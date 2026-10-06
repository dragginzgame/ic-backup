//! Real guarded source descriptors, unchanged journals and exact SDK upload bytes.
use super::*;
use crate::{
    model::ic_snapshot_upload::{IcSnapshotUploadError, IcSnapshotUploadRequest},
    ops::persistence::{IcSnapshotUploadArtifactError, create_operation_plan},
    test_support::{
        create_private_fifo,
        ic_snapshot_upload::{DESTINATION_ID, raw as upload_raw, source_plan},
    },
};
use ic_management_canister_types::{
    UploadCanisterSnapshotDataArgs, UploadCanisterSnapshotMetadataArgs,
};

fn upload_values() -> ReadCanisterSnapshotMetadataResult {
    let mut values: ReadCanisterSnapshotMetadataResult =
        candid::decode_one(&upload_raw("upload-source")).unwrap();
    values.wasm_module_size = 5;
    values.wasm_memory_size = 3;
    values.stable_memory_size = 1;
    values
}

#[test]
fn local_metrics_count_returned_work_without_writes_unique_progress_or_replay_samples() {
    let root = root();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let plan = source_plan();
    let request = IcSnapshotMetadataRequest::new(TARGET, &[0, 255, 17]).unwrap();
    let raw = candid::encode_one(upload_values()).unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let guard = retained(&layout, &plan, &metadata, &raw);
    let before = fs::read(guard.path()).unwrap();
    let empty = guard.ic_snapshot_metrics();
    assert_eq!(empty.verification_success_ns().samples(), 0);
    assert_eq!(empty.prepared_chunk_bytes().latest(), None);
    let upload = guard
        .prepare_ic_snapshot_upload_metadata(&plan, TOKEN, &metadata)
        .unwrap();
    guard
        .verify_ic_snapshot_artifact(&plan, TOKEN, &metadata)
        .unwrap();
    for _ in 0..2 {
        guard
            .prepare_ic_snapshot_upload_data(
                &plan,
                TOKEN,
                &upload,
                DESTINATION_ID,
                SnapshotDataKind::WasmModule { offset: 1, size: 3 },
            )
            .unwrap();
    }
    guard
        .prepare_ic_snapshot_upload_data(
            &plan,
            TOKEN,
            &upload,
            DESTINATION_ID,
            SnapshotDataKind::WasmChunk {
                hash: metadata.metadata().wasm_chunk_store[1].hash.clone(),
            },
        )
        .unwrap();
    let successful = guard.ic_snapshot_metrics();
    assert_eq!(successful.verification_success_ns().samples(), 8);
    assert_eq!(successful.verification_failure_ns().samples(), 0);
    assert_eq!(successful.upload_metadata_success_ns().samples(), 1);
    assert_eq!(successful.upload_data_success_ns().samples(), 3);
    assert_eq!(successful.prepared_chunk_bytes().samples(), 3);
    assert_eq!(successful.prepared_chunk_bytes().total(), 6);
    assert_eq!(successful.prepared_chunk_bytes().latest(), Some(0));
    assert_eq!(successful.prepared_chunk_bytes().maximum(), Some(3));
    assert!(successful.upload_data_success_ns().latest().is_some());
    assert!(matches!(
        guard.prepare_ic_snapshot_upload_data(
            &plan,
            TOKEN,
            &upload,
            &[],
            SnapshotDataKind::WasmModule { offset: 0, size: 1 }
        ),
        Err(IcSnapshotUploadArtifactError::Upload(
            IcSnapshotUploadError::InvalidDestination
        ))
    ));
    assert!(matches!(
        guard.prepare_ic_snapshot_upload_metadata(&plan, "wrong token", &metadata),
        Err(IcSnapshotUploadArtifactError::Artifact(_))
    ));
    let rejected = guard.ic_snapshot_metrics();
    assert_eq!(
        rejected.verification_success_ns(),
        successful.verification_success_ns()
    );
    assert_eq!(rejected.verification_failure_ns().samples(), 1);
    assert_eq!(rejected.upload_metadata_failure_ns().samples(), 1);
    assert_eq!(rejected.upload_data_failure_ns().samples(), 1);
    assert_eq!(
        rejected.prepared_chunk_bytes(),
        successful.prepared_chunk_bytes()
    );
    assert_eq!(successful.upload_data_failure_ns().samples(), 0);
    assert_eq!(guard.ic_snapshot_metrics(), rejected);
    assert_eq!(fs::read(guard.path()).unwrap(), before);
    fs::rename(
        root.join(format!("artifacts/{TARGET}")),
        root.join("retained-original"),
    )
    .unwrap();
    assert_missing_source_records_failure_only(&guard, &plan, &upload, rejected);
    assert_eq!(fs::read(guard.path()).unwrap(), before);
    drop(guard);
    let reopened = DownloadJournalGuard::open(&layout, plan.digest().hash()).unwrap();
    assert!(reopened.record().unwrap().resume_view().is_complete);
    assert_eq!(reopened.ic_snapshot_metrics(), empty);
    assert_eq!(fs::read(reopened.path()).unwrap(), before);
}

