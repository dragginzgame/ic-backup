//! Real retained-source IO and original journal failures; no IC effect simulation.
use super::*;
use crate::model::ic_update_response::IcUpdateResponseInput;
use crate::tests::support::ready::ready;
use crate::{
    model::{
        attempt_journal::{MutationOutcomeRecord, MutationReceiptRequest},
        execution_workflow::{
            ExecutionStageBindingRecord, ExecutionStagePredecessorRecord, ExecutionWorkflowRecord,
        },
        ic_mutation::IcMutationAcknowledgement,
        ic_snapshot_upload::{
            IcSnapshotDataUploadPlan, IcSnapshotDataUploadPlanningError, IcSnapshotUploadAttempt,
            IcSnapshotUploadReply,
        },
    },
    ops::persistence::{
        AttemptJournalGuard, ExecutionStageGuard, IcSnapshotDataUploadPreparationError,
        create_execution_workflow, read_execution_progress,
    },
    ports::{ic_mutation::IcMutationProviderError, ic_snapshot_upload::IcSnapshotUploadProvider},
    tests::support::ic_snapshot_upload::{data_workflow, unhex},
    workflow::ic_snapshot_upload::{
        IcSnapshotDataUploadExecutionError, IcSnapshotDataUploadReplyError,
        IcSnapshotUploadExecutionError, upload_snapshot_data,
    },
};
use ic_platform::candid::contract;
use std::io;

mod execution;
mod fixture;
mod restore;
use execution::complete_data;
use fixture::{with_data_stage, with_data_stage_then};
use restore::check_empty_restore_source;

#[test]
fn insufficient_allocation_refuses_before_any_source_verification_or_journal_changes() {
    let root = root();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let original = source_plan();
    let request = IcSnapshotMetadataRequest::new(TARGET, &[0, 255, 17]).unwrap();
    let raw = contract::encode_one(upload_values()).unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let source = retained(&layout, &original, &metadata, &raw);
    let payload = source
        .prepare_ic_snapshot_upload_metadata(&original, TOKEN, &metadata)
        .unwrap();
    let reply = IcSnapshotUploadReply::decode(
        &payload,
        &unhex("4449444c026c01b6b897890f016d7b0100031500ff"),
    )
    .unwrap();
    let workflow: ExecutionWorkflowRecord = data_workflow(&payload, 1);
    let before = fs::read(source.path()).unwrap();
    fs::rename(
        root.join(format!("artifacts/{TARGET}")),
        root.join("retained-missing-tree"),
    )
    .unwrap();
    assert!(matches!(
        source.prepare_ic_snapshot_data_upload_plan(&workflow, 7, TOKEN, &reply, 4),
        Err(IcSnapshotDataUploadPreparationError::Planning(
            IcSnapshotDataUploadPlanningError::InsufficientAllowance {
                required: 6,
                original: 1
            }
        ))
    ));
    assert_eq!(fs::read(source.path()).unwrap(), before);
}

#[test]
fn batch_bindings_match_canonical_payloads_and_preserve_retained_records() {
    with_data_stage(|_stage, data, source| {
        let before = fs::read(source.path()).unwrap();
        let bindings: Vec<_> = data
            .kinds()
            .iter()
            .map(|kind| {
                source
                    .prepare_source_fixture_chunk(
                        data.metadata().source_plan(),
                        TOKEN,
                        data.metadata(),
                        data.destination(),
                        kind.clone(),
                    )
                    .unwrap()
                    .binding_digest()
            })
            .collect();
        // Each original operation names the exact separately prepared payload.
        assert_eq!(data.plan().unwrap().operations().len(), bindings.len());
        for (operation, binding) in data.plan().unwrap().operations().iter().zip(&bindings) {
            assert_eq!(operation.request(), binding.hash());
        }
        assert_eq!(fs::read(source.path()).unwrap(), before);
    });
}

