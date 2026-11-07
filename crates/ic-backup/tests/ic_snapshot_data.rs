//! Native metadata/data replay preserves exact original spending and retention.

use ic_platform::candid::contract;
use support::hex_bytes as bytes;
#[expect(
    dead_code,
    reason = "this integration target uses only a subset of the shared fixture helpers"
)]
mod support;

use ic_backup::{
    model::{
        ic_request::{IcManagementMethodRecord, IcManagementRequest, IcManagementRequestRecord},
        ic_snapshot_coverage::{IcSnapshotDataCoverage, IcSnapshotDataCoverageError},
        ic_snapshot_data::{IcSnapshotDataReply, IcSnapshotDataRequest},
        ic_snapshot_metadata::{IcSnapshotMetadataReply, IcSnapshotMetadataRequest},
        operation_plan::OperationPlanRecord,
    },
    ops::persistence::{AttemptJournalGuard, BackupLayoutGuard, read_operation_plan},
};
use ic_platform::management_canister_types::SnapshotDataKind;
use serde::Deserialize;
use std::fs;
use support::observation_replay::PendingObservation;

#[derive(Deserialize)]
struct Case {
    name: String,
    kind: SnapshotDataKind,
    reply_hex: String,
    digest: String,
}

#[test]
fn every_registered_data_kind_reopens_exact_evidence_without_settlement_or_new_spending() {
    let cases: Vec<Case> = serde_json::from_str(include_str!(
        "../src/model/ic_snapshot_data/tests/golden.json"
    ))
    .unwrap();
    let sources: Vec<serde_json::Value> = serde_json::from_str(include_str!(
        "../src/model/ic_snapshot_metadata/tests/golden.json"
    ))
    .unwrap();
    let source = sources
        .iter()
        .find(|source| source["name"] == "data-source")
        .unwrap();
    let raw_metadata = bytes(source["reply_hex"].as_str().unwrap());
    for case in cases {
        retained_case(case, &raw_metadata);
    }
}

