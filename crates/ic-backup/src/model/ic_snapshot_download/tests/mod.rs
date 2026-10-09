use super::*;
use crate::model::{
    artifacts::ArtifactChecksumRecord,
    ic_snapshot_metadata::{IcSnapshotMetadataReply, IcSnapshotMetadataRequest},
};
use ic_management_canister_types::{ChunkHash, ReadCanisterSnapshotMetadataResult};
use sha2::Digest;

pub(crate) fn values() -> ReadCanisterSnapshotMetadataResult {
    let cases: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../../ic_snapshot_metadata/tests/golden.json")).unwrap();
    let hex = cases
        .iter()
        .find(|case| case["name"] == "data-source")
        .unwrap()["reply_hex"]
        .as_str()
        .unwrap();
    let bytes: Vec<u8> = hex
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect();
    let mut values: ReadCanisterSnapshotMetadataResult = candid::decode_one(&bytes).unwrap();
    values.wasm_module_size = 64;
    values.wasm_memory_size = 64;
    values.stable_memory_size = 64;
    values
}
pub(crate) fn workflow(mutations: u32) -> ExecutionWorkflowRecord {
    let mut request = crate::model::operation_plan::tests::request();
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
    let mut request = crate::model::execution_workflow::tests::child_request(0);
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
fn predecessor(
    binding: &ExecutionStageBindingRecord,
    metadata: &IcSnapshotMetadataReply<'_>,
) -> ExecutionStagePredecessorRecord {
    ExecutionStagePredecessorRecord::new(
        0,
        binding.digest(),
        ArtifactChecksumRecord::from_bytes(b"settlement"),
        metadata.digest(),
    )
}

#[test]
fn exact_complete_payloads_reuse_original_allocation_and_chain_before_calls() {
    let workflow = workflow(12);
    let request = source();
    let mut values = values();
    values.wasm_module_size = 5;
    values.wasm_memory_size = 3;
    values.stable_memory_size = 0;
    values.wasm_chunk_store = vec![
        ChunkHash { hash: vec![1; 32] },
        ChunkHash { hash: vec![2; 32] },
    ];
    let metadata =
        IcSnapshotMetadataReply::decode(&request, &candid::encode_one(values).unwrap()).unwrap();
    let download = IcSnapshotDownloadPlan::new(&workflow, 7, &metadata, 4).unwrap();
    let expected = [
        SnapshotDataKind::WasmModule { offset: 0, size: 4 },
        SnapshotDataKind::WasmModule { offset: 4, size: 1 },
        SnapshotDataKind::WasmMemory { offset: 0, size: 3 },
        SnapshotDataKind::WasmChunk { hash: vec![1; 32] },
        SnapshotDataKind::WasmChunk { hash: vec![2; 32] },
    ];
    assert_eq!(download.requests().len(), expected.len());
    let plan = download.plan().unwrap();
    assert_eq!(plan.budget().mutations(), 12);
    assert_eq!(plan.budget().observations(), 1);
    assert_eq!(plan.allocated_attempts().mutations, 5);
    assert_eq!(plan.allocated_attempts().observations, 0);
    for (index, (payload, kind)) in download.requests().iter().zip(expected).enumerate() {
        assert_eq!(
            candid::encode_one(payload.kind()).unwrap(),
            candid::encode_one(kind).unwrap()
        );
        let sequence = u64::try_from(index).unwrap();
        assert_eq!(
            plan.operation(sequence).unwrap().request(),
            payload.digest().hash()
        );
        assert_eq!(
            plan.graph().node(sequence).unwrap().depends_on(),
            sequence.checked_sub(1).into_iter().collect::<Vec<_>>()
        );
        assert_eq!(payload.snapshot_id(), request.snapshot_id());
    }
    let original = source_plan(&request);
    let binding = ExecutionStageBindingRecord::new(&workflow, 0, &original, vec![]).unwrap();
    let learned = download
        .bind(&binding, &original, vec![predecessor(&binding, &metadata)])
        .unwrap();
    learned.validate(&workflow, plan).unwrap();
    assert_eq!(
        IcSnapshotDownloadPlan::new(&workflow, 7, &metadata, 4)
            .unwrap()
            .plan(),
        Some(plan)
    );
}

#[test]
fn exact_budget_thresholds_and_extreme_nat64_reject_before_request_allocation() {
    let request = source();
    let mut values = values();
    values.wasm_chunk_store.clear();
    values.wasm_module_size = 8;
    values.wasm_memory_size = 0;
    values.stable_memory_size = 0;
    let metadata =
        IcSnapshotMetadataReply::decode(&request, &candid::encode_one(&values).unwrap()).unwrap();
    let admitted = workflow(2);
    assert_eq!(
        IcSnapshotDownloadPlan::new(&admitted, 7, &metadata, 4)
            .unwrap()
            .requests()
            .len(),
        2
    );
    let too_small = workflow(1);
    assert!(matches!(
        IcSnapshotDownloadPlan::new(&too_small, 7, &metadata, 4),
        Err(IcSnapshotDownloadPlanningError::InsufficientAllowance {
            required: 2,
            original: 1
        })
    ));
    for chunk in [0, 1024 * 1024 + 1, u64::MAX] {
        assert!(matches!(
            IcSnapshotDownloadPlan::new(&admitted, 7, &metadata, chunk),
            Err(IcSnapshotDownloadPlanningError::InvalidChunkSize)
        ));
    }
    values.wasm_module_size = u64::MAX;
    values.wasm_memory_size = u64::MAX;
    let huge =
        IcSnapshotMetadataReply::decode(&request, &candid::encode_one(values).unwrap()).unwrap();
    assert!(matches!(
        IcSnapshotDownloadPlan::new(&admitted, 7, &huge, 1),
        Err(IcSnapshotDownloadPlanningError::CountOverflow)
    ));
    assert!(matches!(
        IcSnapshotDownloadPlan::new(&admitted, 7, &huge, 1024 * 1024),
        Err(IcSnapshotDownloadPlanningError::InsufficientAllowance { .. })
    ));
}

#[test]
fn empty_regions_need_no_placeholder_but_known_empty_chunks_still_need_reads() {
    let workflow = workflow(1);
    let request = source();
    let mut values = values();
    values.wasm_module_size = 0;
    values.wasm_memory_size = 0;
    values.stable_memory_size = 0;
    values.wasm_chunk_store.clear();
    let metadata =
        IcSnapshotMetadataReply::decode(&request, &candid::encode_one(&values).unwrap()).unwrap();
    let empty = IcSnapshotDownloadPlan::new(&workflow, 7, &metadata, 1024 * 1024).unwrap();
    assert!(empty.requests().is_empty());
    assert!(empty.plan().is_none());
    let original = source_plan(&request);
    let binding = ExecutionStageBindingRecord::new(&workflow, 0, &original, vec![]).unwrap();
    assert!(matches!(
        empty.bind(&binding, &original, vec![predecessor(&binding, &metadata)]),
        Err(IcSnapshotDownloadPlanningError::NoDataReads)
    ));
    values.wasm_chunk_store.push(ChunkHash {
        hash: sha2::Digest::finalize(sha2::Sha256::new()).to_vec(),
    });
    let metadata =
        IcSnapshotMetadataReply::decode(&request, &candid::encode_one(values).unwrap()).unwrap();
    assert_eq!(
        IcSnapshotDownloadPlan::new(&workflow, 7, &metadata, 1024 * 1024)
            .unwrap()
            .requests()
            .len(),
        1
    );
}

#[test]
fn changed_metadata_input_or_source_request_cannot_bind_the_same_download() {
    let workflow = workflow(12);
    let request = source();
    let metadata =
        IcSnapshotMetadataReply::decode(&request, &candid::encode_one(values()).unwrap()).unwrap();
    let download = IcSnapshotDownloadPlan::new(&workflow, 7, &metadata, 1024 * 1024).unwrap();
    let original = source_plan(&request);
    let binding = ExecutionStageBindingRecord::new(&workflow, 0, &original, vec![]).unwrap();
    let wrong = ExecutionStagePredecessorRecord::new(
        0,
        binding.digest(),
        ArtifactChecksumRecord::from_bytes(b"settlement"),
        ArtifactChecksumRecord::from_bytes(b"different metadata"),
    );
    assert!(matches!(
        download.bind(&binding, &original, vec![wrong]),
        Err(IcSnapshotDownloadPlanningError::MetadataMismatch)
    ));
    let other_request = IcSnapshotMetadataRequest::new(request.target(), &[9]).unwrap();
    let other_plan = source_plan(&other_request);
    let other_binding =
        ExecutionStageBindingRecord::new(&workflow, 0, &other_plan, vec![]).unwrap();
    assert!(matches!(
        download.bind(
            &other_binding,
            &other_plan,
            vec![predecessor(&other_binding, &metadata)]
        ),
        Err(IcSnapshotDownloadPlanningError::MetadataMismatch)
    ));
}

#[test]
fn wrong_target_unknown_stage_and_missing_original_predecessor_reject() {
    let workflow = workflow(12);
    let request = source();
    let raw = candid::encode_one(values()).unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    assert!(matches!(
        IcSnapshotDownloadPlan::new(&workflow, 99, &metadata, 1024 * 1024),
        Err(IcSnapshotDownloadPlanningError::Plan(_))
    ));
    let other_request = IcSnapshotMetadataRequest::new("2vxsx-fae", &[0, 255, 17]).unwrap();
    let other_metadata = IcSnapshotMetadataReply::decode(&other_request, &raw).unwrap();
    assert!(matches!(
        IcSnapshotDownloadPlan::new(&workflow, 7, &other_metadata, 1024 * 1024),
        Err(IcSnapshotDownloadPlanningError::MetadataMismatch)
    ));
    let download = IcSnapshotDownloadPlan::new(&workflow, 7, &metadata, 1024 * 1024).unwrap();
    let original = source_plan(&request);
    let binding = ExecutionStageBindingRecord::new(&workflow, 0, &original, vec![]).unwrap();
    assert!(matches!(
        download.bind(&binding, &original, vec![]),
        Err(IcSnapshotDownloadPlanningError::MetadataMismatch)
    ));
}
