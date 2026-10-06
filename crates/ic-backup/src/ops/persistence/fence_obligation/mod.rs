//! Bounded no-replace original fence obligations under retained plan/requirement custody.

use super::{
    BackupLayoutGuard, ConsistencyPersistenceError, JournalLock, JournalLockError,
    PersistenceError, RestoreSafetyPersistenceError, create_json_durable,
    read_consistency_requirement, read_json, read_restore_safety_requirement,
};
use crate::model::{
    artifacts::ArtifactChecksumRecord,
    consistency::{ApplicationFenceBinding, ConsistencyRequirementRecord},
    fence_obligation::{FenceObligationError, FenceObligationRecord, MAX_FENCE_OBLIGATION_BYTES},
    operation_plan::OperationPlanRecord,
    restore_safety::RestoreSafetyRequirementRecord,
};
use thiserror::Error;

/// Borrowed original declarations and guarded source custody used at the IO boundary.
///
/// These inputs grant no acquisition, Active fence, spending or release authority.
#[derive(Clone, Copy, Debug)]
pub enum FenceObligationRequirement<'a> {
    /// Exact retained original coordinated capture declaration and chosen fence.
    Capture {
        /// Original requirement, already durably retained with its plan.
        requirement: &'a ConsistencyRequirementRecord,
        /// Exact original integration-retained identity and membership revision.
        fence: &'a ApplicationFenceBinding,
    },
    /// Exact retained original restore/source declaration under both layout guards.
    Restore {
        /// Unchanged source layout exclusion.
        source_layout: &'a BackupLayoutGuard,
        /// Exact original source plan, already retained there.
        source: &'a OperationPlanRecord,
        /// Original fenced restore safety requirement, already durably retained.
        requirement: &'a RestoreSafetyRequirementRecord,
    },
}
impl FenceObligationRequirement<'_> {
    fn validate(
        self,
        layout: &BackupLayoutGuard,
        plan: &OperationPlanRecord,
        record: &FenceObligationRecord,
    ) -> Result<(), FenceObligationPersistenceError> {
        match self {
            Self::Capture { requirement, fence } => {
                record.validate_capture(plan, requirement, fence)?;
                read_consistency_requirement(layout, plan, &requirement.digest())?;
            }
            Self::Restore {
                source_layout,
                source,
                requirement,
            } => {
                record.validate_restore(plan, source, requirement)?;
                read_restore_safety_requirement(
                    layout,
                    source_layout,
                    plan,
                    source,
                    &requirement.digest(),
                )?;
            }
        }
        Ok(())
    }
}
/// Durably publish fixed `fence-obligation.json` without replacing retained obligations.
///
/// Publish before reservation/dispatch of its explicit acquisition operation.
/// Attempt journals separately own every reservation, outcome and reconciliation.
/// This function admits retained original declarations, not executable requests.
/// # Errors
/// Rejects absent/changed originals, inappropriate fence scope, existing/unsafe paths or IO.
pub fn create_fence_obligation(
    layout: &BackupLayoutGuard,
    plan: &OperationPlanRecord,
    requirement: FenceObligationRequirement<'_>,
    record: &FenceObligationRecord,
) -> Result<(), FenceObligationPersistenceError> {
    requirement.validate(layout, plan, record)?;
    let path = layout.root().join("fence-obligation.json");
    let _lock = JournalLock::acquire(&path)?;
    check_size(record)?;
    create_json_durable(&path, record)?;
    Ok(())
}
/// Read the exact bounded retained obligation; absence never recreates or releases it.
///
/// Lost local publication replies reconcile through this read. Recovery must also
/// reopen the exact original acquisition journal; a missing journal is not an
/// unspent allowance. No fresh provider call or fence action occurs here.
/// # Errors
/// Rejects changed originals, another digest, unsafe/missing paths or oversized/invalid bytes.
pub fn read_fence_obligation(
    layout: &BackupLayoutGuard,
    plan: &OperationPlanRecord,
    requirement: FenceObligationRequirement<'_>,
    expected: &ArtifactChecksumRecord,
) -> Result<FenceObligationRecord, FenceObligationPersistenceError> {
    // Validate retained plans before following a location derived from the held layout.
    super::read_operation_plan(layout, &plan.digest())?;
    let path = layout.root().join("fence-obligation.json");
    let _lock = JournalLock::acquire(&path)?;
    let record: FenceObligationRecord = read_json(&path, MAX_FENCE_OBLIGATION_BYTES)?;
    check_size(&record)?;
    requirement.validate(layout, plan, &record)?;
    if &record.digest() != expected {
        return Err(FenceObligationPersistenceError::DigestMismatch);
    }
    Ok(record)
}
fn check_size(record: &FenceObligationRecord) -> Result<(), PersistenceError> {
    super::json::check_json_size(record, MAX_FENCE_OBLIGATION_BYTES)
}
/// Typed denial preserving original obligation bytes and acquisition spending.
#[derive(Debug, Error)]
pub enum FenceObligationPersistenceError {
    /// Retained obligation differs from the original exact expected digest.
    #[error("fence obligation digest mismatch")]
    DigestMismatch,
    /// Exact original fence scope or plan differs.
    #[error(transparent)]
    Obligation(#[from] FenceObligationError),
    /// Original plan cannot be read under its unchanged layout.
    #[error(transparent)]
    Plan(#[from] super::OperationPlanPersistenceError),
    /// Exact original capture requirement cannot be read.
    #[error(transparent)]
    Consistency(#[from] ConsistencyPersistenceError),
    /// Exact original restore/source requirement cannot be read.
    #[error(transparent)]
    Restore(#[from] RestoreSafetyPersistenceError),
    /// Journal exclusion failed.
    #[error(transparent)]
    Lock(#[from] JournalLockError),
    /// Bounded/durable filesystem or JSON operation failed.
    #[error(transparent)]
    Persistence(#[from] PersistenceError),
}

#[cfg(all(test, unix))]
mod tests;
