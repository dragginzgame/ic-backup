//! Exact complete Applied local journal admission; no terminal or release authority.

use crate::{
    model::{
        attempt_journal::AttemptJournalRecord, execution_settlement::ExecutionSettlementRecord,
        operation_plan::OperationPlanRecord,
    },
    policy::execution_progress::{
        self, ExecutionProgressError, ExecutionProgressRequest, ExecutionProgressView,
        OperationProgressState,
    },
};
use std::collections::BTreeMap;
use thiserror::Error;

/// Admit the original complete causal journal set and match every retained history.
///
/// The canonical progress owner checks exact coverage/context/operation/limits and
/// retained prerequisite evidence; no missing input means unused allowance. Every
/// operation must be Applied, even when remaining allowances are nonzero. Exact
/// full history includes negative/uncertain receipts and consumed attempts, not
/// just the final progress shape. This performs no IO/serialization/transitions or
/// remote observations. Receipt authentication, dispatch chronology, artifacts,
/// actual application safety and command quiescence remain separately qualified.
/// # Errors
/// Rejects changed plan, incomplete/invalid/unsettled journals or different exact histories.
pub fn validate(
    plan: &OperationPlanRecord,
    journals: &[&AttemptJournalRecord],
    settlement: &ExecutionSettlementRecord,
) -> Result<ExecutionProgressView, ExecutionSettlementPolicyError> {
    if &plan.digest() != settlement.plan_intent() {
        return Err(ExecutionSettlementPolicyError::IntentMismatch);
    }
    let view = execution_progress::progress(&ExecutionProgressRequest { plan, journals })?;
    if let Some(operation) = view
        .operations
        .iter()
        .find(|operation| operation.state != OperationProgressState::Applied)
    {
        return Err(ExecutionSettlementPolicyError::UnsettledOperation(
            operation.operation_sequence,
        ));
    }
    if settlement.journals().len() != plan.operations().len() {
        return Err(ExecutionSettlementPolicyError::JournalSetMismatch);
    }
    let actual: BTreeMap<_, _> = journals
        .iter()
        .map(|journal| (journal.authority().binding().operation_sequence(), *journal))
        .collect();
    for (row, operation) in settlement.journals().iter().zip(plan.operations()) {
        if row.operation_sequence() != operation.operation_sequence() {
            return Err(ExecutionSettlementPolicyError::JournalSetMismatch);
        }
        let journal = actual
            .get(&row.operation_sequence())
            .ok_or(ExecutionSettlementPolicyError::JournalSetMismatch)?;
        if &journal.digest() != row.history() {
            return Err(ExecutionSettlementPolicyError::HistoryMismatch(
                row.operation_sequence(),
            ));
        }
    }
    Ok(view)
}
/// Typed local settlement denial; no error dispatches, refunds or releases evidence.
#[derive(Debug, Eq, Error, PartialEq)]
pub enum ExecutionSettlementPolicyError {
    /// Checkpoint names another full original plan.
    #[error("execution settlement original plan mismatch")]
    IntentMismatch,
    /// Original complete evidence/causality failed canonical admission.
    #[error(transparent)]
    Progress(#[from] ExecutionProgressError),
    /// Operation has no retained Applied outcome.
    #[error("execution settlement operation {0} is not applied")]
    UnsettledOperation(u64),
    /// Checkpoint rows do not cover exactly the original operation set.
    #[error("execution settlement journal set mismatch")]
    JournalSetMismatch,
    /// Same final progress hides changed exact reservations/receipt evidence.
    #[error("execution settlement journal {0} history mismatch")]
    HistoryMismatch(u64),
}
#[cfg(test)]
mod tests;
