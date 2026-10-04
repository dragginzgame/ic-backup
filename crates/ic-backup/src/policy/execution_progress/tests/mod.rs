//! Fresh local evidence joins; fixtures attest no IC effects or current authority.

use super::*;
use crate::model::{
    attempt_journal::{
        AttemptJournalRecordError, MutationOutcomeRecord, MutationReceiptRequest,
        ObservationOutcomeRecord, ObservationReceiptRequest,
    },
    effect_graph::{EffectGraphRecord, EffectNodeRecord, EffectNodeRequest},
    operation_plan::{OperationPlanRequest, PlanBudgetRecord, tests::request},
};

fn plan() -> OperationPlanRecord {
    OperationPlanRecord::new(request()).expect("original plan")
}
fn journals(plan: &OperationPlanRecord) -> Vec<AttemptJournalRecord> {
    plan.operations()
        .iter()
        .map(|operation| {
            AttemptJournalRecord::new(
                plan.attempt_authority(operation.operation_sequence())
                    .expect("original authority"),
            )
        })
        .collect()
}
fn project(
    plan: &OperationPlanRecord,
    journals: &[AttemptJournalRecord],
) -> Result<ExecutionProgressView, ExecutionProgressError> {
    let references: Vec<_> = journals.iter().collect();
    progress(&ExecutionProgressRequest {
        plan,
        journals: &references,
    })
}
fn resolve(journal: &mut AttemptJournalRecord, outcome: MutationOutcomeRecord) {
    let attempt = journal.reserve_mutation().expect("local reservation");
    journal
        .record_mutation(MutationReceiptRequest {
            attempt,
            request: journal.authority().binding().request().into(),
            outcome,
            evidence: "12".repeat(32),
        })
        .expect("local caller-qualified fixture receipt");
}

#[test]
fn admits_complete_exact_journals_in_graph_order_without_editing_evidence() {
    let plan = plan();
    let mut journals = journals(&plan);
    journals.reverse();
    let original = serde_json::to_vec(&journals).expect("retained fixture bytes");
    let view = project(&plan, &journals).expect("complete original local evidence");
    assert_eq!(view.intent, plan.digest());
    assert_eq!(view.graph, plan.graph().digest());
    assert_eq!(view.applied_operations, 0);
    assert_eq!(view.operations[0].operation_sequence, 0);
    assert_eq!(view.operations[1].operation_sequence, 7);
    assert_eq!(
        view.operations[0].state,
        OperationProgressState::MutationAvailable
    );
    assert_eq!(
        view.operations[1].state,
        OperationProgressState::AwaitingDependencies
    );
    assert_eq!(view.operations[0].attempts, journals[1].view());
    assert_eq!(view.operations[1].attempts, journals[0].view());
    assert_eq!(
        view.attempts,
        ExecutionAttemptTotalsView {
            mutations_used: 0,
            observations_used: 0,
            mutations_remaining: 3,
            observations_remaining: 2,
        }
    );
    assert_eq!(serde_json::to_vec(&journals).expect("unchanged"), original);
    assert_eq!(project(&plan, &journals).expect("repeat"), view);

    let mut request = request();
    request.budget = PlanBudgetRecord::new(100, 100).expect("unassigned headroom");
    let bigger = OperationPlanRecord::new(request).expect("same assignments");
    let bigger_view = project(&bigger, &self::journals(&bigger)).expect("new exact declarations");
    assert_ne!(bigger_view.intent, view.intent);
    assert_eq!(
        bigger_view.attempts, view.attempts,
        "unused headroom grants no allowance"
    );
}

