//! Explicit replacement of one completed capture's target pointer.
use super::{
    BackupStore, CaptureCompletionError, CaptureError, CaptureOperation, CapturePlan,
    CaptureProvider, PRIVATE, StatusHttpTransport, child, exact, lock_target, owner,
    same_directory, same_identity, target_directory,
};
use ic_host_artifacts::artifact::Sha256Digest;
use ic_host_fs::durable::{
    NamedWriteError, PublicationMode, WriteOptions, write_at_validated_with,
};
use std::{ffi::OsStr, io::Write, os::fd::AsFd};
use thiserror::Error;

/// Succession never removes predecessor evidence or refunds its authority.
#[derive(Debug, Error)]
pub enum CaptureSuccessionError<E> {
    /// Intent, exclusion or original custody refused.
    #[error(transparent)]
    Capture(#[from] CaptureError),
    /// Original completion authentication or durability reconciliation refused.
    #[error("predecessor completion refused")]
    Completion(Box<CaptureCompletionError<E>>),
    /// Owner publication failed, preserving its before/after-publication distinction.
    #[error("successor owner publication failed")]
    Publication(Box<NamedWriteError<CaptureError>>),
}

impl BackupStore {
    /// Create a fresh capture after an exact completed predecessor on the same target.
    ///
    /// Drop any predecessor handle first. Takes the target lock, then nonblocking
    /// predecessor custody; authenticates and synchronizes its already-published
    /// completion without remote work. Atomically replaces only `owner.json`, then
    /// creates the new intent through the ordinary creation path. Old plan, journal,
    /// bundle and lock files remain intact. New calls need their own admission.
    ///
    /// # Errors
    /// Refuses wrong approval/target, occupied IDs, held locks, unfinished or changed
    /// predecessor evidence. Failures after owner replacement may leave an occupied
    /// incomplete successor. Reopen only complete original intent; never retry
    /// replacement blindly or remove retained files to recover.
    pub async fn create_capture_after<P: CaptureProvider>(
        &self,
        plan: CapturePlan,
        approval: Sha256Digest,
        predecessor: &CapturePlan,
        transport: &StatusHttpTransport,
        provider: &mut P,
    ) -> Result<CaptureOperation, CaptureSuccessionError<P::Error>> {
        if plan.digest() != approval {
            return Err(CaptureError::ApprovalMismatch.into());
        }
        if plan.request().target != predecessor.request().target
            || plan.request().operation_id == predecessor.request().operation_id
        {
            return Err(CaptureError::InvalidPlan.into());
        }
        let ledger_bytes = super::ledger::initial(&plan).map_err(CaptureError::from)?;
        exact(&self.root, "format", super::super::FORMAT, 128)?;
        let operations = child(&self.root, "operations", false)?;
        if operations
            .open_directory(OsStr::new(&plan.request().operation_id), PRIVATE)
            .map_err(CaptureError::from)?
            .is_some()
        {
            return Err(CaptureError::Occupied.into());
        }
        let target = target_directory(&self.root, predecessor, false)?;
        let (target_lock, _) = lock_target(&target)?;
        exact(&target, "owner.json", &owner(predecessor)?, 1024)?;
        let mut previous =
            self.open_capture(&predecessor.request().operation_id, predecessor.digest())?;
        same_identity(&target, &previous.target)?;
        let state = previous.check()?;
        if !previous.check_assembly(&state)? {
            return Err(CaptureError::Incomplete.into());
        }
        previous
            .complete_capture(transport, provider)
            .await
            .map_err(|e| CaptureSuccessionError::Completion(Box::new(e)))?;
        let owner_bytes = owner(&plan)?;
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
                // Host owns a temporary sibling during this callback. Both locks
                // retain the admitted predecessor; do not rescan that namespace.
                exact(&target, "owner.json", &owner(predecessor)?, 1024)?;
                same_directory(&self.root, "operations", &operations)?;
                same_identity(&target_directory(&self.root, predecessor, false)?, &target)
            },
        )
        .map_err(|e| CaptureSuccessionError::Publication(Box::new(e)))?;
        exact(&target, "owner.json", &owner_bytes, 1024)?;
        self.create_claimed_capture(plan, target, target_lock, &ledger_bytes)
            .map_err(Into::into)
    }
}
