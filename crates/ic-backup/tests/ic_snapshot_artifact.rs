//! Public local artifact publication preserves original pending spending and source retention.

#![cfg(unix)]

use ic_backup::{
    model::{
        download_journal::{ArtifactStateRecord, DownloadArtifactRequest},
        ic_request::{IcManagementMethodRecord, IcManagementRequest, IcManagementRequestRecord},
        ic_snapshot_data::{IcSnapshotDataReply, IcSnapshotDataRequest},
        ic_snapshot_metadata::{IcSnapshotMetadataReply, IcSnapshotMetadataRequest},
        operation_plan::OperationPlanRecord,
    },
    ops::{
        artifacts::checksum_directory,
        persistence::{
            AttemptJournalGuard, BackupLayoutGuard, DownloadJournalGuard, IcSnapshotArtifactError,
            create_operation_plan, read_operation_plan,
        },
    },
};
use ic_management_canister_types::{
    ReadCanisterSnapshotDataResult, ReadCanisterSnapshotMetadataResult, SnapshotDataKind,
};
use serde_json::json;
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one public retained-evidence journey keeps original spending and source retention assertions together"
)]
fn durable_bytes_and_local_reopen_never_settle_capture_or_replenish_spending() {
    const TARGET: &str = "renrk-eyaaa-aaaaa-aaada-cai";
    const TOKEN: &str = "integration-owned-token";
    let root = std::env::temp_dir().canonicalize().unwrap().join(format!(
        "ic-backup-public-artifact-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(root.join("artifacts")).unwrap();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let capture = IcManagementRequestRecord::new(IcManagementRequest {
        method: IcManagementMethodRecord::TakeCanisterSnapshot,
        target: TARGET.into(),
        snapshot_id: None,
    })
    .unwrap();
    let observe = IcManagementRequestRecord::new(IcManagementRequest {
        method: IcManagementMethodRecord::ListCanisterSnapshots,
        target: TARGET.into(),
        snapshot_id: None,
    })
    .unwrap();
    let plan: OperationPlanRecord = serde_json::from_value(json!({
        "version": 1,
        "context": {"network": "ab".repeat(32), "caller": "2vxsx-fae", "release": "cd".repeat(32)},
        "inventory": {"version": 1, "targets": [{"canister_id": TARGET, "parent_canister_id": null, "role": null, "module_hash": null}]},
        "selected_targets": [TARGET],
        "graph": {"version": 1, "nodes": [{"operation_sequence": 7, "depends_on": []}]},
        "operations": [{"operation_sequence": 7, "target": TARGET, "request": capture.digest().hash(), "budget": {"mutations": 1, "observations": 1}}],
        "budget": {"mutations": 1, "observations": 1}
    })).unwrap();
    create_operation_plan(&layout, &plan).unwrap();
    layout
        .retain_restore(&root.join("unfinished-restore.json"), plan.digest().hash())
        .unwrap();
    let references = layout.restore_references().unwrap();
    assert!(!references.is_empty());
    let reference_bytes = fs::read(root.join("restore-references.json")).unwrap();
    let plan_bytes = fs::read(root.join("operation-plan.json")).unwrap();
    let mut attempts =
        AttemptJournalGuard::create(&layout, plan.attempt_authority(7).unwrap()).unwrap();
    let mutation = attempts.reserve_mutation().unwrap();
    let observation = attempts
        .reserve_observation(mutation, observe.digest().hash())
        .unwrap();
    let attempt_bytes = fs::read(attempts.path()).unwrap();
    let attempt_view = attempts.record().unwrap().view();

    let sources: Vec<serde_json::Value> = serde_json::from_str(include_str!(
        "../src/model/ic_snapshot_metadata/tests/golden.json"
    ))
    .unwrap();
    let source = sources
        .iter()
        .find(|source| source["name"] == "data-source")
        .unwrap();
    let raw = source["reply_hex"]
        .as_str()
        .unwrap()
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect::<Vec<_>>();
    let mut values: ReadCanisterSnapshotMetadataResult = candid::decode_one(&raw).unwrap();
    values.wasm_module_size = 3;
    values.wasm_memory_size = 0;
    values.stable_memory_size = 0;
    values.wasm_chunk_store.clear();
    let raw = candid::encode_one(values).unwrap();
    let request = IcSnapshotMetadataRequest::new(TARGET, &[0, 255, 17]).unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let mut downloads = DownloadJournalGuard::create(
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
    let mut writer = downloads
        .stage_ic_snapshot_artifact(TOKEN, &metadata, &raw)
        .unwrap();
    for offset in 0..3 {
        let data =
            IcSnapshotDataRequest::new(&metadata, SnapshotDataKind::WasmModule { offset, size: 1 })
                .unwrap();
        let raw = candid::encode_one(ReadCanisterSnapshotDataResult { chunk: vec![42] }).unwrap();
        let reply = IcSnapshotDataReply::decode(&data, &raw).unwrap();
        writer = writer.append(&reply).unwrap();
    }
    let checksum = writer.finish().unwrap();
    assert_eq!(
        downloads
            .verify_ic_snapshot_artifact(&plan, TOKEN, &metadata)
            .unwrap(),
        checksum
    );
    let canonical = root.join(downloads.record().unwrap().artifacts()[0].artifact_path());
    assert_eq!(checksum_directory(&canonical).unwrap(), checksum);
    assert_eq!(
        fs::read(canonical.join("wasm-module.bin")).unwrap(),
        [42; 3]
    );
    assert_eq!(fs::read(canonical.join("metadata.candid")).unwrap(), raw);
    assert_eq!(
        fs::read(canonical.join("metadata-arguments.candid")).unwrap(),
        request.arguments()
    );
    assert_eq!(attempts.record().unwrap().view(), attempt_view);
    assert_eq!(fs::read(attempts.path()).unwrap(), attempt_bytes);
    assert_eq!(
        fs::read(root.join("operation-plan.json")).unwrap(),
        plan_bytes
    );
    assert_eq!(
        fs::read(root.join("restore-references.json")).unwrap(),
        reference_bytes
    );
    let download_bytes = fs::read(downloads.path()).unwrap();
    drop(downloads);
    drop(attempts);
    drop(layout);

    // Ordinary retained replay needs no artifact reads, provider or transfer reconstruction.
    fs::rename(&canonical, root.join("retained-canonical")).unwrap();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let original = read_operation_plan(&layout, &plan.digest()).unwrap();
    let mut attempts =
        AttemptJournalGuard::open(&layout, &original.attempt_authority(7).unwrap()).unwrap();
    let downloads = DownloadJournalGuard::open(&layout, original.digest().hash()).unwrap();
    assert!(downloads.record().unwrap().resume_view().is_complete);
    assert!(matches!(
        downloads.verify_ic_snapshot_artifact(&original, TOKEN, &metadata),
        Err(IcSnapshotArtifactError::Io(_))
    ));
    let artifact = &downloads.record().unwrap().artifacts()[0];
    assert_eq!(artifact.state(), ArtifactStateRecord::Durable);
    assert_eq!(artifact.snapshot_id(), TOKEN);
    assert_eq!(artifact.checksum(), Some(&checksum));
    assert_eq!(fs::read(downloads.path()).unwrap(), download_bytes);
    assert_eq!(attempts.record().unwrap().view(), attempt_view);
    assert_eq!(attempt_view.pending_mutation, Some(mutation));
    assert_eq!(attempt_view.pending_observation, Some(observation));
    assert!(!attempt_view.applied);
    assert_eq!(
        (
            attempt_view.mutations_remaining,
            attempt_view.observations_remaining
        ),
        (0, 0)
    );
    assert!(attempts.reserve_mutation().is_err());
    assert!(
        attempts
            .reserve_observation(mutation, observe.digest().hash())
            .is_err()
    );
    assert_eq!(fs::read(attempts.path()).unwrap(), attempt_bytes);
    assert_eq!(layout.restore_references().unwrap(), references);
    assert_eq!(
        fs::read(root.join("restore-references.json")).unwrap(),
        reference_bytes
    );
    assert_eq!(
        fs::read(root.join("operation-plan.json")).unwrap(),
        plan_bytes
    );
    drop(downloads);
    drop(attempts);
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}
