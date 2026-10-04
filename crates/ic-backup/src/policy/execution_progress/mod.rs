//! Pure original-plan progress from exact retained journals; no scheduling or dispatch.

use crate::model::{
    artifacts::ArtifactChecksumRecord,
    attempt_journal::{
        AttemptAuthorityRecord, AttemptJournalRecord, AttemptJournalView, OperationBindingRecord,
    },
    effect_graph::MAX_EFFECT_OPERATIONS,
    operation_plan::{OperationPlanRecord, PlanContextRecord, PlannedOperationRecord},
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

/// Passive complete journal set supplied by the local evidence-owning caller.
#[derive(Clone, Debug)]
pub struct ExecutionProgressRequest<'a> {
    /// Original immutable declaration; its hash binds every journal's intent.
    pub plan: &'a OperationPlanRecord,
    /// Exactly one validated retained journal per declared operation, in any order.
    pub journals: &'a [&'a AttemptJournalRecord],
}

/// Derived local attempt condition, without fresh permission or backend qualification.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationProgressState {
    /// At least one declared prerequisite lacks retained Applied evidence.
    AwaitingDependencies,
    /// Prerequisites are Applied and original mutation allowance remains.
    MutationAvailable,
    /// No unresolved mutation or Applied receipt, and mutation allowance is exhausted.
    MutationExhausted,
    /// A mutation is unresolved and original reconciliation allowance remains.
    MutationUnresolved,
    /// An observation reservation remains unresolved, even if its allowance is exhausted.
    ObservationUnresolved,
    /// A mutation remains unresolved with no pending observation or remaining observation allowance.
    ReconciliationExhausted,
    /// This operation has retained integration-qualified Applied evidence.
    Applied,
}

/// Read-only local progress for one exact operation in graph planning order.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OperationProgressView {
    /// Original opaque operation identity, never an array index.
    pub operation_sequence: u64,
    /// Derived local condition; none of these values authorize dispatch.
    pub state: OperationProgressState,
    /// Exact retained accounting, authority digest and unresolved attempt identities.
    pub attempts: AttemptJournalView,
}

/// Sums over original assigned allowances only; unused plan headroom is excluded.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct ExecutionAttemptTotalsView {
    /// Mutation reservations consumed across all original journals, without refunds.
    pub mutations_used: u32,
    /// Observation reservations consumed across all original journals, without refunds.
    pub observations_used: u32,
    /// Sum of remaining originally assigned mutation allowances.
    pub mutations_remaining: u32,
    /// Sum of remaining originally assigned observation allowances.
    pub observations_remaining: u32,
}

/// Read-only projection bound to the complete original declaration and retained journals.
///
/// Applied evidence is caller-qualified and does not prove full run completion,
/// current IC state, artifact durability, application safety or reference release.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ExecutionProgressView {
    /// Full canonical original plan intent.
    pub intent: ArtifactChecksumRecord,
    /// Original explicit dependency graph digest.
    pub graph: ArtifactChecksumRecord,
    /// Number of operations with retained Applied receipts, not a terminal proof.
    pub applied_operations: usize,
    /// Derived totals over the original per-operation ceilings.
    pub attempts: ExecutionAttemptTotalsView,
    /// Exact journal conditions in deterministic graph planning order.
    pub operations: Vec<OperationProgressView>,
}

/// Admit exact complete journal coverage and project causal retained local progress.
///
/// Missing journals are errors, never fresh zero-consumption declarations. Any
/// attempted operation requires Applied evidence for every declared prerequisite.
/// This validates retained causality, not cross-journal dispatch chronology: no
/// remote observations, authority probes, writes or scheduling occur.
///
/// # Errors
/// Rejects excessive, duplicate, unknown, missing or mismatched journals, premature
/// attempts and accounting overflow. Unresolved/exhausted states remain visible.
pub fn progress(
    request: &ExecutionProgressRequest<'_>,
) -> Result<ExecutionProgressView, ExecutionProgressError> {
    if request.journals.len() > MAX_EFFECT_OPERATIONS {
        return Err(ExecutionProgressError::TooManyJournals);
    }
    let plan = request.plan;
    let intent = plan.digest();
    let mut journals = BTreeMap::new();
    for journal in request.journals {
        let sequence = journal.authority().binding().operation_sequence();
        let operation = plan
            .operation(sequence)
            .map_err(|_| ExecutionProgressError::UnknownOperation(sequence))?;
        if journals.contains_key(&sequence) {
            return Err(ExecutionProgressError::DuplicateJournal(sequence));
        }
        let original = OriginalJournalBinding {
            intent: &intent,
            context: plan.context(),
            operation,
        };
        if !original.matches(journal.authority()) {
            return Err(ExecutionProgressError::AuthorityMismatch(sequence));
        }
        journals.insert(sequence, journal.view());
    }
    for operation in plan.operations() {
        if !journals.contains_key(&operation.operation_sequence()) {
            return Err(ExecutionProgressError::MissingJournal(
                operation.operation_sequence(),
            ));
        }
    }
    let applied: BTreeSet<_> = journals
        .iter()
        .filter_map(|(sequence, attempts)| attempts.applied.then_some(*sequence))
        .collect();
    let mut totals = ExecutionAttemptTotalsView {
        mutations_used: 0,
        observations_used: 0,
        mutations_remaining: 0,
        observations_remaining: 0,
    };
    let mut operations = Vec::with_capacity(journals.len());
    for node in plan.graph().ordered_nodes() {
        let sequence = node.operation_sequence();
        // Closed graph/table identity and complete coverage were admitted above.
        let attempts = journals
            .remove(&sequence)
            .ok_or(ExecutionProgressError::MissingJournal(sequence))?;
        let unmet = node
            .depends_on()
            .iter()
            .find(|dependency| !applied.contains(dependency));
        if attempts.mutations_used != 0
            && let Some(prerequisite) = unmet
        {
            return Err(ExecutionProgressError::PrematureAttempt {
                operation_sequence: sequence,
                prerequisite: *prerequisite,
            });
        }
        totals.add(&attempts)?;
        operations.push(OperationProgressView {
            operation_sequence: sequence,
            state: condition(&attempts, unmet.is_some()),
            attempts,
        });
    }
    Ok(ExecutionProgressView {
        intent,
        graph: plan.graph().digest(),
        applied_operations: applied.len(),
        attempts: totals,
        operations,
    })
}

