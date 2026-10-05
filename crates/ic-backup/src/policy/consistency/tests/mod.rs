//! Pure actual consistency matching; native evidence never proves an active application fence.

use super::*;
use crate::{
    model::{
        consistency::{ConsistencyBoundary, ConsistencyRequirementRecord},
        operation_plan::{PlanContextRecord, PlanContextRequest},
    },
    test_support::{
        consistency::{fence, input, parameters},
        membership::{hash, plan},
    },
};

#[test]
fn admits_only_original_lane_and_requires_every_exact_target_stopped() {
    let plan = plan();
    for guarantee in [
        ConsistencyGuaranteeRecord::PerCanister,
        ConsistencyGuaranteeRecord::ApplicationCoordinated,
    ] {
        let requirement = ConsistencyRequirementRecord::new(&plan, guarantee);
        let request = ConsistencyRequest::new(&plan, &requirement, parameters(guarantee)).unwrap();
        let observation = ConsistencyObservation::new(input(&request)).unwrap();
        let view = validate(&request, &observation).unwrap();
        assert_eq!(view.request(), &request.digest());
        assert_eq!(view.requirement(), &requirement.digest());
        assert_eq!(view.targets()[0].stopped_and_drained, hash("ef"));
        assert_eq!(view.evidence(), &hash("34"));
        assert_eq!(view.remote_observations(), 1);
        assert_eq!(
            view.fence().map(|fence| &fence.identity),
            request.expected_fence().map(|fence| &fence.identity)
        );
        for state in [CaptureState::Running, CaptureState::Stopping] {
            let mut data = input(&request);
            data.targets[0].state = state;
            assert_eq!(
                validate(&request, &ConsistencyObservation::new(data).unwrap()).unwrap_err(),
                ConsistencyError::TargetNotStopped
            );
        }
        let mut data = input(&request);
        data.consistency = match guarantee {
            ConsistencyGuaranteeRecord::PerCanister => {
                ConsistencyEvidence::ApplicationCoordinated(Box::new(fence()))
            }
            ConsistencyGuaranteeRecord::ApplicationCoordinated => ConsistencyEvidence::PerCanister,
        };
        assert_eq!(
            validate(&request, &ConsistencyObservation::new(data).unwrap()).unwrap_err(),
            ConsistencyError::GuaranteeMismatch
        );
    }
}

#[test]
fn rejects_stale_boundary_challenge_or_fence_and_changed_actual_revision() {
    let plan = plan();
    let guarantee = ConsistencyGuaranteeRecord::ApplicationCoordinated;
    let requirement = ConsistencyRequirementRecord::new(&plan, guarantee);
    let request = ConsistencyRequest::new(&plan, &requirement, parameters(guarantee)).unwrap();
    let observation = ConsistencyObservation::new(input(&request)).unwrap();
    for variant in 0..4 {
        let mut params = parameters(guarantee);
        match variant {
            0 => params.challenge = hash("34"),
            1 => params.boundary = ConsistencyBoundary::AfterCapture,
            2 => params.expected_fence.as_mut().unwrap().identity = hash("90"),
            _ => params.expected_fence.as_mut().unwrap().membership_revision = hash("90"),
        }
        let changed = ConsistencyRequest::new(&plan, &requirement, params).unwrap();
        assert_eq!(
            validate(&changed, &observation).unwrap_err(),
            ConsistencyError::RequestMismatch
        );
    }
    let mut data = input(&request);
    let mut changed = fence();
    changed.identity = hash("90");
    data.consistency = ConsistencyEvidence::ApplicationCoordinated(Box::new(changed));
    assert_eq!(
        validate(&request, &ConsistencyObservation::new(data).unwrap()).unwrap_err(),
        ConsistencyError::FenceMismatch
    );
    for revision in [None, Some(hash("90"))] {
        let mut data = input(&request);
        data.membership_revision = revision;
        assert_eq!(
            validate(&request, &ConsistencyObservation::new(data).unwrap()).unwrap_err(),
            ConsistencyError::MembershipRevisionMismatch
        );
    }
    let mut data = input(&request);
    let mut inactive = fence();
    inactive.state = ApplicationFenceState::Inactive;
    data.consistency = ConsistencyEvidence::ApplicationCoordinated(Box::new(inactive));
    assert_eq!(
        validate(&request, &ConsistencyObservation::new(data).unwrap()).unwrap_err(),
        ConsistencyError::FenceNotActive
    );
    let mut data = input(&request);
    let mut changed = fence();
    changed.membership_revision = hash("90");
    data.membership_revision = Some(hash("90"));
    data.consistency = ConsistencyEvidence::ApplicationCoordinated(Box::new(changed));
    assert_eq!(
        validate(&request, &ConsistencyObservation::new(data).unwrap()).unwrap_err(),
        ConsistencyError::MembershipRevisionMismatch
    );
}

