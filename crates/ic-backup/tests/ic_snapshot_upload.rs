//! Public retained-source preparation and passive upload recovery; no IC simulation.
#![cfg(unix)]

mod upload_provider;

use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        download_journal::DownloadArtifactRequest,
        ic_mutation::{IcMutationAcknowledgement, IcMutationAcknowledgementInput},
        ic_snapshot_data::{IcSnapshotDataReply, IcSnapshotDataRequest},
        ic_snapshot_metadata::{IcSnapshotMetadataReply, IcSnapshotMetadataRequest},
        ic_snapshot_upload::{IcSnapshotUploadAttempt, IcSnapshotUploadReplyKind},
        operation_plan::OperationPlanRecord,
    },
    ops::persistence::{
        AttemptJournalGuard, BackupLayoutGuard, DownloadJournalGuard, create_operation_plan,
        read_operation_plan,
    },
    policy::ic_snapshot_upload::validate_acknowledgement,
    ports::{ic_mutation::IcMutationProviderError, ic_snapshot_upload::IcSnapshotUploadProvider},
};
use ic_management_canister_types::{
    ReadCanisterSnapshotDataResult, ReadCanisterSnapshotMetadataResult, SnapshotDataKind,
    UploadCanisterSnapshotDataArgs, UploadCanisterSnapshotMetadataResult,
};
use serde_json::json;
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

const TARGET: &str = "renrk-eyaaa-aaaaa-aaada-cai";
const TOKEN: &str = "original-source-token";
const DESTINATION: &[u8] = &[21, 0, 255];

