//! Public byte-reply recovery without remote observations or automatic settlement.

mod support;

use ic_backup::{
    model::{
        artifacts::ChecksumError,
        attempt_journal::AttemptJournalRecordError,
        ic_request::{IcManagementMethodRecord, IcManagementRequest, IcManagementRequestRecord},
        ic_snapshot_reply::{IcSnapshotReply, IcSnapshotReplyError},
        operation_plan::OperationPlanRecord,
    },
    ops::persistence::{
        AttemptJournalError, AttemptJournalGuard, BackupLayoutGuard, create_json_durable,
        create_operation_plan, read_json, read_operation_plan,
    },
};
use ic_management_canister_types::Snapshot;
use serde_json::json;
use std::fs;

fn request(method: IcManagementMethodRecord, target: &str) -> IcManagementRequestRecord {
    IcManagementRequestRecord::new(IcManagementRequest {
        method,
        target: target.into(),
        snapshot_id: None,
    })
    .expect("existing exact wire owner")
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one linear public recovery journey keeps original bytes, spending and replay assertions together"
)]
fn retains_original_spending_and_reply_bytes_across_reopen_and_decode_failures() {
    let root = support::temp_root("ic-backup-public-reply");
    let layout = BackupLayoutGuard::acquire(&root).expect("layout exclusion");
    let capture = request(
        IcManagementMethodRecord::TakeCanisterSnapshot,
        "renrk-eyaaa-aaaaa-aaada-cai",
    );
    let inventory = request(
        IcManagementMethodRecord::ListCanisterSnapshots,
        capture.target(),
    );
    let plan: OperationPlanRecord = serde_json::from_value(json!({
        "version": 1,
        "context": {"network": "ab".repeat(32), "caller": "2vxsx-fae", "release": "cd".repeat(32)},
        "inventory": {"version": 1, "targets": [{"canister_id": capture.target(), "parent_canister_id": null, "role": null, "module_hash": null}]},
        "selected_targets": [capture.target()],
        "graph": {"version": 1, "nodes": [{"operation_sequence": 7, "depends_on": []}]},
        "operations": [{"operation_sequence": 7, "target": capture.target(), "request": capture.digest().hash(), "budget": {"mutations": 1, "observations": 1}}],
        "budget": {"mutations": 1, "observations": 1}
    })).expect("original plan");
    create_operation_plan(&layout, &plan).expect("retain original intent");
    let intent = plan.digest();
    let authority = plan.attempt_authority(7).expect("original allowance");
    capture
        .validate_mutation_binding(authority.binding())
        .expect("original exact capture");
    let mut journal =
        AttemptJournalGuard::create(&layout, authority).expect("durable original journal");
    let mutation = journal
        .reserve_mutation()
        .expect("spend before any integration-owned effect");
    inventory
        .validate_observation_binding(
            journal.record().expect("record").authority().binding(),
            &inventory.digest(),
        )
        .expect("independent exact observation payload");
    let observation = journal
        .reserve_observation(mutation, inventory.digest().hash())
        .expect("spend independent observation allowance");
    let journal_path = journal.path();
    let retained_journal = fs::read(&journal_path).expect("retained exact bytes");
    let retained_view = journal.record().expect("original spending").view();
    let raw = candid::encode_one(Snapshot {
        id: vec![0, 255, 17],
        taken_at_timestamp: 123,
        total_size: 456,
    })
    .expect("local wire fixture");
    let decoded = IcSnapshotReply::decode(&capture, &raw).expect("local byte admission");
    let expected_reply = decoded.digest();
    let path = root.join("reply-bytes.json");
    // Tiny integration-owned bytes use existing immutable bounded JSON IO;
    // this is not a product receipt schema or authenticated transport result.
    create_json_durable(&path, &raw).expect("retain exact wire evidence");
    let stored_bytes = fs::read(&path).expect("retained evidence bytes");
    drop(decoded);
    drop(journal);
    drop(layout);

    let layout = BackupLayoutGuard::acquire(&root).expect("new owner");
    let plan = read_operation_plan(&layout, &intent).expect("same original plan");
    let authority = plan.attempt_authority(7).expect("same original allowance");
    let mut journal =
        AttemptJournalGuard::open(&layout, &authority).expect("spent pending journal");
    let retained: Vec<u8> = read_json(&path, 8192).expect("bounded tiny fixture evidence");
    assert_eq!(retained, raw);
    let reply = IcSnapshotReply::decode(&capture, &retained).expect("local replay without effects");
    assert_eq!(reply.digest(), expected_reply);
    assert_eq!(reply.snapshots()[0].id(), [0, 255, 17]);
    assert_eq!(reply.snapshots()[0].taken_at_timestamp(), 123);
    assert_eq!(reply.snapshots()[0].total_size(), 456);
    let wrong_target = request(IcManagementMethodRecord::TakeCanisterSnapshot, "aaaaa-aa");
    let wrong_association =
        IcSnapshotReply::decode(&wrong_target, &retained).expect("wire has no target identity");
    assert_ne!(wrong_association.digest(), expected_reply);
    assert!(matches!(
        wrong_association.digest().verify(expected_reply.hash()),
        Err(ChecksumError::ChecksumMismatch { .. })
    ));
    assert_eq!(
        IcSnapshotReply::decode(&inventory, &retained).unwrap_err(),
        IcSnapshotReplyError::InvalidReply
    );
    assert_eq!(
        IcSnapshotReply::decode(&capture, b"lost or malformed reply").unwrap_err(),
        IcSnapshotReplyError::InvalidReply
    );
    let duplicate = candid::encode_one(vec![
        Snapshot {
            id: vec![1],
            taken_at_timestamp: 0,
            total_size: 0
        };
        2
    ])
    .expect("ambiguous inventory");
    assert_eq!(
        IcSnapshotReply::decode(&inventory, &duplicate).unwrap_err(),
        IcSnapshotReplyError::DuplicateSnapshotId
    );
    assert_eq!(
        journal.record().expect("unchanged accounting").view(),
        retained_view
    );
    assert_eq!(retained_view.pending_mutation, Some(mutation));
    assert_eq!(retained_view.pending_observation, Some(observation));
    assert_eq!(retained_view.mutations_remaining, 0);
    assert_eq!(retained_view.observations_remaining, 0);
    assert!(
        matches!(journal.reserve_mutation(), Err(AttemptJournalError::Record(AttemptJournalRecordError::ObservationPending { attempt })) if attempt == observation)
    );
    assert_eq!(
        fs::read(&journal_path).expect("unchanged journal bytes"),
        retained_journal
    );
    assert_eq!(
        fs::read(&path).expect("unchanged reply bytes"),
        stored_bytes
    );
    drop(journal);
    drop(layout);
    fs::remove_dir_all(root).expect("clean successful owned fixture");
}
