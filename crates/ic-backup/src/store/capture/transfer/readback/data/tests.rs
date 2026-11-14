//! Re-authored legacy length/hash/identity cases and complete local byte coverage.
use super::super::super::tests::{chunk, metadata, plan};
use super::*;
use ic_platform::{
    candid_codec::{self, Reserved},
    management_canister_types::{ReadCanisterSnapshotDataResult, SnapshotDataKind, SnapshotSource},
};

fn metadata_reply(source: &ReadCanisterSnapshotMetadataResult) -> Vec<u8> {
    let mut destination = source.clone();
    destination.source = Some(SnapshotSource::MetadataUpload(Reserved));
    destination.taken_at_timestamp = 0;
    destination.canister_version = u64::MAX;
    destination.wasm_chunk_store.reverse();
    candid_codec::encode_one(&destination, 1024 * 1024).unwrap()
}

fn data_reply(bytes: &[u8]) -> Vec<u8> {
    candid_codec::encode_one(
        &ReadCanisterSnapshotDataResult {
            chunk: bytes.to_vec(),
        },
        2 * 1024 * 1024,
    )
    .unwrap()
}

#[test]
fn all_regions_and_chunks_compare_in_source_order_with_exact_destination_identity() {
    let p = plan(4);
    let mut m = metadata(5, 4, 3);
    m.wasm_chunk_store = vec![chunk(b"abc"), chunk(b"")];
    let source_before = candid_codec::encode_one(&m, 4096).unwrap();
    let transfer = p.plan_transfer(&[0, 255, 1], &m).unwrap();
    let destination = [255; 256];
    let mut readback = transfer
        .begin_readback(&destination, &metadata_reply(&m))
        .unwrap();
    let source_requests: Vec<_> = transfer.requests().collect();
    assert_eq!(source_requests.len(), 6);
    for (index, source_request) in source_requests.iter().enumerate() {
        let (ordinal, request) = readback.next_request().unwrap();
        assert_eq!(ordinal as usize, index);
        assert_eq!(request.canister_id, p.request().target.canister);
        assert_eq!(request.snapshot_id, destination);
        assert_eq!(source_request.snapshot_id, [0, 255, 1]);
        assert_eq!(
            candid_codec::encode_one(&request.kind, 4096).unwrap(),
            candid_codec::encode_one(&source_request.kind, 4096).unwrap(),
        );
        assert_eq!(
            candid_codec::encode_one(&readback.next_request().unwrap().1, 4096).unwrap(),
            candid_codec::encode_one(&request, 4096).unwrap(),
        );
        let expected = match &request.kind {
            SnapshotDataKind::WasmModule { offset, size } => {
                &b"modul"[*offset as usize..(offset + size) as usize]
            }
            SnapshotDataKind::WasmMemory { offset, size } => {
                &b"heap"[*offset as usize..(offset + size) as usize]
            }
            SnapshotDataKind::StableMemory { offset, size } => {
                &b"mem"[*offset as usize..(offset + size) as usize]
            }
            SnapshotDataKind::WasmChunk { hash } if *hash == chunk(b"abc").hash => {
                b"abc".as_slice()
            }
            SnapshotDataKind::WasmChunk { hash } => {
                assert_eq!(*hash, chunk(b"").hash);
                b"".as_slice()
            }
        };
        readback
            .compare_data(ordinal, expected, &data_reply(expected))
            .unwrap();
        assert_eq!(readback.compared_data_calls(), ordinal + 1);
    }
    assert!(readback.next_request().is_none());
    assert_eq!(
        readback.compare_data(transfer.data_calls(), b"", b""),
        Err(ReadbackDataError::Ordinal)
    );
    let result = readback.finish().unwrap();
    assert_eq!(result.taken_at_timestamp, 0);
    assert_eq!(result.canister_version, u64::MAX);
    assert_eq!(result.wasm_chunk_store[0].hash, m.wasm_chunk_store[1].hash);
    assert_eq!(candid_codec::encode_one(&m, 4096).unwrap(), source_before);
    for (after, before) in transfer.requests().zip(source_requests) {
        assert_eq!(
            candid_codec::encode_one(&after, 4096).unwrap(),
            candid_codec::encode_one(&before, 4096).unwrap(),
        );
    }
}

