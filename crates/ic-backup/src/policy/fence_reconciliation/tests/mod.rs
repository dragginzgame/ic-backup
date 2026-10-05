use super::*;
use crate::{
    model::{attempt_journal::ObservationReceiptRequest, fence_reconciliation::*},
    test_support::{
        consistency,
        fence_reconciliation::{input, original, reserve},
        membership::hash,
        restore_safety,
    },
};
use serde_json::json;

#[test]
fn all_passive_claims_match_original_purpose_without_mutating_or_refunding_journal() {
    for restore in [false, true] {
        let mut original = original(restore);
        reserve(&mut original, "12");
        let intent = FenceReconciliationIntent::new(
            &original.plan,
            &original.obligation,
            &original.journal,
            hash("12"),
        )
        .unwrap();
        let request = FenceReconciliationRequest::new(&intent, &original.journal).unwrap();
        let before = original.journal.clone();
        let mut actual = input(&request);
        for (settlement, outcome) in [
            (actual.settlement.clone(), ObservationOutcomeRecord::Applied),
            (
                FenceReconciliationEvidence::NotAcquired {
                    exclusion: hash("34"),
                },
                ObservationOutcomeRecord::NotApplied,
            ),
            (
                FenceReconciliationEvidence::Unresolved {
                    uncertainty: hash("34"),
                },
                ObservationOutcomeRecord::Uncertain,
            ),
        ] {
            actual.settlement = settlement;
            let observation = FenceReconciliationObservation::new(actual.clone()).unwrap();
            let view = validate(&request, &original.journal, &observation).unwrap();
            assert_eq!(view.outcome(), outcome);
            assert_eq!(view.request(), &request.digest());
            assert_eq!(view.observation_attempt(), request.observation_attempt());
            assert_eq!(view.evidence(), &hash("ab"));
            assert_eq!(original.journal, before);
        }
    }
}
#[test]
fn exact_request_attempts_context_inventory_selection_and_single_call_bound_reject_drift() {
    let mut original = original(false);
    reserve(&mut original, "12");
    let intent = FenceReconciliationIntent::new(
        &original.plan,
        &original.obligation,
        &original.journal,
        hash("12"),
    )
    .unwrap();
    let request = FenceReconciliationRequest::new(&intent, &original.journal).unwrap();
    let mut actual = input(&request);
    actual.request = hash("90");
    assert!(matches!(
        validate(
            &request,
            &original.journal,
            &FenceReconciliationObservation::new(actual).unwrap()
        ),
        Err(FenceReconciliationError::RequestMismatch)
    ));
    for (mutation, observation) in [(1, 3), (2, 3)] {
        let mut actual = input(&request);
        actual.mutation_attempt = mutation;
        actual.observation_attempt = observation;
        assert!(matches!(
            validate(
                &request,
                &original.journal,
                &FenceReconciliationObservation::new(actual).unwrap()
            ),
            Err(FenceReconciliationError::AttemptMismatch)
        ));
    }
    for (field, replacement) in [
        ("network", json!(hash("90").hash())),
        ("caller", json!("aaaaa-aa")),
        ("release", json!(hash("90").hash())),
    ] {
        let mut actual = input(&request);
        let mut context = serde_json::to_value(&actual.context).unwrap();
        context[field] = replacement;
        actual.context = serde_json::from_value(context).unwrap();
        assert!(matches!(
            validate(
                &request,
                &original.journal,
                &FenceReconciliationObservation::new(actual).unwrap()
            ),
            Err(FenceReconciliationError::ContextMismatch)
        ));
    }
    let mut actual = input(&request);
    let mut inventory = serde_json::to_value(&actual.inventory).unwrap();
    inventory["targets"][0]["role"] = json!("unselected-parent-drift");
    actual.inventory = serde_json::from_value(inventory).unwrap();
    assert!(matches!(
        validate(
            &request,
            &original.journal,
            &FenceReconciliationObservation::new(actual).unwrap()
        ),
        Err(FenceReconciliationError::InventoryMismatch)
    ));
    let mut actual = input(&request);
    actual.selected_targets = vec!["aaaaa-aa".into()];
    assert!(matches!(
        validate(
            &request,
            &original.journal,
            &FenceReconciliationObservation::new(actual).unwrap()
        ),
        Err(FenceReconciliationError::SelectionMismatch)
    ));
    let mut actual = input(&request);
    actual.remote_observations = 2;
    assert!(matches!(
        validate(
            &request,
            &original.journal,
            &FenceReconciliationObservation::new(actual).unwrap()
        ),
        Err(FenceReconciliationError::ObservationLimitExceeded)
    ));
}
#[test]
fn capture_acquisition_rejects_other_purpose_inactive_or_rebound_fence() {
    let mut original = original(false);
    reserve(&mut original, "12");
    let intent = FenceReconciliationIntent::new(
        &original.plan,
        &original.obligation,
        &original.journal,
        hash("12"),
    )
    .unwrap();
    let request = FenceReconciliationRequest::new(&intent, &original.journal).unwrap();
    for (state, identity, revision, expected) in [
        (
            ApplicationFenceState::Inactive,
            hash("56"),
            hash("78"),
            "inactive",
        ),
        (
            ApplicationFenceState::Active,
            hash("90"),
            hash("78"),
            "identity",
        ),
        (
            ApplicationFenceState::Active,
            hash("56"),
            hash("90"),
            "revision",
        ),
    ] {
        let mut actual = input(&request);
        actual.settlement = FenceReconciliationEvidence::AcquiredCapture {
            fence: Box::new(crate::model::consistency::ApplicationFenceEvidence {
                state,
                identity,
                membership_revision: revision,
                ..consistency::fence()
            }),
            attribution: hash("34"),
        };
        let observation = FenceReconciliationObservation::new(actual).unwrap();
        let error = validate(&request, &original.journal, &observation).unwrap_err();
        assert!(matches!(
            (expected, error),
            ("inactive", FenceReconciliationError::FenceNotActive)
                | ("identity", FenceReconciliationError::FenceMismatch)
                | (
                    "revision",
                    FenceReconciliationError::MembershipRevisionMismatch
                )
        ));
    }
    let mut actual = input(&request);
    actual.settlement = FenceReconciliationEvidence::AcquiredRestore {
        fence: Box::new(restore_safety::fence()),
        attribution: hash("34"),
    };
    assert!(matches!(
        validate(
            &request,
            &original.journal,
            &FenceReconciliationObservation::new(actual).unwrap()
        ),
        Err(FenceReconciliationError::PurposeMismatch)
    ));
}
#[test]
fn restore_acquisition_rejects_changed_original_fence_and_external_revisions() {
    let mut original = original(true);
    reserve(&mut original, "12");
    let intent = FenceReconciliationIntent::new(
        &original.plan,
        &original.obligation,
        &original.journal,
        hash("12"),
    )
    .unwrap();
    let request = FenceReconciliationRequest::new(&intent, &original.journal).unwrap();
    for field in ["identity", "membership", "external", "inactive"] {
        let mut fence = restore_safety::fence();
        match field {
            "identity" => fence.binding.identity = hash("ef"),
            "membership" => fence.binding.membership_revision = hash("ef"),
            "external" => fence.binding.external_obligations_revision = hash("ef"),
            _ => fence.state = ApplicationFenceState::Inactive,
        }
        let mut actual = input(&request);
        actual.settlement = FenceReconciliationEvidence::AcquiredRestore {
            fence: Box::new(fence),
            attribution: hash("34"),
        };
        let error = validate(
            &request,
            &original.journal,
            &FenceReconciliationObservation::new(actual).unwrap(),
        )
        .unwrap_err();
        assert!(matches!(
            (field, error),
            ("identity", FenceReconciliationError::FenceMismatch)
                | (
                    "membership",
                    FenceReconciliationError::MembershipRevisionMismatch
                )
                | (
                    "external",
                    FenceReconciliationError::ExternalObligationsRevisionMismatch
                )
                | ("inactive", FenceReconciliationError::FenceNotActive)
        ));
    }
}
#[test]
fn old_result_cannot_replay_after_observation_settlement_or_under_new_reservation() {
    let mut original = original_with_two_observations();
    reserve(&mut original, "12");
    let intent = FenceReconciliationIntent::new(
        &original.plan,
        &original.obligation,
        &original.journal,
        hash("12"),
    )
    .unwrap();
    let request = FenceReconciliationRequest::new(&intent, &original.journal).unwrap();
    let observation = FenceReconciliationObservation::new(input(&request)).unwrap();
    original
        .journal
        .record_observation(ObservationReceiptRequest {
            attempt: request.observation_attempt(),
            request: request.digest().hash().into(),
            outcome: ObservationOutcomeRecord::Uncertain,
            evidence: hash("ab").hash().into(),
        })
        .unwrap();
    assert!(matches!(
        validate(&request, &original.journal, &observation),
        Err(FenceReconciliationError::Reservation(
            FenceReconciliationRequestError::ObservationMismatch
        ))
    ));
    // Even deliberately reusing a challenge cannot replay a prior observation attempt.
    let next_intent = FenceReconciliationIntent::new(
        &original.plan,
        &original.obligation,
        &original.journal,
        hash("12"),
    )
    .unwrap();
    original
        .journal
        .reserve_observation(next_intent.mutation_attempt(), next_intent.digest().hash())
        .unwrap();
    let next = FenceReconciliationRequest::new(&next_intent, &original.journal).unwrap();
    assert!(matches!(
        validate(&next, &original.journal, &observation),
        Err(FenceReconciliationError::AttemptMismatch)
    ));
}
fn original_with_two_observations() -> crate::test_support::fence_reconciliation::Original {
    use crate::model::{
        consistency::{
            ApplicationFenceBinding, ConsistencyGuaranteeRecord, ConsistencyRequirementRecord,
        },
        fence_obligation::FenceObligationRecord,
    };
    let mut value = serde_json::to_value(crate::test_support::membership::plan()).unwrap();
    value["operations"][0]["budget"]["observations"] = json!(2);
    value["budget"]["observations"] = json!(3);
    let plan = serde_json::from_value(value).unwrap();
    let obligation = FenceObligationRecord::for_capture(
        &plan,
        &ConsistencyRequirementRecord::new(
            &plan,
            ConsistencyGuaranteeRecord::ApplicationCoordinated,
        ),
        0,
        &ApplicationFenceBinding {
            identity: hash("56"),
            membership_revision: hash("78"),
        },
    )
    .unwrap();
    let mut journal = AttemptJournalRecord::new(plan.attempt_authority(0).unwrap());
    journal.reserve_mutation().unwrap();
    crate::test_support::fence_reconciliation::Original {
        plan,
        obligation,
        journal,
    }
}
