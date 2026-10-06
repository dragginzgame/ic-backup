//! Bounded immutable original-plan consistency requirement retention; no fence authority.

use super::{
    BackupLayoutGuard, JournalLock, JournalLockError, OperationPlanPersistenceError,
    PersistenceError, create_json_durable, read_json, read_operation_plan,
};
use crate::model::{
    artifacts::ArtifactChecksumRecord,
    consistency::{
        ConsistencyRequirementError, ConsistencyRequirementRecord,
        MAX_CONSISTENCY_REQUIREMENT_BYTES,
    },
    operation_plan::OperationPlanRecord,
};
use thiserror::Error;

/// Durably create fixed `consistency-requirement.json` without replacing original evidence.
///
/// Requires the exact original plan already retained under layout exclusion. This
/// persists the reviewed guarantee, not an active fence, paid allowance or release permit.
/// # Errors
/// Rejects mismatched/missing original plan, excessive bytes, unsafe/existing paths and IO/locks.
pub fn create_consistency_requirement(
    layout: &BackupLayoutGuard,
    plan: &OperationPlanRecord,
    record: &ConsistencyRequirementRecord,
) -> Result<(), ConsistencyPersistenceError> {
    record.validate_plan(plan)?;
    read_operation_plan(layout, &plan.digest())?;
    let path = layout.root().join("consistency-requirement.json");
    let _lock = JournalLock::acquire(&path)?;
    check_size(record)?;
    create_json_durable(&path, record)?;
    Ok(())
}
/// Read bounded validated original requirement under its exact expected digest and retained plan.
///
/// Lost local creation replies reconcile by this read; never overwrite/downgrade an
/// existing guarantee or recover a fence/consumed authority from absence.
/// # Errors
/// Rejects unsafe/missing/oversized/invalid declarations, original plan or requirement mismatch.
pub fn read_consistency_requirement(
    layout: &BackupLayoutGuard,
    plan: &OperationPlanRecord,
    expected: &ArtifactChecksumRecord,
) -> Result<ConsistencyRequirementRecord, ConsistencyPersistenceError> {
    read_operation_plan(layout, &plan.digest())?;
    let path = layout.root().join("consistency-requirement.json");
    let _lock = JournalLock::acquire(&path)?;
    let record: ConsistencyRequirementRecord = read_json(&path, MAX_CONSISTENCY_REQUIREMENT_BYTES)?;
    check_size(&record)?;
    record.validate_plan(plan)?;
    if &record.digest() != expected {
        return Err(ConsistencyPersistenceError::DigestMismatch);
    }
    Ok(record)
}
fn check_size(record: &ConsistencyRequirementRecord) -> Result<(), PersistenceError> {
    super::json::check_json_size(record, MAX_CONSISTENCY_REQUIREMENT_BYTES)
}
/// Typed exact retained requirement or bounded local storage denial.
#[derive(Debug, Error)]
pub enum ConsistencyPersistenceError {
    /// Retained guarantee differs from the exact originally expected requirement.
    #[error("consistency requirement digest mismatch")]
    DigestMismatch,
    /// Original full plan identity differs.
    #[error(transparent)]
    Requirement(#[from] ConsistencyRequirementError),
    /// Original plan is not admitted from the held layout.
    #[error(transparent)]
    Plan(#[from] OperationPlanPersistenceError),
    /// Layout/journal exclusion failed.
    #[error(transparent)]
    Lock(#[from] JournalLockError),
    /// Bounded JSON or durable filesystem access failed.
    #[error(transparent)]
    Persistence(#[from] PersistenceError),
}

#[cfg(all(test, unix))]
mod tests;