#[test]
fn every_partial_prefix_refuses_completion_including_a_named_empty_chunk() {
    let p = plan(4);
    let mut m = metadata(1, 0, 0);
    m.wasm_chunk_store = vec![chunk(b"")];
    let transfer = p.plan_transfer(&[1], &m).unwrap();
    assert_eq!(transfer.data_calls(), 2); // No reads for empty heap/stable regions.
    for count in 0..=2 {
        let mut readback = transfer.begin_readback(&[2], &metadata_reply(&m)).unwrap();
        if count >= 1 {
            readback.compare_data(0, b"m", &data_reply(b"m")).unwrap();
        }
        if count == 2 {
            readback.compare_data(1, b"", &data_reply(b"")).unwrap();
        }
        assert_eq!(readback.compared_data_calls(), count);
        match readback.finish() {
            Ok(_) => assert_eq!(count, 2),
            Err(error) => {
                assert!(count < 2);
                assert_eq!(error, ReadbackDataError::Incomplete);
            }
        }
    }
}

#[test]
fn skipped_repeated_and_out_of_range_positions_never_advance_coverage() {
    let p = plan(1);
    let m = metadata(2, 0, 0);
    let transfer = p.plan_transfer(&[1], &m).unwrap();
    let mut readback = transfer.begin_readback(&[2], &metadata_reply(&m)).unwrap();
    for ordinal in [1, 2, u32::MAX] {
        assert_eq!(
            readback.compare_data(ordinal, b"", b""),
            Err(ReadbackDataError::Ordinal)
        );
        assert_eq!(readback.compared_data_calls(), 0);
    }
    readback.compare_data(0, b"a", &data_reply(b"a")).unwrap();
    assert_eq!(
        readback.compare_data(0, b"a", &data_reply(b"a")),
        Err(ReadbackDataError::Ordinal)
    );
    assert_eq!(readback.next_request().unwrap().0, 1);
    readback.compare_data(1, b"b", &data_reply(b"b")).unwrap();
    assert!(readback.finish().is_ok());
}

#[test]
fn same_length_corruption_in_each_region_refuses_without_advancing() {
    let p = plan(4);
    let m = metadata(4, 4, 4);
    let transfer = p.plan_transfer(&[1], &m).unwrap();
    let mut readback = transfer.begin_readback(&[2], &metadata_reply(&m)).unwrap();
    for ordinal in 0..3 {
        for position in 0..4 {
            let mut corrupt = *b"data";
            corrupt[position] ^= 1;
            assert_eq!(
                readback.compare_data(ordinal, b"data", &data_reply(&corrupt)),
                Err(ReadbackDataError::Mismatch)
            );
            assert_eq!(readback.compared_data_calls(), ordinal);
        }
        // Corrected local evidence can be compared; no network call is performed.
        readback
            .compare_data(ordinal, b"data", &data_reply(b"data"))
            .unwrap();
    }
    assert!(readback.finish().is_ok());
}

#[test]
fn source_shape_precedes_wire_decoding_and_destination_length_is_independent() {
    let p = plan(4);
    let m = metadata(4, 0, 0);
    let transfer = p.plan_transfer(&[1], &m).unwrap();
    let mut readback = transfer.begin_readback(&[2], &metadata_reply(&m)).unwrap();
    for bytes in [b"".as_slice(), b"abc", b"abcde"] {
        let error = TransferDataError::Length {
            expected: 4,
            actual: bytes.len(),
        };
        assert_eq!(
            readback.compare_data(0, bytes, b"invalid"),
            Err(ReadbackDataError::Source(error))
        );
        assert_eq!(
            readback.compare_data(0, b"data", &data_reply(bytes)),
            Err(ReadbackDataError::Reply(TransferDataError::Length {
                expected: 4,
                actual: bytes.len()
            }))
        );
        assert_eq!(readback.compared_data_calls(), 0);
    }
    assert_eq!(
        readback.finish().unwrap_err(),
        ReadbackDataError::Incomplete
    );
}

