//! Pure boundary matching and typed unsafe denials; no application/IC observations.

use super::*;
use crate::{
    model::{operation_plan::OperationPlanRecord, restore_safety::*},
    test_support::{
        membership::{APP, hash},
        restore_safety::{fence, input, parameters, plan, requirement, source, wire},
    },
};
use serde_json::json;

#[test]
fn exact_load_start_and_original_lanes_admit_read_only_evidence() {
    let plan = plan();
    let source = source();
    for lane in [
        RestoreSafetyLaneRecord::NoIrreversibleEffects,
        RestoreSafetyLaneRecord::ApplicationFenced,
    ] {
        let requirement = requirement(&plan, &source, lane);
        for method in [
            IcManagementMethodRecord::LoadCanisterSnapshot,
            IcManagementMethodRecord::StartCanister,
        ] {
            let wire = wire(method);
            let request =
                RestoreSafetyRequest::new(&plan, &source, &requirement, &wire, parameters(method))
                    .unwrap();
            let observed = RestoreSafetyObservation::new(input(&request)).unwrap();
            let view = validate(&request, &observed).unwrap();
            assert_eq!(view.request(), &request.digest());
            assert_eq!(view.requirement(), &requirement.digest());
            assert_eq!(view.targets()[0].target, APP);
            assert_eq!(view.evidence(), &hash("89"));
            assert_eq!(view.remote_observations(), 1);
            assert_eq!(
                view.fence().map(|f| &f.binding),
                requirement.expected_fence()
            );
            for state in [CanisterStatusType::Running, CanisterStatusType::Stopping] {
                let mut data = input(&request);
                data.targets[0].state = state;
                assert_eq!(
                    validate(&request, &RestoreSafetyObservation::new(data).unwrap()).unwrap_err(),
                    RestoreSafetyError::TargetNotStopped
                );
            }
        }
    }
}
#[test]
fn changed_actual_context_full_inventory_source_or_selection_rejects() {
    let plan = plan();
    let source = source();
    let requirement = requirement(
        &plan,
        &source,
        RestoreSafetyLaneRecord::NoIrreversibleEffects,
    );
    let wire = wire(IcManagementMethodRecord::LoadCanisterSnapshot);
    let request = RestoreSafetyRequest::new(
        &plan,
        &source,
        &requirement,
        &wire,
        parameters(wire.method()),
    )
    .unwrap();
    for (field, replacement) in [
        ("network", json!("34".repeat(32))),
        ("caller", json!("aaaaa-aa")),
        ("release", json!("34".repeat(32))),
    ] {
        let mut data = input(&request);
        let mut context = serde_json::to_value(&data.context).unwrap();
        context[field] = replacement;
        data.context = serde_json::from_value(context).unwrap();
        assert_eq!(
            validate(&request, &RestoreSafetyObservation::new(data).unwrap()).unwrap_err(),
            RestoreSafetyError::ContextMismatch(field)
        );
    }
    let mut data = input(&request);
    let mut inventory = serde_json::to_value(&data.inventory).unwrap();
    inventory["targets"][0]["role"] = json!("changed-unselected-parent");
    data.inventory = serde_json::from_value(inventory).unwrap();
    assert_eq!(
        validate(&request, &RestoreSafetyObservation::new(data).unwrap()).unwrap_err(),
        RestoreSafetyError::InventoryMismatch
    );
    let mut data = input(&request);
    data.source_plan_intent = hash("34");
    assert_eq!(
        validate(&request, &RestoreSafetyObservation::new(data).unwrap()).unwrap_err(),
        RestoreSafetyError::SourcePlanMismatch
    );
    let mut data = input(&request);
    data.source_artifacts = hash("34");
    assert_eq!(
        validate(&request, &RestoreSafetyObservation::new(data).unwrap()).unwrap_err(),
        RestoreSafetyError::SourceArtifactsMismatch
    );
    let mut data = input(&request);
    data.targets[0].target = "aaaaa-aa".into();
    assert_eq!(
        validate(&request, &RestoreSafetyObservation::new(data).unwrap()).unwrap_err(),
        RestoreSafetyError::SelectionMismatch
    );
}
#[test]
fn stale_challenge_original_requirement_and_descriptive_calls_reject_without_new_allowance() {
    let plan = plan();
    let source = source();
    let requirement = requirement(
        &plan,
        &source,
        RestoreSafetyLaneRecord::NoIrreversibleEffects,
    );
    let wire = wire(IcManagementMethodRecord::LoadCanisterSnapshot);
    let request = RestoreSafetyRequest::new(
        &plan,
        &source,
        &requirement,
        &wire,
        parameters(wire.method()),
    )
    .unwrap();
    let old = RestoreSafetyObservation::new(input(&request)).unwrap();
    let mut params = parameters(wire.method());
    params.challenge = hash("34");
    let fresh = RestoreSafetyRequest::new(&plan, &source, &requirement, &wire, params).unwrap();
    assert_eq!(
        validate(&fresh, &old).unwrap_err(),
        RestoreSafetyError::RequestMismatch
    );
    let mut params = parameters(wire.method());
    params.max_remote_observations = 0;
    let zero = RestoreSafetyRequest::new(&plan, &source, &requirement, &wire, params).unwrap();
    let mut data = input(&zero);
    assert_eq!(
        validate(&zero, &RestoreSafetyObservation::new(data.clone()).unwrap()).unwrap_err(),
        RestoreSafetyError::ObservationLimitExceeded {
            limit: 0,
            reported: 1
        }
    );
    data.remote_observations = 0;
    assert!(validate(&zero, &RestoreSafetyObservation::new(data).unwrap()).is_ok());
    let mut data = input(&request);
    data.remote_observations = 2;
    assert_eq!(
        validate(&request, &RestoreSafetyObservation::new(data).unwrap()).unwrap_err(),
        RestoreSafetyError::ObservationLimitExceeded {
            limit: 1,
            reported: 2
        }
    );
    let mut value = serde_json::to_value(&requirement).unwrap();
    value["source_artifacts"] = json!(hash("34"));
    let changed = serde_json::from_value(value).unwrap();
    let changed_request =
        RestoreSafetyRequest::new(&plan, &source, &changed, &wire, parameters(wire.method()))
            .unwrap();
    assert_eq!(
        validate(&changed_request, &old).unwrap_err(),
        RestoreSafetyError::RequestMismatch
    );
}
#[test]
fn unresolved_obligations_lane_changes_inactive_or_rebound_fences_deny() {
    let plan = plan();
    let source = source();
    let wire = wire(IcManagementMethodRecord::LoadCanisterSnapshot);
    for lane in [
        RestoreSafetyLaneRecord::NoIrreversibleEffects,
        RestoreSafetyLaneRecord::ApplicationFenced,
    ] {
        let requirement = requirement(&plan, &source, lane);
        let request = RestoreSafetyRequest::new(
            &plan,
            &source,
            &requirement,
            &wire,
            parameters(wire.method()),
        )
        .unwrap();
        let mut data = input(&request);
        data.safety = RestoreSafetyEvidence::Unresolved(hash("34"));
        assert_eq!(
            validate(&request, &RestoreSafetyObservation::new(data).unwrap()).unwrap_err(),
            RestoreSafetyError::UnresolvedExternalObligations
        );
        let mut data = input(&request);
        data.safety = match lane {
            RestoreSafetyLaneRecord::NoIrreversibleEffects => {
                RestoreSafetyEvidence::ApplicationFenced(Box::new(fence()))
            }
            RestoreSafetyLaneRecord::ApplicationFenced => {
                RestoreSafetyEvidence::NoIrreversibleEffects(hash("34"))
            }
        };
        assert_eq!(
            validate(&request, &RestoreSafetyObservation::new(data).unwrap()).unwrap_err(),
            RestoreSafetyError::SafetyLaneMismatch
        );
    }
    let requirement = requirement(&plan, &source, RestoreSafetyLaneRecord::ApplicationFenced);
    let request = RestoreSafetyRequest::new(
        &plan,
        &source,
        &requirement,
        &wire,
        parameters(wire.method()),
    )
    .unwrap();
    for variant in 0..4 {
        let mut data = input(&request);
        let mut changed = fence();
        let error = match variant {
            0 => {
                changed.binding.identity = hash("34");
                RestoreSafetyError::FenceMismatch
            }
            1 => {
                changed.binding.membership_revision = hash("34");
                RestoreSafetyError::MembershipRevisionMismatch
            }
            2 => {
                changed.binding.external_obligations_revision = hash("34");
                RestoreSafetyError::ExternalObligationsRevisionMismatch
            }
            _ => {
                changed.state = ApplicationFenceState::Inactive;
                RestoreSafetyError::FenceNotActive
            }
        };
        data.safety = RestoreSafetyEvidence::ApplicationFenced(Box::new(changed));
        assert_eq!(
            validate(&request, &RestoreSafetyObservation::new(data).unwrap()).unwrap_err(),
            error
        );
    }
}
#[test]
fn start_requires_source_specific_application_acceptance_and_fenced_execution() {
    let plan = plan();
    let source = source();
    for lane in [
        RestoreSafetyLaneRecord::NoIrreversibleEffects,
        RestoreSafetyLaneRecord::ApplicationFenced,
    ] {
        let requirement = requirement(&plan, &source, lane);
        for method in [
            IcManagementMethodRecord::LoadCanisterSnapshot,
            IcManagementMethodRecord::StartCanister,
        ] {
            let wire = wire(method);
            let request =
                RestoreSafetyRequest::new(&plan, &source, &requirement, &wire, parameters(method))
                    .unwrap();
            let mut data = input(&request);
            data.targets[0].restored_acceptance = None;
            if method == IcManagementMethodRecord::StartCanister {
                assert_eq!(
                    validate(&request, &RestoreSafetyObservation::new(data).unwrap()).unwrap_err(),
                    RestoreSafetyError::RestoredAcceptanceRequired
                );
            } else {
                assert!(validate(&request, &RestoreSafetyObservation::new(data).unwrap()).is_ok());
            }
            if lane == RestoreSafetyLaneRecord::ApplicationFenced {
                let mut data = input(&request);
                let mut missing = fence();
                missing.controlled_execution = None;
                data.safety = RestoreSafetyEvidence::ApplicationFenced(Box::new(missing));
                if method == IcManagementMethodRecord::StartCanister {
                    assert_eq!(
                        validate(&request, &RestoreSafetyObservation::new(data).unwrap())
                            .unwrap_err(),
                        RestoreSafetyError::ControlledExecutionRequired
                    );
                } else {
                    assert!(
                        validate(&request, &RestoreSafetyObservation::new(data).unwrap()).is_ok()
                    );
                }
            }
        }
    }
}
#[test]
fn whole_selected_unit_needs_stopped_load_and_acceptance_before_ordered_starts() {
    let mut value = serde_json::to_value(plan()).unwrap();
    let other = "r7inp-6aaaa-aaaaa-aaabq-cai";
    value["inventory"]["targets"].as_array_mut().unwrap().push(
        json!({"canister_id":other,"parent_canister_id":"aaaaa-aa","role":null,"module_hash":null}),
    );
    value["selected_targets"]
        .as_array_mut()
        .unwrap()
        .push(json!(other));
    value["graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .push(json!({"operation_sequence":9,"depends_on":[]}));
    value["graph"]["nodes"][1]["depends_on"] = json!([7, 9]);
    value["operations"].as_array_mut().unwrap().push(json!({"operation_sequence":9,"target":other,"request":"12".repeat(32),"budget":{"mutations":1,"observations":1}}));
    value["budget"] = json!({"mutations":3,"observations":3});
    let plan: OperationPlanRecord = serde_json::from_value(value).unwrap();
    let source = plan.clone();
    let requirement = requirement(&plan, &source, RestoreSafetyLaneRecord::ApplicationFenced);
    let load = wire(IcManagementMethodRecord::LoadCanisterSnapshot);
    let request = RestoreSafetyRequest::new(
        &plan,
        &source,
        &requirement,
        &load,
        parameters(load.method()),
    )
    .unwrap();
    let mut data = input(&request);
    let i = data
        .targets
        .iter()
        .position(|target| target.target == other)
        .unwrap();
    data.targets[i].state = CanisterStatusType::Running;
    assert_eq!(
        validate(&request, &RestoreSafetyObservation::new(data).unwrap()).unwrap_err(),
        RestoreSafetyError::TargetNotStopped
    );
    let start = wire(IcManagementMethodRecord::StartCanister);
    let request = RestoreSafetyRequest::new(
        &plan,
        &source,
        &requirement,
        &start,
        parameters(start.method()),
    )
    .unwrap();
    let mut data = input(&request);
    data.targets[i].state = CanisterStatusType::Running;
    assert!(
        validate(
            &request,
            &RestoreSafetyObservation::new(data.clone()).unwrap()
        )
        .is_ok()
    );
    let mut stopping = data.clone();
    stopping.targets[i].state = CanisterStatusType::Stopping;
    assert_eq!(
        validate(&request, &RestoreSafetyObservation::new(stopping).unwrap()).unwrap_err(),
        RestoreSafetyError::TargetNotStopped
    );
    data.targets[i].restored_acceptance = None;
    assert_eq!(
        validate(&request, &RestoreSafetyObservation::new(data).unwrap()).unwrap_err(),
        RestoreSafetyError::RestoredAcceptanceRequired
    );
    let mut data = input(&request);
    data.targets.remove(i);
    assert_eq!(
        validate(&request, &RestoreSafetyObservation::new(data).unwrap()).unwrap_err(),
        RestoreSafetyError::SelectionMismatch
    );
}
