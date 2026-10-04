//! Pure exact-result admission; fixtures certify no actual membership or IC effects.

use super::*;
use crate::{
    model::{
        inventory::{InventoryTargetRecord, InventoryTargetRequest},
        membership::MembershipBoundary,
        operation_plan::{PlanContextRecord, PlanContextRequest},
    },
    test_support::membership::{APP, hash, plan},
};

fn observation(request: &MembershipObservationRequest<'_>) -> MembershipObservation {
    MembershipObservation {
        request: request.digest(),
        context: PlanContextRecord::new(&PlanContextRequest {
            network: request.binding().network().into(),
            caller: request.binding().caller().into(),
            release: request.binding().release().into(),
        })
        .unwrap(),
        inventory: request.inventory().clone(),
        revision: Some(hash("34")),
        evidence: hash("56"),
        remote_observations: 1,
    }
}

#[test]
fn projects_exact_inventory_selection_revision_evidence_and_reported_calls() {
    let plan = plan();
    let request = MembershipObservationRequest::new(
        &plan,
        7,
        hash("12"),
        MembershipBoundary::BeforeEffect,
        1,
    )
    .unwrap();
    let observed = observation(&request);
    let view = validate(&request, &observed).unwrap();
    assert_eq!(view.request(), &request.digest());
    assert!(std::ptr::eq(
        std::ptr::from_ref(view.inventory()),
        std::ptr::from_ref(&observed.inventory)
    ));
    assert_eq!(view.selected_targets(), &[APP.to_owned()]);
    assert_eq!(view.revision(), Some(&hash("34")));
    assert_eq!(view.evidence(), &hash("56"));
    assert_eq!(view.remote_observations(), 1);
    let original_bytes = serde_json::to_vec(&plan).unwrap();
    let authority = plan.attempt_authority(7).unwrap();
    validate(&request, &observed).unwrap();
    assert_eq!(serde_json::to_vec(&plan).unwrap(), original_bytes);
    assert_eq!(plan.attempt_authority(7).unwrap(), authority);
}

#[test]
fn rejects_replayed_challenge_other_boundary_operation_and_changed_original_plan() {
    let plan = plan();
    let request = MembershipObservationRequest::new(
        &plan,
        7,
        hash("12"),
        MembershipBoundary::BeforeEffect,
        1,
    )
    .unwrap();
    let observed = observation(&request);
    for (sequence, challenge, boundary, limit) in [
        (7, "23", MembershipBoundary::BeforeEffect, 1),
        (7, "12", MembershipBoundary::AfterEffect, 1),
        (0, "12", MembershipBoundary::BeforeEffect, 1),
        (7, "12", MembershipBoundary::BeforeEffect, 2),
    ] {
        let other =
            MembershipObservationRequest::new(&plan, sequence, hash(challenge), boundary, limit)
                .unwrap();
        assert_eq!(
            validate(&other, &observed).unwrap_err(),
            MembershipError::RequestMismatch
        );
    }
    let mut value = serde_json::to_value(&plan).unwrap();
    value["budget"]["observations"] = serde_json::json!(3);
    let changed = serde_json::from_value(value).unwrap();
    let other = MembershipObservationRequest::new(
        &changed,
        7,
        hash("12"),
        MembershipBoundary::BeforeEffect,
        1,
    )
    .unwrap();
    assert_eq!(
        validate(&other, &observed).unwrap_err(),
        MembershipError::RequestMismatch
    );
}

#[test]
fn rejects_actually_observed_network_caller_and_release_mismatch() {
    let plan = plan();
    let request = MembershipObservationRequest::new(
        &plan,
        7,
        hash("12"),
        MembershipBoundary::BeforeEffect,
        1,
    )
    .unwrap();
    for field in ["network", "caller", "release"] {
        let mut observed = observation(&request);
        let mut value = serde_json::to_value(&observed.context).unwrap();
        value[field] = serde_json::json!(if field == "caller" {
            "aaaaa-aa".into()
        } else {
            "89".repeat(32)
        });
        observed.context = serde_json::from_value(value).unwrap();
        assert_eq!(
            validate(&request, &observed).unwrap_err(),
            MembershipError::ContextMismatch(field)
        );
    }
}

