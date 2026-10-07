//! Public retained-baseline comparison without capture attribution or settlement.

mod support;

use ic_backup::{
    model::{
        attempt_journal::AttemptJournalRecordError,
        ic_request::{IcManagementMethodRecord, IcManagementRequest, IcManagementRequestRecord},
        ic_snapshot_reply::{IcSnapshotInfo, IcSnapshotReply},
        operation_plan::OperationPlanRecord,
    },
    ops::persistence::{
        AttemptJournalError, AttemptJournalGuard, BackupLayoutGuard, create_json_durable,
        create_operation_plan, read_json, read_operation_plan,
    },
    policy::snapshot_inventory_delta::{SnapshotInventoryDeltaError, compare},
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
    .expect("exact request")
}

fn wire(ids: &[u8]) -> Vec<u8> {
    candid::encode_one(
        ids.iter()
            .map(|id| Snapshot {
                id: vec![*id],
                taken_at_timestamp: 123,
                total_size: 456,
            })
            .collect::<Vec<_>>(),
    )
    .expect("local inventory fixture")
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one public recovery journey retains baseline, observation bytes and original spent authority together"
)]
fn singleton_and_ambiguous_deltas_preserve_pending_capture_and_original_evidence() {
    let root = support::temp_root("ic-backup-public-delta");
    let layout = BackupLayoutGuard::acquire(&root).expect("layout exclusion");
    let capture = request(
        IcManagementMethodRecord::TakeCanisterSnapshot,
        "renrk-eyaaa-aaaaa-aaada-cai",
    );
    let list = request(
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
    // Integration-owned original baseline fixture; no live observation or receipt.
    let baseline_wire = wire(&[1]);
    let baseline_path = root.join("baseline-bytes.json");
    create_json_durable(&baseline_path, &baseline_wire).expect("retain original baseline");
    let authority = plan.attempt_authority(7).expect("original allowance");
    capture
        .validate_mutation_binding(authority.binding())
        .expect("original capture");
    let mut journal = AttemptJournalGuard::create(&layout, authority).expect("original journal");
    let mutation = journal
        .reserve_mutation()
        .expect("reserve before any effect");
    list.validate_observation_binding(
        journal.record().expect("record").authority().binding(),
        &list.digest(),
    )
    .expect("exact independent list request");
    let observation = journal
        .reserve_observation(mutation, list.digest().hash())
        .expect("reserve original observation");
    let observed_wire = wire(&[2, 1]);
    let observed_path = root.join("observed-bytes.json");
    create_json_durable(&observed_path, &observed_wire).expect("retain observation fixture bytes");
    let journal_path = journal.path();
    let journal_bytes = fs::read(&journal_path).expect("spent journal bytes");
    let baseline_bytes = fs::read(&baseline_path).expect("baseline evidence bytes");
    let observed_bytes = fs::read(&observed_path).expect("observation evidence bytes");
    let original_view = journal.record().expect("original spending").view();
    drop(journal);
    drop(layout);

    let layout = BackupLayoutGuard::acquire(&root).expect("new owner");
    let plan = read_operation_plan(&layout, &intent).expect("original retained plan");
    let mut journal = AttemptJournalGuard::open(
        &layout,
        &plan
            .attempt_authority(7)
            .expect("unchanged original allowance"),
    )
    .expect("pending journal");
    let before: Vec<u8> = read_json(&baseline_path, 8192).expect("bounded baseline fixture read");
    let after: Vec<u8> = read_json(&observed_path, 8192).expect("bounded observation fixture read");
    let baseline = IcSnapshotReply::decode(&list, &before).expect("exact retained baseline");
    let observed = IcSnapshotReply::decode(&list, &after).expect("exact retained observation");
    let delta = compare(&capture, &baseline, &observed).expect("singleton candidate");
    assert_eq!(delta.candidates().len(), 1);
    assert_eq!(delta.candidates()[0].id(), [2]);
    assert_eq!(delta.capture().digest(), capture.digest());
    assert_eq!(
        delta.baseline().payload_checksum(),
        &ic_backup::model::artifacts::ArtifactChecksumRecord::from_bytes(&baseline_wire)
    );
    assert_eq!(
        delta.observed().payload_checksum(),
        &ic_backup::model::artifacts::ArtifactChecksumRecord::from_bytes(&observed_wire)
    );

    let multiple = IcSnapshotReply::decode(&list, &wire(&[3, 1, 2])).expect("multiple candidates");
    let ambiguous = compare(&capture, &baseline, &multiple).expect("explicit ambiguity");
    assert_eq!(
        ambiguous
            .candidates()
            .iter()
            .map(|entry| entry.id())
            .collect::<Vec<_>>(),
        vec![&[2][..], &[3][..]]
    );
    let missing = IcSnapshotReply::decode(&list, &wire(&[2])).expect("lost baseline fixture");
    assert_eq!(
        compare(&capture, &baseline, &missing).unwrap_err(),
        SnapshotInventoryDeltaError::LostBaseline
    );
    let unchanged = compare(&capture, &baseline, &baseline).expect("zero candidates");
    let no_candidates: &[&IcSnapshotInfo] = &[];
    assert_eq!(unchanged.candidates(), no_candidates);
    assert_eq!(
        journal
            .record()
            .expect("unchanged pending accounting")
            .view(),
        original_view
    );
    assert_eq!(original_view.pending_mutation, Some(mutation));
    assert_eq!(original_view.pending_observation, Some(observation));
    assert_eq!(original_view.mutations_remaining, 0);
    assert_eq!(original_view.observations_remaining, 0);
    assert!(matches!(journal.reserve_mutation(),
        Err(AttemptJournalError::Record(AttemptJournalRecordError::ObservationPending { attempt }))
        if attempt == observation));
    assert_eq!(
        fs::read(&journal_path).expect("journal preserved"),
        journal_bytes
    );
    assert_eq!(
        fs::read(&baseline_path).expect("baseline preserved"),
        baseline_bytes
    );
    assert_eq!(
        fs::read(&observed_path).expect("observation preserved"),
        observed_bytes
    );
    drop(journal);
    drop(layout);
    fs::remove_dir_all(root).expect("clean successful owned fixture");
}
