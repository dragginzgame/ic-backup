//! Original restore approval/source custody over the shared recorder.
use super::{RestorePlan, RestorePlanError, procedure::Procedure};
use crate::store::BackupDigest;
use crate::store::operation::{
    replay::{self, Error as ReplayError},
    runtime::{Action, Connection, Error, Guard, Progress, Signer, now},
};
use crate::store::{
    BackupStore, CaptureOperation, CapturePlan, CaptureSourceError, OperationReplayError,
    OperationStoreError, PRIVATE, PayloadSource, RestoreTransfer,
    capture::{
        child, exact, lock_target, owner, owner_bytes, same_directory, same_identity, target_for,
    },
    open_optional,
    operation::{FactRecord, Journal, custody::Recorder, intent::Intent},
};
use ic_host_fs::Directory;
use ic_platform_protocol::management_snapshots::SnapshotMetadata;
use std::ffi::OsStr;

/// Original immutable approval, source custody or local publication refused.
#[derive(Debug, thiserror::Error)]
pub enum RestorePersistenceError {
    /// Original local custody or finite declaration refused.
    #[error(transparent)]
    Capture(#[from] OperationStoreError),
    /// Exact approved restore declaration refused.
    #[error(transparent)]
    Plan(#[from] RestorePlanError),
    /// Selected original source refused.
    #[error("restore source refused")]
    Source(Box<CaptureSourceError>),
    /// Original completion or call evidence refused.
    #[error(transparent)]
    Completion(#[from] OperationReplayError),
    /// Owner replacement may have published before failure.
    #[error("restore owner publication failed")]
    Publication(Box<crate::store::StoreError>),
    /// Original custody validation failed before replacing owner authority.
    #[error("owner admission refused: {source}")]
    OwnerAdmission {
        /// Original custody refusal.
        #[source]
        source: Box<OperationStoreError>,
        /// Explicit stage cleanup refusal, if any.
        cleanup: Option<ic_host_fs::Error>,
    },
}
#[derive(Debug)]
pub(in crate::store) struct Source<'store> {
    pub capture: CaptureOperation,
    pub payload: PayloadSource<'store>,
    pub metadata: SnapshotMetadata,
}
impl Source<'_> {
    pub(in crate::store) fn check(&self) -> Result<(), OperationStoreError> {
        self.capture.check_custody()?;
        self.payload.check_files()?;
        Ok(())
    }
    pub(in crate::store) fn transfer(
        &self,
        plan: &RestorePlan,
    ) -> Result<RestoreTransfer, crate::store::RestoreTransferError> {
        crate::store::capture::source::transfer::plan_payload(
            &self.payload,
            &plan.transfer_request(),
        )
    }
}
/// Held restore approval and original source; dropping never erases ownership or evidence.
#[derive(Debug)]
pub struct RestoreOperation<'store> {
    pub(in crate::store) record: Recorder<RestorePlan>,
    source: Source<'store>,
}
/// Exact local outcome derived from the closed original restore journal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadedStopped {
    /// Original restore operation identity.
    pub operation_id: String,
    /// Exact restored network and canister.
    pub target: crate::store::BackupTarget,
    /// Original capture operation identity.
    pub source_operation_id: String,
    /// Exact authenticated original payload manifest.
    pub source_manifest_sha256: BackupDigest,
    /// Newly allocated destination snapshot, retained by the IC.
    pub snapshot_id: Vec<u8>,
    /// Original local completion timestamp, never renewed on replay.
    pub completed_at_ns: u64,
    /// Exact immutable restore approval.
    pub plan_sha256: BackupDigest,
    /// Closing hash-chain commitment to original calls and local completion.
    pub journal_sha256: BackupDigest,
}
impl RestoreOperation<'_> {
    /// Exact immutable original restore approval.
    #[must_use]
    pub fn plan(&self) -> &RestorePlan {
        self.record.plan()
    }
    fn project<'a>(
        &mut self,
        journal: &Journal,
        plan: &'a RestorePlan,
    ) -> Result<Procedure<'a>, ReplayError> {
        self.source.check()?;
        let mut flow = Procedure::new(plan, &self.source)?;
        for (index, call) in journal.calls.iter().enumerate() {
            if flow.next()? != Some(call.step) {
                return Err(ReplayError::History);
            }
            let (method, arguments) = flow.request(&mut self.source)?;
            let result =
                replay::resolve_call(plan.view(), call, method, &arguments, &mut |call, role| {
                    self.record.read_role(call, role)
                })?;
            if let Some(result) = result {
                flow.accept(
                    call.step,
                    result.rejected,
                    result.request_id,
                    &result.response,
                    &mut self.source,
                )?;
            } else {
                if index + 1 != journal.calls.len() || journal.completed_at_ns.is_some() {
                    return Err(ReplayError::History);
                }
                return Ok(flow);
            }
        }
        if journal.completed_at_ns.is_some() && flow.next()?.is_some() {
            return Err(ReplayError::History);
        }
        self.source.check()?;
        Ok(flow)
    }
    fn completion(
        &self,
        journal: &Journal,
        flow: &Procedure<'_>,
    ) -> Result<Option<LoadedStopped>, ReplayError> {
        let Some(completed_at_ns) = journal.completed_at_ns else {
            return Ok(None);
        };
        if flow.next()?.is_some() {
            return Err(ReplayError::History);
        }
        let plan = self.plan();
        Ok(Some(LoadedStopped {
            operation_id: plan.request().operation_id.clone(),
            target: plan.request().target,
            source_operation_id: plan.source_operation_id().into(),
            source_manifest_sha256: plan.source_manifest_sha256(),
            snapshot_id: flow.destination()?.to_vec(),
            completed_at_ns,
            plan_sha256: plan.digest(),
            journal_sha256: journal.head,
        }))
    }
    /// Authenticate original progress and source bytes with no effects, clock or writes.
    /// # Errors
    /// Refuses changed custody, invalid original evidence or failed restore policy.
    pub fn inspect(&mut self) -> Result<crate::store::OperationView<LoadedStopped>, ReplayError> {
        let journal = self.record.check()?;
        let plan = self.plan().clone();
        let flow = self.project(&journal, &plan)?;
        self.record
            .view(&journal, flow.next()?, self.completion(&journal, &flow)?)
    }
    /// Advance the fixed restore with at most one original management submission.
    /// Completion verifies original evidence offline and leaves the application stopped.
    /// # Errors
    /// Refuses changed source, live authority, exhausted allowance or failed readback/load policy.
    pub async fn advance<S: Signer, G: Guard>(
        &mut self,
        connection: &mut Connection<S, G>,
    ) -> Result<Progress<LoadedStopped>, Error<S::Error, G::Error>> {
        let plan = self.plan().clone();
        let journal = self.record.check()?;
        let mut flow = self.project(&journal, &plan)?;
        if let Some(receipt) = self.completion(&journal, &flow)? {
            return Ok(Progress::Complete(Box::new(receipt)));
        }
        let Some(step) = flow.next()? else {
            let completed_at_ns = now().ok_or(Error::Deadline)?;
            self.source.check()?;
            let closed = self.record.append(
                &journal,
                FactRecord::LoadedStopped { completed_at_ns },
                None,
            )?;
            return Ok(Progress::Complete(Box::new(
                self.completion(&closed, &flow)?
                    .ok_or(OperationStoreError::Incomplete)?,
            )));
        };
        // Admit the whole known upload/readback/load schedule before allocating anything.
        if step == crate::store::OperationStep::AllocateSnapshot
            && journal.calls.last().is_none_or(|c| c.step != step)
        {
            journal.remaining(
                flow.data_calls()
                    .checked_mul(2)
                    .and_then(|n| n.checked_add(5))
                    .ok_or(OperationStoreError::StatusAdmission)?,
            )?;
        }
        let (method, arguments) = flow.request(&mut self.source)?;
        let source = &self.source;
        let result = self
            .record
            .run(
                journal,
                connection,
                step,
                action(step),
                method,
                &arguments,
                || source.check(),
            )
            .await?;
        let Some(result) = result else {
            return Ok(Progress::NeedsAttention(step));
        };
        flow.accept(
            step,
            result.rejected,
            result.request_id,
            &result.response,
            &mut self.source,
        )?;
        Ok(Progress::Advanced)
    }
    /// Settle the original pending call from surviving terminal certificate bytes.
    /// # Errors
    /// Refuses resolved or changed history; retained policy failures remain settled.
    pub fn reconcile_reply(&mut self, certificate: &[u8]) -> Result<(), ReplayError> {
        self.reconcile_certificate(certificate, false)
    }

    /// Import the surviving certificate from the original submitted read-state lookup.
    /// Uses its separate retained time policy without clock, signing or network work.
    /// # Errors
    /// Refuses missing/resolved lookup, changed source/custody or invalid evidence/policy.
    pub fn reconcile_observation_reply(&mut self, certificate: &[u8]) -> Result<(), ReplayError> {
        self.reconcile_certificate(certificate, true)
    }

    fn reconcile_certificate(
        &mut self,
        certificate: &[u8],
        observation: bool,
    ) -> Result<(), ReplayError> {
        let plan = self.plan().clone();
        let journal = self.record.check()?;
        let mut flow = self.project(&journal, &plan)?;
        let step = journal
            .calls
            .last()
            .ok_or(OperationStoreError::Incomplete)?
            .step;
        if flow.next()? != Some(step) {
            return Err(ReplayError::History);
        }
        let (method, arguments) = flow.request(&mut self.source)?;
        let result = if observation {
            self.record.import_observation(&journal, certificate)?
        } else {
            Some(
                self.record
                    .import_reply(&journal, method, &arguments, certificate)?,
            )
        };
        let Some(result) = result else {
            return Ok(());
        };
        flow.accept(
            step,
            result.rejected,
            result.request_id,
            &result.response,
            &mut self.source,
        )?;
        Ok(())
    }
    /// Make the sole budgeted observation of the original pending request.
    /// # Errors
    /// Refuses renewed attempts, changed custody, invalid proof or unavailable live authority.
    pub async fn reconcile_observation<S: Signer, G: Guard>(
        &mut self,
        connection: &mut Connection<S, G>,
    ) -> Result<Progress<LoadedStopped>, Error<S::Error, G::Error>> {
        let plan = self.plan().clone();
        let journal = self.record.check()?;
        let mut flow = self.project(&journal, &plan)?;
        let step = journal
            .calls
            .last()
            .ok_or(OperationStoreError::Incomplete)?
            .step;
        if flow.next()? != Some(step) {
            return Err(Error::Resolved);
        }
        let source = &self.source;
        let result = self
            .record
            .observe(journal, connection, || source.check())
            .await?;
        let Some(result) = result else {
            return Ok(Progress::NeedsAttention(step));
        };
        flow.accept(
            step,
            result.rejected,
            result.request_id,
            &result.response,
            &mut self.source,
        )?;
        Ok(Progress::Advanced)
    }
    pub(in crate::store) fn sync_completed(&mut self) -> Result<(), ReplayError> {
        if self.inspect()?.completed.is_none() {
            return Err(OperationStoreError::Incomplete.into());
        }
        self.record.sync_evidence()?;
        self.source.capture.sync_published()?;
        self.source.check()?;
        self.inspect()?;
        Ok(())
    }
}
fn action(step: crate::store::OperationStep) -> Action {
    use crate::store::OperationStep::*;
    match step {
        AllocateSnapshot => Action::AllocateSnapshot,
        Upload(_) => Action::UploadSnapshot,
        LoadSnapshot => Action::LoadSnapshot,
        ReadbackMetadata | Readback(_) => Action::ReadSnapshot,
        _ => Action::Inspect,
    }
}
impl BackupStore {
    fn restore_source<'s>(
        &'s self,
        plan: &RestorePlan,
        mut capture: CaptureOperation,
    ) -> Result<Source<'s>, RestorePersistenceError> {
        let summary = capture
            .inspect()?
            .completed
            .ok_or(OperationStoreError::Incomplete)?;
        if summary.manifest_sha256 != plan.source_manifest_sha256() {
            return Err(RestorePlanError::SourceBinding.into());
        }
        let admitted = self
            .open_capture_source(
                &mut capture,
                &summary,
                plan.request()
                    .limits
                    .payload_bytes
                    .try_into()
                    .map_err(|_| OperationStoreError::InvalidPlan)?,
            )
            .map_err(|e| RestorePersistenceError::Source(Box::new(e)))?;
        plan.validate_source(&admitted)?;
        let (payload, metadata) = admitted.into_parts();
        Ok(Source {
            capture,
            payload,
            metadata,
        })
    }
    /// Persist the exact approved restore after a completed capture owner on the same target.
    /// # Errors
    /// Refuses occupied IDs, wrong approval/source/owner or uncertain local publication.
    pub fn create_restore(
        &self,
        plan: &RestorePlan,
        approval: BackupDigest,
        predecessor: &CapturePlan,
    ) -> Result<RestoreOperation<'_>, RestorePersistenceError> {
        approve(plan, approval)?;
        if plan.request().target != predecessor.request().target
            || plan.request().operation_id == predecessor.request().operation_id
        {
            return Err(OperationStoreError::InvalidPlan.into());
        }
        let initial = Journal::create(plan).map_err(OperationStoreError::from)?;
        let operations = child(&self.root, "operations", false)?;
        if operations
            .open_directory(OsStr::new(&plan.request().operation_id), PRIVATE)
            .map_err(OperationStoreError::from)?
            .is_some()
        {
            return Err(OperationStoreError::Occupied.into());
        }
        let target = target_for(&self.root, plan.request().target, false)?;
        let (target_lock, _) = lock_target(&target)?;
        exact(&target, "owner.json", &owner(predecessor)?, 1024)?;
        let mut previous =
            self.open_capture(&predecessor.request().operation_id, predecessor.digest())?;
        previous.sync_published()?;
        let same = plan.source_operation_id() == predecessor.request().operation_id;
        let capture = if same {
            previous
        } else {
            self.open_capture(plan.source_operation_id(), plan.source_plan_sha256())?
        };
        let mut source = self.restore_source(plan, capture)?;
        if !same {
            source.capture.sync_published()?;
        }
        source.check()?;
        replace_owner(
            &self.root,
            &target,
            &operations,
            predecessor.view(),
            plan.view(),
        )?;
        let record = Recorder::create_claimed(&self.root, plan.clone(), target_lock, &initial)?;
        Ok(RestoreOperation { record, source })
    }
    /// Reopen by original operation ID and approval without a connection or reconstructed plan.
    /// Selection reads bounded original intent; target/source/operation locks then
    /// admit those same exact bytes and all retained evidence without writes or repair.
    ///
    /// ```no_run
    /// use ic_backup::store::{BackupStore, LoadedStopped, RestorePersistenceError};
    /// use ic_backup::store::BackupDigest;
    ///
    /// fn restored_outcome(
    ///     store: &BackupStore, original_id: &str, original_approval: BackupDigest,
    /// ) -> Result<Option<LoadedStopped>, RestorePersistenceError> {
    ///     let mut operation = store.open_restore(original_id, original_approval)?;
    ///     // The store supplies the original plan; no signer or live guard is needed.
    ///     Ok(operation.inspect()?.completed)
    /// }
    /// ```
    ///
    /// # Errors
    /// Refuses mismatched approval, missing original files or invalid retained evidence.
    pub fn open_restore(
        &self,
        id: &str,
        approval: BackupDigest,
    ) -> Result<RestoreOperation<'_>, RestorePersistenceError> {
        let plan = Recorder::<RestorePlan>::select_intent(&self.root, id)?;
        approve(&plan, approval)?;
        let target = target_for(&self.root, plan.request().target, false)?;
        let target_lock = crate::store::capture::acquire(
            open_optional(&target, "lock")
                .map_err(OperationStoreError::from)?
                .ok_or(OperationStoreError::Incomplete)?,
        )?;
        self.open_restore_locked(&plan, approval, &target, &target_lock)
    }
    pub(in crate::store) fn open_restore_locked(
        &self,
        plan: &RestorePlan,
        approval: BackupDigest,
        target: &Directory,
        target_lock: &ic_host_fs::Lock,
    ) -> Result<RestoreOperation<'_>, RestorePersistenceError> {
        approve(plan, approval)?;
        let source = self.restore_source(
            plan,
            self.open_capture(plan.source_operation_id(), plan.source_plan_sha256())?,
        )?;
        let record = Recorder::<RestorePlan>::open(&self.root, &plan.request().operation_id)?;
        if record.plan().encoded() != plan.encoded() {
            return Err(OperationStoreError::ApprovalMismatch.into());
        }
        same_identity(
            target,
            &target_for(&self.root, plan.request().target, false)?,
        )?;
        record.check_target_lock(target_lock)?;
        let mut operation = RestoreOperation { record, source };
        operation.inspect()?;
        Ok(operation)
    }
}
fn approve(plan: &RestorePlan, approval: BackupDigest) -> Result<(), OperationStoreError> {
    if plan.digest() != approval {
        return Err(OperationStoreError::ApprovalMismatch);
    }
    Ok(())
}
pub(in crate::store) fn replace_owner(
    store: &Directory,
    target: &Directory,
    operations: &Directory,
    previous: crate::store::operation::intent::Plan<'_>,
    next: crate::store::operation::intent::Plan<'_>,
) -> Result<(), RestorePersistenceError> {
    let before = owner_bytes(previous.operation_id(), previous.digest())?;
    let bytes = owner_bytes(next.operation_id(), next.digest())?;
    let stage_owner = || -> Result<ic_host_fs::Sealed, crate::store::StoreError> {
        let stage = target
            .stage(
                OsStr::new("owner.pending"),
                OsStr::new("owner.json"),
                ic_host_fs::Policy {
                    destination: ic_host_fs::Destination::Replace,
                    permissions: 0o600,
                    retention: ic_host_fs::Retention::Discard,
                },
            )?
            .write_from(
                bytes.as_slice(),
                ic_host_artifacts::Length::Exactly(bytes.len() as u64),
            )?
            .seal()?;
        // All original custody checks below run before granting replacement visibility.
        Ok(stage)
    };
    let stage = stage_owner().map_err(|e| RestorePersistenceError::Publication(Box::new(e)))?;
    let admission = || {
        exact(target, "owner.json", &before, 1024)?;
        same_directory(store, "operations", operations)?;
        same_identity(&target_for(store, previous.target(), false)?, target)
    };
    if let Err(source) = admission() {
        return Err(RestorePersistenceError::OwnerAdmission {
            source: Box::new(source),
            cleanup: stage.abort().err(),
        });
    }
    match stage.commit() {
        ic_host_fs::Commit::Committed(_) => (),
        ic_host_fs::Commit::NotPublished { cause, stage } => {
            return Err(RestorePersistenceError::Publication(Box::new(
                crate::store::StoreError::NotPublished {
                    cause,
                    cleanup: stage.abort().err(),
                },
            )));
        }
        ic_host_fs::Commit::PublishedUnconfirmed {
            receipt,
            phase,
            cause,
            sync_error,
        } => {
            return Err(RestorePersistenceError::Publication(Box::new(
                crate::store::StoreError::PublishedUnconfirmed {
                    receipt: Box::new(receipt),
                    phase,
                    cause,
                    sync_error,
                },
            )));
        }
    }
    exact(target, "owner.json", &bytes, 1024)?;
    Ok(())
}
