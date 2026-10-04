//! Immutable original declarations, independently encoded requests and bounded targets.

use super::*;
use crate::test_support::{
    consistency::{input, parameters},
    membership::{APP, hash, plan},
};
use serde_json::json;

#[test]
fn closed_requirement_normalizes_intent_and_matches_independent_binary_goldens() {
    let plan = plan();
    for (guarantee, expected) in [
        (
            ConsistencyGuaranteeRecord::PerCanister,
            "bac43771b54e06dd441102d2cde0f76daa143b9e34fe1dfe796a13ea1af90924",
        ),
        (
            ConsistencyGuaranteeRecord::ApplicationCoordinated,
            "206da6d94c3bc11ae48d492f15c730b2b73c5ab7a44a25755eb6cf44a82b1e4c",
        ),
    ] {
        let requirement = ConsistencyRequirementRecord::new(&plan, guarantee);
        assert_eq!(requirement.plan_intent(), plan.digest().hash());
        assert_eq!(requirement.digest().hash(), expected);
        let mut value = serde_json::to_value(&requirement).unwrap();
        value["plan_intent"] = json!(plan.digest().hash().to_uppercase());
        assert_eq!(
            serde_json::from_value::<ConsistencyRequirementRecord>(value.clone()).unwrap(),
            requirement
        );
        for missing in ["version", "plan_intent", "guarantee"] {
            let mut absent = value.clone();
            absent.as_object_mut().unwrap().remove(missing);
            assert!(serde_json::from_value::<ConsistencyRequirementRecord>(absent).is_err());
        }
        for (field, replacement) in [
            ("version", json!(2)),
            ("plan_intent", json!("invalid")),
            ("guarantee", json!("root-coordinated")),
            ("accepted", json!(true)),
        ] {
            let mut bad = value.clone();
            bad[field] = replacement;
            assert!(serde_json::from_value::<ConsistencyRequirementRecord>(bad).is_err());
        }
        assert!(
            serde_json::to_vec_pretty(&requirement).unwrap().len() as u64
                <= MAX_CONSISTENCY_REQUIREMENT_BYTES
        );
    }
}

#[test]
fn request_binds_guarantee_operation_challenge_boundary_fence_and_original_allowances() {
    let plan = plan();
    for (guarantee, expected) in [
        (
            ConsistencyGuaranteeRecord::PerCanister,
            "b074f1a4e8f4154843706f25b9e55d35ec11c7e2539254685dc87b237281f482",
        ),
        (
            ConsistencyGuaranteeRecord::ApplicationCoordinated,
            "f2b614e4d09e43eee29807d8860b14c7f2d238d111546258d2be22e6e7ee8ed4",
        ),
    ] {
        let requirement = ConsistencyRequirementRecord::new(&plan, guarantee);
        let request = ConsistencyRequest::new(&plan, &requirement, parameters(guarantee)).unwrap();
        assert_eq!(request.digest().hash(), expected); // Independent Python encoding from original plan golden.
        assert_eq!(
            request.binding(),
            plan.attempt_authority(7).unwrap().binding()
        );
        assert_eq!(request.inventory(), plan.inventory());
        assert_eq!(request.selected_targets(), plan.selected_targets());
        assert_eq!(request.challenge(), &hash("12"));
        assert_eq!(request.boundary(), ConsistencyBoundary::BeforeCapture);
        assert_eq!(request.max_remote_observations(), 1);
        for variant in 0..5 {
            let mut params = parameters(guarantee);
            match variant {
                0 => params.challenge = hash("34"),
                1 => params.boundary = ConsistencyBoundary::AfterCapture,
                2 => params.operation_sequence = 0,
                3 => params.max_remote_observations = 2,
                _ => params.max_remote_observations = 0,
            }
            assert_ne!(
                request.digest(),
                ConsistencyRequest::new(&plan, &requirement, params)
                    .unwrap()
                    .digest()
            );
        }
        let mut changed = serde_json::to_value(&plan).unwrap();
        changed["budget"]["observations"] = json!(3);
        let changed: OperationPlanRecord = serde_json::from_value(changed).unwrap();
        assert!(matches!(
            requirement.validate_plan(&changed),
            Err(ConsistencyRequirementError::PlanMismatch)
        ));
        let new_requirement = ConsistencyRequirementRecord::new(&changed, guarantee);
        assert_ne!(
            request.digest(),
            ConsistencyRequest::new(&changed, &new_requirement, parameters(guarantee))
                .unwrap()
                .digest()
        );
    }
}