fn assert_missing_source_records_failure_only(
    guard: &DownloadJournalGuard<'_>,
    plan: &crate::model::operation_plan::OperationPlanRecord,
    upload: &IcSnapshotUploadRequest<'_>,
    rejected: crate::ops::persistence::IcSnapshotLocalMetrics,
) {
    assert!(matches!(
        guard.prepare_ic_snapshot_upload_data(
            plan,
            TOKEN,
            upload,
            DESTINATION_ID,
            SnapshotDataKind::WasmModule { offset: 0, size: 1 }
        ),
        Err(IcSnapshotUploadArtifactError::Artifact(_))
    ));
    let missing = guard.ic_snapshot_metrics();
    assert_eq!(missing.verification_failure_ns().samples(), 2);
    assert_eq!(missing.upload_data_failure_ns().samples(), 2);
    assert_eq!(
        missing.verification_success_ns(),
        rejected.verification_success_ns()
    );
    assert_eq!(
        missing.upload_data_success_ns(),
        rejected.upload_data_success_ns()
    );
    assert_eq!(
        missing.prepared_chunk_bytes(),
        rejected.prepared_chunk_bytes()
    );
}

fn retained<'layout>(
    layout: &'layout BackupLayoutGuard,
    plan: &crate::model::operation_plan::OperationPlanRecord,
    metadata: &IcSnapshotMetadataReply<'_>,
    raw: &[u8],
) -> DownloadJournalGuard<'layout> {
    create_operation_plan(layout, plan).unwrap();
    let mut guard = DownloadJournalGuard::create(
        layout,
        plan.digest().hash(),
        vec![DownloadArtifactRequest {
            canister_id: TARGET.into(),
            snapshot_id: TOKEN.into(),
            snapshot_taken_at_timestamp: metadata.metadata().taken_at_timestamp,
            snapshot_total_size_bytes: u64::MAX,
        }],
    )
    .unwrap();
    complete(
        guard
            .stage_ic_snapshot_artifact(TOKEN, metadata, raw)
            .unwrap(),
    )
    .finish()
    .unwrap();
    guard
}

#[test]
fn fresh_metadata_and_all_region_chunk_bytes_preserve_original_records_and_empty_values() {
    let root = root();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let plan = source_plan();
    let request = IcSnapshotMetadataRequest::new(TARGET, &[0, 255, 17]).unwrap();
    let raw = candid::encode_one(upload_values()).unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let guard = retained(&layout, &plan, &metadata, &raw);
    let journal_bytes = fs::read(guard.path()).unwrap();
    let plan_bytes = fs::read(root.join("operation-plan.json")).unwrap();
    let upload = guard
        .prepare_ic_snapshot_upload_metadata(&plan, TOKEN, &metadata)
        .unwrap();
    let args: UploadCanisterSnapshotMetadataArgs = candid::decode_one(upload.arguments()).unwrap();
    assert_eq!(args.replace_snapshot, None);
    assert_eq!(args.globals.len(), metadata.metadata().globals.len());
    assert_eq!(
        upload.source_checksum(),
        guard.record().unwrap().artifacts()[0].checksum().unwrap()
    );
    let cases = [
        (
            SnapshotDataKind::WasmModule { offset: 1, size: 3 },
            vec![8, 10, 10],
        ),
        (
            SnapshotDataKind::WasmMemory { offset: 1, size: 2 },
            vec![7, 7],
        ),
        (
            SnapshotDataKind::StableMemory { offset: 0, size: 1 },
            vec![9],
        ),
        (
            SnapshotDataKind::WasmChunk {
                hash: metadata.metadata().wasm_chunk_store[0].hash.clone(),
            },
            vec![0, 255, 17],
        ),
        (
            SnapshotDataKind::WasmChunk {
                hash: metadata.metadata().wasm_chunk_store[1].hash.clone(),
            },
            vec![],
        ),
    ];
    for (kind, expected) in cases {
        let payload = guard
            .prepare_ic_snapshot_upload_data(&plan, TOKEN, &upload, DESTINATION_ID, kind)
            .unwrap();
        let args: UploadCanisterSnapshotDataArgs = candid::decode_one(payload.arguments()).unwrap();
        assert_eq!(args.chunk, expected);
        assert_eq!(args.snapshot_id, DESTINATION_ID);
        assert_eq!(args.canister_id.to_text(), TARGET);
        assert_eq!(payload.source_checksum(), upload.source_checksum());
        assert_eq!(fs::read(guard.path()).unwrap(), journal_bytes);
        assert_eq!(
            fs::read(root.join("operation-plan.json")).unwrap(),
            plan_bytes
        );
    }
    drop(guard);
    let reopened = DownloadJournalGuard::open(&layout, plan.digest().hash()).unwrap();
    assert_eq!(
        reopened
            .prepare_ic_snapshot_upload_metadata(&plan, TOKEN, &metadata)
            .unwrap()
            .binding_digest(),
        upload.binding_digest()
    );
    assert!(reopened.record().unwrap().resume_view().is_complete);
    assert_eq!(fs::read(reopened.path()).unwrap(), journal_bytes);
}