#[test]
fn rejects_missing_duplicate_unknown_and_every_changed_original_binding() {
    let plan = plan();
    let journals = journals(&plan);
    assert_eq!(
        project(&plan, &[]).expect_err("no implied empty journals"),
        ExecutionProgressError::MissingJournal(0)
    );
    assert_eq!(
        project(&plan, &journals[..1]).expect_err("missing dependent"),
        ExecutionProgressError::MissingJournal(7)
    );
    assert_eq!(
        project(&plan, &[journals[0].clone(), journals[0].clone()]).expect_err("duplicate"),
        ExecutionProgressError::DuplicateJournal(0)
    );
    let references = vec![&journals[0]; MAX_EFFECT_OPERATIONS + 1];
    assert_eq!(
        progress(&ExecutionProgressRequest {
            plan: &plan,
            journals: &references
        })
        .expect_err("bounded input"),
        ExecutionProgressError::TooManyJournals
    );
    for (field, changed) in [
        ("intent", "34".repeat(32)),
        ("network", "56".repeat(32)),
        ("caller", "aaaaa-aa".into()),
        ("target", "aaaaa-aa".into()),
        ("release", "78".repeat(32)),
        ("request", "90".repeat(32)),
    ] {
        let mut value = serde_json::to_value(journals[0].authority()).expect("authority");
        value["binding"][field] = changed.into();
        let changed: AttemptAuthorityRecord =
            serde_json::from_value(value).expect("valid different identity");
        let changed = AttemptJournalRecord::new(changed);
        assert_eq!(
            project(&plan, &[changed, journals[1].clone()]).expect_err(field),
            ExecutionProgressError::AuthorityMismatch(0)
        );
    }
    for kind in ["mutations", "observations"] {
        let mut value = serde_json::to_value(journals[0].authority()).expect("authority");
        value["budget"][kind] = 2.into();
        let changed = AttemptJournalRecord::new(
            serde_json::from_value(value).expect("valid altered allowance"),
        );
        assert_eq!(
            project(&plan, &[changed, journals[1].clone()]).expect_err(kind),
            ExecutionProgressError::AuthorityMismatch(0)
        );
    }
    let mut value = serde_json::to_value(journals[0].authority()).expect("authority");
    value["binding"]["operation_sequence"] = 9.into();
    let changed =
        AttemptJournalRecord::new(serde_json::from_value(value).expect("valid unknown sequence"));
    assert_eq!(
        project(&plan, &[changed, journals[1].clone()]).expect_err("unknown"),
        ExecutionProgressError::UnknownOperation(9)
    );
    let mut value = serde_json::to_value(journals[0].authority()).expect("authority");
    value["binding"]["operation_sequence"] = 7.into();
    let changed =
        AttemptJournalRecord::new(serde_json::from_value(value).expect("known wrong operation"));
    assert_eq!(
        project(&plan, &[journals[0].clone(), changed]).expect_err("sequence/request mismatch"),
        ExecutionProgressError::AuthorityMismatch(7)
    );
}

#[test]
fn rejects_any_attempted_dependent_without_applied_prerequisites() {
    let plan = plan();
    let original = journals(&plan);
    for outcome in [
        None,
        Some(MutationOutcomeRecord::NotApplied),
        Some(MutationOutcomeRecord::Applied),
    ] {
        let mut journals = original.clone();
        if let Some(outcome) = outcome {
            resolve(&mut journals[1], outcome);
        } else {
            journals[1]
                .reserve_mutation()
                .expect("pending local reservation");
        }
        let before = serde_json::to_vec(&journals).expect("evidence");
        assert_eq!(
            project(&plan, &journals).expect_err("premature attempt"),
            ExecutionProgressError::PrematureAttempt {
                operation_sequence: 7,
                prerequisite: 0
            }
        );
        assert_eq!(
            serde_json::to_vec(&journals).expect("unchanged evidence"),
            before
        );
    }
    let mut journals = original;
    resolve(&mut journals[0], MutationOutcomeRecord::NotApplied);
    journals[1]
        .reserve_mutation()
        .expect("dependent reservation");
    assert_eq!(
        project(&plan, &journals).expect_err("NotApplied prerequisite"),
        ExecutionProgressError::PrematureAttempt {
            operation_sequence: 7,
            prerequisite: 0
        }
    );
}

