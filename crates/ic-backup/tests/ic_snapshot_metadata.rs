//! Local metadata evidence replay retains consumed authority; no IC effects.

#[expect(
    dead_code,
    reason = "this integration target uses only a subset of the shared fixture helpers"
)]
mod support;

use ic_backup::{
    model::{
        ic_request::{IcManagementMethodRecord, IcManagementRequest, IcManagementRequestRecord},
        ic_snapshot_metadata::{IcSnapshotMetadataReply, IcSnapshotMetadataRequest},
        operation_plan::OperationPlanRecord,
    },
    ops::persistence::{AttemptJournalGuard, BackupLayoutGuard, read_operation_plan},
};
use std::fs;
use support::observation_replay::PendingObservation;

#[test]
fn replays_exact_metadata_bytes_without_settlement_new_spending_or_reference_release() {
    let root = support::temp_root("ic-backup-metadata");
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let capture = IcManagementRequestRecord::new(IcManagementRequest {
        method: IcManagementMethodRecord::TakeCanisterSnapshot,
        target: "renrk-eyaaa-aaaaa-aaada-cai".into(),
        snapshot_id: None,
    })
    .unwrap();
    let request = IcSnapshotMetadataRequest::new(capture.target(), &[0, 255, 17]).unwrap();
    let plan: OperationPlanRecord =
        support::single_operation_plan(capture.target(), 7, capture.digest().hash());
    let (journal, original) = PendingObservation::reserve(&layout, &plan, request.digest().hash());
    let cases: Vec<serde_json::Value> = serde_json::from_str(include_str!(
        "../src/model/ic_snapshot_metadata/tests/golden.json"
    ))
    .unwrap();
    let raw = support::hex_bytes(cases[0]["reply_hex"].as_str().unwrap());
    let reply = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let digest = reply.digest();
    let arguments = request.arguments().to_vec();
    let intent = plan.digest();
    // Tiny test-owned evidence uses no product receipt/transfer record or provider.
    fs::write(root.join("metadata.candid"), &raw).unwrap();
    drop(reply);
    drop(journal);
    drop(layout);

    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let plan = read_operation_plan(&layout, &intent).unwrap();
    let authority = plan.attempt_authority(7).unwrap();
    let mut journal = AttemptJournalGuard::open(&layout, &authority).unwrap();
    let reconstructed =
        IcSnapshotMetadataRequest::new(request.target(), request.snapshot_id()).unwrap();
    assert_eq!(reconstructed.arguments(), arguments);
    assert_eq!(
        journal.record().unwrap().pending_observation_request(),
        Some(reconstructed.digest().hash())
    );
    let retained = fs::read(root.join("metadata.candid")).unwrap();
    assert_eq!(retained, raw);
    let admitted = IcSnapshotMetadataReply::decode(&reconstructed, &retained).unwrap();
    assert_eq!(admitted.digest(), digest);
    assert_eq!(admitted.metadata().wasm_module_size, u64::MAX);
    assert!(admitted.metadata().globals[5].is_none());
    original.assert_unchanged(&layout, &mut journal, request.digest().hash());
    assert_eq!(fs::read(root.join("metadata.candid")).unwrap(), raw);
    drop(admitted);
    drop(journal);
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}
