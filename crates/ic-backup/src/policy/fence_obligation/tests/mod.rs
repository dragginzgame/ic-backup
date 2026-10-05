use super::*;
use crate::{
    model::{
        attempt_journal::{ObservationOutcomeRecord, ObservationReceiptRequest},
        consistency::*,
    },
    test_support::membership::{hash, plan},
};
use serde_json::json;

fn obligation(plan: &OperationPlanRecord) -> FenceObligationRecord {
    FenceObligationRecord::for_capture(
        plan,
        &ConsistencyRequirementRecord::new(
            plan,
            ConsistencyGuaranteeRecord::ApplicationCoordinated,
        ),
        0,
        &ApplicationFenceBinding {
            identity: hash("56"),
            membership_revision: hash("78"),
        },
    )
    .unwrap()
}
#[test]
fn exact_recovery_keeps_lost_observation_pending_and_never_refunds_uncertain() {
    let plan = plan();
    let obligation = obligation(&plan);
    let mut journal = AttemptJournalRecord::new(plan.attempt_authority(0).unwrap());
    let mutation = journal.reserve_mutation().unwrap();
    let observation = journal
        .reserve_observation(mutation, hash("12").hash())
        .unwrap();
    let pending = acquisition_progress(&plan, &obligation, &journal).unwrap();
    assert_eq!((pending.mutations_used, pending.observations_used), (1, 1));
    assert_eq!(
        (pending.mutations_remaining, pending.observations_remaining),
        (0, 0)
    );
    assert_eq!(pending.pending_observation, Some(observation));
    journal
        .record_observation(ObservationReceiptRequest {
            attempt: observation,
            request: hash("12").hash().into(),
            outcome: ObservationOutcomeRecord::Uncertain,
            evidence: hash("34").hash().into(),
        })
        .unwrap();
    let settled = acquisition_progress(&plan, &obligation, &journal).unwrap();
    assert_eq!(settled.pending_observation, None);
    assert_eq!(settled.pending_mutation, Some(mutation));
    assert!(!settled.applied);
    assert_eq!(
        (settled.mutations_remaining, settled.observations_remaining),
        (0, 0)
    );
}
#[test]
fn other_original_operation_context_request_or_limits_reject() {
    let plan = plan();
    let obligation = obligation(&plan);
    let journal = AttemptJournalRecord::new(plan.attempt_authority(0).unwrap());
    assert_eq!(
        acquisition_progress(&plan, &obligation, &journal).unwrap(),
        journal.view()
    );
    for (pointer, replacement) in [
        ("/authority/binding/intent", json!(hash("90").hash())),
        ("/authority/binding/network", json!(hash("90").hash())),
        ("/authority/binding/caller", json!("aaaaa-aa")),
        ("/authority/binding/target", json!("aaaaa-aa")),
        ("/authority/binding/release", json!(hash("90").hash())),
        ("/authority/binding/request", json!(hash("90").hash())),
        ("/authority/binding/operation_sequence", json!(7)),
        ("/authority/budget/mutations", json!(2)),
        ("/authority/budget/observations", json!(2)),
    ] {
        let mut value = serde_json::to_value(&journal).unwrap();
        *value.pointer_mut(pointer).unwrap() = replacement;
        let changed = serde_json::from_value(value).unwrap();
        assert!(matches!(
            acquisition_progress(&plan, &obligation, &changed),
            Err(FenceObligationProgressError::AuthorityMismatch)
        ));
    }
}
