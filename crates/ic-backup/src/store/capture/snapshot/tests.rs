//! Re-authored frozen-legacy wire cases; goldens retain original bytes and hashes.

use super::{super::status::tests::plan, *};
use crate::store::{BackupDigest, capture::tests::wire::frozen};
use serde::Deserialize;

fn snapshot(id: Vec<u8>) -> Snapshot {
    Snapshot {
        id,
        taken_at_timestamp: 0,
        total_size: u64::MAX,
    }
}

fn encode(value: &impl crate::store::capture::tests::wire::EncodeFixture) -> Vec<u8> {
    crate::store::capture::tests::wire::encode(value, MAX_REPLY_BYTES)
}

fn hex(value: &str) -> Vec<u8> {
    let (pairs, rest) = value.as_bytes().as_chunks::<2>();
    assert!(rest.is_empty());

    pairs
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}
#[derive(Deserialize)]
struct Golden {
    name: String,
    method: String,
    payload_checksum: String,
    reply_hex: String,
    snapshots: Vec<Snapshot>,
}

fn goldens() -> Vec<Golden> {
    serde_json::from_str(include_str!("golden.json")).unwrap()
}

#[test]
fn every_legacy_golden_preserves_exact_snapshot_fields_and_wire_order() {
    for case in goldens() {
        let raw = hex(&case.reply_hex);
        assert_eq!(
            BackupDigest::compute(&raw).to_string(),
            case.payload_checksum,
            "{}",
            case.name
        );
        let values = match case.method.as_str() {
            "take_canister_snapshot" => vec![decode_taken_snapshot(&raw, &[]).unwrap()],
            "list_canister_snapshots" => decode_snapshot_list(&raw).unwrap(),
            other => panic!("unregistered method {other}"),
        };
        assert_eq!(values, case.snapshots, "{}", case.name);
    }
}

#[test]
fn exact_inventory_and_identifier_caps_preserve_full_numeric_ranges() {
    let max = Snapshot {
        id: vec![255; MAX_ID_BYTES],
        taken_at_timestamp: u64::MAX,
        total_size: u64::MAX,
    };
    assert_eq!(decode_taken_snapshot(&encode(&max), &[]).unwrap(), max);
    assert_eq!(
        decode_taken_snapshot(&encode(&snapshot(vec![])), &[]).unwrap_err(),
        SnapshotReplyError::SnapshotId
    );
    assert_eq!(
        decode_taken_snapshot(&encode(&snapshot(vec![0; MAX_ID_BYTES + 1])), &[]).unwrap_err(),
        SnapshotReplyError::Decode(DecodeError::Limit)
    );
    let mut values: Vec<_> = (0..MAX_SNAPSHOTS)
        .map(|index| {
            let mut id = vec![0; MAX_ID_BYTES];
            id[..8].copy_from_slice(&(index as u64).to_le_bytes());

            snapshot(id)
        })
        .collect();
    assert_eq!(decode_snapshot_list(&encode(&values)).unwrap(), values);
    values.push(snapshot(vec![255]));
    assert!(matches!(
        decode_snapshot_list(&encode(&values)),
        Err(SnapshotReplyError::Decode(DecodeError::Limit))
    ));
}

#[test]
fn repeated_ids_refuse_even_when_timestamp_and_size_differ() {
    let first = snapshot(vec![0, 0]);
    let second = Snapshot {
        id: first.id.clone(),
        taken_at_timestamp: u64::MAX,
        total_size: 0,
    };
    let values = vec![first, second];
    assert_eq!(
        decode_snapshot_list(&encode(&values)).unwrap_err(),
        SnapshotReplyError::DuplicateSnapshotId
    );
    assert_eq!(
        plan().take_snapshot_arguments(&values, 3).unwrap_err(),
        SnapshotReplyError::DuplicateSnapshotId
    );
    let values = vec![snapshot(vec![255]), snapshot(vec![0]), snapshot(vec![0, 0])];
    assert_eq!(decode_snapshot_list(&encode(&values)).unwrap(), values);
}

#[test]
fn refuses_partial_extra_wrong_shape_and_trailing_replies() {
    let value = snapshot(vec![0, 255, 17]);
    let raw = encode(&value);
    for length in 0..raw.len() {
        assert!(matches!(
            decode_taken_snapshot(&raw[..length], &[]),
            Err(SnapshotReplyError::Decode(_))
        ));
    }
    let mut trailing = raw.clone();
    trailing.push(0);
    for invalid in [
        trailing,
        encode(&vec![value]),
        frozen("snapshot-extra-argument"),
        b"DIDL\0\0".to_vec(),
    ] {
        assert!(matches!(
            decode_taken_snapshot(&invalid, &[]),
            Err(SnapshotReplyError::Decode(_))
        ));
    }
    assert!(matches!(
        decode_snapshot_list(&raw),
        Err(SnapshotReplyError::Decode(_))
    ));

    for length in [MAX_REPLY_BYTES, MAX_REPLY_BYTES + 1] {
        let error = decode_snapshot_list(&vec![0; length]).unwrap_err();

        if length == MAX_REPLY_BYTES {
            assert!(matches!(error, SnapshotReplyError::Decode(_)));
        } else {
            assert_eq!(error, SnapshotReplyError::ReplyTooLarge);
        }
    }
}

