//! Local metadata evidence replay retains consumed authority; no IC effects.

mod support;

use ic_backup::{
    model::{
        ic_request::{IcManagementMethodRecord, IcManagementRequest, IcManagementRequestRecord},
        ic_snapshot_metadata::{
            IcSnapshotMetadataError, IcSnapshotMetadataReply, IcSnapshotMetadataRequest,
        },
        operation_plan::OperationPlanRecord,
    },
    ops::persistence::{
        AttemptJournalGuard, BackupLayoutGuard, create_operation_plan, read_operation_plan,
    },
};
use serde_json::json;
use std::fs;

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one original byte/spending/reopen journey retains all custody assertions together"
)]
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
    let original_journal = fs::read(journal.path()).unwrap();
    let original_plan = fs::read(root.join("operation-plan.json")).unwrap();
    let original_view = journal.record().unwrap().view();
    let cases: Vec<serde_json::Value> = serde_json::from_str(include_str!(
        "../src/model/ic_snapshot_metadata/tests/golden.json"
    ))
    .unwrap();
    let raw: Vec<u8> = cases[0]["reply_hex"]
        .as_str()
        .unwrap()
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect();
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
    assert_eq!(
        IcSnapshotMetadataReply::decode(&reconstructed, &retained[..retained.len() - 1])
            .unwrap_err(),
        IcSnapshotMetadataError::InvalidReply
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
    assert_eq!(fs::read(root.join("metadata.candid")).unwrap(), raw);
    drop(admitted);
    drop(journal);
    drop(layout);
    fs::remove_dir_all(root).unwrap();
}
