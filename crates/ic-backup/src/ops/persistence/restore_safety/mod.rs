//! Immutable original restore/source safety retention under both layout guards.

use super::{
    BackupLayoutGuard, JournalLock, JournalLockError, OperationPlanPersistenceError,
    PersistenceError, create_json_durable, read_json, read_operation_plan,
};
use crate::model::{
    artifacts::ArtifactChecksumRecord,
    operation_plan::OperationPlanRecord,
    restore_safety::{
        MAX_RESTORE_SAFETY_REQUIREMENT_BYTES, RestoreSafetyRequirementError,
        RestoreSafetyRequirementRecord,
    },
};
use thiserror::Error;

/// Durably create fixed `restore-safety-requirement.json` without replacement.
///
/// Both exact original plans must already be retained under their layout guards.
/// Artifact completeness, source-reference retention and current application/fence
/// custody remain separately admitted by their owners. This persists declarations.
/// # Errors
/// Rejects changed/missing plans, inappropriate source, excessive bytes or unsafe/existing paths.
pub fn create_restore_safety_requirement(
    layout: &BackupLayoutGuard,
    source_layout: &BackupLayoutGuard,
    plan: &OperationPlanRecord,
    source: &OperationPlanRecord,
    record: &RestoreSafetyRequirementRecord,
) -> Result<(), RestoreSafetyPersistenceError> {
    record.validate_plans(plan, source)?;
    read_operation_plan(layout, &plan.digest())?;
    read_operation_plan(source_layout, &source.digest())?;
    let path = layout.root().join("restore-safety-requirement.json");
    let _lock = JournalLock::acquire(&path)?;
    check_size(record)?;
    create_json_durable(&path, record)?;
    Ok(())
}
/// Read bounded exact original requirement under both unchanged retained plans.
///
/// Lost creation replies reconcile through this read; absence never recreates a
/// requirement, grants fresh source/fence authority or replenishes original attempts.
/// # Errors
/// Rejects missing/unsafe/oversized records, changed plans or exact requirement digest mismatch.
pub fn read_restore_safety_requirement(
    layout: &BackupLayoutGuard,
    source_layout: &BackupLayoutGuard,
    plan: &OperationPlanRecord,
    source: &OperationPlanRecord,
    expected: &ArtifactChecksumRecord,
) -> Result<RestoreSafetyRequirementRecord, RestoreSafetyPersistenceError> {
    read_operation_plan(layout, &plan.digest())?;
    read_operation_plan(source_layout, &source.digest())?;
    let path = layout.root().join("restore-safety-requirement.json");
    let _lock = JournalLock::acquire(&path)?;
    let record: RestoreSafetyRequirementRecord =
        read_json(&path, MAX_RESTORE_SAFETY_REQUIREMENT_BYTES)?;
    check_size(&record)?;
    record.validate_plans(plan, source)?;
    if &record.digest() != expected {
        return Err(RestoreSafetyPersistenceError::DigestMismatch);
    }
    Ok(record)
}
fn check_size(record: &RestoreSafetyRequirementRecord) -> Result<(), PersistenceError> {
    super::json::check_json_size(record, MAX_RESTORE_SAFETY_REQUIREMENT_BYTES)
}
/// Typed original restore/source safety persistence denial.
#[derive(Debug, Error)]
pub enum RestoreSafetyPersistenceError {
    /// Exact original retained requirement differs.
    #[error("restore safety requirement digest mismatch")]
    DigestMismatch,
    /// Original source or restore declarations differ.
    #[error(transparent)]
    Requirement(#[from] RestoreSafetyRequirementError),
    /// An exact original plan is not retained under its unchanged layout.
    #[error(transparent)]
    Plan(#[from] OperationPlanPersistenceError),
    /// Journal exclusion failed.
    #[error(transparent)]
    Lock(#[from] JournalLockError),
    /// Bounded JSON/durable IO failed.
    #[error(transparent)]
    Persistence(#[from] PersistenceError),
}

#[cfg(all(test, unix))]
mod tests;