#[test]
fn hostile_declared_counts_refuse_before_missing_elements_are_read() {
    // The registered empty-inventory vector has no body beyond its length.
    let case = goldens()
        .into_iter()
        .find(|case| case.name == "empty-inventory")
        .unwrap();
    let mut raw = hex(&case.reply_hex);
    assert_eq!(raw.pop(), Some(0));
    raw.extend_from_slice(&[0xff, 0xff, 0xff, 0xff, 0x0f]);
    assert_eq!(
        decode_snapshot_list(&raw).unwrap_err(),
        SnapshotReplyError::Decode(DecodeError::Limit)
    );
    let raw = encode(&snapshot(vec![1]));
    let body = raw.len() - 18;
    assert_eq!(raw[body], 1);
    let mut huge_id = raw[..body].to_vec();
    huge_id.extend_from_slice(&[0xff, 0xff, 0xff, 0xff, 0x0f]);
    assert_eq!(
        decode_taken_snapshot(&huge_id, &[]).unwrap_err(),
        SnapshotReplyError::Decode(DecodeError::Limit)
    );
}

#[test]
fn unknown_missing_and_wrong_typed_fields_refuse() {
    for raw in [
        frozen("snapshot-extra-field"),
        frozen("snapshot-missing-size"),
        frozen("snapshot-signed-timestamp"),
    ] {
        assert!(matches!(
            decode_taken_snapshot(&raw, &[]),
            Err(SnapshotReplyError::Decode(_))
        ));
    }
}

#[test]
fn preflight_keeps_exact_target_code_and_prior_snapshots_without_effects() {
    let p = plan();
    let inventory = vec![snapshot(vec![0, 255])];
    let expected = TakeSnapshotArgs {
        canister_id: p.request().target.canister,
        replace_snapshot: None,
        uninstall_code: Some(false),
        sender_canister_version: None,
    };
    assert_eq!(p.take_snapshot_arguments(&[], 1).unwrap(), expected);
    assert_eq!(p.take_snapshot_arguments(&inventory, 2).unwrap(), expected);
    let encoded = encode(&expected);
    assert_eq!(encoded, frozen("take-snapshot-arguments"));
    assert_eq!(inventory, [snapshot(vec![0, 255])]);
    assert_eq!(
        p.take_snapshot_arguments(&inventory, 1).unwrap_err(),
        SnapshotReplyError::NoCapacity
    );

    for capacity in [0, MAX_SNAPSHOTS as u32 + 1, u32::MAX] {
        assert_eq!(
            p.take_snapshot_arguments(&[], capacity).unwrap_err(),
            SnapshotReplyError::CapacityLimit
        );
    }
}

#[test]
fn take_reply_never_infers_success_from_inventory_difference_or_reuses_an_old_id() {
    let prior = vec![snapshot(vec![1])];
    let mut reused = prior[0].clone();
    reused.taken_at_timestamp = 99;
    assert_eq!(
        decode_taken_snapshot(&encode(&reused), &prior).unwrap_err(),
        SnapshotReplyError::AlreadyExisted
    );
    let new = snapshot(vec![1, 0]);
    assert_eq!(decode_taken_snapshot(&encode(&new), &prior).unwrap(), new);
    assert!(matches!(
        decode_taken_snapshot(&encode(&vec![prior[0].clone(), new]), &prior),
        Err(SnapshotReplyError::Decode(_))
    ));
    for bad in [
        vec![snapshot(vec![])],
        vec![snapshot(vec![0; MAX_ID_BYTES + 1])],
    ] {
        assert_eq!(
            plan().take_snapshot_arguments(&bad, 2).unwrap_err(),
            SnapshotReplyError::SnapshotId
        );
        assert_eq!(
            decode_taken_snapshot(&encode(&reused), &bad).unwrap_err(),
            SnapshotReplyError::SnapshotId
        );
    }
    let huge = vec![snapshot(vec![1]); MAX_SNAPSHOTS + 1];
    assert_eq!(
        plan()
            .take_snapshot_arguments(&huge, MAX_SNAPSHOTS as u32)
            .unwrap_err(),
        SnapshotReplyError::InventoryLimit
    );
}
