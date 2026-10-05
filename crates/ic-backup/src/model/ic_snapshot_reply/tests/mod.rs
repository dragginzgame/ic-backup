//! Native wire admission, independent goldens and finite-bound regressions.

use super::*;
use crate::model::ic_request::{IcManagementRequest, MAX_IC_SNAPSHOT_ID_BYTES};
use candid::CandidType;
use ic_management_canister_types::Snapshot;
use serde::Deserialize;

#[derive(Deserialize)]
struct GoldenCase {
    name: String,
    method: IcManagementMethodRecord,
    target: String,
    request_digest: String,
    reply_hex: String,
    payload_checksum: String,
    digest: String,
    snapshots: Vec<Snapshot>,
}

fn request(method: IcManagementMethodRecord, target: &str) -> IcManagementRequestRecord {
    IcManagementRequestRecord::new(IcManagementRequest {
        method,
        target: target.into(),
        snapshot_id: (method == IcManagementMethodRecord::LoadCanisterSnapshot).then(|| vec![1]),
    })
    .expect("existing canonical wire request")
}

fn inventory_request() -> IcManagementRequestRecord {
    request(
        IcManagementMethodRecord::ListCanisterSnapshots,
        "renrk-eyaaa-aaaaa-aaada-cai",
    )
}

fn bytes(hex: &str) -> Vec<u8> {
    let (pairs, remainder) = hex.as_bytes().as_chunks::<2>();
    assert!(remainder.is_empty(), "complete golden bytes");
    pairs
        .iter()
        .map(|pair| {
            u8::from_str_radix(std::str::from_utf8(pair).expect("ASCII"), 16).expect("hex byte")
        })
        .collect()
}

fn snapshot(id: Vec<u8>) -> Snapshot {
    Snapshot {
        id,
        taken_at_timestamp: 0,
        total_size: u64::MAX,
    }
}

#[test]
fn admits_every_registered_independent_wire_and_evidence_digest() {
    let cases: Vec<GoldenCase> =
        serde_json::from_str(include_str!("golden.json")).expect("registered cases");
    for case in cases {
        let request = request(case.method, &case.target);
        let raw = bytes(&case.reply_hex);
        assert_eq!(
            request.digest().hash(),
            case.request_digest,
            "{}",
            case.name
        );
        let decoded = IcSnapshotReply::decode(&request, &raw).expect("golden reply");
        let official = if case.method == IcManagementMethodRecord::TakeCanisterSnapshot {
            vec![candid::decode_one::<Snapshot>(&raw).expect("official capture shape")]
        } else {
            candid::decode_one::<Vec<Snapshot>>(&raw).expect("official inventory shape")
        };
        assert_eq!(official, case.snapshots);
        assert_eq!(decoded.payload_checksum().hash(), case.payload_checksum);
        assert_eq!(decoded.digest().hash(), case.digest);
        let mut expected = case.snapshots;
        expected.sort_by(|left, right| left.id.cmp(&right.id));
        for (actual, expected) in decoded.snapshots().iter().zip(&expected) {
            assert_eq!(actual.id(), expected.id);
            assert_eq!(actual.taken_at_timestamp(), expected.taken_at_timestamp);
            assert_eq!(actual.total_size(), expected.total_size);
        }
        assert_eq!(decoded.snapshots().len(), expected.len());
        assert_eq!(decoded.request(), &request);
    }
}

#[test]
fn canonical_projection_preserves_raw_reply_and_request_identity() {
    let wire_request = inventory_request();
    let mut snapshots = vec![snapshot(vec![255]), snapshot(vec![0]), snapshot(vec![0, 0])];
    let first_bytes = candid::encode_one(&snapshots).expect("wire");
    snapshots.reverse();
    let second_bytes = candid::encode_one(&snapshots).expect("reordered wire");
    let first = IcSnapshotReply::decode(&wire_request, &first_bytes).expect("inventory");
    let second = IcSnapshotReply::decode(&wire_request, &second_bytes).expect("inventory");
    assert_eq!(first.snapshots(), second.snapshots());
    assert_eq!(first.snapshots()[0].id(), [0]);
    assert_eq!(first.snapshots()[1].id(), [0, 0]);
    assert_ne!(first.payload_checksum(), second.payload_checksum());
    assert_ne!(first.digest(), second.digest());
    let other = request(IcManagementMethodRecord::ListCanisterSnapshots, "aaaaa-aa");
    let other = IcSnapshotReply::decode(&other, &first_bytes).expect("declared other association");
    assert_ne!(first.digest(), other.digest());
    assert_eq!(first.payload_checksum(), other.payload_checksum());
}

