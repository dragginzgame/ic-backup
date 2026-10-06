//! Fresh retained format, metadata, extent and closed-tree admission on native files.

use super::*;
use crate::{
    model::operation_plan::OperationPlanRecord,
    ops::persistence::{DownloadIntegrityError, create_operation_plan},
    test_support::create_private_fifo,
};
use serde_json::json;

fn plan() -> OperationPlanRecord {
    serde_json::from_value(json!({
        "version": 1,
        "context": {"network": "ab".repeat(32), "caller": "2vxsx-fae", "release": "cd".repeat(32)},
        "inventory": {"version": 1, "targets": [{"canister_id": TARGET, "parent_canister_id": null, "role": null, "module_hash": null}]},
        "selected_targets": [TARGET],
        "graph": {"version": 1, "nodes": [{"operation_sequence": 7, "depends_on": []}]},
        "operations": [{"operation_sequence": 7, "target": TARGET, "request": "ef".repeat(32), "budget": {"mutations": 1, "observations": 0}}],
        "budget": {"mutations": 1, "observations": 0}
    })).unwrap()
}

fn retained<'a>(
    layout: &'a BackupLayoutGuard,
    plan: &OperationPlanRecord,
    metadata: &IcSnapshotMetadataReply<'_>,
    raw: &[u8],
) -> DownloadJournalGuard<'a> {
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
fn explicit_retained_verification_keeps_exact_metadata_journal_and_plan_bytes() {
    let root = root();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let plan = plan();
    let request = IcSnapshotMetadataRequest::new(TARGET, &[0, 255, 17]).unwrap();
    let raw = candid::encode_one(values()).unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let guard = retained(&layout, &plan, &metadata, &raw);
    let bytes = fs::read(guard.path()).unwrap();
    let plan_bytes = fs::read(root.join("operation-plan.json")).unwrap();
    let checksum = guard.record().unwrap().artifacts()[0]
        .checksum()
        .unwrap()
        .clone();
    drop(guard);
    let guard = DownloadJournalGuard::open(&layout, plan.digest().hash()).unwrap();
    assert_eq!(
        guard
            .verify_ic_snapshot_artifact(&plan, TOKEN, &metadata)
            .unwrap(),
        checksum
    );
    assert_eq!(fs::read(guard.path()).unwrap(), bytes);
    assert_eq!(
        fs::read(root.join("operation-plan.json")).unwrap(),
        plan_bytes
    );
    assert_eq!(
        fs::read(root.join(format!("artifacts/{TARGET}/metadata.candid"))).unwrap(),
        raw
    );
}

#[test]
fn changed_unsafe_missing_extra_and_unbounded_children_reject_without_repair() {
    for case in [
        "changed",
        "truncated",
        "long",
        "format",
        "metadata",
        "arguments",
        "chunk",
        "missing",
        "extra",
        "empty-directory",
        "directory",
        "symlink",
        "fifo",
        "huge-chunk",
        "huge-metadata",
    ] {
        let root = root();
        let layout = BackupLayoutGuard::acquire(&root).unwrap();
        let plan = plan();
        let request = IcSnapshotMetadataRequest::new(TARGET, &[0, 255, 17]).unwrap();
        let raw = candid::encode_one(values()).unwrap();
        let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
        let guard = retained(&layout, &plan, &metadata, &raw);
        let journal_bytes = fs::read(guard.path()).unwrap();
        let directory = root.join(format!("artifacts/{TARGET}"));
        let module = directory.join(REGIONS[0]);
        let chunk = directory.join(format!(
            "chunk-{}.bin",
            hex_bytes(&metadata.metadata().wasm_chunk_store[0].hash)
        ));
        match case {
            "changed" => fs::write(&module, [11; 5]).unwrap(),
            "truncated" => fs::write(&module, [8; 4]).unwrap(),
            "long" => fs::write(&module, [8; 6]).unwrap(),
            "format" => fs::write(directory.join("format"), b"incorrect format").unwrap(),
            "metadata" => fs::write(directory.join("metadata.candid"), b"wrong metadata").unwrap(),
            "arguments" => fs::write(
                directory.join("metadata-arguments.candid"),
                b"wrong request",
            )
            .unwrap(),
            "chunk" => fs::write(&chunk, [17, 255, 0]).unwrap(),
            "extra" => fs::write(directory.join("extra"), b"").unwrap(),
            "empty-directory" => fs::create_dir(directory.join("extra-empty")).unwrap(),
            "huge-chunk" | "huge-metadata" => {
                let path = if case == "huge-chunk" {
                    chunk
                } else {
                    directory.join("metadata.candid")
                };
                File::options()
                    .write(true)
                    .open(path)
                    .unwrap()
                    .set_len(MAX_IC_SNAPSHOT_METADATA_BYTES as u64 + 1)
                    .unwrap();
            }
            _ => {
                fs::rename(&module, root.join("retained-module")).unwrap();
                match case {
                    "missing" => {}
                    "directory" => fs::create_dir(&module).unwrap(),
                    "symlink" => symlink(root.join("retained-module"), &module).unwrap(),
                    "fifo" => create_private_fifo(&module),
                    _ => unreachable!(),
                }
            }
        }
        let result = guard.verify_ic_snapshot_artifact(&plan, TOKEN, &metadata);
        match case {
            "changed" | "metadata" | "chunk" => assert!(
                matches!(
                    result,
                    Err(IcSnapshotArtifactError::Checksum(
                        ChecksumError::ChecksumMismatch { .. }
                    ))
                ),
                "{case}: {result:?}"
            ),
            "missing" | "extra" | "empty-directory" => assert!(
                matches!(result, Err(IcSnapshotArtifactError::UnexpectedEntry)),
                "{case}: {result:?}"
            ),
            "symlink" => assert!(
                matches!(result, Err(IcSnapshotArtifactError::Io(_))),
                "{case}: {result:?}"
            ),
            _ => assert!(
                matches!(result, Err(IcSnapshotArtifactError::FileShape)),
                "{case}: {result:?}"
            ),
        }
        assert_eq!(fs::read(guard.path()).unwrap(), journal_bytes);
        assert!(guard.record().unwrap().resume_view().is_complete);
        assert!(directory.exists());
    }
}

