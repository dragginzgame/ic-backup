//! Original retained plan/journal admission for local progress and guarded reservations.

use super::{AttemptJournalError, BackupLayoutGuard, OperationPlanPersistenceError};
use crate::{
    model::{
        artifacts::ArtifactChecksumRecord, attempt_journal::AttemptJournalRecord,
        operation_plan::OperationPlanError,
    },
    policy::execution_progress::{
        ExecutionProgressError, ExecutionProgressView, progress_with_journals,
    },
};
use thiserror::Error;

/// Read complete original retained execution evidence without remote effects or artifact reads.
///
/// Requires the exact persisted plan and every original attempt journal. Missing
/// journals never supply zero consumption. Journal locks are taken sequentially;
/// hold no attempt guards when calling this function. The layout retains cooperating
/// exclusion, while noncooperating byte custody remains integration-owned.
/// This grants no fresh permission, executable call, authenticated receipt or release.
/// # Errors
/// Rejects missing/unsafe/changed originals, contention and invalid retained causality.
pub fn read_execution_progress(
    layout: &BackupLayoutGuard,
    expected_plan: &ArtifactChecksumRecord,
) -> Result<ExecutionProgressView, ExecutionProgressPersistenceError> {
    read_with_current(layout, expected_plan, None)
}

pub(super) fn read_with_current(
    layout: &BackupLayoutGuard,
    expected_plan: &ArtifactChecksumRecord,
    current: Option<&AttemptJournalRecord>,
) -> Result<ExecutionProgressView, ExecutionProgressPersistenceError> {
    let (plan, journals) = super::attempt_journal::read_original_plan_journals::<
        ExecutionProgressPersistenceError,
    >(layout, expected_plan, current)?;
    Ok(progress_with_journals(&plan, journals.iter())?.0)
}

/// Typed local evidence or reservation rejection; no failure resets original spending.
#[derive(Debug, Error)]
pub enum ExecutionProgressPersistenceError {
    /// Exact original persisted plan cannot be admitted.
    #[error(transparent)]
    Plan(#[from] OperationPlanPersistenceError),
    /// Original bulk authority derivation failed.
    #[error(transparent)]
    Authority(#[from] OperationPlanError),
    /// Original journal admission or durable reservation failed.
    #[error(transparent)]
    Journal(#[from] AttemptJournalError),
    /// Complete retained accounting/causality cannot be projected.
    #[error(transparent)]
    Policy(#[from] ExecutionProgressError),
    /// The selected operation has a prerequisite without retained Applied evidence.
    #[error("operation {0} has unapplied prerequisites")]
    DependenciesUnapplied(u64),
}

#[cfg(all(test, unix))]
mod tests;
