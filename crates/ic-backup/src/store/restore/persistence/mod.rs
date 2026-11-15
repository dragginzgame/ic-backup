//! Local original restore intent; no management-effect methods.
use super::{PLAN_LIMIT, RestorePlan, RestorePlanError, RestorePlanRecord, hash};
use crate::store::{
    BackupStore, CaptureCompletionError, CaptureError, CaptureOperation, CapturePlan,
    CaptureProvider, CaptureSourceError, PRIVATE, PayloadSource, StatusHttpTransport,
    capture::{
        acquire, child, exact, ledger::initial_bytes, owner, owner_bytes, retain, same_directory,
        same_identity, target_for,
    },
    open_optional,
};
use ic_host_artifacts::artifact::Sha256Digest;
use ic_host_fs::{
    capability::Directory,
    durable::{
        ExclusiveFileLock, NamedWriteError, PublicationMode, WriteOptions, create_retained_at,
        write_at_validated_with,
    },
    identity::FileIdentity,
};
use std::{ffi::OsStr, fs::File, io::Write, os::fd::AsFd};
use thiserror::Error;

/// Local creation/reopen refusal; retained evidence is never repaired or removed.
#[derive(Debug, Error)]
pub enum RestorePersistenceError<E> {
    /// Original intent, lock, filesystem or ownership refused.
    #[error(transparent)]
    Capture(#[from] CaptureError),
    /// Original immutable declaration refused.
    #[error(transparent)]
    Plan(#[from] RestorePlanError),
    /// Exact source custody or authentication refused.
    #[error("restore source refused")]
    Source(Box<CaptureSourceError<E>>),
    /// Published predecessor/source reconciliation refused.
    #[error("restore predecessor completion refused")]
    Completion(Box<CaptureCompletionError<E>>),
    /// Owner replacement failed; may have published before the failure.
    #[error("restore owner publication failed")]
    Publication(Box<NamedWriteError<CaptureError>>),
}

/// Held original restore intent and source. No remote effects are implemented.
/// Dropping releases process locks, never the durable target claim or source binding.
#[derive(Debug)]
pub struct RestoreOperation<'store> {
    store: &'store BackupStore,
    directory: Directory,
    target: Directory,
    lock: ExclusiveFileLock,
    target_lock_file: File,
    ledger_file: File,
    plan_file: File,
    source: CaptureOperation,
    payload: PayloadSource<'store>,
    plan: RestorePlan,
}
impl RestoreOperation<'_> {
    /// Recheck exact local intent, owner, source and held file identities.
    /// This is inspection only; it does not sync, append or grant effect authority.
    /// # Errors
    /// Refuses incomplete, changed or unsafe custody and every unknown ledger suffix.
    pub fn inspect(&self) -> Result<&RestorePlan, CaptureError> {
        exact(&self.store.root, "format", crate::store::FORMAT, 128)?;
        let operations = child(&self.store.root, "operations", false)?;
        same_directory(
            &operations,
            &self.plan.request().operation_id,
            &self.directory,
        )?;
        same_identity(
            &target_for(&self.store.root, self.plan.request().target, false)?,
            &self.target,
        )?;
        for (dir, name, held) in [
            (&self.target, "lock", &self.target_lock_file),
            (&self.directory, "lock", self.lock.file()),
            (&self.directory, "plan.json", &self.plan_file),
            (&self.directory, "ledger.bin", &self.ledger_file),
        ] {
            let current = open_optional(dir, name)?.ok_or(CaptureError::Incomplete)?;
            if FileIdentity::from_file(&current)? != FileIdentity::from_file(held)? {
                return Err(CaptureError::Changed);
            }
        }
        exact(&self.target, "lock", b"", 0)?;
        exact(&self.directory, "lock", b"", 0)?;
        exact(
            &self.target,
            "owner.json",
            &restore_owner(&self.plan)?,
            1024,
        )?;
        exact(
            &self.directory,
            "plan.json",
            self.plan.encoded(),
            PLAN_LIMIT,
        )?;
        let initial = initial(&self.plan)?;
        exact(&self.directory, "ledger.bin", &initial, initial.len())?;
        for entry in self.target.entries(2)? {
            if !matches!(entry?.name.to_str(), Some("lock" | "owner.json")) {
                return Err(CaptureError::Changed);
            }
        }
        for entry in self.directory.entries(3)? {
            if !matches!(
                entry?.name.to_str(),
                Some("lock" | "plan.json" | "ledger.bin")
            ) {
                return Err(CaptureError::Changed);
            }
        }
        self.source.inspect()?;
        self.payload.check_files()?;
        Ok(&self.plan)
    }
}