fn retained_case(case: Case, raw_metadata: &[u8]) {
    let root = support::temp_root(&format!("ic-backup-data-{}", case.name));
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let capture = IcManagementRequestRecord::new(IcManagementRequest {
        method: IcManagementMethodRecord::TakeCanisterSnapshot,
        target: "renrk-eyaaa-aaaaa-aaada-cai".into(),
        snapshot_id: None,
    })
    .unwrap();
    let metadata_request = IcSnapshotMetadataRequest::new(capture.target(), &[0, 255, 17]).unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&metadata_request, raw_metadata).unwrap();
    let request = IcSnapshotDataRequest::new(&metadata, case.kind.clone()).unwrap();
    let arguments = request.arguments().to_vec();
    let plan: OperationPlanRecord =
        support::single_operation_plan(capture.target(), 7, capture.digest().hash());
    let (journal, original) = PendingObservation::reserve(&layout, &plan, request.digest().hash());
    let raw = bytes(&case.reply_hex);
    let reply = IcSnapshotDataReply::decode(&request, &raw).unwrap();
    assert_eq!(reply.digest().hash(), case.digest);
    let mut coverage = IcSnapshotDataCoverage::new(&metadata);
    let coverage_result = coverage.admit(&reply);
    if matches!(request.kind(), SnapshotDataKind::WasmChunk { .. }) {
        assert_eq!(coverage_result, Ok(()));
        assert_eq!(coverage.covered_chunks(), 1);
    } else {
        assert_eq!(
            coverage_result,
            Err(IcSnapshotDataCoverageError::NoncontiguousRange)
        );
        assert_eq!(coverage.covered_region_bytes(), [0; 3]);
    }
    assert!(coverage.complete().is_none());
    drop(coverage);
    let checksum = reply.chunk_checksum().clone();
    let metadata_digest = metadata.digest();
    let intent = plan.digest();
    // Tiny local fixture evidence is not a product progress/completion record.
    fs::write(root.join("metadata.candid"), raw_metadata).unwrap();
    fs::write(root.join("data.candid"), &raw).unwrap();
    drop(reply);
    drop(request);
    drop(metadata);
    drop(journal);
    drop(layout);

    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let plan = read_operation_plan(&layout, &intent).unwrap();
    let authority = plan.attempt_authority(7).unwrap();
    let mut journal = AttemptJournalGuard::open(&layout, &authority).unwrap();
    let stored_metadata = fs::read(root.join("metadata.candid")).unwrap();
    assert_eq!(stored_metadata, raw_metadata);
    let metadata = IcSnapshotMetadataReply::decode(&metadata_request, &stored_metadata).unwrap();
    assert_eq!(metadata.digest(), metadata_digest);
    let request = IcSnapshotDataRequest::new(&metadata, case.kind).unwrap();
    assert_eq!(request.arguments(), arguments);
    assert_eq!(
        journal.record().unwrap().pending_observation_request(),
        Some(request.digest().hash())
    );
    let stored = fs::read(root.join("data.candid")).unwrap();
    assert_eq!(stored, raw);
    let admitted = IcSnapshotDataReply::decode(&request, &stored).unwrap();
    let mut reopened_coverage = IcSnapshotDataCoverage::new(&metadata);
    assert_eq!(reopened_coverage.covered_region_bytes(), [0; 3]);
    assert_eq!(reopened_coverage.covered_chunks(), 0);
    assert_eq!(reopened_coverage.admit(&admitted), coverage_result);
    assert!(reopened_coverage.complete().is_none());
    assert_eq!(admitted.digest().hash(), case.digest);
    assert_eq!(admitted.chunk_checksum(), &checksum);
    assert_eq!(admitted.chunk(), [0, 255, 17]);

    original.assert_unchanged(&layout, &mut journal, request.digest().hash());
    assert_eq!(fs::read(root.join("data.candid")).unwrap(), raw);
    assert_eq!(
        fs::read(root.join("metadata.candid")).unwrap(),
        raw_metadata
    );
    drop(admitted);
    drop(reopened_coverage);
    drop(request);
    drop(metadata);
    drop(journal);
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn complete_coverage_streams_without_a_persisted_or_retained_byte_claim() {
    use ic_platform::management_canister_types::{
        ReadCanisterSnapshotDataResult, ReadCanisterSnapshotMetadataResult,
    };
    let sources: Vec<serde_json::Value> = serde_json::from_str(include_str!(
        "../src/model/ic_snapshot_metadata/tests/golden.json"
    ))
    .unwrap();
    let source = sources
        .iter()
        .find(|source| source["name"] == "data-source")
        .unwrap();
    let mut values: ReadCanisterSnapshotMetadataResult = ic_platform::candid::decode_one(
        &bytes(source["reply_hex"].as_str().unwrap()),
        support::WIRE_ORACLE_LIMITS,
    )
    .unwrap();
    values.wasm_module_size = 3;
    values.wasm_memory_size = 0;
    values.stable_memory_size = 0;
    values.wasm_chunk_store.clear();
    let request =
        IcSnapshotMetadataRequest::new("renrk-eyaaa-aaaaa-aaada-cai", &[0, 255, 17]).unwrap();
    let raw_metadata = contract::encode_one(values).unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw_metadata).unwrap();
    let mut coverage = IcSnapshotDataCoverage::new(&metadata);
    for offset in 0..3 {
        let data =
            IcSnapshotDataRequest::new(&metadata, SnapshotDataKind::WasmModule { offset, size: 1 })
                .unwrap();
        let raw = contract::encode_one(ReadCanisterSnapshotDataResult { chunk: vec![42] }).unwrap();
        let reply = IcSnapshotDataReply::decode(&data, &raw).unwrap();
        coverage.admit(&reply).unwrap();
    }
    assert_eq!(
        coverage.complete().unwrap().metadata().digest(),
        metadata.digest()
    );
    assert_eq!(coverage.covered_region_bytes(), [3, 0, 0]);
    drop(coverage);
    let retained = IcSnapshotMetadataReply::decode(&request, &raw_metadata).unwrap();
    assert!(IcSnapshotDataCoverage::new(&retained).complete().is_none());
}
