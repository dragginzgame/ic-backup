use super::*;
use crate::{
    model::attempt_journal::{ObservationOutcomeRecord, ObservationReceiptRequest},
    test_support::{
        fence_reconciliation::{input, original, reserve},
        membership::hash,
    },
};
use serde_json::json;

#[test]
fn independent_binary_goldens_bind_all_original_intent_fields() {
    let contract: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../../docs/contracts/fence-reconciliation-port.json"
    ))
    .unwrap();
    for golden in contract["digest_goldens"].as_array().unwrap() {
        let hashes: Vec<_> = ["obligation", "authority", "challenge"]
            .into_iter()
            .map(|key| ArtifactChecksumRecord::from_hash(golden[key].as_str().unwrap()).unwrap())
            .collect();
        let mutation = u32::try_from(golden["mutation_attempt"].as_u64().unwrap()).unwrap();
        assert_eq!(
            observation_digest(&hashes[0], &hashes[1], mutation, &hashes[2]).hash(),
            golden["sha256"].as_str().unwrap()
        );
    }
    let original = original(false);
    let a = FenceReconciliationIntent::new(
        &original.plan,
        &original.obligation,
        &original.journal,
        hash("12"),
    )
    .unwrap();
    let b = FenceReconciliationIntent::new(
        &original.plan,
        &original.obligation,
        &original.journal,
        hash("34"),
    )
    .unwrap();
    assert_ne!(a.digest(), b.digest());
    assert_eq!(a.authority(), original.journal.authority());
    assert_eq!(a.challenge(), &hash("12"));
}
#[test]
fn missing_pending_mutation_or_original_authority_never_creates_intent() {
    let original = original(false);
    let empty = AttemptJournalRecord::new(original.plan.attempt_authority(0).unwrap());
    assert!(matches!(
        FenceReconciliationIntent::new(&original.plan, &original.obligation, &empty, hash("12")),
        Err(FenceReconciliationRequestError::NoPendingMutation)
    ));
    for (pointer, replacement) in [
        ("/authority/binding/network", json!(hash("90").hash())),
        ("/authority/binding/caller", json!("aaaaa-aa")),
        ("/authority/binding/request", json!(hash("90").hash())),
        ("/authority/budget/mutations", json!(2)),
        ("/authority/budget/observations", json!(2)),
    ] {
        let mut value = serde_json::to_value(&original.journal).unwrap();
        *value.pointer_mut(pointer).unwrap() = replacement;
        let changed = serde_json::from_value(value).unwrap();
        assert!(matches!(
            FenceReconciliationIntent::new(
                &original.plan,
                &original.obligation,
                &changed,
                hash("12")
            ),
            Err(FenceReconciliationRequestError::AuthorityMismatch)
        ));
    }
}
#[test]
fn only_exact_reserved_observation_binds_and_settlement_invalidates_old_request() {
    let mut original = original(false);
    let intent = FenceReconciliationIntent::new(
        &original.plan,
        &original.obligation,
        &original.journal,
        hash("12"),
    )
    .unwrap();
    assert!(matches!(
        FenceReconciliationRequest::new(&intent, &original.journal),
        Err(FenceReconciliationRequestError::NoPendingObservation)
    ));
    assert_eq!(original.journal.pending_observation_request(), None);
    let attempt = original
        .journal
        .reserve_observation(intent.mutation_attempt(), intent.digest().hash())
        .unwrap();
    let request = FenceReconciliationRequest::new(&intent, &original.journal).unwrap();
    assert_eq!(
        original.journal.pending_observation_request(),
        Some(request.digest().hash())
    );
    let other = FenceReconciliationIntent::new(
        &original.plan,
        &original.obligation,
        &original.journal,
        hash("34"),
    )
    .unwrap();
    assert!(matches!(
        FenceReconciliationRequest::new(&other, &original.journal),
        Err(FenceReconciliationRequestError::RequestMismatch)
    ));
    let view = original.journal.view();
    assert_eq!(
        (view.mutations_remaining, view.observations_remaining),
        (0, 0)
    );
    original
        .journal
        .record_observation(ObservationReceiptRequest {
            attempt,
            request: request.digest().hash().into(),
            outcome: ObservationOutcomeRecord::Uncertain,
            evidence: hash("ab").hash().into(),
        })
        .unwrap();
    assert_eq!(original.journal.pending_observation_request(), None);
    assert!(matches!(
        request.validate_journal(&original.journal),
        Err(FenceReconciliationRequestError::ObservationMismatch)
    ));
    assert_eq!(
        original.journal.view().pending_mutation,
        Some(intent.mutation_attempt())
    );
}
#[test]
fn bounded_canonical_actual_unit_and_chronological_attempt_ids_reject_bad_inputs() {
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
    actual.selected_targets[0] = actual.selected_targets[0].to_uppercase();
    assert_eq!(
        FenceReconciliationObservation::new(actual)
            .unwrap()
            .input()
            .selected_targets,
        original.plan.selected_targets()
    );
    for targets in [
        vec![],
        vec![original.plan.selected_targets()[0].clone(); MAX_INVENTORY_TARGETS + 1],
    ] {
        let mut actual = input(&request);
        actual.selected_targets = targets;
        assert!(matches!(
            FenceReconciliationObservation::new(actual),
            Err(FenceReconciliationObservationError::InvalidTargetCount)
        ));
    }
    let mut actual = input(&request);
    actual
        .selected_targets
        .push(actual.selected_targets[0].to_uppercase());
    assert!(matches!(
        FenceReconciliationObservation::new(actual),
        Err(FenceReconciliationObservationError::DuplicateTarget)
    ));
    for (mutation, observation) in [(0, 2), (2, 2), (2, 1), (1, MAX_OPERATION_ATTEMPTS + 1)] {
        let mut actual = input(&request);
        actual.mutation_attempt = mutation;
        actual.observation_attempt = observation;
        assert!(matches!(
            FenceReconciliationObservation::new(actual),
            Err(FenceReconciliationObservationError::InvalidAttempts)
        ));
    }
    let mut actual = input(&request);
    actual.selected_targets = vec!["2vxsx-fae".into()];
    assert!(matches!(
        FenceReconciliationObservation::new(actual),
        Err(FenceReconciliationObservationError::Inventory(_))
    ));
}
#[test]
fn admits_exact_inventory_ceiling_in_canonical_order_without_dispatch_authority() {
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
    let targets: Vec<_> = (0..MAX_INVENTORY_TARGETS).map(|index| {
        let principal = ic_principal::Principal::from_slice(&index.to_be_bytes()).to_text();
        json!({"canister_id":principal,"parent_canister_id":null,"role":null,"module_hash":null})
    }).collect();
    let mut actual = input(&request);
    actual.selected_targets = targets
        .iter()
        .rev()
        .map(|target| target["canister_id"].as_str().unwrap().to_uppercase())
        .collect();
    actual.inventory = serde_json::from_value(json!({"version":1,"targets":targets})).unwrap();
    let actual = FenceReconciliationObservation::new(actual).unwrap();
    assert_eq!(actual.input().selected_targets.len(), MAX_INVENTORY_TARGETS);
    assert!(
        actual
            .input()
            .selected_targets
            .windows(2)
            .all(|pair| pair[0] < pair[1])
    );
}