#[test]
fn maximum_chunks_and_region_bytes_verify_without_an_aggregate_buffer() {
    let root = root();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let plan = plan();
    create_operation_plan(&layout, &plan).unwrap();
    let mut values = values();
    values.wasm_module_size = MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES as u64;
    values.wasm_memory_size = 0;
    values.stable_memory_size = 0;
    values.wasm_chunk_store = (0_u16..1024)
        .map(|value| ChunkHash {
            hash: Sha256::digest(value.to_be_bytes()).to_vec(),
        })
        .collect();
    let raw = candid::encode_one(values).unwrap();
    let request = IcSnapshotMetadataRequest::new(TARGET, &[0, 255, 17]).unwrap();
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
    writer = append(
        writer,
        SnapshotDataKind::WasmModule {
            offset: 0,
            size: MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES as u64,
        },
        vec![42; MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES],
    )
    .unwrap();
    for value in 0_u16..1024 {
        let chunk = value.to_be_bytes().to_vec();
        writer = append(
            writer,
            SnapshotDataKind::WasmChunk {
                hash: Sha256::digest(&chunk).to_vec(),
            },
            chunk,
        )
        .unwrap();
    }
    let checksum = writer.finish().unwrap();
    assert_eq!(
        guard
            .verify_ic_snapshot_artifact(&plan, TOKEN, &metadata)
            .unwrap(),
        checksum
    );
    let mut extreme = candid::decode_one::<ReadCanisterSnapshotMetadataResult>(&raw).unwrap();
    extreme.stable_memory_size = u64::MAX;
    let extreme_raw = candid::encode_one(extreme).unwrap();
    let extreme_metadata = IcSnapshotMetadataReply::decode(&request, &extreme_raw).unwrap();
    let canonical = root.join(format!("artifacts/{TARGET}"));
    fs::write(canonical.join("metadata.candid"), &extreme_raw).unwrap();
    assert!(matches!(
        guard.verify_ic_snapshot_artifact(&plan, TOKEN, &extreme_metadata),
        Err(IcSnapshotArtifactError::FileShape)
    ));
    assert_eq!(
        fs::metadata(canonical.join("stable-memory.bin"))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        guard.record().unwrap().artifacts()[0].checksum(),
        Some(&checksum)
    );
}

#[test]
fn matching_forged_tree_checksums_cannot_bypass_metadata_shape_or_chunk_identity() {
    for case in [
        "extent",
        "format",
        "metadata",
        "arguments",
        "chunk",
        "extra-directory",
    ] {
        let root = root();
        let layout = BackupLayoutGuard::acquire(&root).unwrap();
        let plan = plan();
        let request = IcSnapshotMetadataRequest::new(TARGET, &[0, 255, 17]).unwrap();
        let raw = candid::encode_one(values()).unwrap();
        let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
        let mut guard = retained(&layout, &plan, &metadata, &raw);
        let directory = root.join(format!("artifacts/{TARGET}"));
        let path = match case {
            "extent" => directory.join(REGIONS[0]),
            "format" => directory.join("format"),
            "metadata" => directory.join("metadata.candid"),
            "arguments" => directory.join("metadata-arguments.candid"),
            "chunk" => directory.join(format!(
                "chunk-{}.bin",
                hex_bytes(&metadata.metadata().wasm_chunk_store[0].hash)
            )),
            _ => directory.join("extra-empty"),
        };
        if case == "extra-directory" {
            fs::create_dir(&path).unwrap();
        } else {
            fs::write(
                &path,
                if case == "extent" {
                    b"wrong extent".as_slice()
                } else {
                    b"wrong".as_slice()
                },
            )
            .unwrap();
        }
        let checksum = checksum_directory(&directory).unwrap();
        let mut record = serde_json::to_value(&guard.record).unwrap();
        record["artifacts"][0]["checksum"] = serde_json::to_value(&checksum).unwrap();
        guard.record = serde_json::from_value(record).unwrap();
        // Deliberately corrupt both retained declarations; shape/evidence remains independent.
        write_json_durable(&guard.path(), &guard.record).unwrap();
        let bytes = fs::read(guard.path()).unwrap();
        let result = guard.verify_ic_snapshot_artifact(&plan, TOKEN, &metadata);
        match case {
            "extent" | "format" | "arguments" => assert!(
                matches!(result, Err(IcSnapshotArtifactError::FileShape)),
                "{case}: {result:?}"
            ),
            "metadata" | "chunk" => assert!(
                matches!(
                    result,
                    Err(IcSnapshotArtifactError::Checksum(
                        ChecksumError::ChecksumMismatch { .. }
                    ))
                ),
                "{case}: {result:?}"
            ),
            _ => assert!(
                matches!(result, Err(IcSnapshotArtifactError::UnexpectedEntry)),
                "{case}: {result:?}"
            ),
        }
        assert_eq!(fs::read(guard.path()).unwrap(), bytes);
        assert!(path.exists());
    }
}