#[test]
fn rejects_full_inventory_drift_including_unselected_parent_and_metadata() {
    let plan = plan();
    let request = MembershipObservationRequest::new(
        &plan,
        7,
        hash("12"),
        MembershipBoundary::BeforeEffect,
        1,
    )
    .unwrap();
    for (pointer, replacement) in [
        (
            "/targets/0/role",
            serde_json::json!("changed unselected parent"),
        ),
        ("/targets/0/module_hash", serde_json::json!("ab".repeat(32))),
        ("/targets/1/parent_canister_id", serde_json::Value::Null),
        (
            "/targets/1/role",
            serde_json::json!("changed selected role"),
        ),
    ] {
        let mut observed = observation(&request);
        let mut value = serde_json::to_value(&observed.inventory).unwrap();
        *value.pointer_mut(pointer).unwrap() = replacement;
        observed.inventory = serde_json::from_value(value).unwrap();
        assert_eq!(
            validate(&request, &observed).unwrap_err(),
            MembershipError::InventoryMismatch,
            "{pointer}"
        );
    }
    let mut observed = observation(&request);
    observed.inventory = InventoryRecord::new(vec![
        InventoryTargetRecord::new(&InventoryTargetRequest {
            canister_id: APP.into(),
            parent_canister_id: None,
            role: None,
            module_hash: None,
        })
        .unwrap(),
    ])
    .unwrap();
    assert_eq!(
        validate(&request, &observed).unwrap_err(),
        MembershipError::InventoryMismatch
    );
}

#[test]
fn accepts_canonical_aliases_without_inventing_revision_continuity_or_extra_calls() {
    let plan = plan();
    let request = MembershipObservationRequest::new(
        &plan,
        7,
        hash("12"),
        MembershipBoundary::BeforeEffect,
        0,
    )
    .unwrap();
    let mut observed = observation(&request);
    observed.remote_observations = 0;
    observed.revision = None;
    let mut value = serde_json::to_value(&observed.inventory).unwrap();
    value["targets"][1]["canister_id"] = serde_json::json!(APP.to_uppercase());
    observed.inventory = serde_json::from_value(value).unwrap();
    assert_eq!(validate(&request, &observed).unwrap().revision(), None);
    observed.remote_observations = 1;
    assert_eq!(
        validate(&request, &observed).unwrap_err(),
        MembershipError::ObservationLimitExceeded {
            limit: 0,
            reported: 1
        }
    );
    let max_request = MembershipObservationRequest::new(
        &plan,
        7,
        hash("12"),
        MembershipBoundary::AfterEffect,
        1024,
    )
    .unwrap();
    let mut max = observation(&max_request);
    max.remote_observations = 1024;
    assert_eq!(
        validate(&max_request, &max).unwrap().remote_observations(),
        1024
    );
    max.remote_observations = u32::MAX;
    assert_eq!(
        validate(&max_request, &max).unwrap_err(),
        MembershipError::ObservationLimitExceeded {
            limit: 1024,
            reported: u32::MAX
        }
    );
}

#[test]
fn admits_the_full_inventory_bound_and_rejects_one_missing_unselected_target() {
    let plan = plan();
    let mut value = serde_json::to_value(&plan).unwrap();
    let rows = value["inventory"]["targets"].as_array_mut().unwrap();
    for id in 0_u32..1022 {
        rows.push(serde_json::json!({
            "canister_id": ic_principal::Principal::from_slice(&id.to_be_bytes()).to_text(),
            "parent_canister_id": null, "role": null, "module_hash": null,
        }));
    }
    let plan = serde_json::from_value(value).unwrap();
    let request = MembershipObservationRequest::new(
        &plan,
        7,
        hash("12"),
        MembershipBoundary::BeforeEffect,
        0,
    )
    .unwrap();
    let mut observed = observation(&request);
    observed.remote_observations = 0;
    assert_eq!(
        validate(&request, &observed)
            .unwrap()
            .inventory()
            .targets()
            .len(),
        1024
    );
    let missing = ic_principal::Principal::from_slice(&0_u32.to_be_bytes()).to_text();
    let smaller = InventoryRecord::new(
        observed
            .inventory
            .targets()
            .iter()
            .filter(|target| target.canister_id() != missing)
            .cloned()
            .collect(),
    )
    .unwrap();
    observed.inventory = smaller;
    assert_eq!(
        validate(&request, &observed).unwrap_err(),
        MembershipError::InventoryMismatch
    );
}