impl BackupStore {
    /// Persist approved restore intent after an explicitly selected completed capture owner.
    ///
    /// Drop held capture/source handles first. Target, predecessor, source and new
    /// operation locks are acquired in that order. Source may be an older capture.
    /// Original plan bytes retain the source dependency; no bundle/reference is added.
    /// Only local IO and historical authentication occur. No signing or HTTP occurs.
    /// # Errors
    /// Refuses wrong approval, occupied IDs, incomplete predecessor/source or changed
    /// custody. Failure can retain a new owner with partial intent; never retry blindly.
    pub async fn create_restore<P: CaptureProvider>(
        &self,
        plan: &RestorePlan,
        approval: Sha256Digest,
        predecessor: &CapturePlan,
        transport: &StatusHttpTransport,
        provider: &mut P,
    ) -> Result<RestoreOperation<'_>, RestorePersistenceError<P::Error>> {
        approve(plan, approval)?;
        let initial = initial(plan)?;
        if plan.request().target != predecessor.request().target
            || plan.request().operation_id == predecessor.request().operation_id
        {
            return Err(CaptureError::InvalidPlan.into());
        }
        let operations = child(&self.root, "operations", false)?;
        if operations
            .open_directory(OsStr::new(&plan.request().operation_id), PRIVATE)
            .map_err(CaptureError::from)?
            .is_some()
        {
            return Err(CaptureError::Occupied.into());
        }
        let target = target_for(&self.root, plan.request().target, false)?;
        let target_lock = acquire(
            open_optional(&target, "lock")
                .map_err(CaptureError::from)?
                .ok_or(CaptureError::Incomplete)?,
        )?;
        exact(&target, "owner.json", &owner(predecessor)?, 1024)?;
        let mut previous =
            self.open_capture(&predecessor.request().operation_id, predecessor.digest())?;
        // Verify a published receipt before completion reconciliation: never finish
        // a merely ready predecessor as an implicit part of claiming a target.
        previous.require_published()?;
        previous
            .complete_capture(transport, provider)
            .await
            .map_err(|e| RestorePersistenceError::Completion(Box::new(e)))?;
        let mut source = if plan.source().bundle_id == predecessor.request().operation_id {
            previous
        } else {
            self.open_capture(&plan.source().bundle_id, source_digest(plan)?)?
        };
        let admitted = self
            .open_capture_source(
                &mut source,
                plan.source(),
                plan.request()
                    .limits
                    .payload_bytes
                    .try_into()
                    .map_err(|_| CaptureError::InvalidPlan)?,
                transport,
                provider,
            )
            .await
            .map_err(|e| RestorePersistenceError::Source(Box::new(e)))?;
        let plan = RestorePlan::decode(&admitted, plan.encoded().to_vec())?;
        let payload = admitted.into_payload();
        source
            .complete_capture(transport, provider)
            .await
            .map_err(|e| RestorePersistenceError::Completion(Box::new(e)))?;
        payload.check_files().map_err(CaptureError::from)?;
        let owner_bytes = restore_owner(&plan)?;
        let parent = target.open_syncable().map_err(CaptureError::from)?;
        write_at_validated_with(
            parent.as_fd(),
            OsStr::new("owner.json"),
            WriteOptions {
                mode: PublicationMode::Replace,
                permissions: 0o600,
            },
            |file| {
                file.write_all(&owner_bytes)?;
                Ok(())
            },
            |()| {
                exact(&target, "owner.json", &owner(predecessor)?, 1024)?;
                same_directory(&self.root, "operations", &operations)?;
                same_identity(
                    &target_for(&self.root, plan.request().target, false)?,
                    &target,
                )
            },
        )
        .map_err(|e| RestorePersistenceError::Publication(Box::new(e)))?;
        let directory = child(&operations, &plan.request().operation_id, true)?;
        let lock = create_lock(&directory)?;
        retain(&directory, "plan.json", plan.encoded())?;
        retain(&directory, "ledger.bin", &initial)?;
        directory.sync().map_err(CaptureError::from)?;
        operations.sync().map_err(CaptureError::from)?;
        let operation = held(
            self,
            plan,
            directory,
            target,
            lock,
            &target_lock,
            source,
            payload,
        )?;
        operation.inspect()?;
        Ok(operation)
    }

    /// Reopen complete original restore intent using the caller's exact approved plan.
    ///
    /// No file is created or synchronized. Re-admits the original source under
    /// target-before-source-before-restore locks. The caller may retain the plan or
    /// decode its original bytes against a source before dropping that source handle.
    /// # Errors
    /// Refuses changed approval, files, source, owner, unknown tails or busy locks.
    pub async fn open_restore<P: CaptureProvider>(
        &self,
        plan: &RestorePlan,
        approval: Sha256Digest,
        transport: &StatusHttpTransport,
        provider: &mut P,
    ) -> Result<RestoreOperation<'_>, RestorePersistenceError<P::Error>> {
        approve(plan, approval)?;
        let target = target_for(&self.root, plan.request().target, false)?;
        // Opening must never create a missing established target lock.
        let target_lock = acquire(
            open_optional(&target, "lock")
                .map_err(CaptureError::from)?
                .ok_or(CaptureError::Incomplete)?,
        )?;
        exact(&target, "owner.json", &restore_owner(plan)?, 1024)?;
        let mut source = self.open_capture(&plan.source().bundle_id, source_digest(plan)?)?;
        let admitted = self
            .open_capture_source(
                &mut source,
                plan.source(),
                plan.request()
                    .limits
                    .payload_bytes
                    .try_into()
                    .map_err(|_| CaptureError::InvalidPlan)?,
                transport,
                provider,
            )
            .await
            .map_err(|e| RestorePersistenceError::Source(Box::new(e)))?;
        let plan = RestorePlan::decode(&admitted, plan.encoded().to_vec())?;
        let payload = admitted.into_payload();
        let operations = child(&self.root, "operations", false)?;
        let directory = child(&operations, &plan.request().operation_id, false)?;
        let lock = acquire(
            open_optional(&directory, "lock")
                .map_err(CaptureError::from)?
                .ok_or(CaptureError::Incomplete)?,
        )?;
        let operation = held(
            self,
            plan,
            directory,
            target,
            lock,
            &target_lock,
            source,
            payload,
        )?;
        operation.inspect()?;
        Ok(operation)
    }
}
fn approve(plan: &RestorePlan, approval: Sha256Digest) -> Result<(), CaptureError> {
    if plan.digest() != approval {
        return Err(CaptureError::ApprovalMismatch);
    }
    Ok(())
}
fn source_digest(plan: &RestorePlan) -> Result<Sha256Digest, RestorePlanError> {
    let record: RestorePlanRecord = serde_json::from_slice(plan.encoded())?;
    hash(&record.source_plan_sha256)
}
fn initial(plan: &RestorePlan) -> Result<Vec<u8>, CaptureError> {
    Ok(initial_bytes(
        plan.encoded(),
        plan.request().limits.evidence_bytes,
    )?)
}
fn restore_owner(plan: &RestorePlan) -> Result<Vec<u8>, CaptureError> {
    owner_bytes(&plan.request().operation_id, plan.digest())
}
fn create_lock(directory: &Directory) -> Result<ExclusiveFileLock, CaptureError> {
    let parent = directory.open_syncable()?;
    let file = create_retained_at(parent.as_fd(), OsStr::new("lock"), 0o600)?;
    file.sync_all()?;
    acquire(file)
}
#[expect(
    clippy::too_many_arguments,
    reason = "construct one held operation without a parallel persisted state"
)]
fn held<'s>(
    store: &'s BackupStore,
    plan: RestorePlan,
    directory: Directory,
    target: Directory,
    lock: ExclusiveFileLock,
    target_lock: &ExclusiveFileLock,
    source: CaptureOperation,
    payload: PayloadSource<'s>,
) -> Result<RestoreOperation<'s>, CaptureError> {
    let plan_file = open_optional(&directory, "plan.json")?.ok_or(CaptureError::Incomplete)?;
    let ledger_file = open_optional(&directory, "ledger.bin")?.ok_or(CaptureError::Incomplete)?;
    Ok(RestoreOperation {
        store,
        directory,
        target,
        lock,
        target_lock_file: target_lock.file().try_clone()?,
        ledger_file,
        plan_file,
        source,
        payload,
        plan,
    })
}