#[test]
fn changed_unsafe_missing_and_extra_sources_reject_with_original_evidence_retained() {
    for case in [
        "changed",
        "missing",
        "symlink",
        "fifo",
        "directory",
        "extra",
        "metadata",
        "layout",
    ] {
        let root = root();
        let layout = BackupLayoutGuard::acquire(&root).unwrap();
        let plan = source_plan();
        let request = IcSnapshotMetadataRequest::new(TARGET, &[0, 255, 17]).unwrap();
        let raw = candid::encode_one(upload_values()).unwrap();
        let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
        let guard = retained(&layout, &plan, &metadata, &raw);
        let upload = guard
            .prepare_ic_snapshot_upload_metadata(&plan, TOKEN, &metadata)
            .unwrap();
        let before = fs::read(guard.path()).unwrap();
        let directory = root.join(format!("artifacts/{TARGET}"));
        let module = directory.join(REGIONS[0]);
        if matches!(case, "missing" | "symlink" | "fifo" | "directory") {
            fs::remove_file(&module).unwrap();
        }
        match case {
            "changed" => fs::write(&module, [42; 5]).unwrap(),
            "symlink" => symlink(directory.join(REGIONS[1]), &module).unwrap(),
            "fifo" => create_private_fifo(&module),
            "directory" => fs::create_dir(&module).unwrap(),
            "extra" => fs::write(directory.join("extra"), []).unwrap(),
            "metadata" => fs::write(directory.join("metadata.candid"), []).unwrap(),
            "layout" => {
                fs::rename(
                    root.join("operation-plan.json"),
                    root.join("retained-plan.json"),
                )
                .unwrap();
            }
            "missing" => {}
            _ => unreachable!(),
        }
        assert!(matches!(
            guard.prepare_ic_snapshot_upload_metadata(&plan, TOKEN, &metadata),
            Err(IcSnapshotUploadArtifactError::Artifact(_))
        ));
        assert!(matches!(
            guard.prepare_ic_snapshot_upload_data(
                &plan,
                TOKEN,
                &upload,
                DESTINATION_ID,
                SnapshotDataKind::WasmModule { offset: 0, size: 1 }
            ),
            Err(IcSnapshotUploadArtifactError::Artifact(_))
        ));
        assert_eq!(fs::read(guard.path()).unwrap(), before);
        assert_eq!(
            guard.record().unwrap().artifacts()[0].state(),
            ArtifactStateRecord::Durable
        );
        if case == "missing" {
            assert!(!module.exists());
        }
        if case == "changed" {
            assert_eq!(fs::read(module).unwrap(), [42; 5]);
        }
    }
}