#[test]
fn non_durable_originals_and_replaced_layout_reject_before_artifact_reads() {
    use crate::policy::download_integrity::DownloadIntegrityPolicyError;
    let root = root();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let plan = plan();
    create_operation_plan(&layout, &plan).unwrap();
    let request = IcSnapshotMetadataRequest::new(TARGET, &[0, 255, 17]).unwrap();
    let raw = candid::encode_one(values()).unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let guard = DownloadJournalGuard::create(
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
    assert!(matches!(
        guard.verify_ic_snapshot_artifact(&plan, TOKEN, &metadata),
        Err(IcSnapshotArtifactError::Integrity(
            DownloadIntegrityError::Policy(DownloadIntegrityPolicyError::NonDurableArtifact { .. })
        ))
    ));
    let original_path = root.with_extension("retained-original");
    fs::rename(&root, &original_path).unwrap();
    fs::create_dir(&root).unwrap();
    assert!(matches!(
        guard.verify_ic_snapshot_artifact(&plan, TOKEN, &metadata),
        Err(IcSnapshotArtifactError::Integrity(
            DownloadIntegrityError::Journal(_)
        ))
    ));
    assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
    assert!(original_path.join("download-journal.json").exists());
}

#[test]
fn exact_original_metadata_request_and_retained_declarations_are_required() {
    let root = root();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let plan = plan();
    let request = IcSnapshotMetadataRequest::new(TARGET, &[0, 255, 17]).unwrap();
    let raw = candid::encode_one(values()).unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let guard = retained(&layout, &plan, &metadata, &raw);
    let wrong_request = IcSnapshotMetadataRequest::new(TARGET, &[17, 255, 0]).unwrap();
    let wrong_metadata = IcSnapshotMetadataReply::decode(&wrong_request, &raw).unwrap();
    assert!(matches!(
        guard.verify_ic_snapshot_artifact(&plan, TOKEN, &wrong_metadata),
        Err(IcSnapshotArtifactError::Checksum(_))
    ));
    let mut different = values();
    different.taken_at_timestamp -= 1;
    let different_raw = candid::encode_one(different).unwrap();
    let different_metadata = IcSnapshotMetadataReply::decode(&request, &different_raw).unwrap();
    assert!(matches!(
        guard.verify_ic_snapshot_artifact(&plan, TOKEN, &different_metadata),
        Err(IcSnapshotArtifactError::OriginalMismatch)
    ));
    assert!(matches!(
        guard.verify_ic_snapshot_artifact(&plan, "other-token", &metadata),
        Err(IcSnapshotArtifactError::Journal(_))
    ));
    let mut changed_plan = serde_json::to_value(&plan).unwrap();
    changed_plan["context"]["release"] = json!("02".repeat(32));
    let changed_plan: OperationPlanRecord = serde_json::from_value(changed_plan).unwrap();
    assert!(matches!(
        guard.verify_ic_snapshot_artifact(&changed_plan, TOKEN, &metadata),
        Err(IcSnapshotArtifactError::Integrity(
            DownloadIntegrityError::Plan(_)
        ))
    ));
    let bytes = fs::read(guard.path()).unwrap();
    let mut changed_journal = serde_json::to_value(&guard.record).unwrap();
    changed_journal["artifacts"][0]["snapshot_total_size_bytes"] = json!(7);
    fs::write(guard.path(), serde_json::to_vec(&changed_journal).unwrap()).unwrap();
    assert!(matches!(
        guard.verify_ic_snapshot_artifact(&plan, TOKEN, &metadata),
        Err(IcSnapshotArtifactError::Integrity(
            DownloadIntegrityError::JournalChanged
        ))
    ));
    fs::write(guard.path(), &bytes).unwrap();
    fs::rename(
        root.join("operation-plan.json"),
        root.join("retained-plan.json"),
    )
    .unwrap();
    assert!(matches!(
        guard.verify_ic_snapshot_artifact(&plan, TOKEN, &metadata),
        Err(IcSnapshotArtifactError::Integrity(
            DownloadIntegrityError::Plan(_)
        ))
    ));
    assert_eq!(fs::read(guard.path()).unwrap(), bytes);
}
