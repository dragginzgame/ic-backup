//! Explicit target succession after exact authenticated local completion.

use super::{
    BackupStore, CaptureOperation, CapturePlan, OperationReplayError, OperationStoreError, PRIVATE,
    child, exact, lock_target, target_for,
};
use crate::store::{
    BackupDigest,
    operation::intent::{Intent, Plan},
};
use std::ffi::OsStr;
use thiserror::Error;

///
/// CaptureSuccessionError
/// Succession never removes predecessor evidence or refunds its original authority.
///

#[derive(Debug, Error)]
pub enum CaptureSuccessionError {
    /// Original intent, exclusion or local custody refused.
    #[error(transparent)]
    Capture(#[from] OperationStoreError),
    /// Original certified completion or payload refused.
    #[error(transparent)]
    Completion(#[from] OperationReplayError),
    /// Restore predecessor or common owner publication refused.
    #[error(transparent)]
    Restore(Box<crate::store::RestorePersistenceError>),
}

impl BackupStore {
    /// Create fresh capture intent after an exact published predecessor on the same target.
    ///
    /// Takes target then predecessor locks, verifies and synchronizes the original
    /// published receipt, atomically replaces only the owner and creates new intent.
    /// No signing, guard, clock, HTTP, pruning or old-operation mutation occurs.
    /// # Errors
    /// Refuses wrong approval/target, held locks, occupied IDs or unfinished/changed
    /// completion. Failure may leave an occupied partial successor; never reset it.
    pub fn create_capture_after(
        &self,
        plan: CapturePlan,
        approval: BackupDigest,
        predecessor: Plan<'_>,
    ) -> Result<CaptureOperation, CaptureSuccessionError> {
        if plan.digest() != approval {
            return Err(OperationStoreError::ApprovalMismatch.into());
        }
        if plan.request().target != predecessor.target()
            || plan.request().operation_id == predecessor.operation_id()
        {
            return Err(OperationStoreError::InvalidPlan.into());
        }
        let initial =
            crate::store::operation::Journal::create(&plan).map_err(OperationStoreError::from)?;
        exact(&self.root, "format", super::super::FORMAT, 128)?;
        let operations = child(&self.root, "operations", false)?;
        if operations
            .open_directory(OsStr::new(&plan.request().operation_id), PRIVATE)
            .map_err(OperationStoreError::from)?
            .is_some()
        {
            return Err(OperationStoreError::Occupied.into());
        }
        let target = target_for(&self.root, predecessor.target(), false)?;
        let (target_lock, _) = lock_target(&target)?;
        exact(
            &target,
            "owner.json",
            &super::owner_bytes(predecessor.operation_id(), predecessor.digest())?,
            1024,
        )?;
        let finish = |target_lock| {
            crate::store::restore::persistence::replace_owner(
                &self.root,
                &target,
                &operations,
                predecessor,
                plan.view(),
            )
            .map_err(|e| CaptureSuccessionError::Restore(Box::new(e)))?;

            Ok(CaptureOperation::create_claimed(
                &self.root,
                plan,
                target_lock,
                &initial,
            )?)
        };

        match predecessor {
            Plan::Capture(p) => {
                let mut previous = self.open_capture(&p.request().operation_id, p.digest())?;
                previous.sync_published()?;
                let result = finish(target_lock);
                drop(previous);

                result
            }
            Plan::Restore(p) => {
                let mut previous = self
                    .open_restore_locked(p, p.digest(), &target, &target_lock)
                    .map_err(|e| CaptureSuccessionError::Restore(Box::new(e)))?;
                previous.sync_completed()?;
                let result = finish(target_lock);
                drop(previous);

                result
            }
        }
    }
}
