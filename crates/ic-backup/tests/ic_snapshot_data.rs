//! Native metadata/data replay preserves exact original spending and retention.

mod support;

use ic_backup::{
    model::{
        ic_request::{IcManagementMethodRecord, IcManagementRequest, IcManagementRequestRecord},
        ic_snapshot_coverage::{IcSnapshotDataCoverage, IcSnapshotDataCoverageError},
        ic_snapshot_data::{IcSnapshotDataError, IcSnapshotDataReply, IcSnapshotDataRequest},
        ic_snapshot_metadata::{IcSnapshotMetadataReply, IcSnapshotMetadataRequest},
        operation_plan::OperationPlanRecord,
    },
    ops::persistence::{
        AttemptJournalGuard, BackupLayoutGuard, create_operation_plan, read_operation_plan,
    },
};
use ic_management_canister_types::SnapshotDataKind;
use serde::Deserialize;
use serde_json::json;
use std::fs;

#[derive(Deserialize)]
struct Case {
    name: String,
    kind: SnapshotDataKind,
    reply_hex: String,
    digest: String,
}

fn bytes(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
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

#[expect(
    clippy::too_many_lines,
    reason = "one original evidence/reopen journey keeps all spending and retention assertions together"
)]
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
    let plan: OperationPlanRecord = serde_json::from_value(json!({
        "version": 1,
        "context": {"network": "ab".repeat(32), "caller": "2vxsx-fae", "release": "cd".repeat(32)},
        "inventory": {"version": 1, "targets": [{"canister_id": capture.target(), "parent_canister_id": null, "role": null, "module_hash": null}]},
        "selected_targets": [capture.target()],
        "graph": {"version": 1, "nodes": [{"operation_sequence": 7, "depends_on": []}]},
        "operations": [{"operation_sequence": 7, "target": capture.target(), "request": capture.digest().hash(), "budget": {"mutations": 1, "observations": 1}}],
        "budget": {"mutations": 1, "observations": 1}
    })).unwrap();
    create_operation_plan(&layout, &plan).unwrap();
    layout
        .retain_restore(&root.join("restore-reference.json"), plan.digest().hash())
        .unwrap();
    let references = layout.restore_references().unwrap();
    let authority = plan.attempt_authority(7).unwrap();
    let mut journal = AttemptJournalGuard::create(&layout, authority).unwrap();
    let mutation = journal.reserve_mutation().unwrap();
    let observation = journal
        .reserve_observation(mutation, request.digest().hash())
        .unwrap();
    let original_view = journal.record().unwrap().view();
    let original_journal = fs::read(journal.path()).unwrap();
    let original_plan = fs::read(root.join("operation-plan.json")).unwrap();
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
    assert_eq!(
        IcSnapshotDataReply::decode(&request, &stored[..stored.len() - 1]).unwrap_err(),
        IcSnapshotDataError::InvalidReply
    );
    assert_eq!(journal.record().unwrap().view(), original_view);
    assert_eq!(original_view.pending_mutation, Some(mutation));
    assert_eq!(original_view.pending_observation, Some(observation));
    assert!(!original_view.applied);
    assert_eq!(
        (
            original_view.mutations_remaining,
            original_view.observations_remaining
        ),
        (0, 0)
    );
    assert!(journal.reserve_mutation().is_err());
    assert!(
        journal
            .reserve_observation(mutation, request.digest().hash())
            .is_err()
    );
    assert_eq!(fs::read(journal.path()).unwrap(), original_journal);
    assert_eq!(
        fs::read(root.join("operation-plan.json")).unwrap(),
        original_plan
    );
    assert_eq!(layout.restore_references().unwrap(), references);
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
    use ic_management_canister_types::{
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
    let mut values: ReadCanisterSnapshotMetadataResult =
        candid::decode_one(&bytes(source["reply_hex"].as_str().unwrap())).unwrap();
    values.wasm_module_size = 3;
    values.wasm_memory_size = 0;
    values.stable_memory_size = 0;
    values.wasm_chunk_store.clear();
    let request =
        IcSnapshotMetadataRequest::new("renrk-eyaaa-aaaaa-aaada-cai", &[0, 255, 17]).unwrap();
    let raw_metadata = candid::encode_one(values).unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw_metadata).unwrap();
    let mut coverage = IcSnapshotDataCoverage::new(&metadata);
    for offset in 0..3 {
        let data =
            IcSnapshotDataRequest::new(&metadata, SnapshotDataKind::WasmModule { offset, size: 1 })
                .unwrap();
        let raw = candid::encode_one(ReadCanisterSnapshotDataResult { chunk: vec![42] }).unwrap();
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