#[test]
fn missing_or_unexpected_fence_unknown_operation_and_excess_ceiling_reject() {
    let plan = plan();
    for guarantee in [
        ConsistencyGuaranteeRecord::PerCanister,
        ConsistencyGuaranteeRecord::ApplicationCoordinated,
    ] {
        let requirement = ConsistencyRequirementRecord::new(&plan, guarantee);
        let mut params = parameters(guarantee);
        params.operation_sequence = 99;
        assert!(matches!(
            ConsistencyRequest::new(&plan, &requirement, params),
            Err(ConsistencyRequestError::Plan(
                OperationPlanError::UnknownOperation(99)
            ))
        ));
        let mut params = parameters(guarantee);
        params.max_remote_observations = 1025;
        assert!(matches!(
            ConsistencyRequest::new(&plan, &requirement, params),
            Err(ConsistencyRequestError::ObservationLimitTooLarge)
        ));
        for limit in [0, MAX_CONSISTENCY_REMOTE_OBSERVATIONS] {
            let mut params = parameters(guarantee);
            params.max_remote_observations = limit;
            assert_eq!(
                ConsistencyRequest::new(&plan, &requirement, params)
                    .unwrap()
                    .max_remote_observations(),
                limit
            );
        }
        let mut params = parameters(guarantee);
        params.expected_fence = match guarantee {
            ConsistencyGuaranteeRecord::PerCanister => Some(ApplicationFenceBinding {
                identity: hash("56"),
                membership_revision: hash("78"),
            }),
            ConsistencyGuaranteeRecord::ApplicationCoordinated => None,
        };
        let error = ConsistencyRequest::new(&plan, &requirement, params).unwrap_err();
        assert!(matches!(
            (guarantee, error),
            (
                ConsistencyGuaranteeRecord::PerCanister,
                ConsistencyRequestError::UnexpectedFence
            ) | (
                ConsistencyGuaranteeRecord::ApplicationCoordinated,
                ConsistencyRequestError::FenceRequired
            )
        ));
    }
}

#[test]
fn canonical_target_evidence_rejects_empty_duplicate_malformed_unknown_and_excessive_sets() {
    let plan = plan();
    let requirement =
        ConsistencyRequirementRecord::new(&plan, ConsistencyGuaranteeRecord::PerCanister);
    let request =
        ConsistencyRequest::new(&plan, &requirement, parameters(requirement.guarantee())).unwrap();
    let mut data = input(&request);
    data.targets[0].target = APP.to_uppercase();
    assert_eq!(
        ConsistencyObservation::new(data).unwrap().targets()[0].target,
        APP
    );
    let mut data = input(&request);
    data.targets.clear();
    assert_eq!(
        ConsistencyObservation::new(data).unwrap_err(),
        ConsistencyObservationError::InvalidTargetCount
    );
    let mut data = input(&request);
    let mut alias = data.targets[0].clone();
    alias.target = APP.to_uppercase();
    data.targets.push(alias);
    assert_eq!(
        ConsistencyObservation::new(data).unwrap_err(),
        ConsistencyObservationError::DuplicateTarget
    );
    let mut data = input(&request);
    data.targets[0].target = "invalid".into();
    assert_eq!(
        ConsistencyObservation::new(data).unwrap_err(),
        ConsistencyObservationError::InvalidPrincipal
    );
    let mut data = input(&request);
    data.targets[0].target = "2vxsx-fae".into();
    assert_eq!(
        ConsistencyObservation::new(data).unwrap_err(),
        ConsistencyObservationError::TargetAbsentFromInventory
    );
    let mut data = input(&request);
    data.targets = vec![data.targets[0].clone(); MAX_INVENTORY_TARGETS + 1];
    assert_eq!(
        ConsistencyObservation::new(data).unwrap_err(),
        ConsistencyObservationError::InvalidTargetCount
    );
}

#[test]
fn admits_maximum_canonical_actual_target_set_in_reordered_input() {
    let plan = plan();
    let requirement =
        ConsistencyRequirementRecord::new(&plan, ConsistencyGuaranteeRecord::PerCanister);
    let request =
        ConsistencyRequest::new(&plan, &requirement, parameters(requirement.guarantee())).unwrap();
    let mut data = input(&request);
    let ids: Vec<_> = (0_u32..1024)
        .map(|id| ic_principal::Principal::from_slice(&id.to_be_bytes()).to_text())
        .collect();
    data.inventory = InventoryRecord::new(
        ids.iter()
            .map(|id| {
                crate::model::inventory::InventoryTargetRecord::new(
                    &crate::model::inventory::InventoryTargetRequest {
                        canister_id: id.clone(),
                        parent_canister_id: None,
                        role: None,
                        module_hash: None,
                    },
                )
                .unwrap()
            })
            .collect(),
    )
    .unwrap();
    data.targets = ids
        .into_iter()
        .rev()
        .map(|target| TargetCaptureEvidence {
            target,
            state: CaptureState::Stopped,
            stopped_and_drained: hash("ef"),
        })
        .collect();
    let observation = ConsistencyObservation::new(data).unwrap();
    assert_eq!(observation.targets().len(), MAX_INVENTORY_TARGETS);
    assert!(
        observation
            .targets()
            .windows(2)
            .all(|pair| pair[0].target < pair[1].target)
    );
}
