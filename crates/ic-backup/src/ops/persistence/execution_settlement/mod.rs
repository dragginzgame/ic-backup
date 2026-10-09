//! Immutable original all-Applied journal checkpoint publication and local replay.

use super::json::check_json_size;
use super::{
    AttemptJournalError, BackupLayoutGuard, JournalLock, JournalLockError,
    OperationPlanPersistenceError, PersistenceError, create_json_durable, read_json,
    read_operation_plan,
};
use crate::{
    model::{
        artifacts::ArtifactChecksumRecord,
        execution_settlement::{ExecutionSettlementRecord, MAX_EXECUTION_SETTLEMENT_BYTES},
    },
    policy::execution_settlement::{ExecutionSettlementPolicyError, validate},
};
use std::path::Path;
use thiserror::Error;

/// Publish fixed `execution-settlement.json` under original retained plan/journal evidence.
///
/// The exclusive layout owns cooperating write exclusion. Journal locks are acquired
/// sequentially in canonical sequence order, bounding descriptor use; every admitted
/// Applied journal rejects further owner transitions. Hold no journal guards when
/// invoking this operation. Noncooperating byte custody and authentic receipts remain
/// integration-owned. This creates no journal, budget, product terminal or release permit.
/// # Errors
/// Rejects missing/unsafe/oversized originals, contention, unsettled/changed evidence or existing publication.
pub fn create_execution_settlement(
    layout: &BackupLayoutGuard,
    record: &ExecutionSettlementRecord,
) -> Result<(), ExecutionSettlementPersistenceError> {
    create_with(layout, record, create_json_durable)
}
fn create_with(
    layout: &BackupLayoutGuard,
    record: &ExecutionSettlementRecord,
    writer: impl FnOnce(&Path, &ExecutionSettlementRecord) -> Result<(), PersistenceError>,
) -> Result<(), ExecutionSettlementPersistenceError> {
    layout.check_root()?;
    let path = layout.root().join("execution-settlement.json");
    let _lock = JournalLock::acquire(&path)?;
    check_json_size(record, MAX_EXECUTION_SETTLEMENT_BYTES)?;
    validate_retained(layout, record)?;
    writer(&path, record)?;
    Ok(())
}
/// Reopen exact checkpoint and validate complete retained original evidence using local IO only.
///
/// A lost publication reply can be reconciled without rewriting evidence or observing
/// remote state. This is original journal settlement replay, not fresh verification
/// of artifacts/application state, full run completion or release admission.
/// # Errors
/// Rejects changed expected identity, absent/invalid/unsafe evidence, contention or journal drift.
pub fn read_execution_settlement(
    layout: &BackupLayoutGuard,
    expected_plan: &ArtifactChecksumRecord,
    expected: &ArtifactChecksumRecord,
) -> Result<ExecutionSettlementRecord, ExecutionSettlementPersistenceError> {
    layout.check_root()?;
    let path = layout.root().join("execution-settlement.json");
    let _lock = JournalLock::acquire(&path)?;
    let record: ExecutionSettlementRecord = read_json(&path, MAX_EXECUTION_SETTLEMENT_BYTES)?;
    check_json_size(&record, MAX_EXECUTION_SETTLEMENT_BYTES)?;
    if record.plan_intent() != expected_plan || &record.digest() != expected {
        return Err(ExecutionSettlementPersistenceError::DigestMismatch);
    }
    validate_retained(layout, &record)?;
    Ok(record)
}
fn validate_retained(
    layout: &BackupLayoutGuard,
    record: &ExecutionSettlementRecord,
) -> Result<(), ExecutionSettlementPersistenceError> {
    let plan = read_operation_plan(layout, record.plan_intent())?;
    let authorities = plan.attempt_authorities()?;
    let journals = super::attempt_journal::read_original_journals(layout, &authorities, None)?;
    let references: Vec<_> = journals.iter().collect();
    validate(&plan, &references, record)?;
    layout.check_root()?;
    Ok(())
}
/// Typed immutable publication/replay failure, preserving all original journals and obligations.
#[derive(Debug, Error)]
pub enum ExecutionSettlementPersistenceError {
    /// Expected original plan/checkpoint identity differs.
    #[error("execution settlement digest mismatch")]
    DigestMismatch,
    /// Original retained plan cannot be admitted.
    #[error(transparent)]
    Plan(#[from] OperationPlanPersistenceError),
    /// Original plan authority derivation failed.
    #[error(transparent)]
    Authority(#[from] crate::model::operation_plan::OperationPlanError),
    /// An original journal is absent, unsafe, mismatched or held by another owner.
    #[error(transparent)]
    Journal(#[from] AttemptJournalError),
    /// Complete original settlement evidence failed pure policy.
    #[error(transparent)]
    Policy(#[from] ExecutionSettlementPolicyError),
    /// Publication/replay checkpoint lock failed.
    #[error(transparent)]
    Lock(#[from] JournalLockError),
    /// Bounded local IO or canonical encoding failed.
    #[error(transparent)]
    Persistence(#[from] PersistenceError),
}
#[cfg(all(test, unix))]
mod tests;
