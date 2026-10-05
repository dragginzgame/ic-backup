//! Original source/safety admission, exact request identities and bounded actual rows.

use super::*;
use crate::{
    model::ic_request::IcManagementRequest,
    test_support::{
        membership::{APP, hash},
        restore_safety::{binding, input, parameters, plan, requirement, source, wire},
    },
};
use serde_json::json;

#[test]
fn matches_independent_frozen_requirement_schema_goldens() {
    let schema: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../../docs/contracts/restore-safety-requirement.schema.json"
    ))
    .unwrap();
    let examples = schema["examples"].as_array().unwrap();
    let goldens = schema["x-canonical-digest"]["goldens"].as_array().unwrap();
    assert_eq!(examples.len(), goldens.len());
    for (example, golden) in examples.iter().zip(goldens) {
        let record: RestoreSafetyRequirementRecord =
            serde_json::from_value(example.clone()).unwrap();
        assert_eq!(record.digest().hash(), golden["digest"].as_str().unwrap());
    }
}

#[test]
fn strict_original_requirements_roundtrip_normalize_and_bind_all_original_fields() {
    let plan = plan();
    let source = source();
    for lane in [
        RestoreSafetyLaneRecord::NoIrreversibleEffects,
        RestoreSafetyLaneRecord::ApplicationFenced,
    ] {
        let original = requirement(&plan, &source, lane);
        let value = serde_json::to_value(&original).unwrap();
        assert_eq!(original.plan_intent(), plan.digest().hash());
        assert_eq!(original.source_plan_intent(), source.digest().hash());
        assert_eq!(original.source_artifacts(), &hash("ab"));
        assert!(
            serde_json::to_vec_pretty(&original).unwrap().len() as u64
                <= MAX_RESTORE_SAFETY_REQUIREMENT_BYTES
        );
        assert_eq!(
            serde_json::from_value::<RestoreSafetyRequirementRecord>(value.clone()).unwrap(),
            original
        );
        let mut upper = value.clone();
        upper["plan_intent"] = json!(plan.digest().hash().to_uppercase());
        assert_eq!(
            serde_json::from_value::<RestoreSafetyRequirementRecord>(upper).unwrap(),
            original
        );
        for field in [
            "version",
            "plan_intent",
            "source_plan_intent",
            "source_artifacts",
            "safety",
            "expected_fence",
        ] {
            let mut missing = value.clone();
            missing.as_object_mut().unwrap().remove(field);
            assert!(
                serde_json::from_value::<RestoreSafetyRequirementRecord>(missing).is_err(),
                "{field}"
            );
        }
        for (field, replacement) in [
            ("version", json!(2)),
            ("plan_intent", json!("bad")),
            ("safety", json!("settled")),
            ("Proven", json!(true)),
        ] {
            let mut invalid = value.clone();
            invalid[field] = replacement;
            assert!(serde_json::from_value::<RestoreSafetyRequirementRecord>(invalid).is_err());
        }
        let mut invalid = value.clone();
        invalid["expected_fence"] = match lane {
            RestoreSafetyLaneRecord::ApplicationFenced => json!(null),
            RestoreSafetyLaneRecord::NoIrreversibleEffects => json!(binding()),
        };
        assert!(serde_json::from_value::<RestoreSafetyRequirementRecord>(invalid).is_err());
    }
}
#[test]
fn original_plan_source_artifacts_and_fence_revisions_cannot_be_rebound() {
    let plan = plan();
    let source = source();
    for lane in [
        RestoreSafetyLaneRecord::NoIrreversibleEffects,
        RestoreSafetyLaneRecord::ApplicationFenced,
    ] {
        let original = requirement(&plan, &source, lane);
        let value = serde_json::to_value(&original).unwrap();
        let mut changed = serde_json::to_value(&plan).unwrap();
        changed["operations"][0]["budget"]["mutations"] = json!(2);
        changed["budget"]["mutations"] = json!(3);
        let changed: OperationPlanRecord = serde_json::from_value(changed).unwrap();
        assert!(matches!(
            original.validate_plans(&changed, &source),
            Err(RestoreSafetyRequirementError::PlanMismatch)
        ));
        let mut changed_source = serde_json::to_value(&source).unwrap();
        changed_source["context"]["caller"] = json!("2vxsx-fae");
        let changed_source = serde_json::from_value(changed_source).unwrap();
        assert!(matches!(
            original.validate_plans(&plan, &changed_source),
            Err(RestoreSafetyRequirementError::SourcePlanMismatch)
        ));
        assert_ne!(
            original.digest(),
            requirement(&plan, &changed_source, lane).digest()
        );
        let mut changed = value.clone();
        changed["source_artifacts"]["hash"] = json!(hash("34").hash());
        assert_ne!(
            original.digest(),
            serde_json::from_value::<RestoreSafetyRequirementRecord>(changed)
                .unwrap()
                .digest()
        );
        if lane == RestoreSafetyLaneRecord::ApplicationFenced {
            for field in [
                "identity",
                "membership_revision",
                "external_obligations_revision",
            ] {
                let mut changed = value.clone();
                changed["expected_fence"][field]["hash"] = json!(hash("34").hash());
                assert_ne!(
                    original.digest(),
                    serde_json::from_value::<RestoreSafetyRequirementRecord>(changed)
                        .unwrap()
                        .digest()
                );
            }
        }
    }
}
#[test]
fn same_network_release_and_source_selected_ids_are_required_without_source_caller_equality() {
    let plan = plan();
    let source = source();
    assert_ne!(plan.context().caller(), source.context().caller());
    let input = || RestoreSafetyRequirementRequest {
        source_artifacts: hash("ab"),
        safety: RestoreSafetyLaneRecord::NoIrreversibleEffects,
        expected_fence: None,
    };
    for (field, expected) in [("network", "network"), ("release", "release")] {
        let mut changed = serde_json::to_value(&source).unwrap();
        changed["context"][field] = json!("34".repeat(32));
        let changed = serde_json::from_value(changed).unwrap();
        let err = RestoreSafetyRequirementRecord::new(&plan, &changed, input()).unwrap_err();
        match (expected, err) {
            ("network", RestoreSafetyRequirementError::SourceNetworkMismatch)
            | ("release", RestoreSafetyRequirementError::SourceReleaseMismatch) => {}
            (_, error) => panic!("{error}"),
        }
    }
    let mut different = serde_json::to_value(&source).unwrap();
    different["selected_targets"] = json!(["aaaaa-aa"]);
    different["operations"][0]["target"] = json!("aaaaa-aa");
    let different = serde_json::from_value(different).unwrap();
    assert!(matches!(
        RestoreSafetyRequirementRecord::new(&plan, &different, input()),
        Err(RestoreSafetyRequirementError::SourceSelectionMismatch)
    ));
    let mut larger = serde_json::to_value(&source).unwrap();
    larger["selected_targets"] = json!([APP, "aaaaa-aa"]);
    larger["graph"]["nodes"]
        .as_array_mut()
        .unwrap()
        .push(json!({"operation_sequence":9,"depends_on":[]}));
    larger["operations"].as_array_mut().unwrap().push(json!({"operation_sequence":9,"target":"aaaaa-aa","request":"12".repeat(32),"budget":{"mutations":1,"observations":0}}));
    larger["budget"]["mutations"] = json!(2);
    let larger = serde_json::from_value(larger).unwrap();
    assert!(RestoreSafetyRequirementRecord::new(&plan, &larger, input()).is_ok());
}
#[test]
fn binds_exact_load_start_source_safety_challenge_and_ceiling() {
    let plan = plan();
    let source = source();
    let requirement = requirement(&plan, &source, RestoreSafetyLaneRecord::ApplicationFenced);
    let load = wire(IcManagementMethodRecord::LoadCanisterSnapshot);
    let start = wire(IcManagementMethodRecord::StartCanister);
    let request = RestoreSafetyRequest::new(
        &plan,
        &source,
        &requirement,
        &load,
        parameters(load.method()),
    )
    .unwrap();
    assert_eq!(
        request.binding(),
        plan.attempt_authority(7).unwrap().binding()
    );
    assert_eq!(request.inventory(), plan.inventory());
    assert_eq!(request.selected_targets(), plan.selected_targets());
    assert_eq!(request.wire(), &load);
    let other = RestoreSafetyRequest::new(
        &plan,
        &source,
        &requirement,
        &start,
        parameters(start.method()),
    )
    .unwrap();
    assert_ne!(request.digest(), other.digest());
    for variant in 0..2 {
        let mut params = parameters(load.method());
        if variant == 0 {
            params.challenge = hash("34");
        } else {
            params.max_remote_observations = 1024;
        }
        assert_ne!(
            request.digest(),
            RestoreSafetyRequest::new(&plan, &source, &requirement, &load, params)
                .unwrap()
                .digest()
        );
    }
    let mut bytes = b"ic-backup/restore-safety-requirement/v1\0".to_vec();
    bytes.extend_from_slice(plan.digest().hash().as_bytes());
    bytes.extend_from_slice(source.digest().hash().as_bytes());
    bytes.extend_from_slice(hash("ab").hash().as_bytes());
    bytes.push(1);
    for pair in ["56", "78", "90"] {
        bytes.extend_from_slice(hash(pair).hash().as_bytes());
    }
    assert_eq!(
        requirement.digest(),
        ArtifactChecksumRecord::from_bytes(&bytes)
    );
    let mut bytes = b"ic-backup/restore-safety-request/v1\0".to_vec();
    bytes.extend_from_slice(requirement.digest().hash().as_bytes());
    bytes.extend_from_slice(&7_u64.to_be_bytes());
    bytes.extend_from_slice(load.digest().hash().as_bytes());
    bytes.extend_from_slice(hash("12").hash().as_bytes());
    bytes.extend_from_slice(&1_u32.to_be_bytes());
    assert_eq!(request.digest(), ArtifactChecksumRecord::from_bytes(&bytes));
}
#[test]
fn rejects_changed_mutation_unknown_operation_other_methods_and_excessive_ceiling() {
    let plan = plan();
    let source = source();
    let requirement = requirement(
        &plan,
        &source,
        RestoreSafetyLaneRecord::NoIrreversibleEffects,
    );
    let load = wire(IcManagementMethodRecord::LoadCanisterSnapshot);
    for method in [
        IcManagementMethodRecord::CanisterStatus,
        IcManagementMethodRecord::ListCanisterSnapshots,
        IcManagementMethodRecord::StopCanister,
        IcManagementMethodRecord::TakeCanisterSnapshot,
    ] {
        let wire = IcManagementRequestRecord::new(IcManagementRequest {
            method,
            target: APP.into(),
            snapshot_id: None,
        })
        .unwrap();
        assert!(matches!(
            RestoreSafetyRequest::new(
                &plan,
                &source,
                &requirement,
                &wire,
                parameters(load.method())
            ),
            Err(RestoreSafetyRequestError::UnsupportedMethod)
        ));
    }
    let changed = IcManagementRequestRecord::new(IcManagementRequest {
        method: load.method(),
        target: APP.into(),
        snapshot_id: Some(vec![99]),
    })
    .unwrap();
    assert!(matches!(
        RestoreSafetyRequest::new(
            &plan,
            &source,
            &requirement,
            &changed,
            parameters(load.method())
        ),
        Err(RestoreSafetyRequestError::Payload(
            IcRequestError::DigestMismatch
        ))
    ));
    let changed = IcManagementRequestRecord::new(IcManagementRequest {
        method: load.method(),
        target: "aaaaa-aa".into(),
        snapshot_id: Some(vec![99]),
    })
    .unwrap();
    assert!(matches!(
        RestoreSafetyRequest::new(
            &plan,
            &source,
            &requirement,
            &changed,
            parameters(load.method())
        ),
        Err(RestoreSafetyRequestError::Payload(
            IcRequestError::TargetMismatch
        ))
    ));
    let mut params = parameters(load.method());
    params.operation_sequence = 99;
    assert!(matches!(
        RestoreSafetyRequest::new(&plan, &source, &requirement, &load, params),
        Err(RestoreSafetyRequestError::Plan(
            OperationPlanError::UnknownOperation(99)
        ))
    ));
    let mut params = parameters(load.method());
    params.max_remote_observations = 1025;
    assert!(matches!(
        RestoreSafetyRequest::new(&plan, &source, &requirement, &load, params),
        Err(RestoreSafetyRequestError::ObservationLimitTooLarge)
    ));
}
#[test]
fn admits_only_canonical_nonempty_bounded_unique_inventory_backed_rows() {
    let plan = plan();
    let source = source();
    let requirement = requirement(
        &plan,
        &source,
        RestoreSafetyLaneRecord::NoIrreversibleEffects,
    );
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
    data.targets[0].target = APP.to_uppercase();
    assert_eq!(
        RestoreSafetyObservation::new(data).unwrap().targets()[0].target,
        APP
    );
    let mut data = input(&request);
    data.targets.push(data.targets[0].clone());
    data.targets[1].target = APP.to_uppercase();
    assert_eq!(
        RestoreSafetyObservation::new(data).unwrap_err(),
        RestoreSafetyObservationError::DuplicateTarget
    );
    let mut data = input(&request);
    data.targets.clear();
    assert_eq!(
        RestoreSafetyObservation::new(data).unwrap_err(),
        RestoreSafetyObservationError::InvalidTargetCount
    );
    for (target, error) in [
        ("bad", RestoreSafetyObservationError::InvalidPrincipal),
        (
            "2vxsx-fae",
            RestoreSafetyObservationError::TargetAbsentFromInventory,
        ),
    ] {
        let mut data = input(&request);
        data.targets[0].target = target.into();
        assert_eq!(RestoreSafetyObservation::new(data).unwrap_err(), error);
    }
    let ids: Vec<_> = (0_u32..1024)
        .map(|id| ic_principal::Principal::from_slice(&id.to_be_bytes()).to_text())
        .collect();
    let inventory=serde_json::from_value(json!({"version":1,"targets":ids.iter().map(|id|json!({"canister_id":id,"parent_canister_id":null,"role":null,"module_hash":null})).collect::<Vec<_>>()})).unwrap();
    let mut data = input(&request);
    let row = data.targets[0].clone();
    data.inventory = inventory;
    data.targets = ids
        .iter()
        .rev()
        .map(|id| TargetRestoreEvidence {
            target: id.clone(),
            ..row.clone()
        })
        .collect();
    let observed = RestoreSafetyObservation::new(data.clone()).unwrap();
    assert_eq!(observed.targets().len(), MAX_INVENTORY_TARGETS);
    assert!(
        observed
            .targets()
            .windows(2)
            .all(|pair| pair[0].target < pair[1].target)
    );
    data.targets.push(row);
    assert_eq!(
        RestoreSafetyObservation::new(data).unwrap_err(),
        RestoreSafetyObservationError::InvalidTargetCount
    );
}
