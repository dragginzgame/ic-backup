use super::*;
use crate::model::{
    attempt_journal::AttemptBudgetRecord,
    effect_graph::{EffectGraphRecord, EffectNodeRecord, EffectNodeRequest},
    operation_plan::{OperationPlanRequest, PlanBudgetRecord, PlannedOperationRequest},
};

pub(crate) fn workflow() -> ExecutionWorkflowRecord {
    ExecutionWorkflowRecord::new(
        OperationPlanRecord::new(crate::model::operation_plan::tests::request()).unwrap(),
    )
}
pub(crate) fn child_request(sequence: u64) -> OperationPlanRequest {
    let mut request = crate::model::operation_plan::tests::request();
    let stage = workflow().stage(sequence).unwrap().clone();
    request.graph = EffectGraphRecord::new(vec![
        EffectNodeRecord::new(EffectNodeRequest {
            operation_sequence: 42,
            depends_on: vec![],
        })
        .unwrap(),
    ])
    .unwrap();
    request.operations = vec![
        PlannedOperationRecord::new(PlannedOperationRequest {
            operation_sequence: 42,
            target: stage.target().into(),
            request: "78".repeat(32),
            budget: stage.budget().clone(),
        })
        .unwrap(),
    ];
    request.budget =
        PlanBudgetRecord::new(stage.budget().mutations(), stage.budget().observations()).unwrap();
    request
}
pub(crate) fn child(sequence: u64) -> OperationPlanRecord {
    OperationPlanRecord::new(child_request(sequence)).unwrap()
}
fn predecessor() -> ExecutionStagePredecessorRecord {
    ExecutionStagePredecessorRecord::new(
        0,
        ArtifactChecksumRecord::from_bytes(b"binding"),
        ArtifactChecksumRecord::from_bytes(b"settlement"),
        ArtifactChecksumRecord::from_bytes(b"learned"),
    )
}

#[test]
fn distinct_canonical_catalog_and_exact_learned_binding_retain_original_budgets() {
    let workflow = workflow();
    let plan = child(7);
    let binding =
        ExecutionStageBindingRecord::new(&workflow, 7, &plan, vec![predecessor()]).unwrap();
    assert_ne!(workflow.digest(), workflow.allocation.digest());
    assert_ne!(binding.digest(), plan.digest());
    assert_eq!(
        serde_json::from_slice::<ExecutionWorkflowRecord>(&serde_json::to_vec(&workflow).unwrap())
            .unwrap(),
        workflow
    );
    let decoded: ExecutionStageBindingRecord =
        serde_json::from_slice(&serde_json::to_vec(&binding).unwrap()).unwrap();
    assert_eq!(decoded, binding);
    decoded.validate(&workflow, &plan).unwrap();
    assert_eq!(
        plan.attempt_authority(42).unwrap().budget(),
        workflow.stage(7).unwrap().budget()
    );
    let mut bytes = b"ic-backup/execution-stage/v1\0".to_vec();
    bytes.extend_from_slice(workflow.digest().hash().as_bytes());
    bytes.extend_from_slice(&7_u64.to_be_bytes());
    bytes.extend_from_slice(plan.digest().hash().as_bytes());
    bytes.extend_from_slice(&1_u64.to_be_bytes());
    let row = predecessor();
    bytes.extend_from_slice(&0_u64.to_be_bytes());
    for digest in [row.binding(), row.settlement(), row.learned_evidence()] {
        bytes.extend_from_slice(digest.hash().as_bytes());
    }
    assert_eq!(binding.digest(), ArtifactChecksumRecord::from_bytes(&bytes));
}

#[test]
fn child_cannot_change_original_identity_or_borrow_unassigned_headroom() {
    let workflow = workflow();
    let mut request = child_request(0);
    request.budget = PlanBudgetRecord::new(2, 1).unwrap();
    assert_eq!(
        ExecutionStageBindingRecord::new(
            &workflow,
            0,
            &OperationPlanRecord::new(request).unwrap(),
            vec![]
        ),
        Err(ExecutionWorkflowError::ChildBudgetMismatch)
    );
    let mut request = child_request(0);
    let mut context = serde_json::to_value(&request.context).unwrap();
    context["network"] = serde_json::json!("90".repeat(32));
    request.context = serde_json::from_value(context).unwrap();
    assert_eq!(
        ExecutionStageBindingRecord::new(
            &workflow,
            0,
            &OperationPlanRecord::new(request).unwrap(),
            vec![]
        ),
        Err(ExecutionWorkflowError::ChildIdentityMismatch)
    );
    let mut request = child_request(0);
    request.selected_targets = vec!["aaaaa-aa".into()];
    request.operations = vec![
        PlannedOperationRecord::new(PlannedOperationRequest {
            operation_sequence: 42,
            target: "aaaaa-aa".into(),
            request: "78".repeat(32),
            budget: AttemptBudgetRecord::new(1, 1).unwrap(),
        })
        .unwrap(),
    ];
    assert_eq!(
        ExecutionStageBindingRecord::new(
            &workflow,
            0,
            &OperationPlanRecord::new(request).unwrap(),
            vec![]
        ),
        Err(ExecutionWorkflowError::ChildIdentityMismatch)
    );
    let mut request = child_request(0);
    request.operations = vec![
        PlannedOperationRecord::new(PlannedOperationRequest {
            operation_sequence: 42,
            target: request.selected_targets[0].clone(),
            request: "78".repeat(32),
            budget: AttemptBudgetRecord::new(0, 0).unwrap(),
        })
        .unwrap(),
    ];
    assert_eq!(
        ExecutionStageBindingRecord::new(
            &workflow,
            0,
            &OperationPlanRecord::new(request).unwrap(),
            vec![]
        ),
        Err(ExecutionWorkflowError::UnallocatedOperation)
    );
}