#[test]
fn projects_pending_observations_and_exhaustion_without_blind_retry() {
    let plan = plan();
    let mut journals = journals(&plan);
    let mutation = journals[0]
        .reserve_mutation()
        .expect("consume original allowance");
    let view = project(&plan, &journals).expect("unresolved mutation");
    assert_eq!(
        view.operations[0].state,
        OperationProgressState::MutationUnresolved
    );
    assert_eq!(view.operations[0].attempts.pending_mutation, Some(mutation));
    assert_eq!(view.operations[0].attempts.mutations_remaining, 0);
    let observation_request = "23".repeat(32);
    let observation = journals[0]
        .reserve_observation(mutation, &observation_request)
        .expect("separate original observation allowance");
    let view = project(&plan, &journals).expect("lost observation response stays pending");
    assert_eq!(
        view.operations[0].state,
        OperationProgressState::ObservationUnresolved
    );
    assert_eq!(
        view.operations[0].attempts.pending_observation,
        Some(observation)
    );
    assert_eq!(view.operations[0].attempts.observations_remaining, 0);
    journals[0]
        .record_observation(ObservationReceiptRequest {
            attempt: observation,
            request: observation_request,
            outcome: ObservationOutcomeRecord::Uncertain,
            evidence: "45".repeat(32),
        })
        .expect("qualified settled unresolved fixture");
    let view = project(&plan, &journals).expect("unresolved exhausted accounting");
    assert_eq!(
        view.operations[0].state,
        OperationProgressState::ReconciliationExhausted
    );
    assert_eq!(view.operations[0].attempts.pending_mutation, Some(mutation));
    assert_eq!(view.operations[0].attempts.pending_observation, None);
    assert_eq!(view.attempts.mutations_used, 1);
    assert_eq!(view.attempts.observations_used, 1);
    assert_eq!(view.attempts.mutations_remaining, 2);
    assert_eq!(view.attempts.observations_remaining, 1);
    assert!(
        matches!(journals[0].reserve_mutation(), Err(AttemptJournalRecordError::MutationPending { attempt }) if attempt == mutation)
    );
    let restored: Vec<AttemptJournalRecord> =
        serde_json::from_slice(&serde_json::to_vec(&journals).expect("evidence bytes"))
            .expect("local replay");
    assert_eq!(
        project(&plan, &restored).expect("original reopened accounting"),
        view
    );
}

#[test]
fn derives_applied_and_available_progress_from_actual_local_journal_receipts() {
    let plan = plan();
    let mut journals = journals(&plan);
    resolve(&mut journals[0], MutationOutcomeRecord::Applied);
    let view = project(&plan, &journals).expect("applied prerequisite");
    assert_eq!(view.applied_operations, 1);
    assert_eq!(view.operations[0].state, OperationProgressState::Applied);
    assert_eq!(
        view.operations[1].state,
        OperationProgressState::MutationAvailable
    );
    resolve(&mut journals[1], MutationOutcomeRecord::NotApplied);
    let view = project(&plan, &journals).expect("spent not-applied attempt");
    assert_eq!(
        view.operations[1].state,
        OperationProgressState::MutationAvailable
    );
    assert_eq!(view.operations[1].attempts.mutations_remaining, 1);
    resolve(&mut journals[1], MutationOutcomeRecord::Applied);
    let view = project(&plan, &journals).expect("all operation receipts applied");
    assert_eq!(view.applied_operations, 2);
    assert!(
        view.operations
            .iter()
            .all(|op| op.state == OperationProgressState::Applied)
    );
    assert_eq!(view.attempts.mutations_used, 3);
    assert_eq!(view.attempts.mutations_remaining, 0);
    assert_eq!(
        view.attempts.observations_remaining, 2,
        "unused original allowance is not a dispatch permit"
    );
    assert!(matches!(
        journals[1].reserve_mutation(),
        Err(AttemptJournalRecordError::AlreadyApplied)
    ));
}

#[test]
fn distinguishes_settled_not_applied_from_unresolved_without_replenishing_limits() {
    let plan = plan();
    for outcome in [
        ObservationOutcomeRecord::Applied,
        ObservationOutcomeRecord::NotApplied,
    ] {
        let mut journals = journals(&plan);
        let mutation = journals[0].reserve_mutation().expect("mutation");
        let request = "67".repeat(32);
        let observation = journals[0]
            .reserve_observation(mutation, &request)
            .expect("observation");
        journals[0]
            .record_observation(ObservationReceiptRequest {
                attempt: observation,
                request,
                outcome,
                evidence: "89".repeat(32),
            })
            .expect("qualified fixture settlement");
        let view = project(&plan, &journals).expect("settled exact observation");
        assert_eq!(view.operations[0].attempts.pending_mutation, None);
        assert_eq!(view.operations[0].attempts.pending_observation, None);
        assert_eq!(view.operations[0].attempts.mutations_remaining, 0);
        assert_eq!(view.operations[0].attempts.observations_remaining, 0);
        let expected = if outcome == ObservationOutcomeRecord::Applied {
            OperationProgressState::Applied
        } else {
            OperationProgressState::MutationExhausted
        };
        assert_eq!(view.operations[0].state, expected);
    }
}