#[test]
fn rejects_actual_context_full_inventory_and_missing_or_extra_selected_targets() {
    let plan = plan();
    let requirement =
        ConsistencyRequirementRecord::new(&plan, ConsistencyGuaranteeRecord::PerCanister);
    let request =
        ConsistencyRequest::new(&plan, &requirement, parameters(requirement.guarantee())).unwrap();
    for field in ["network", "caller", "release"] {
        let mut data = input(&request);
        data.context = PlanContextRecord::new(&PlanContextRequest {
            network: if field == "network" {
                "90".repeat(32)
            } else {
                plan.context().network().into()
            },
            caller: if field == "caller" {
                "aaaaa-aa".into()
            } else {
                plan.context().caller().into()
            },
            release: if field == "release" {
                "90".repeat(32)
            } else {
                plan.context().release().into()
            },
        })
        .unwrap();
        assert_eq!(
            validate(&request, &ConsistencyObservation::new(data).unwrap()).unwrap_err(),
            ConsistencyError::ContextMismatch(field)
        );
    }
    let mut data = input(&request);
    let mut inventory = serde_json::to_value(&data.inventory).unwrap();
    inventory["targets"][0]["role"] = serde_json::json!("changed-unselected-parent");
    data.inventory = serde_json::from_value(inventory).unwrap();
    assert_eq!(
        validate(&request, &ConsistencyObservation::new(data).unwrap()).unwrap_err(),
        ConsistencyError::InventoryMismatch
    );
    for extra in [false, true] {
        let mut data = input(&request);
        let mut target = data.targets[0].clone();
        target.target = "aaaaa-aa".into();
        if extra {
            data.targets.push(target);
        } else {
            data.targets = vec![target];
        }
        assert_eq!(
            validate(&request, &ConsistencyObservation::new(data).unwrap()).unwrap_err(),
            ConsistencyError::SelectionMismatch
        );
    }
}

#[test]
fn zero_and_max_call_reporting_do_not_change_original_allowances() {
    let plan = plan();
    let requirement =
        ConsistencyRequirementRecord::new(&plan, ConsistencyGuaranteeRecord::PerCanister);
    for limit in [0, 1024] {
        let mut params = parameters(requirement.guarantee());
        params.max_remote_observations = limit;
        let request = ConsistencyRequest::new(&plan, &requirement, params).unwrap();
        let mut data = input(&request);
        data.remote_observations = limit;
        let observation = ConsistencyObservation::new(data.clone()).unwrap();
        assert_eq!(
            validate(&request, &observation)
                .unwrap()
                .remote_observations(),
            limit
        );
        data.remote_observations = limit + 1;
        assert_eq!(
            validate(&request, &ConsistencyObservation::new(data).unwrap()).unwrap_err(),
            ConsistencyError::ObservationLimitExceeded {
                limit,
                reported: limit + 1
            }
        );
        assert_eq!(
            plan.attempt_authority(7).unwrap().budget().observations(),
            1
        );
    }
}

#[test]
fn every_member_of_a_multi_target_selection_requires_stopped_evidence() {
    let mut value = serde_json::to_value(plan()).unwrap();
    value["selected_targets"] =
        serde_json::json!(["aaaaa-aa", crate::test_support::membership::APP]);
    value["operations"][0]["target"] = serde_json::json!("aaaaa-aa");
    let plan = serde_json::from_value(value).unwrap();
    for guarantee in [
        ConsistencyGuaranteeRecord::PerCanister,
        ConsistencyGuaranteeRecord::ApplicationCoordinated,
    ] {
        let requirement = ConsistencyRequirementRecord::new(&plan, guarantee);
        let request = ConsistencyRequest::new(&plan, &requirement, parameters(guarantee)).unwrap();
        let original = input(&request);
        assert_eq!(original.targets.len(), 2);
        assert_eq!(
            validate(
                &request,
                &ConsistencyObservation::new(original.clone()).unwrap()
            )
            .unwrap()
            .targets()
            .len(),
            2
        );
        for index in 0..original.targets.len() {
            let mut changed = original.clone();
            changed.targets[index].state = CaptureState::Stopping;
            assert_eq!(
                validate(&request, &ConsistencyObservation::new(changed).unwrap()).unwrap_err(),
                ConsistencyError::TargetNotStopped
            );
        }
    }
}