#[test]
fn accepts_full_nat64_values_and_exact_raw_identifier_bound() {
    let request = request(IcManagementMethodRecord::TakeCanisterSnapshot, "2vxsx-fae");
    let expected = Snapshot {
        id: vec![255; MAX_IC_SNAPSHOT_ID_BYTES],
        taken_at_timestamp: u64::MAX,
        total_size: u64::MAX,
    };
    let raw = candid::encode_one(&expected).expect("official wire");
    let reply = IcSnapshotReply::decode(&request, &raw).expect("bounded exact descriptor");
    assert_eq!(reply.snapshots().len(), 1);
    assert_eq!(reply.snapshots()[0].id(), expected.id);
    assert_eq!(reply.snapshots()[0].taken_at_timestamp(), u64::MAX);
    assert_eq!(reply.snapshots()[0].total_size(), u64::MAX);
    for id in [vec![], vec![0; MAX_IC_SNAPSHOT_ID_BYTES + 1]] {
        let raw = candid::encode_one(snapshot(id)).expect("invalid-bound wire");
        assert_eq!(
            IcSnapshotReply::decode(&request, &raw).unwrap_err(),
            IcSnapshotReplyError::InvalidReply
        );
    }
}

#[test]
fn accepts_maximum_combined_inventory_bounds_and_rejects_one_more() {
    let request = inventory_request();
    let mut snapshots: Vec<_> = (0..MAX_IC_SNAPSHOT_REPLY_ENTRIES)
        .map(|index| {
            let mut id = vec![0; MAX_IC_SNAPSHOT_ID_BYTES];
            id[..2].copy_from_slice(&index.to_le_bytes()[..2]);
            snapshot(id)
        })
        .collect();
    let raw = candid::encode_one(&snapshots).expect("maximum bounded reply");
    let reply =
        IcSnapshotReply::decode(&request, &raw).expect("all limits together fit decoder quota");
    assert_eq!(reply.snapshots().len(), MAX_IC_SNAPSHOT_REPLY_ENTRIES);
    snapshots.push(snapshot(vec![255]));
    let raw = candid::encode_one(snapshots).expect("too many entries");
    assert_eq!(
        IcSnapshotReply::decode(&request, &raw).unwrap_err(),
        IcSnapshotReplyError::InvalidReply
    );
}

#[test]
fn rejects_duplicate_ids_even_with_changed_declared_metadata() {
    let request = inventory_request();
    let mut duplicate = snapshot(vec![0, 0]);
    duplicate.total_size = 0;
    let raw =
        candid::encode_one(vec![snapshot(vec![0, 0]), duplicate]).expect("duplicate inventory");
    assert_eq!(
        IcSnapshotReply::decode(&request, &raw).unwrap_err(),
        IcSnapshotReplyError::DuplicateSnapshotId
    );
}

#[test]
fn rejects_every_other_method_before_parsing() {
    for method in [
        IcManagementMethodRecord::CanisterStatus,
        IcManagementMethodRecord::LoadCanisterSnapshot,
        IcManagementMethodRecord::StopCanister,
        IcManagementMethodRecord::StartCanister,
    ] {
        let request = request(method, "2vxsx-fae");
        assert_eq!(
            IcSnapshotReply::decode(&request, b"invalid").unwrap_err(),
            IcSnapshotReplyError::UnsupportedMethod { method }
        );
    }
}

#[test]
fn rejects_raw_size_overflow_and_malformed_input_at_the_raw_bound() {
    let request = inventory_request();
    assert_eq!(
        IcSnapshotReply::decode(&request, &vec![0; MAX_IC_SNAPSHOT_REPLY_BYTES + 1]).unwrap_err(),
        IcSnapshotReplyError::ReplyTooLarge
    );
    assert_eq!(
        IcSnapshotReply::decode(&request, &vec![0; MAX_IC_SNAPSHOT_REPLY_BYTES]).unwrap_err(),
        IcSnapshotReplyError::InvalidReply
    );
}

