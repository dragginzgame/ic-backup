//! Shared original-plan fixture builders; assertions remain beside their owners.

use crate::model::{
    attempt_journal::AttemptBudgetRecord,
    execution_workflow::ExecutionWorkflowRecord,
    ic_snapshot_metadata::IcSnapshotMetadataRequest,
    operation_plan::{
        OperationPlanRecord, PlanBudgetRecord, PlannedOperationRecord, PlannedOperationRequest,
    },
};
use ic_platform::management_canister_types::ReadCanisterSnapshotMetadataResult;

pub(crate) fn values() -> ReadCanisterSnapshotMetadataResult {
    let cases: Vec<serde_json::Value> = serde_json::from_str(include_str!(
        "../../model/ic_snapshot_metadata/tests/golden.json"
    ))
    .unwrap();
    let hex = cases
        .iter()
        .find(|case| case["name"] == "data-source")
        .unwrap()["reply_hex"]
        .as_str()
        .unwrap();
    let bytes = crate::test_support::hex_bytes(hex);
    let mut values: ReadCanisterSnapshotMetadataResult =
        ic_platform::candid::decode_one(&bytes, crate::test_support::WIRE_ORACLE_LIMITS).unwrap();
    values.wasm_module_size = 64;
    values.wasm_memory_size = 64;
    values.stable_memory_size = 64;
    values
}
pub(crate) fn workflow(mutations: u32) -> ExecutionWorkflowRecord {
    let mut request = crate::test_support::operation_plan::request();
    let target = request.selected_targets[0].clone();
    request.operations = [(0, 1, 0), (7, mutations, 1)]
        .into_iter()
        .map(|(sequence, mutations, observations)| {
            PlannedOperationRecord::new(PlannedOperationRequest {
                operation_sequence: sequence,
                target: target.clone(),
                request: "77".repeat(32),
                budget: AttemptBudgetRecord::new(mutations, observations).unwrap(),
            })
            .unwrap()
        })
        .collect();
    request.budget = PlanBudgetRecord::new(mutations + 10, 10).unwrap();
    ExecutionWorkflowRecord::new(OperationPlanRecord::new(request).unwrap())
}
pub(crate) fn source_plan(source: &IcSnapshotMetadataRequest) -> OperationPlanRecord {
    let mut request = crate::test_support::execution_workflow::child_request(0);
    request.operations = vec![
        PlannedOperationRecord::new(PlannedOperationRequest {
            operation_sequence: 42,
            target: source.target().into(),
            request: source.digest().hash().into(),
            budget: AttemptBudgetRecord::new(1, 0).unwrap(),
        })
        .unwrap(),
    ];
    request.budget = PlanBudgetRecord::new(1, 0).unwrap();
    OperationPlanRecord::new(request).unwrap()
}
pub(crate) fn source() -> IcSnapshotMetadataRequest {
    IcSnapshotMetadataRequest::new("renrk-eyaaa-aaaaa-aaada-cai", &[0, 255, 17]).unwrap()
}
