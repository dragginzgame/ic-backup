//! Immutable bounded operation plan publication and original local intent admission.

use super::{
    BackupLayoutGuard, JournalLock, JournalLockError, PersistenceError, create_json_durable,
    read_json,
};
use crate::model::{
    artifacts::ArtifactChecksumRecord,
    operation_plan::{MAX_OPERATION_PLAN_BYTES, OperationPlanRecord},
};
use thiserror::Error;

/// Durably create fixed `operation-plan.json` without replacing prior evidence.
///
/// The embedded inventory/graph are the declaration's original bindings; separate
/// retained inventory/graph files are not adopted or rewritten by this operation.
///
/// # Errors
/// Rejects excessive canonical bytes, unsafe/existing entries, replaced roots and IO/locks.
pub fn create_operation_plan(
    layout: &BackupLayoutGuard,
    record: &OperationPlanRecord,
) -> Result<(), OperationPlanPersistenceError> {
    layout.check_root()?;
    let path = layout.root().join("operation-plan.json");
    let _lock = JournalLock::acquire(&path)?;
    check_size(record)?;
    create_json_durable(&path, record)?;
    Ok(())
}

/// Admit bounded validated local plan under its exact original expected intent digest.
///
/// Lost creation responses reconcile through this exact local read. This observes
/// no remote authority, does not create/reset journals and grants no dispatch permit.
///
/// # Errors
/// Rejects unsafe/missing/oversized/invalid declarations, digest mismatch and ownership failures.
pub fn read_operation_plan(
    layout: &BackupLayoutGuard,
    expected: &ArtifactChecksumRecord,
) -> Result<OperationPlanRecord, OperationPlanPersistenceError> {
    layout.check_root()?;
    let path = layout.root().join("operation-plan.json");
    let _lock = JournalLock::acquire(&path)?;
    let record: OperationPlanRecord = read_json(&path, MAX_OPERATION_PLAN_BYTES)?;
    check_size(&record)?;
    if &record.digest() != expected {
        return Err(OperationPlanPersistenceError::DigestMismatch);
    }
    Ok(record)
}
fn check_size(record: &OperationPlanRecord) -> Result<(), PersistenceError> {
    super::json::check_json_size(record, MAX_OPERATION_PLAN_BYTES)
}

/// Typed original operation-plan identity or bounded immutable local admission failure.
#[derive(Debug, Error)]
pub enum OperationPlanPersistenceError {
    /// Retained declaration differs from the exact original selected intent digest.
    #[error("operation plan digest mismatch")]
    DigestMismatch,
    /// Cooperating layout/journal ownership failed.
    #[error(transparent)]
    Lock(#[from] JournalLockError),
    /// Bounded JSON, model admission or durable filesystem access failed.
    #[error(transparent)]
    Persistence(#[from] PersistenceError),
}

#[cfg(all(test, unix))]
mod tests;