fn root(lane: &str) -> PathBuf {
    let root = std::env::temp_dir().canonicalize().unwrap().join(format!(
        "ic-backup-public-upload-{lane}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(root.join("artifacts")).unwrap();
    root
}
fn plan(request: &str) -> OperationPlanRecord {
    serde_json::from_value(json!({
        "version":1,"context":{"network":"ab".repeat(32),"caller":"2vxsx-fae","release":"cd".repeat(32)},
        "inventory":{"version":1,"targets":[{"canister_id":TARGET,"parent_canister_id":null,"role":null,"module_hash":null}]},
        "selected_targets":[TARGET],"graph":{"version":1,"nodes":[{"operation_sequence":7,"depends_on":[]}]},
        "operations":[{"operation_sequence":7,"target":TARGET,"request":request,"budget":{"mutations":1,"observations":1}}],
        "budget":{"mutations":1,"observations":1}
    })).unwrap()
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one public source/upload reopen journey checks unchanged retained evidence at every boundary"
)]
fn exact_source_new_destination_and_independent_pending_attempts_survive_reopen() {
    let source_root = root("source");
    let metadata_root = root("metadata");
    let data_root = root("data");
    let source_layout = BackupLayoutGuard::acquire(&source_root).unwrap();
    let original = plan(&"ef".repeat(32));
    create_operation_plan(&source_layout, &original).unwrap();
    let cases: Vec<serde_json::Value> = serde_json::from_str(include_str!(
        "../src/model/ic_snapshot_metadata/tests/golden.json"
    ))
    .unwrap();
    let fixture = cases
        .iter()
        .find(|case| case["name"] == "upload-source-absent")
        .unwrap();
    let bytes = fixture["reply_hex"]
        .as_str()
        .unwrap()
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect::<Vec<_>>();
    let mut values: ReadCanisterSnapshotMetadataResult = candid::decode_one(&bytes).unwrap();
    values.wasm_module_size = 3;
    values.wasm_memory_size = 0;
    values.stable_memory_size = 0;
    values.wasm_chunk_store.clear();
    let raw = candid::encode_one(values).unwrap();
    let metadata_request = IcSnapshotMetadataRequest::new(TARGET, &[0, 255, 17]).unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&metadata_request, &raw).unwrap();
    let mut downloads = DownloadJournalGuard::create(
        &source_layout,
        original.digest().hash(),
        vec![DownloadArtifactRequest {
            canister_id: TARGET.into(),
            snapshot_id: TOKEN.into(),
            snapshot_taken_at_timestamp: metadata.metadata().taken_at_timestamp,
            snapshot_total_size_bytes: u64::MAX,
        }],
    )
    .unwrap();
    let writer = downloads
        .stage_ic_snapshot_artifact(TOKEN, &metadata, &raw)
        .unwrap();
    let read = IcSnapshotDataRequest::new(
        &metadata,
        SnapshotDataKind::WasmModule { offset: 0, size: 3 },
    )
    .unwrap();
    let reply = IcSnapshotDataReply::decode(
        &read,
        &candid::encode_one(ReadCanisterSnapshotDataResult { chunk: vec![42; 3] }).unwrap(),
    )
    .unwrap();
    writer.append(&reply).unwrap().finish().unwrap();
    let upload = downloads
        .prepare_ic_snapshot_upload_metadata(&original, TOKEN, &metadata)
        .unwrap();
    let upload_plan = plan(upload.binding_digest().hash());
    let upload_layout = BackupLayoutGuard::acquire(&metadata_root).unwrap();
    create_operation_plan(&upload_layout, &upload_plan).unwrap();
    source_layout
        .retain_restore(
            &metadata_root.join("operation-plan.json"),
            upload_plan.digest().hash(),
        )
        .unwrap();
    let source_plan_bytes = fs::read(source_root.join("operation-plan.json")).unwrap();
    let source_journal_bytes = fs::read(downloads.path()).unwrap();
    let references = fs::read(source_root.join("restore-references.json")).unwrap();
    let mut attempts =
        AttemptJournalGuard::create(&upload_layout, upload_plan.attempt_authority(7).unwrap())
            .unwrap();
    attempts.reserve_mutation().unwrap();
    let attempt_bytes = fs::read(attempts.path()).unwrap();
    let retained_attempt = attempts.record().unwrap().clone();
    let attempt =
        IcSnapshotUploadAttempt::new(&upload_plan, 7, &retained_attempt, &upload).unwrap();
    let mut metadata_provider =
        upload_provider::RefusingProvider::new(IcMutationProviderError::Unavailable);
    assert_eq!(
        metadata_provider.submit_upload(&attempt).unwrap_err(),
        IcMutationProviderError::Unavailable
    );
    assert_eq!(fs::read(attempts.path()).unwrap(), attempt_bytes);
    // Associate separately retained bytes, without invoking or retrying the provider.
    let acknowledgement = IcMutationAcknowledgement::new(IcMutationAcknowledgementInput {
        authority: attempt.authority().digest(),
        mutation_attempt: 1,
        context: upload_plan.context().clone(),
        target: TARGET.into(),
        reply: candid::encode_one(UploadCanisterSnapshotMetadataResult {
            snapshot_id: DESTINATION.to_vec(),
        })
        .unwrap(),
        evidence: ArtifactChecksumRecord::from_bytes(b"passive allocation declaration"),
    })
    .unwrap();
    let view = validate_acknowledgement(&attempt, &retained_attempt, &acknowledgement).unwrap();
    assert_eq!(
        view.reply().kind(),
        &IcSnapshotUploadReplyKind::Metadata {
            snapshot_id: DESTINATION.to_vec()
        }
    );
    assert_eq!(fs::read(attempts.path()).unwrap(), attempt_bytes);
    assert!(!attempts.record().unwrap().view().applied);
    assert!(attempts.reserve_mutation().is_err());

    // The test declares an independently qualified destination; the passive reply supplied no permit.
    let data = downloads
        .prepare_ic_snapshot_upload_data(
            &original,
            TOKEN,
            &upload,
            DESTINATION,
            SnapshotDataKind::WasmModule { offset: 1, size: 2 },
        )
        .unwrap();
    let args: UploadCanisterSnapshotDataArgs = candid::decode_one(data.arguments()).unwrap();
    assert_eq!(args.chunk, [42; 2]);
    assert_eq!(args.snapshot_id, DESTINATION);
    assert!(IcSnapshotUploadAttempt::new(&upload_plan, 7, &retained_attempt, &data).is_err());
    let data_plan = plan(data.binding_digest().hash());
    let data_layout = BackupLayoutGuard::acquire(&data_root).unwrap();
    create_operation_plan(&data_layout, &data_plan).unwrap();
    let mut data_attempts =
        AttemptJournalGuard::create(&data_layout, data_plan.attempt_authority(7).unwrap()).unwrap();
    data_attempts.reserve_mutation().unwrap();
    let data_attempt_bytes = fs::read(data_attempts.path()).unwrap();
    let record = data_attempts.record().unwrap();
    let attempt = IcSnapshotUploadAttempt::new(&data_plan, 7, record, &data).unwrap();
    let mut data_provider =
        upload_provider::RefusingProvider::new(IcMutationProviderError::Indeterminate);
    assert_eq!(
        data_provider.submit_upload(&attempt).unwrap_err(),
        IcMutationProviderError::Indeterminate
    );
    assert_eq!(fs::read(data_attempts.path()).unwrap(), data_attempt_bytes);
    let acknowledgement = IcMutationAcknowledgement::new(IcMutationAcknowledgementInput {
        authority: attempt.authority().digest(),
        mutation_attempt: 1,
        context: data_plan.context().clone(),
        target: TARGET.into(),
        reply: b"DIDL\0\0".to_vec(),
        evidence: ArtifactChecksumRecord::from_bytes(b"passive write declaration"),
    })
    .unwrap();
    assert_eq!(
        validate_acknowledgement(&attempt, record, &acknowledgement)
            .unwrap()
            .reply()
            .kind(),
        &IcSnapshotUploadReplyKind::DataAcknowledgement
    );
    assert_eq!(fs::read(data_attempts.path()).unwrap(), data_attempt_bytes);
    assert_eq!(fs::read(downloads.path()).unwrap(), source_journal_bytes);
    assert_eq!(
        fs::read(source_root.join("operation-plan.json")).unwrap(),
        source_plan_bytes
    );
    assert_eq!(
        fs::read(source_root.join("restore-references.json")).unwrap(),
        references
    );
    let payload_digest = data.binding_digest();
    drop(data_attempts);
    drop(data_layout);
    drop(attempts);
    drop(upload_layout);
    drop(downloads);
    drop(source_layout);

    // Lost provider replies and ordinary local reopen leave both paid effects pending.
    let source_layout = BackupLayoutGuard::acquire(&source_root).unwrap();
    let retained = read_operation_plan(&source_layout, &original.digest()).unwrap();
    let downloads = DownloadJournalGuard::open(&source_layout, retained.digest().hash()).unwrap();
    assert_eq!(
        downloads
            .ic_snapshot_metrics()
            .upload_metadata_success_ns()
            .samples(),
        0
    );
    assert_eq!(
        downloads
            .ic_snapshot_metrics()
            .prepared_chunk_bytes()
            .latest(),
        None
    );
    let data_layout = BackupLayoutGuard::acquire(&data_root).unwrap();
    let retained_data = read_operation_plan(&data_layout, &data_plan.digest()).unwrap();
    let mut data_attempts =
        AttemptJournalGuard::open(&data_layout, &retained_data.attempt_authority(7).unwrap())
            .unwrap();
    let upload = downloads
        .prepare_ic_snapshot_upload_metadata(&retained, TOKEN, &metadata)
        .unwrap();
    let data = downloads
        .prepare_ic_snapshot_upload_data(
            &retained,
            TOKEN,
            &upload,
            DESTINATION,
            SnapshotDataKind::WasmModule { offset: 1, size: 2 },
        )
        .unwrap();
    assert_eq!(data.binding_digest(), payload_digest);
    let measurements = downloads.ic_snapshot_metrics();
    assert_eq!(measurements.upload_metadata_success_ns().samples(), 1);
    assert_eq!(measurements.upload_data_success_ns().samples(), 1);
    assert_eq!(measurements.verification_success_ns().samples(), 3);
    assert_eq!(measurements.prepared_chunk_bytes().total(), 2);
    assert_eq!(
        data_attempts.record().unwrap().view().pending_mutation,
        Some(1)
    );
    assert!(data_attempts.reserve_mutation().is_err());
    data_attempts
        .reserve_observation(
            1,
            ArtifactChecksumRecord::from_bytes(b"original recovery request").hash(),
        )
        .unwrap();
    let before = fs::read(data_attempts.path()).unwrap();
    assert!(
        IcSnapshotUploadAttempt::new(&retained_data, 7, data_attempts.record().unwrap(), &data)
            .is_err()
    );
    assert_eq!(fs::read(data_attempts.path()).unwrap(), before);
    assert_eq!(
        data_attempts.record().unwrap().view().pending_observation,
        Some(2)
    );
    assert_eq!(
        data_attempts.record().unwrap().view().mutations_remaining,
        0
    );
    assert_eq!(
        data_attempts
            .record()
            .unwrap()
            .view()
            .observations_remaining,
        0
    );
    assert_eq!(
        fs::read(source_root.join("restore-references.json")).unwrap(),
        references
    );
    assert_eq!(fs::read(downloads.path()).unwrap(), source_journal_bytes);
    let upload_layout = BackupLayoutGuard::acquire(&metadata_root).unwrap();
    let retained_metadata = read_operation_plan(&upload_layout, &upload_plan.digest()).unwrap();
    let attempts = AttemptJournalGuard::open(
        &upload_layout,
        &retained_metadata.attempt_authority(7).unwrap(),
    )
    .unwrap();
    assert_eq!(fs::read(attempts.path()).unwrap(), attempt_bytes);
    assert_eq!(attempts.record().unwrap().view().pending_mutation, Some(1));
    assert_eq!(downloads.ic_snapshot_metrics(), measurements);
    assert_eq!(metadata_provider.calls, 1);
    assert_eq!(data_provider.calls, 1);
}