#[test]
fn declared_source_drift_invalid_destination_ranges_and_unavailable_globals_reject() {
    let root = root();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let plan = source_plan();
    let request = IcSnapshotMetadataRequest::new(TARGET, &[0, 255, 17]).unwrap();
    let raw = candid::encode_one(upload_values()).unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let guard = retained(&layout, &plan, &metadata, &raw);
    let before = fs::read(guard.path()).unwrap();
    let upload = guard
        .prepare_ic_snapshot_upload_metadata(&plan, TOKEN, &metadata)
        .unwrap();
    let wrong = IcSnapshotUploadRequest::metadata(
        &plan,
        &metadata,
        &ArtifactChecksumRecord::from_bytes(b"different tree"),
    )
    .unwrap();
    assert!(matches!(
        guard.prepare_ic_snapshot_upload_data(
            &plan,
            TOKEN,
            &wrong,
            DESTINATION_ID,
            SnapshotDataKind::WasmModule { offset: 0, size: 1 }
        ),
        Err(IcSnapshotUploadArtifactError::SourceMismatch)
    ));
    let other = crate::test_support::ic_snapshot_upload::plan(&"12".repeat(32));
    assert!(matches!(
        guard.prepare_ic_snapshot_upload_data(
            &other,
            TOKEN,
            &upload,
            DESTINATION_ID,
            SnapshotDataKind::WasmModule { offset: 0, size: 1 }
        ),
        Err(IcSnapshotUploadArtifactError::SourceMismatch)
    ));
    assert!(matches!(
        guard.prepare_ic_snapshot_upload_data(
            &plan,
            TOKEN,
            &upload,
            &[0, 255, 17],
            SnapshotDataKind::WasmModule { offset: 0, size: 1 }
        ),
        Err(IcSnapshotUploadArtifactError::Upload(
            IcSnapshotUploadError::DestinationReusesSource
        ))
    ));
    assert!(matches!(
        guard.prepare_ic_snapshot_upload_data(
            &plan,
            TOKEN,
            &upload,
            DESTINATION_ID,
            SnapshotDataKind::WasmModule { offset: 5, size: 1 }
        ),
        Err(IcSnapshotUploadArtifactError::Upload(
            IcSnapshotUploadError::Data(_)
        ))
    ));
    assert!(matches!(
        guard.prepare_ic_snapshot_upload_metadata(&plan, "wrong token", &metadata),
        Err(IcSnapshotUploadArtifactError::Artifact(_))
    ));
    assert_eq!(fs::read(guard.path()).unwrap(), before);

    let root = super::root();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let raw = candid::encode_one(values()).unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let guard = retained(&layout, &plan, &metadata, &raw);
    let before = fs::read(guard.path()).unwrap();
    assert!(matches!(
        guard.prepare_ic_snapshot_upload_metadata(&plan, TOKEN, &metadata),
        Err(IcSnapshotUploadArtifactError::Upload(
            IcSnapshotUploadError::UnavailableGlobal
        ))
    ));
    assert_eq!(fs::read(guard.path()).unwrap(), before);
}

#[test]
fn maximum_bounded_descriptor_slice_does_not_buffer_the_whole_region() {
    let root = root();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let plan = source_plan();
    create_operation_plan(&layout, &plan).unwrap();
    let request = IcSnapshotMetadataRequest::new(TARGET, &[0, 255, 17]).unwrap();
    let mut values = upload_values();
    values.wasm_module_size = 2 * MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES as u64;
    values.wasm_memory_size = 0;
    values.stable_memory_size = 0;
    values.wasm_chunk_store.clear();
    let raw = candid::encode_one(values).unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let mut guard = DownloadJournalGuard::create(
        &layout,
        plan.digest().hash(),
        vec![DownloadArtifactRequest {
            canister_id: TARGET.into(),
            snapshot_id: TOKEN.into(),
            snapshot_taken_at_timestamp: metadata.metadata().taken_at_timestamp,
            snapshot_total_size_bytes: u64::MAX,
        }],
    )
    .unwrap();
    let mut writer = guard
        .stage_ic_snapshot_artifact(TOKEN, &metadata, &raw)
        .unwrap();
    for (offset, byte) in [(0, 42), (MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES as u64, 43)] {
        writer = append(
            writer,
            SnapshotDataKind::WasmModule {
                offset,
                size: MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES as u64,
            },
            vec![byte; MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES],
        )
        .unwrap();
    }
    writer.finish().unwrap();
    let upload = guard
        .prepare_ic_snapshot_upload_metadata(&plan, TOKEN, &metadata)
        .unwrap();
    let payload = guard
        .prepare_ic_snapshot_upload_data(
            &plan,
            TOKEN,
            &upload,
            DESTINATION_ID,
            SnapshotDataKind::WasmModule {
                offset: MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES as u64,
                size: MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES as u64,
            },
        )
        .unwrap();
    let args: UploadCanisterSnapshotDataArgs = candid::decode_one(payload.arguments()).unwrap();
    assert_eq!(args.chunk, vec![43; MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES]);
    assert!(payload.arguments().len() > MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES);
}