#[test]
fn dependencies_are_exact_and_changed_requests_or_learned_inputs_change_binding() {
    let workflow = workflow();
    let plan = child(7);
    for rows in [
        vec![],
        vec![predecessor(), predecessor()],
        vec![ExecutionStagePredecessorRecord::new(
            42,
            predecessor().binding,
            predecessor().settlement,
            predecessor().learned_evidence,
        )],
    ] {
        assert_eq!(
            ExecutionStageBindingRecord::new(&workflow, 7, &plan, rows),
            Err(ExecutionWorkflowError::PredecessorMismatch)
        );
    }
    assert!(matches!(
        ExecutionStageBindingRecord::new(&workflow, 9, &plan, vec![]),
        Err(ExecutionWorkflowError::UnknownStage(9))
    ));
    let binding =
        ExecutionStageBindingRecord::new(&workflow, 7, &plan, vec![predecessor()]).unwrap();
    let mut changed = serde_json::to_value(&plan).unwrap();
    changed["operations"][0]["request"] = serde_json::json!("90".repeat(32));
    let changed: OperationPlanRecord = serde_json::from_value(changed).unwrap();
    assert_eq!(
        binding.validate(&workflow, &changed),
        Err(ExecutionWorkflowError::BindingMismatch)
    );
    let mut row = predecessor();
    row.learned_evidence = ArtifactChecksumRecord::from_bytes(b"different ID/dimensions");
    assert_ne!(
        ExecutionStageBindingRecord::new(&workflow, 7, &plan, vec![row])
            .unwrap()
            .digest(),
        binding.digest()
    );
}

#[test]
fn strict_version_fields_and_bounded_predecessor_decode_reject() {
    let binding =
        ExecutionStageBindingRecord::new(&workflow(), 7, &child(7), vec![predecessor()]).unwrap();
    let original = serde_json::to_value(binding).unwrap();
    for field in ["version", "unknown", "predecessors"] {
        let mut value = original.clone();
        value[field] = match field {
            "version" => serde_json::json!(2),
            "unknown" => serde_json::json!(true),
            _ => serde_json::json!(vec![predecessor(); MAX_EFFECT_DEPENDENCIES + 1]),
        };
        assert!(serde_json::from_value::<ExecutionStageBindingRecord>(value).is_err());
    }
    let mut value = serde_json::to_value(workflow()).unwrap();
    value["version"] = serde_json::json!(2);
    assert!(serde_json::from_value::<ExecutionWorkflowRecord>(value).is_err());
}

#[test]
fn independent_schema_binary_goldens_match_exact_original_records() {
    let schema: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../../docs/contracts/execution-workflow.schema.json"
    ))
    .unwrap();
    let golden = &schema["x-canonical-digest"]["golden-vector"];
    let retained: ExecutionWorkflowRecord =
        serde_json::from_value(golden["record"].clone()).unwrap();
    assert_eq!(retained, workflow());
    assert_eq!(
        retained.digest().hash(),
        golden["workflow-sha256"].as_str().unwrap()
    );
    let schema: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../../docs/contracts/execution-stage.schema.json"
    ))
    .unwrap();
    let golden = &schema["x-canonical-digest"]["golden-vector"];
    let plan: OperationPlanRecord = serde_json::from_value(golden["child-plan"].clone()).unwrap();
    assert_eq!(plan, child(7));
    assert_eq!(
        plan.digest().hash(),
        golden["child-plan-sha256"].as_str().unwrap()
    );
    let binding: ExecutionStageBindingRecord =
        serde_json::from_value(golden["record"].clone()).unwrap();
    binding.validate(&retained, &plan).unwrap();
    assert_eq!(
        binding,
        ExecutionStageBindingRecord::new(&retained, 7, &plan, vec![predecessor()]).unwrap()
    );
    assert_eq!(
        binding.digest().hash(),
        golden["stage-sha256"].as_str().unwrap()
    );
}
