//! Immutable original all-Applied journal checkpoint publication and local replay.

use super::json::check_json_size;
use super::{
    AttemptJournalError, BackupLayoutGuard, JournalLock, JournalLockError,
    OperationPlanPersistenceError, PersistenceError, create_json_durable, read_json,
};
use crate::{
    model::{
        artifacts::ArtifactChecksumRecord,
        execution_settlement::{
            ExecutionSettlementError, ExecutionSettlementJournalRecord, ExecutionSettlementRecord,
            MAX_EXECUTION_SETTLEMENT_BYTES,
        },
    },
    policy::execution_settlement::{ExecutionSettlementPolicyError, validate},
};
use std::{io, path::Path};
use thiserror::Error;

/// Derive and publish an all-Applied checkpoint from the exact retained original journals.
///
/// Reads every original journal sequentially, then delegates immutable publication
/// to [`create_execution_settlement`], which rechecks the complete original histories.
/// Hold no journal guards at admission. This creates no receipts, spending allowance,
/// product terminal proof or release permission. An occupied checkpoint is never replaced;
/// use [`read_execution_settlement`] with its retained identity for local replay.
/// # Errors
/// Rejects missing, held, changed or unsettled originals and publication failures.
pub fn checkpoint_execution_settlement(
    layout: &BackupLayoutGuard,
    expected_plan: &ArtifactChecksumRecord,
) -> Result<ExecutionSettlementRecord, ExecutionSettlementCheckpointError> {
    let record = derive_execution_settlement(layout, expected_plan)?;
    create_execution_settlement(layout, &record)?;
    Ok(record)
}

/// Reconcile a lost checkpoint response from complete original journals and the
/// existing immutable checkpoint, without publishing or changing any evidence.
///
/// The derived digest is checked against the retained checkpoint, then its full
/// original histories are admitted again by the existing reader. This local
/// sequential replay supplies no authenticated receipt, terminal proof or release.
/// # Errors
/// Rejects absent/changed checkpoints, missing/held/unsettled journals and drift.
pub fn replay_execution_settlement(
    layout: &BackupLayoutGuard,
    expected_plan: &ArtifactChecksumRecord,
) -> Result<ExecutionSettlementRecord, ExecutionSettlementCheckpointError> {
    let derived = derive_execution_settlement(layout, expected_plan)?;
    Ok(read_execution_settlement(
        layout,
        expected_plan,
        &derived.digest(),
    )?)
}

/// Complete or replay the exact local checkpoint after an uncertain publication response.
///
/// Derive the expected identity from the complete retained original journals first.
/// An existing exact checkpoint is read without replacement. Only a genuinely
/// absent checkpoint may be created; changed or unsafe entries refuse. A new
/// publication is read back through the same exact-original replay owner.
/// This performs no provider call, receipt transition, refund, terminal admission
/// or reference release. Hold no attempt guards on entry.
/// # Errors
/// Rejects incomplete/held originals, changed or unsafe occupied checkpoints and
/// local publication/readback failures while retaining every original record.
pub fn reconcile_execution_settlement(
    layout: &BackupLayoutGuard,
    expected_plan: &ArtifactChecksumRecord,
) -> Result<ExecutionSettlementRecord, ExecutionSettlementCheckpointError> {
    let derived = derive_execution_settlement(layout, expected_plan)?;
    match read_execution_settlement(layout, expected_plan, &derived.digest()) {
        Ok(existing) => Ok(existing),
        Err(ExecutionSettlementPersistenceError::Persistence(PersistenceError::Io(error)))
            if error.kind() == io::ErrorKind::NotFound =>
        {
            create_execution_settlement(layout, &derived)?;
            Ok(read_execution_settlement(
                layout,
                expected_plan,
                &derived.digest(),
            )?)
        }
        Err(error) => Err(error.into()),
    }
}

fn derive_execution_settlement(
    layout: &BackupLayoutGuard,
    expected_plan: &ArtifactChecksumRecord,
) -> Result<ExecutionSettlementRecord, ExecutionSettlementCheckpointError> {
    let (plan, journals) = super::attempt_journal::read_original_plan_journals::<
        ExecutionSettlementPersistenceError,
    >(layout, expected_plan, None)?;
    let rows = journals
        .iter()
        .map(ExecutionSettlementJournalRecord::from_journal)
        .collect();
    Ok(ExecutionSettlementRecord::new(plan.digest(), rows)?)
}

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
    let (plan, journals) = super::attempt_journal::read_original_plan_journals::<
        ExecutionSettlementPersistenceError,
    >(layout, record.plan_intent(), None)?;
    let references: Vec<_> = journals.iter().collect();
    validate(&plan, &references, record)?;
    layout.check_root()?;
    Ok(())
}
/// Failure deriving, publishing or replaying a checkpoint; originals remain retained.
#[derive(Debug, Error)]
pub enum ExecutionSettlementCheckpointError {
    /// Original local evidence or immutable publication cannot be admitted.
    #[error(transparent)]
    Persistence(#[from] ExecutionSettlementPersistenceError),
    /// The derived checkpoint cannot satisfy the existing bounded record schema.
    #[error(transparent)]
    Record(#[from] ExecutionSettlementError),
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
