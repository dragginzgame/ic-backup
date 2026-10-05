use super::*;
use crate::test_support::{
    membership::{hash, plan},
    restore_safety,
};
use serde_json::json;

fn capture(plan: &OperationPlanRecord) -> ConsistencyRequirementRecord {
    ConsistencyRequirementRecord::new(plan, ConsistencyGuaranteeRecord::ApplicationCoordinated)
}
fn fence() -> ApplicationFenceBinding {
    ApplicationFenceBinding {
        identity: hash("56"),
        membership_revision: hash("78"),
    }
}
#[test]
fn strict_schema_and_independent_binary_goldens_admit_only_original_fields() {
    let schema: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../../docs/contracts/fence-obligation.schema.json"
    ))
    .unwrap();
    for sample in schema["x-canonical-digest"].as_array().unwrap() {
        let record: FenceObligationRecord =
            serde_json::from_value(sample["record"].clone()).unwrap();
        assert_eq!(record.digest().hash(), sample["sha256"].as_str().unwrap());
        let mut value = sample["record"].clone();
        value["plan_intent"]["hash"] = json!(
            value["plan_intent"]["hash"]
                .as_str()
                .unwrap()
                .to_uppercase()
        );
        assert_eq!(
            serde_json::from_value::<FenceObligationRecord>(value).unwrap(),
            record
        );
        for field in ["version", "plan_intent", "acquisition_operation", "scope"] {
            let mut value = sample["record"].clone();
            value.as_object_mut().unwrap().remove(field);
            assert!(serde_json::from_value::<FenceObligationRecord>(value).is_err());
        }
        for location in ["", "/scope", "/plan_intent"] {
            let mut value = sample["record"].clone();
            value.pointer_mut(location).unwrap()["released"] = json!(true);
            assert!(serde_json::from_value::<FenceObligationRecord>(value).is_err());
        }
        let mut value = sample["record"].clone();
        value["version"] = json!(2);
        assert!(serde_json::from_value::<FenceObligationRecord>(value).is_err());
        let mut value = sample["record"].clone();
        value["scope"]["purpose"] = json!("released");
        assert!(serde_json::from_value::<FenceObligationRecord>(value).is_err());
        let mut value = sample["record"].clone();
        value["plan_intent"]["hash"] = json!("invalid");
        assert!(serde_json::from_value::<FenceObligationRecord>(value).is_err());
    }
}
#[test]
fn capture_binds_full_original_plan_requirement_fence_and_explicit_acquisition() {
    let plan = plan();
    let requirement = capture(&plan);
    let record = FenceObligationRecord::for_capture(&plan, &requirement, 0, &fence()).unwrap();
    record
        .validate_capture(&plan, &requirement, &fence())
        .unwrap();
    assert_eq!(record.plan_intent(), &plan.digest());
    assert_eq!(record.acquisition_operation(), 0);
    let changed = ApplicationFenceBinding {
        identity: hash("90"),
        ..fence()
    };
    assert!(matches!(
        record.validate_capture(&plan, &requirement, &changed),
        Err(FenceObligationError::BindingMismatch)
    ));
    let changed = ApplicationFenceBinding {
        membership_revision: hash("90"),
        ..fence()
    };
    assert!(
        record
            .validate_capture(&plan, &requirement, &changed)
            .is_err()
    );
    let weaker = ConsistencyRequirementRecord::new(&plan, ConsistencyGuaranteeRecord::PerCanister);
    assert!(matches!(
        FenceObligationRecord::for_capture(&plan, &weaker, 0, &fence()),
        Err(FenceObligationError::FenceNotRequired)
    ));
    assert!(matches!(
        FenceObligationRecord::for_capture(&plan, &requirement, 99, &fence()),
        Err(FenceObligationError::Plan(_))
    ));
    for pointer in [
        "/context/network",
        "/context/release",
        "/operations/0/request",
    ] {
        let mut value = serde_json::to_value(&plan).unwrap();
        *value.pointer_mut(pointer).unwrap() = json!(hash("90").hash());
        let changed = serde_json::from_value(value).unwrap();
        assert!(matches!(
            record.validate_plan(&changed),
            Err(FenceObligationError::PlanMismatch)
        ));
    }
}
#[test]
fn zero_original_mutation_allowance_cannot_declare_acquisition() {
    let mut value = serde_json::to_value(plan()).unwrap();
    value["operations"][0]["budget"]["mutations"] = json!(0);
    let plan = serde_json::from_value(value).unwrap();
    assert!(matches!(
        FenceObligationRecord::for_capture(&plan, &capture(&plan), 0, &fence()),
        Err(FenceObligationError::NoAcquisitionAllowance)
    ));
}
#[test]
fn restore_derives_exact_source_and_fence_revisions_from_original_requirement() {
    let plan = plan();
    let source = restore_safety::source();
    let requirement =
        restore_safety::requirement(&plan, &source, RestoreSafetyLaneRecord::ApplicationFenced);
    let record = FenceObligationRecord::for_restore(&plan, &source, &requirement, 0).unwrap();
    record
        .validate_restore(&plan, &source, &requirement)
        .unwrap();
    for pointer in [
        "/source_artifacts/hash",
        "/expected_fence/identity/hash",
        "/expected_fence/membership_revision/hash",
        "/expected_fence/external_obligations_revision/hash",
    ] {
        let mut value = serde_json::to_value(&requirement).unwrap();
        *value.pointer_mut(pointer).unwrap() = json!(hash("ef").hash());
        let changed = serde_json::from_value(value).unwrap();
        assert!(matches!(
            record.validate_restore(&plan, &source, &changed),
            Err(FenceObligationError::BindingMismatch)
        ));
    }
    let weaker = restore_safety::requirement(
        &plan,
        &source,
        RestoreSafetyLaneRecord::NoIrreversibleEffects,
    );
    assert!(matches!(
        FenceObligationRecord::for_restore(&plan, &source, &weaker, 0),
        Err(FenceObligationError::FenceNotRequired)
    ));
    assert!(
        record
            .validate_capture(&plan, &capture(&plan), &fence())
            .is_err()
    );
}