struct OriginalJournalBinding<'a> {
    intent: &'a ArtifactChecksumRecord,
    context: &'a PlanContextRecord,
    operation: &'a PlannedOperationRecord,
}
impl OriginalJournalBinding<'_> {
    fn matches(&self, authority: &AttemptAuthorityRecord) -> bool {
        self.identity_matches(authority.binding())
            && self.context_matches(authority.binding())
            && authority.budget() == self.operation.budget()
    }
    fn identity_matches(&self, binding: &OperationBindingRecord) -> bool {
        binding.intent() == self.intent.hash()
            && binding.operation_sequence() == self.operation.operation_sequence()
            && binding.target() == self.operation.target()
            && binding.request() == self.operation.request()
    }
    fn context_matches(&self, binding: &OperationBindingRecord) -> bool {
        binding.network() == self.context.network()
            && binding.caller() == self.context.caller()
            && binding.release() == self.context.release()
    }
}
impl ExecutionAttemptTotalsView {
    fn add(&mut self, attempts: &AttemptJournalView) -> Result<(), ExecutionProgressError> {
        self.mutations_used = sum(self.mutations_used, attempts.mutations_used)?;
        self.observations_used = sum(self.observations_used, attempts.observations_used)?;
        self.mutations_remaining = sum(self.mutations_remaining, attempts.mutations_remaining)?;
        self.observations_remaining =
            sum(self.observations_remaining, attempts.observations_remaining)?;
        Ok(())
    }
}
fn sum(left: u32, right: u32) -> Result<u32, ExecutionProgressError> {
    left.checked_add(right)
        .ok_or(ExecutionProgressError::AccountingOverflow)
}
fn condition(attempts: &AttemptJournalView, unmet_dependencies: bool) -> OperationProgressState {
    if attempts.applied {
        OperationProgressState::Applied
    } else if attempts.pending_observation.is_some() {
        OperationProgressState::ObservationUnresolved
    } else if attempts.pending_mutation.is_some() {
        if attempts.observations_remaining == 0 {
            OperationProgressState::ReconciliationExhausted
        } else {
            OperationProgressState::MutationUnresolved
        }
    } else if unmet_dependencies {
        OperationProgressState::AwaitingDependencies
    } else if attempts.mutations_remaining == 0 {
        OperationProgressState::MutationExhausted
    } else {
        OperationProgressState::MutationAvailable
    }
}

/// Typed original-plan journal admission, retained causality or accounting failure.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum ExecutionProgressError {
    /// The journal input exceeds the original bounded operation count.
    #[error("execution progress exceeds {MAX_EFFECT_OPERATIONS} journals")]
    TooManyJournals,
    /// Multiple supplied journals claim the same original operation identity.
    #[error("duplicate journal for operation {0}")]
    DuplicateJournal(u64),
    /// A supplied journal's operation is absent from the original plan.
    #[error("journal operation {0} is absent from the original plan")]
    UnknownOperation(u64),
    /// Retained evidence for an original operation was not supplied.
    #[error("missing original journal for operation {0}")]
    MissingJournal(u64),
    /// Intent, context, target, request or original allowance differs from the plan.
    #[error("original journal authority mismatch for operation {0}")]
    AuthorityMismatch(u64),
    /// A reservation exists while a declared prerequisite lacks Applied evidence.
    #[error(
        "operation {operation_sequence} was attempted without applied prerequisite {prerequisite}"
    )]
    PrematureAttempt {
        /// Operation with retained consumed mutation allowance.
        operation_sequence: u64,
        /// Exact declared prerequisite lacking retained Applied evidence.
        prerequisite: u64,
    },
    /// Derived accounting could not be represented without overflow.
    #[error("execution attempt accounting overflow")]
    AccountingOverflow,
}

#[cfg(test)]
mod tests;