#[test]
fn rejects_truncation_extra_arguments_wrong_shape_and_trailing_bytes() {
    let capture = request(IcManagementMethodRecord::TakeCanisterSnapshot, "2vxsx-fae");
    let inventory = inventory_request();
    let expected = snapshot(vec![0, 255, 17]);
    let raw = candid::encode_one(&expected).expect("capture wire");
    for length in 0..raw.len() {
        assert_eq!(
            IcSnapshotReply::decode(&capture, &raw[..length]).unwrap_err(),
            IcSnapshotReplyError::InvalidReply
        );
    }
    let extra = candid::encode_args((&expected, 0_u64)).expect("extra argument");
    let mut trailing = raw.clone();
    trailing.push(0);
    let wrong_shape = candid::encode_one(vec![expected]).expect("inventory wire");
    for invalid in [extra, trailing, wrong_shape] {
        assert_eq!(
            IcSnapshotReply::decode(&capture, &invalid).unwrap_err(),
            IcSnapshotReplyError::InvalidReply
        );
    }
    assert_eq!(
        IcSnapshotReply::decode(&inventory, &raw).unwrap_err(),
        IcSnapshotReplyError::InvalidReply
    );
    let no_args = candid::encode_args(()).expect("empty tuple");
    assert_eq!(
        IcSnapshotReply::decode(&capture, &no_args).unwrap_err(),
        IcSnapshotReplyError::InvalidReply
    );
}

#[test]
fn rejects_unknown_fields_missing_metadata_and_wrong_numeric_types() {
    #[derive(CandidType)]
    struct Extended {
        id: Vec<u8>,
        taken_at_timestamp: u64,
        total_size: u64,
        accepted: bool,
    }
    #[derive(CandidType)]
    struct Incomplete {
        id: Vec<u8>,
        taken_at_timestamp: u64,
    }
    #[derive(CandidType)]
    struct WrongNumber {
        id: Vec<u8>,
        taken_at_timestamp: i64,
        total_size: u64,
    }
    let request = request(IcManagementMethodRecord::TakeCanisterSnapshot, "2vxsx-fae");
    for raw in [
        candid::encode_one(Extended {
            id: vec![1],
            taken_at_timestamp: 0,
            total_size: 0,
            accepted: true,
        })
        .expect("extended"),
        candid::encode_one(Incomplete {
            id: vec![1],
            taken_at_timestamp: 0,
        })
        .expect("missing metadata"),
        candid::encode_one(WrongNumber {
            id: vec![1],
            taken_at_timestamp: -1,
            total_size: 0,
        })
        .expect("wrong type"),
    ] {
        assert_eq!(
            IcSnapshotReply::decode(&request, &raw).unwrap_err(),
            IcSnapshotReplyError::InvalidReply
        );
    }
}

#[test]
fn rejects_excess_type_tables_and_huge_declared_sequences() {
    let request = request(IcManagementMethodRecord::TakeCanisterSnapshot, "2vxsx-fae");
    let raw = candid::encode_one(snapshot(vec![1])).expect("official capture wire");
    // The fixture body has a one-byte vector length, one ID byte and two nat64
    // values. Two one-byte argument declarations precede that body. Add unused
    // vec nat8 definitions without changing the original argument or value.
    let definitions_end = raw.len() - (1 + 1 + 8 + 8) - 2;
    assert_eq!(raw[4], 2);
    let mut excess = raw[..definitions_end].to_vec();
    excess[4] = 17;
    for _ in 0..15 {
        excess.extend_from_slice(&[0x6d, 0x7b]);
    }
    excess.extend_from_slice(&raw[definitions_end..]);
    assert_eq!(
        candid::decode_one::<Snapshot>(&excess).expect("valid unused types"),
        snapshot(vec![1])
    );
    assert_eq!(
        IcSnapshotReply::decode(&request, &excess).unwrap_err(),
        IcSnapshotReplyError::InvalidReply
    );
    let mut huge = raw[..definitions_end + 2].to_vec();
    huge.extend_from_slice(&[0xff, 0xff, 0xff, 0xff, 0x0f]);
    assert_eq!(
        IcSnapshotReply::decode(&request, &huge).unwrap_err(),
        IcSnapshotReplyError::InvalidReply
    );
}