#[test]
fn batch_plans_empty_regions_and_known_empty_chunks_without_placeholder_writes() {
    for known_empty in [false, true] {
        let root = root();
        let layout = BackupLayoutGuard::acquire(&root).unwrap();
        let plan = source_plan();
        create_operation_plan(&layout, &plan).unwrap();
        let request = IcSnapshotMetadataRequest::new(TARGET, &[0, 255, 17]).unwrap();
        let mut values = upload_values();
        values.wasm_module_size = 0;
        values.wasm_memory_size = 0;
        values.stable_memory_size = 0;
        let empty = values.wasm_chunk_store.pop().unwrap();
        values.wasm_chunk_store = if known_empty { vec![empty] } else { vec![] };
        let raw = contract::encode_one(values).unwrap();
        let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
        let mut source = journal(
            &layout,
            plan.digest().hash(),
            metadata.metadata().taken_at_timestamp,
        );
        let writer = source
            .stage_ic_snapshot_artifact(TOKEN, &metadata, &raw)
            .unwrap();
        let writer = if known_empty {
            append(
                writer,
                SnapshotDataKind::WasmChunk {
                    hash: metadata.metadata().wasm_chunk_store[0].hash.clone(),
                },
                vec![],
            )
            .unwrap()
        } else {
            writer
        };
        writer.finish().unwrap();
        let before = fs::read(source.path()).unwrap();
        let upload = source
            .prepare_ic_snapshot_upload_metadata(&plan, TOKEN, &metadata)
            .unwrap();
        let workflow = data_workflow(&upload, 12);
        let wire = unhex("4449444c026c01b6b897890f016d7b0100031500ff");
        let allocation = IcSnapshotUploadReply::decode(&upload, &wire).unwrap();
        let data = source
            .prepare_ic_snapshot_data_upload_plan(&workflow, 7, TOKEN, &allocation, 4)
            .unwrap();
        assert_eq!(data.kinds().len(), usize::from(known_empty));
        assert_eq!(data.plan().is_some(), known_empty);
        if !known_empty {
            check_empty_restore_source(&source, &workflow, &data, &allocation);
        }
        if let Some(plan) = data.plan() {
            let payload = source
                .prepare_source_fixture_chunk(
                    upload.source_plan(),
                    TOKEN,
                    &upload,
                    DESTINATION_ID,
                    data.kinds()[0].clone(),
                )
                .unwrap();
            assert_eq!(
                plan.operations()[0].request(),
                payload.binding_digest().hash()
            );
        }
        assert_eq!(fs::read(source.path()).unwrap(), before);
    }
}

#[test]
fn complete_upload_reads_are_linear_in_bytes_for_different_chunk_counts() {
    for chunk in [1, 2, 4] {
        with_data_stage_then(
            chunk,
            |stage, data, source| {
                let entry = source
                    .record()
                    .unwrap()
                    .artifact(TARGET, TOKEN)
                    .unwrap()
                    .clone();
                let directory = source.layout.root().join(entry.artifact_path());
                let total: u64 = fs::read_dir(&directory)
                    .unwrap()
                    .map(|entry| entry.unwrap().metadata().unwrap().len())
                    .sum();
                let values = data.metadata().source().metadata();
                let payload = values.wasm_module_size
                    + values.wasm_memory_size
                    + values.stable_memory_size
                    + values
                        .wasm_chunk_store
                        .iter()
                        .map(|chunk| {
                            fs::metadata(directory.join(format!(
                                    "chunk-{}.bin",
                                    ArtifactChecksumRecord::from_digest(
                                        chunk.hash.clone().try_into().unwrap()
                                    )
                                    .hash()
                                )))
                            .unwrap()
                            .len()
                        })
                        .sum::<u64>();
                let (_, reads) =
                    super::super::read_count::measure(|| complete_data(stage, data, source));
                // Two complete admission passes and one closing pass, plus each exact payload once.
                assert_eq!(reads, 3 * total + payload);
                assert!(data.kinds().len() > 1);
            },
            |_, _, _, ()| {},
        );
    }
}
