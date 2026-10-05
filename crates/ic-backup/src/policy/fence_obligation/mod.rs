//! Read retained acquisition spending under an exact original fence obligation.

use crate::model::{
    attempt_journal::{AttemptJournalRecord, AttemptJournalView},
    fence_obligation::{FenceObligationError, FenceObligationRecord},
    operation_plan::{OperationPlanError, OperationPlanRecord},
};
use thiserror::Error;

/// Join the original obligation to its exact acquisition journal without IO or effects.
///
/// Missing journals are not accepted as zero consumption. Applied is a retained
/// acquisition receipt projection, never current Active custody or permission to
/// release. A pending observation remains pending even when allowances are exhausted.
/// Full dependency admission remains owned by `execution_progress`; this view does
/// not prove prerequisite completion, chronology, authenticity or dispatch safety.
/// Original requirement/fence matching is separately admitted by the record and
/// guarded persistence boundary; this projection checks plan and journal identity.
/// # Errors
/// Rejects another plan, operation, context, request or original budget.
pub fn acquisition_progress(
    plan: &OperationPlanRecord,
    obligation: &FenceObligationRecord,
    journal: &AttemptJournalRecord,
) -> Result<AttemptJournalView, FenceObligationProgressError> {
    obligation.validate_plan(plan)?;
    let original = plan.attempt_authority(obligation.acquisition_operation())?;
    if journal.authority() != &original {
        return Err(FenceObligationProgressError::AuthorityMismatch);
    }
    Ok(journal.view())
}

/// Typed denial of an original acquisition spending projection.
#[derive(Debug, Error)]
pub enum FenceObligationProgressError {
    /// Journal does not retain the exact original operation and allowances.
    #[error("fence acquisition original journal authority mismatch")]
    AuthorityMismatch,
    /// Obligation does not bind the original plan.
    #[error(transparent)]
    Obligation(#[from] FenceObligationError),
    /// Original authority cannot be derived.
    #[error(transparent)]
    Plan(#[from] OperationPlanError),
}

#[cfg(test)]
mod tests;
