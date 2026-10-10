//! Original allowance thresholds, canonical complete extents and exact learned binding.
use super::*;
use crate::{
    model::ic_snapshot_metadata::{IcSnapshotMetadataReply, IcSnapshotMetadataRequest},
    test_support::ic_snapshot_upload::{
        SOURCE_ID, TARGET, data_workflow, raw, source_plan, unhex, upload_plan,
    },
};
use ic_management_canister_types::{ChunkHash, ReadCanisterSnapshotMetadataResult};

fn with_values(
    values: ReadCanisterSnapshotMetadataResult,
    check: impl FnOnce(&IcSnapshotUploadRequest<'_>),
) {
    let source = source_plan();
    let request = IcSnapshotMetadataRequest::new(TARGET, SOURCE_ID).unwrap();
    let metadata =
        IcSnapshotMetadataReply::decode(&request, &candid::encode_one(values).unwrap()).unwrap();
    let upload = IcSnapshotUploadRequest::metadata(
        &source,
        &metadata,
        &ArtifactChecksumRecord::from_bytes(b"original tree"),
    )
    .unwrap();
    check(&upload);
}
fn values() -> ReadCanisterSnapshotMetadataResult {
    candid::decode_one(&raw("upload-source")).unwrap()
}
fn allocation<'a, 'b>(payload: &'a IcSnapshotUploadRequest<'b>) -> IcSnapshotUploadReply<'a, 'b> {
    IcSnapshotUploadReply::decode(
        payload,
        &unhex("4449444c026c01b6b897890f016d7b0100031500ff"),
    )
    .unwrap()
}
#[test]
fn complete_exact_extents_keep_stage_headroom_and_allocation_reply_binding() {
    let mut values = values();
    values.wasm_module_size = 5;
    values.wasm_memory_size = 3;
    values.stable_memory_size = 0;
    values.wasm_chunk_store = vec![
        ChunkHash { hash: vec![1; 32] },
        ChunkHash { hash: vec![2; 32] },
    ];
    with_values(values, |metadata| {
        let workflow = data_workflow(metadata, 12);
        let reply = allocation(metadata);
        let kinds = IcSnapshotDataUploadPlan::derive_kinds(&workflow, 7, &reply, 4).unwrap();
        let expected = vec![
            SnapshotDataKind::WasmModule { offset: 0, size: 4 },
            SnapshotDataKind::WasmModule { offset: 4, size: 1 },
            SnapshotDataKind::WasmMemory { offset: 0, size: 3 },
            SnapshotDataKind::WasmChunk { hash: vec![1; 32] },
            SnapshotDataKind::WasmChunk { hash: vec![2; 32] },
        ];
        assert_eq!(
            candid::encode_one(&kinds).unwrap(),
            candid::encode_one(expected).unwrap()
        );
        let bindings: Vec<_> = (0..kinds.len())
            .map(|i| ArtifactChecksumRecord::from_bytes(&i.to_le_bytes()))
            .collect();
        let data =
            IcSnapshotDataUploadPlan::from_bindings(&workflow, 7, &reply, 4, &bindings).unwrap();
        assert_eq!(data.destination(), &[21, 0, 255]);
        let plan = data.plan().unwrap();
        assert_eq!(plan.budget().mutations(), 12);
        assert_eq!(plan.budget().observations(), 1);
        assert_eq!(plan.allocated_attempts().mutations, 5);
        assert_eq!(plan.allocated_attempts().observations, 0);
        for (i, binding) in bindings.iter().enumerate() {
            let id = u64::try_from(i).unwrap();
            assert_eq!(plan.operation(id).unwrap().request(), binding.hash());
            assert_eq!(
                plan.graph().node(id).unwrap().depends_on(),
                id.checked_sub(1).into_iter().collect::<Vec<_>>()
            );
        }
        let mut original = serde_json::to_value(upload_plan(metadata)).unwrap();
        original["operations"][0]["operation_sequence"] = serde_json::json!(0);
        original["graph"]["nodes"][0]["operation_sequence"] = serde_json::json!(0);
        original["operations"][0]["budget"]["observations"] = serde_json::json!(0);
        original["budget"]["observations"] = serde_json::json!(0);
        let original: OperationPlanRecord = serde_json::from_value(original).unwrap();
        let binding = ExecutionStageBindingRecord::new(&workflow, 0, &original, vec![]).unwrap();
        let predecessor = ExecutionStagePredecessorRecord::new(
            0,
            binding.digest(),
            ArtifactChecksumRecord::from_bytes(b"qualified settlement"),
            reply.digest(),
        );
        let bound = data.bind(&binding, &original, vec![predecessor]).unwrap();
        data.validate_binding(&bound).unwrap();
        assert!(data.bind(&binding, &original, vec![]).is_err());
        let wrong = ExecutionStagePredecessorRecord::new(
            0,
            binding.digest(),
            ArtifactChecksumRecord::from_bytes(b"qualified settlement"),
            ArtifactChecksumRecord::from_bytes(b"other reply"),
        );
        assert!(matches!(
            data.bind(&binding, &original, vec![wrong]),
            Err(IcSnapshotDataUploadPlanningError::OriginalMismatch)
        ));
        assert!(
            IcSnapshotDataUploadPlan::from_bindings(&workflow, 7, &reply, 4, &bindings[..4])
                .is_err()
        );
    });
}
#[test]
fn complete_count_rejects_insufficient_original_allowance_and_extreme_nat64() {
    let mut input = values();
    input.wasm_chunk_store.clear();
    input.wasm_module_size = 8;
    input.wasm_memory_size = 0;
    input.stable_memory_size = 0;
    with_values(input.clone(), |metadata| {
        let reply = allocation(metadata);
        assert_eq!(
            IcSnapshotDataUploadPlan::derive_kinds(&data_workflow(metadata, 2), 7, &reply, 4)
                .unwrap()
                .len(),
            2
        );
        assert!(matches!(
            IcSnapshotDataUploadPlan::derive_kinds(&data_workflow(metadata, 1), 7, &reply, 4),
            Err(IcSnapshotDataUploadPlanningError::InsufficientAllowance {
                required: 2,
                original: 1
            })
        ));
        for chunk in [0, 1024 * 1024 + 1, u64::MAX] {
            assert!(matches!(
                IcSnapshotDataUploadPlan::derive_kinds(
                    &data_workflow(metadata, 2),
                    7,
                    &reply,
                    chunk
                ),
                Err(IcSnapshotDataUploadPlanningError::InvalidChunkSize)
            ));
        }
        assert!(matches!(
            IcSnapshotDataUploadPlan::derive_kinds(&data_workflow(metadata, 2), 99, &reply, 4),
            Err(IcSnapshotDataUploadPlanningError::Plan(_))
        ));
    });
    input.wasm_module_size = u64::MAX;
    input.wasm_memory_size = u64::MAX;
    with_values(input, |metadata| {
        let reply = allocation(metadata);
        let workflow = data_workflow(metadata, 2);
        assert!(matches!(
            IcSnapshotDataUploadPlan::derive_kinds(&workflow, 7, &reply, 1),
            Err(IcSnapshotDataUploadPlanningError::CountOverflow)
        ));
        assert!(matches!(
            IcSnapshotDataUploadPlan::derive_kinds(&workflow, 7, &reply, 1024 * 1024),
            Err(IcSnapshotDataUploadPlanningError::InsufficientAllowance { .. })
        ));
    });
}
#[test]
fn empty_regions_make_no_placeholder_but_known_empty_chunk_requires_a_write() {
    let mut input = values();
    input.wasm_module_size = 0;
    input.wasm_memory_size = 0;
    input.stable_memory_size = 0;
    input.wasm_chunk_store.clear();
    with_values(input.clone(), |metadata| {
        let reply = allocation(metadata);
        let workflow = data_workflow(metadata, 1);
        let plan = IcSnapshotDataUploadPlan::from_bindings(&workflow, 7, &reply, 1024 * 1024, &[])
            .unwrap();
        assert!(plan.kinds().is_empty());
        assert!(plan.plan().is_none());
    });
    input.wasm_chunk_store.push(ChunkHash {
        hash: <sha2::Sha256 as sha2::Digest>::digest(b"").to_vec(),
    });
    with_values(input, |metadata| {
        let reply = allocation(metadata);
        let workflow = data_workflow(metadata, 1);
        assert_eq!(
            IcSnapshotDataUploadPlan::derive_kinds(&workflow, 7, &reply, 1024 * 1024)
                .unwrap()
                .len(),
            1
        );
    });
}
#[test]
fn changed_original_context_and_data_acknowledgement_cannot_plan_allocation() {
    let mut values = values();
    values.wasm_module_size = 8;
    values.wasm_memory_size = 8;
    values.stable_memory_size = 8;
    with_values(values, |metadata| {
        let workflow = data_workflow(metadata, 100);
        let mut caller_fields = serde_json::to_value(&workflow).unwrap();
        caller_fields["allocation"]["context"]["caller"] = serde_json::json!(TARGET);
        let another_caller = serde_json::from_value(caller_fields).unwrap();
        assert!(
            IcSnapshotDataUploadPlan::derive_kinds(
                &another_caller,
                7,
                &allocation(metadata),
                1024 * 1024
            )
            .is_ok()
        );
        let mut fields = serde_json::to_value(&workflow).unwrap();
        fields["allocation"]["context"]["release"] = serde_json::json!("99".repeat(32));
        let changed = serde_json::from_value(fields).unwrap();
        assert!(matches!(
            IcSnapshotDataUploadPlan::derive_kinds(&changed, 7, &allocation(metadata), 1024 * 1024),
            Err(IcSnapshotDataUploadPlanningError::OriginalMismatch)
        ));
        let data = IcSnapshotUploadRequest::data(
            metadata,
            &[21, 0, 255],
            SnapshotDataKind::WasmModule { offset: 0, size: 1 },
            &[7],
        )
        .unwrap();
        let reply = IcSnapshotUploadReply::decode(
            &data,
            crate::model::ic_lifecycle_reply::EMPTY_CANDID_REPLY,
        )
        .unwrap();
        assert!(matches!(
            IcSnapshotDataUploadPlan::derive_kinds(&workflow, 7, &reply, 1024 * 1024),
            Err(IcSnapshotDataUploadPlanningError::OriginalMismatch)
        ));
    });
}