#[test]
fn whole_chunk_source_and_destination_hashes_and_caps_are_checked_separately() {
    let p = plan(4);
    let mut m = metadata(1, 0, 0);
    m.wasm_chunk_store = vec![chunk(b"data")];
    let transfer = p.plan_transfer(&[1], &m).unwrap();
    let mut readback = transfer.begin_readback(&[2], &metadata_reply(&m)).unwrap();
    readback.compare_data(0, b"m", &data_reply(b"m")).unwrap();
    assert_eq!(
        readback.compare_data(1, b"nope", b"invalid"),
        Err(ReadbackDataError::Source(TransferDataError::ChunkHash))
    );
    assert_eq!(
        readback.compare_data(1, b"data", &data_reply(b"nope")),
        Err(ReadbackDataError::Reply(TransferDataError::ChunkHash))
    );
    assert_eq!(
        readback.compare_data(1, b"large", b"invalid"),
        Err(ReadbackDataError::Source(TransferDataError::ChunkLimit {
            limit: 4,
            actual: 5
        }))
    );
    assert_eq!(
        readback.compare_data(1, b"data", &data_reply(b"large")),
        Err(ReadbackDataError::Reply(TransferDataError::ChunkLimit {
            limit: 4,
            actual: 5
        }))
    );
    assert_eq!(readback.compared_data_calls(), 1);
    readback
        .compare_data(1, b"data", &data_reply(b"data"))
        .unwrap();
    assert!(readback.finish().is_ok());
}

#[test]
fn exact_mebibyte_regions_and_chunks_use_the_existing_bounded_decoder() {
    let bytes = vec![127; 1024 * 1024];
    let reply = data_reply(&bytes);
    let p = plan(1024 * 1024);
    let mut m = metadata(bytes.len() as u64, 0, 0);
    m.wasm_chunk_store = vec![chunk(&bytes)];
    let transfer = p.plan_transfer(&[1], &m).unwrap();
    let mut readback = transfer.begin_readback(&[2], &metadata_reply(&m)).unwrap();
    for ordinal in 0..2 {
        assert_eq!(
            readback.compare_data(ordinal, &bytes, &vec![0; 2 * 1024 * 1024 + 1]),
            Err(ReadbackDataError::Reply(TransferDataError::ReplyTooLarge))
        );
        assert!(matches!(
            readback.compare_data(ordinal, &bytes, b"DIDL"),
            Err(ReadbackDataError::Reply(TransferDataError::Decode(_)))
        ));
        assert_eq!(readback.compared_data_calls(), ordinal);
        readback.compare_data(ordinal, &bytes, &reply).unwrap();
    }
    assert!(readback.finish().is_ok());
}

#[test]
fn destination_identity_and_metadata_are_admitted_before_data_coverage_exists() {
    let p = plan(1);
    let m = metadata(1, 0, 0);
    let transfer = p.plan_transfer(&[1], &m).unwrap();
    for id in [b"".as_slice(), &[1], &[2; 257]] {
        assert_eq!(
            transfer.begin_readback(id, b"invalid").unwrap_err(),
            ReadbackDataError::Destination
        );
    }
    let taken = candid_codec::encode_one(&m, 4096).unwrap();
    assert_eq!(
        transfer.begin_readback(&[2], &taken).unwrap_err(),
        ReadbackDataError::Metadata(ReadbackMetadataError::Origin)
    );
    let changed = metadata(2, 0, 0);
    assert_eq!(
        transfer
            .begin_readback(&[2], &metadata_reply(&changed))
            .unwrap_err(),
        ReadbackDataError::Metadata(ReadbackMetadataError::RegionSizes)
    );
    let readback = transfer.begin_readback(&[2], &metadata_reply(&m)).unwrap();
    assert_eq!(readback.compared_data_calls(), 0);
    assert_eq!(
        readback.finish().unwrap_err(),
        ReadbackDataError::Incomplete
    );
}
