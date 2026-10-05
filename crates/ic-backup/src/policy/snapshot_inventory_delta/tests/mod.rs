use super::*;
use crate::model::{
    ic_request::IcManagementRequest, ic_snapshot_reply::MAX_IC_SNAPSHOT_REPLY_ENTRIES,
};
use ic_management_canister_types::Snapshot;

const TARGET: &str = "renrk-eyaaa-aaaaa-aaada-cai";

fn request(method: IcManagementMethodRecord, target: &str) -> IcManagementRequestRecord {
    IcManagementRequestRecord::new(IcManagementRequest {
        method,
        target: target.into(),
        snapshot_id: (method == IcManagementMethodRecord::LoadCanisterSnapshot).then(|| vec![1]),
    })
    .expect("exact request")
}

fn snapshot(id: &[u8]) -> Snapshot {
    Snapshot {
        id: id.into(),
        taken_at_timestamp: 123,
        total_size: 456,
    }
}

fn inventory(request: &IcManagementRequestRecord, snapshots: Vec<Snapshot>) -> IcSnapshotReply<'_> {
    IcSnapshotReply::decode(
        request,
        &candid::encode_one(snapshots).expect("fixture wire"),
    )
    .expect("admitted inventory")
}

#[test]
fn exposes_zero_single_and_multiple_candidates_with_original_evidence() {
    let capture = request(IcManagementMethodRecord::TakeCanisterSnapshot, TARGET);
    let list = request(IcManagementMethodRecord::ListCanisterSnapshots, TARGET);
    let baseline = inventory(&list, vec![snapshot(&[255]), snapshot(&[0, 0])]);
    for additions in [vec![], vec![vec![0]], vec![vec![1], vec![0]]] {
        let mut snapshots = vec![snapshot(&[255]), snapshot(&[0, 0])];
        snapshots.extend(additions.iter().map(|id| snapshot(id)));
        let observed = inventory(&list, snapshots);
        let delta = compare(&capture, &baseline, &observed).expect("pure delta");
        let mut expected = additions;
        expected.sort();
        assert_eq!(
            delta
                .candidates()
                .iter()
                .map(|entry| entry.id())
                .collect::<Vec<_>>(),
            expected.iter().map(Vec::as_slice).collect::<Vec<_>>()
        );
        assert_eq!(delta.capture().digest(), capture.digest());
        assert_eq!(delta.baseline().digest(), baseline.digest());
        assert_eq!(delta.observed().digest(), observed.digest());
    }
}

#[test]
fn projects_canonical_ids_without_erasing_raw_wire_order() {
    let capture = request(IcManagementMethodRecord::TakeCanisterSnapshot, TARGET);
    let list = request(IcManagementMethodRecord::ListCanisterSnapshots, TARGET);
    let baseline = inventory(&list, vec![]);
    let left = inventory(
        &list,
        vec![snapshot(&[255]), snapshot(&[0, 0]), snapshot(&[0])],
    );
    let right = inventory(
        &list,
        vec![snapshot(&[0]), snapshot(&[0, 0]), snapshot(&[255])],
    );
    let left_delta = compare(&capture, &baseline, &left).expect("left");
    let right_delta = compare(&capture, &baseline, &right).expect("right");
    assert_eq!(left_delta.candidates(), right_delta.candidates());
    assert_eq!(left_delta.candidates()[0].id(), [0]);
    assert_eq!(left_delta.candidates()[1].id(), [0, 0]);
    assert_eq!(left_delta.candidates()[2].id(), [255]);
    assert_ne!(
        left_delta.observed().digest(),
        right_delta.observed().digest()
    );
}

#[test]
fn rejects_lost_baseline_at_every_merge_position_even_with_one_new_candidate() {
    let capture = request(IcManagementMethodRecord::TakeCanisterSnapshot, TARGET);
    let list = request(IcManagementMethodRecord::ListCanisterSnapshots, TARGET);
    let baseline = inventory(&list, vec![snapshot(&[1]), snapshot(&[3]), snapshot(&[5])]);
    for missing in [1, 3, 5] {
        let mut snapshots = [1, 3, 5]
            .into_iter()
            .filter(|id| *id != missing)
            .map(|id| snapshot(&[id]))
            .collect::<Vec<_>>();
        snapshots.push(snapshot(&[2]));
        let observed = inventory(&list, snapshots);
        assert_eq!(
            compare(&capture, &baseline, &observed).unwrap_err(),
            SnapshotInventoryDeltaError::LostBaseline
        );
    }
    let empty = inventory(&list, vec![]);
    assert_eq!(
        compare(&capture, &baseline, &empty).unwrap_err(),
        SnapshotInventoryDeltaError::LostBaseline
    );
}