#[test]
fn preserves_explicit_reverse_dependencies_and_zero_allowance_states() {
    let mut request = request();
    request.graph = EffectGraphRecord::new(vec![
        EffectNodeRecord::new(EffectNodeRequest {
            operation_sequence: 0,
            depends_on: vec![7],
        })
        .expect("explicit later identity dependency"),
        EffectNodeRecord::new(EffectNodeRequest {
            operation_sequence: 7,
            depends_on: vec![],
        })
        .expect("first prerequisite"),
    ])
    .expect("acyclic reverse numeric order");
    let plan = OperationPlanRecord::new(request).expect("exact reverse declared plan");
    let mut journals = journals(&plan);
    let view = project(&plan, &journals).expect("graph order");
    assert_eq!(view.operations[0].operation_sequence, 7);
    assert_eq!(view.operations[1].operation_sequence, 0);
    assert_eq!(
        view.operations[1].state,
        OperationProgressState::AwaitingDependencies
    );
    resolve(&mut journals[1], MutationOutcomeRecord::Applied);
    assert_eq!(
        project(&plan, &journals)
            .expect("reverse prerequisite applied")
            .operations[1]
            .state,
        OperationProgressState::MutationAvailable
    );

    let mut request: OperationPlanRequest = self::request();
    request.operations = request
        .operations
        .into_iter()
        .map(|op| {
            let mut value = serde_json::to_value(op).expect("fixture operation");
            value["budget"]["mutations"] = 0.into();
            value["budget"]["observations"] = 0.into();
            serde_json::from_value(value).expect("zero original allowance")
        })
        .collect();
    request.budget = PlanBudgetRecord::new(0, 0).expect("zero aggregate");
    let plan = OperationPlanRecord::new(request).expect("zero declaration");
    let view = project(&plan, &self::journals(&plan)).expect("no fake completion for zero budget");
    assert_eq!(
        view.operations[0].state,
        OperationProgressState::MutationExhausted
    );
    assert_eq!(
        view.operations[1].state,
        OperationProgressState::AwaitingDependencies
    );
    assert_eq!(view.applied_operations, 0);
}

#[test]
fn admits_maximum_journal_count_and_exact_original_aggregate_allowance() {
    use crate::model::attempt_journal::{AttemptBudgetRecord, OperationBindingRequest};
    for (count, mutations, ceiling) in [(MAX_EFFECT_OPERATIONS, 0, 0), (64, 1024, 65536)] {
        let mut request = request();
        let template = serde_json::to_value(&request.operations[0]).expect("operation fixture");
        let mut nodes = Vec::with_capacity(count);
        let mut operations = Vec::with_capacity(count);
        for index in 0..count {
            let sequence = u64::try_from(index).expect("bounded operation count");
            nodes.push(
                EffectNodeRecord::new(EffectNodeRequest {
                    operation_sequence: sequence,
                    depends_on: vec![],
                })
                .expect("independent node"),
            );
            let mut operation = template.clone();
            operation["operation_sequence"] = sequence.into();
            operation["budget"]["mutations"] = mutations.into();
            operation["budget"]["observations"] = 0.into();
            operations.push(serde_json::from_value(operation).expect("exact operation"));
        }
        request.graph = EffectGraphRecord::new(nodes).expect("bounded graph");
        request.operations = operations;
        request.budget = PlanBudgetRecord::new(ceiling, 0).expect("aggregate limit");
        let plan = OperationPlanRecord::new(request).expect("original declaration");
        let intent = plan.digest();
        // Build exact validated fixture authorities using one original intent hash,
        // avoiding repeated full-plan hashing in the maximum-count test setup.
        let journals: Vec<_> = plan
            .operations()
            .iter()
            .rev()
            .map(|operation| {
                let binding = OperationBindingRecord::new(&OperationBindingRequest {
                    intent: intent.hash().into(),
                    operation_sequence: operation.operation_sequence(),
                    network: plan.context().network().into(),
                    caller: plan.context().caller().into(),
                    target: operation.target().into(),
                    release: plan.context().release().into(),
                    request: operation.request().into(),
                })
                .expect("exact original binding");
                AttemptJournalRecord::new(AttemptAuthorityRecord::new(
                    binding,
                    AttemptBudgetRecord::new(mutations, 0).expect("original limits"),
                ))
            })
            .collect();
        let view = project(&plan, &journals).expect("maximum exact coverage");
        assert_eq!(view.operations.len(), count);
        assert_eq!(view.operations[0].operation_sequence, 0);
        assert_eq!(
            view.operations
                .last()
                .expect("last operation")
                .operation_sequence,
            u64::try_from(count - 1).expect("sequence")
        );
        assert_eq!(view.attempts.mutations_remaining, ceiling);
        assert_eq!(view.attempts.mutations_used, 0);
        assert_eq!(view.attempts.observations_used, 0);
        assert_eq!(view.attempts.observations_remaining, 0);
        assert_eq!(view.applied_operations, 0);
    }
}
