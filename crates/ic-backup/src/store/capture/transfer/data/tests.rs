//! Re-authored legacy data-admission cases against the extracted implementation.

use super::{
    super::tests::{chunk, metadata, plan},
    *,
};
use ic_platform_protocol::management_snapshots::{SnapshotData, fixtures};

fn reply(bytes: &[u8]) -> Vec<u8> {
    fixtures::snapshot_data(
        &SnapshotData {
            chunk: bytes.to_vec(),
        },
        MAX_REPLY_BYTES,
    )
    .unwrap()
}

fn hex(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

#[test]
fn frozen_wire_vectors_accept_different_type_table_order() {
    let p = plan(3);
    let m = metadata(3, 0, 0);
    let transfer = p.plan_transfer(&[1], &m).unwrap();

    // Independent wire fixtures retained from the reviewed legacy regressions.
    for raw in [
        "4449444c026c01ed8c8bae04016d7b01000300ff11",
        "4449444c026d7b6c01ed8c8bae040001010300ff11",
    ] {
        assert_eq!(transfer.decode_data(0, &hex(raw)).unwrap(), [0, 255, 17]);
    }
}

#[test]
fn each_region_and_final_partial_extent_require_exact_length() {
    let p = plan(4);
    let m = metadata(5, 6, 7);
    let transfer = p.plan_transfer(&[1], &m).unwrap();
    for (ordinal, expected) in [4, 1, 4, 2, 4, 3].into_iter().enumerate() {
        let ordinal = u32::try_from(ordinal).unwrap();
        assert_eq!(
            transfer
                .decode_data(ordinal, &reply(&vec![42; expected]))
                .unwrap(),
            vec![42; expected]
        );

        for actual in [0, expected - 1, expected + 1] {
            assert_eq!(
                transfer.decode_data(ordinal, &reply(&vec![42; actual])),
                Err(TransferDataError::Length {
                    expected: expected as u64,
                    actual
                })
            );
        }
    }

    // Invalid ordinals refuse before even parsing a reply.
    for ordinal in [6, u32::MAX] {
        assert_eq!(
            transfer.decode_data(ordinal, &[]),
            Err(TransferDataError::Ordinal)
        );
    }
}

#[test]
fn whole_chunks_require_original_digest_including_empty_chunk() {
    let p = plan(4);

    for bytes in [b"".as_slice(), b"abcd"] {
        let mut m = metadata(1, 0, 0);
        m.wasm_chunk_store = vec![chunk(bytes)];
        let transfer = p.plan_transfer(&[1], &m).unwrap();
        assert_eq!(transfer.decode_data(1, &reply(bytes)).unwrap(), bytes);
        assert_eq!(
            transfer.decode_data(1, &reply(b"bad")),
            Err(TransferDataError::ChunkHash)
        );
    }
}

#[test]
fn matching_chunk_hash_does_not_bypass_original_extent_limit() {
    let p = plan(4);
    let mut m = metadata(1, 0, 0);
    m.wasm_chunk_store = vec![chunk(b"12345")];
    let transfer = p.plan_transfer(&[1], &m).unwrap();
    assert_eq!(
        transfer.decode_data(1, &reply(b"12345")),
        Err(TransferDataError::ChunkLimit {
            limit: 4,
            actual: 5
        })
    );
}

#[test]
fn maximum_region_and_chunk_are_admitted() {
    let bytes = vec![0xa5; MAX_DATA_BYTES];
    let p = plan(u32::try_from(MAX_DATA_BYTES).unwrap());
    let mut m = metadata(MAX_DATA_BYTES as u64, 0, 0);
    m.wasm_chunk_store = vec![chunk(&bytes)];
    let transfer = p.plan_transfer(&[1], &m).unwrap();
    let raw = reply(&bytes);

    for ordinal in [0, 1] {
        assert_eq!(transfer.decode_data(ordinal, &raw).unwrap(), bytes);
    }
}

#[test]
fn raw_type_and_declared_vector_bounds_refuse_before_payload_allocation() {
    let p = plan(4);
    let m = metadata(1, 0, 0);
    let transfer = p.plan_transfer(&[1], &m).unwrap();
    assert_eq!(
        transfer.decode_data(0, &vec![0; MAX_REPLY_BYTES + 1]),
        Err(TransferDataError::ReplyTooLarge)
    );
    // Valid header, declared 1 MiB + 1 vector, no body: element cap wins over truncation.
    let huge_vector = hex("4449444c026c01ed8c8bae04016d7b0100818040");
    assert_eq!(
        transfer.decode_data(0, &huge_vector),
        Err(TransferDataError::Decode(DecodeError::Limit))
    );
    // Declared 17 types is rejected before reading any entries.
    assert_eq!(
        transfer.decode_data(0, b"DIDL\x11"),
        Err(TransferDataError::Decode(DecodeError::Limit))
    );
}

#[test]
fn truncation_trailing_bytes_and_wrong_arity_refuse() {
    let p = plan(3);
    let m = metadata(3, 0, 0);
    let transfer = p.plan_transfer(&[1], &m).unwrap();
    let raw = hex("4449444c026c01ed8c8bae04016d7b01000300ff11");
    for end in 0..raw.len() {
        assert!(
            matches!(
                transfer.decode_data(0, &raw[..end]),
                Err(TransferDataError::Decode(_))
            ),
            "prefix {end}"
        );
    }
    let mut trailing = raw;
    trailing.push(0);
    assert_eq!(
        transfer.decode_data(0, &trailing),
        Err(TransferDataError::Decode(DecodeError::Trailing))
    );

    for wire in ["4449444c0000", "4449444c00027f7f"] {
        assert_eq!(
            transfer.decode_data(0, &hex(wire)),
            Err(TransferDataError::Decode(DecodeError::Arity))
        );
    }
}

#[test]
fn missing_wrong_and_unknown_fields_refuse_including_zero_byte_skip() {
    let p = plan(1);
    let m = metadata(1, 0, 0);
    let transfer = p.plan_transfer(&[1], &m).unwrap();
    // Empty record and chunk:nat8 instead of chunk:vec nat8.
    for wire in ["4449444c016c000100", "4449444c016c01ed8c8bae047b010000"] {
        assert_eq!(
            transfer.decode_data(0, &hex(wire)),
            Err(TransferDataError::Decode(DecodeError::Type))
        );
    }
    // Extra field 0:null has no value bytes but still spends skipped work.
    let extra = hex("4449444c026c02007fed8c8bae04016d7b01000142");
    assert_eq!(
        transfer.decode_data(0, &extra),
        Err(TransferDataError::Decode(DecodeError::Limit))
    );
}

#[test]
fn decoding_is_effect_free_and_equal_length_region_bytes_are_not_authenticated() {
    let p = plan(3);
    let m = metadata(3, 0, 0);
    let transfer = p.plan_transfer(&[1], &m).unwrap();
    let before = format!("{:?}", transfer.requests().collect::<Vec<_>>());
    assert!(transfer.decode_data(0, &reply(b"x")).is_err());
    assert_eq!(transfer.decode_data(0, &reply(b"abc")).unwrap(), b"abc");
    assert_eq!(transfer.decode_data(0, &reply(b"xyz")).unwrap(), b"xyz");
    assert_eq!(
        format!("{:?}", transfer.requests().collect::<Vec<_>>()),
        before
    );
    assert_eq!(transfer.data_calls(), 1);
    assert_eq!(transfer.capture_calls, 6);
}