#[test]
fn rejects_changed_timestamp_or_size_for_an_exact_retained_id() {
    let capture = request(IcManagementMethodRecord::TakeCanisterSnapshot, TARGET);
    let list = request(IcManagementMethodRecord::ListCanisterSnapshots, TARGET);
    let baseline = inventory(&list, vec![snapshot(&[0, 255])]);
    for change_timestamp in [true, false] {
        let mut old = snapshot(&[0, 255]);
        if change_timestamp {
            old.taken_at_timestamp += 1;
        } else {
            old.total_size += 1;
        }
        let observed = inventory(&list, vec![old, snapshot(&[1])]);
        assert_eq!(
            compare(&capture, &baseline, &observed).unwrap_err(),
            SnapshotInventoryDeltaError::ChangedBaselineMetadata
        );
    }
}

#[test]
fn rejects_wrong_capture_inventory_methods_and_either_target() {
    let capture = request(IcManagementMethodRecord::TakeCanisterSnapshot, TARGET);
    let list = request(IcManagementMethodRecord::ListCanisterSnapshots, TARGET);
    let other_list = request(IcManagementMethodRecord::ListCanisterSnapshots, "aaaaa-aa");
    let baseline = inventory(&list, vec![]);
    let other = inventory(&other_list, vec![]);
    for wrong in [
        IcManagementMethodRecord::CanisterStatus,
        IcManagementMethodRecord::ListCanisterSnapshots,
        IcManagementMethodRecord::LoadCanisterSnapshot,
        IcManagementMethodRecord::StartCanister,
        IcManagementMethodRecord::StopCanister,
    ] {
        let wrong = request(wrong, TARGET);
        assert_eq!(
            compare(&wrong, &baseline, &baseline).unwrap_err(),
            SnapshotInventoryDeltaError::WrongCaptureMethod
        );
    }
    let captured = IcSnapshotReply::decode(
        &capture,
        &candid::encode_one(snapshot(&[1])).expect("fixture wire"),
    )
    .expect("capture reply");
    for (before, after) in [(&captured, &baseline), (&baseline, &captured)] {
        assert_eq!(
            compare(&capture, before, after).unwrap_err(),
            SnapshotInventoryDeltaError::WrongInventoryMethod
        );
    }
    for (before, after) in [(&other, &baseline), (&baseline, &other)] {
        assert_eq!(
            compare(&capture, before, after).unwrap_err(),
            SnapshotInventoryDeltaError::TargetMismatch
        );
    }
}

#[test]
fn admits_full_inventory_and_interleaved_baseline_without_extra_capacity() {
    let capture = request(IcManagementMethodRecord::TakeCanisterSnapshot, TARGET);
    let list = request(IcManagementMethodRecord::ListCanisterSnapshots, TARGET);
    let snapshots = (0..MAX_IC_SNAPSHOT_REPLY_ENTRIES)
        .map(|id| {
            let mut bytes = vec![0; crate::model::ic_request::MAX_IC_SNAPSHOT_ID_BYTES];
            bytes[..std::mem::size_of::<usize>()].copy_from_slice(&id.to_be_bytes());
            snapshot(&bytes)
        })
        .collect::<Vec<_>>();
    let baseline = inventory(&list, snapshots.iter().step_by(2).cloned().collect());
    let observed = inventory(&list, snapshots.clone());
    let delta = compare(&capture, &baseline, &observed).expect("bounded interleaved delta");
    assert_eq!(delta.candidates().len(), MAX_IC_SNAPSHOT_REPLY_ENTRIES / 2);
    assert_eq!(delta.candidates()[0].id(), snapshots[1].id);
    let empty = inventory(&list, vec![]);
    let all_new = compare(&capture, &empty, &observed).expect("maximum candidates");
    assert_eq!(all_new.candidates().len(), MAX_IC_SNAPSHOT_REPLY_ENTRIES);
    assert!(
        all_new
            .candidates()
            .iter()
            .all(|entry| entry.id().len() == crate::model::ic_request::MAX_IC_SNAPSHOT_ID_BYTES)
    );
}
