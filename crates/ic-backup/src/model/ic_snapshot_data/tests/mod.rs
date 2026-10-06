//! Native wire/extent identity tests; no IC backend simulation or effects.

use super::*;
use crate::model::ic_snapshot_metadata::{IcSnapshotMetadataReply, IcSnapshotMetadataRequest};
use candid::CandidType;
use ic_management_canister_types::{
    ChunkHash, ReadCanisterSnapshotDataResult, ReadCanisterSnapshotMetadataResult,
};
use serde::Deserialize;
use sha2::Digest;

#[derive(Deserialize)]
struct GoldenCase {
    name: String,
    kind: SnapshotDataKind,
    arguments_hex: String,
    request_digest: String,
    reply_hex: String,
    payload_checksum: String,
    chunk: Vec<u8>,
    chunk_checksum: String,
    metadata_digest: String,
    digest: String,
}

fn hex_bytes(hex: &str) -> Vec<u8> {
    let (pairs, rest) = hex.as_bytes().as_chunks::<2>();
    assert_eq!(rest, b"");
    pairs
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

fn metadata_bytes() -> Vec<u8> {
    let cases: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../../ic_snapshot_metadata/tests/golden.json")).unwrap();
    let source = cases
        .iter()
        .find(|case| case["name"] == "data-source")
        .unwrap();
    hex_bytes(source["reply_hex"].as_str().unwrap())
}

fn metadata_request() -> IcSnapshotMetadataRequest {
    IcSnapshotMetadataRequest::new("renrk-eyaaa-aaaaa-aaada-cai", &[0, 255, 17]).unwrap()
}

#[test]
fn admits_every_registered_exact_argument_reply_and_metadata_evidence_digest() {
    let source = metadata_request();
    let metadata = IcSnapshotMetadataReply::decode(&source, &metadata_bytes()).unwrap();
    let cases: Vec<GoldenCase> = serde_json::from_str(include_str!("golden.json")).unwrap();
    for case in cases {
        let request = IcSnapshotDataRequest::new(&metadata, case.kind).unwrap();
        assert_eq!(
            request.arguments(),
            hex_bytes(&case.arguments_hex),
            "{}",
            case.name
        );
        let args: ReadCanisterSnapshotDataArgs = candid::decode_one(request.arguments()).unwrap();
        assert_eq!(args.canister_id.to_text(), source.target());
        assert_eq!(args.snapshot_id, source.snapshot_id());
        assert_eq!(
            candid::encode_one(args.kind).unwrap(),
            candid::encode_one(request.kind()).unwrap()
        );
        assert_eq!(request.receiver(), "aaaaa-aa");
        assert_eq!(request.method(), "read_canister_snapshot_data");
        assert_eq!(request.target(), source.target());
        assert_eq!(request.snapshot_id(), source.snapshot_id());
        assert_eq!(request.digest().hash(), case.request_digest);
        assert_eq!(request.metadata().digest().hash(), case.metadata_digest);
        let raw = hex_bytes(&case.reply_hex);
        let reply = IcSnapshotDataReply::decode(&request, &raw).unwrap();
        let official: ReadCanisterSnapshotDataResult = candid::decode_one(&raw).unwrap();
        assert_eq!(official.chunk, case.chunk);
        assert_eq!(reply.chunk(), case.chunk);
        assert_eq!(reply.chunk_checksum().hash(), case.chunk_checksum);
        assert_eq!(reply.payload_checksum().hash(), case.payload_checksum);
        assert_eq!(reply.digest().hash(), case.digest);
        assert_eq!(reply.request().arguments(), request.arguments());
    }
}

fn region(index: usize, offset: u64, size: u64) -> SnapshotDataKind {
    match index {
        0 => SnapshotDataKind::WasmModule { offset, size },
        1 => SnapshotDataKind::WasmMemory { offset, size },
        2 => SnapshotDataKind::StableMemory { offset, size },
        _ => unreachable!("three upstream bounded regions"),
    }
}

#[test]
fn checked_ranges_reject_zero_excess_overflow_and_wrong_metadata_region() {
    let source = metadata_request();
    let mut values: ReadCanisterSnapshotMetadataResult =
        candid::decode_one(&metadata_bytes()).unwrap();
    values.wasm_module_size = 0;
    values.wasm_memory_size = 64;
    values.stable_memory_size = u64::MAX;
    let metadata =
        IcSnapshotMetadataReply::decode(&source, &candid::encode_one(&values).unwrap()).unwrap();
    for index in 0..3 {
        for size in [0, 1024 * 1024 + 1, u64::MAX] {
            assert_eq!(
                IcSnapshotDataRequest::new(&metadata, region(index, 0, size)).unwrap_err(),
                IcSnapshotDataError::InvalidRangeSize
            );
        }
        for offset in [u64::MAX, u64::MAX - 1] {
            assert_eq!(
                IcSnapshotDataRequest::new(&metadata, region(index, offset, 2)).unwrap_err(),
                IcSnapshotDataError::RangeOutsideMetadata
            );
        }
    }
    assert_eq!(
        IcSnapshotDataRequest::new(&metadata, region(0, 0, 1)).unwrap_err(),
        IcSnapshotDataError::RangeOutsideMetadata
    );
    assert!(IcSnapshotDataRequest::new(&metadata, region(1, 63, 1)).is_ok());
    assert_eq!(
        IcSnapshotDataRequest::new(&metadata, region(1, 63, 2)).unwrap_err(),
        IcSnapshotDataError::RangeOutsideMetadata
    );
    assert!(IcSnapshotDataRequest::new(&metadata, region(2, u64::MAX - 1, 1)).is_ok());
    assert!(IcSnapshotDataRequest::new(&metadata, region(2, 0, 1024 * 1024)).is_ok());
}

#[test]
fn exact_lengths_and_actual_chunk_hashes_admit_no_missing_extra_or_substitute_bytes() {
    let source = metadata_request();
    let metadata = IcSnapshotMetadataReply::decode(&source, &metadata_bytes()).unwrap();
    for index in 0..3 {
        let request = IcSnapshotDataRequest::new(&metadata, region(index, 0, 3)).unwrap();
        for chunk in [vec![], vec![0; 2], vec![0; 4]] {
            let raw = candid::encode_one(ReadCanisterSnapshotDataResult { chunk }).unwrap();
            assert_eq!(
                IcSnapshotDataReply::decode(&request, &raw).unwrap_err(),
                IcSnapshotDataError::LengthMismatch
            );
        }
        let raw = candid::encode_one(ReadCanisterSnapshotDataResult {
            chunk: vec![0, 255, 17],
        })
        .unwrap();
        assert_eq!(
            IcSnapshotDataReply::decode(&request, &raw).unwrap().chunk(),
            [0, 255, 17]
        );
    }
    for hash in [vec![], vec![0; 31], vec![0; 33]] {
        assert_eq!(
            IcSnapshotDataRequest::new(&metadata, SnapshotDataKind::WasmChunk { hash })
                .unwrap_err(),
            IcSnapshotDataError::InvalidChunkHash
        );
    }
    assert_eq!(
        IcSnapshotDataRequest::new(&metadata, SnapshotDataKind::WasmChunk { hash: vec![0; 32] })
            .unwrap_err(),
        IcSnapshotDataError::ChunkNotInMetadata
    );
    for (index, chunk) in [vec![0, 255, 17], vec![]].into_iter().enumerate() {
        let request = IcSnapshotDataRequest::new(
            &metadata,
            SnapshotDataKind::WasmChunk {
                hash: metadata.metadata().wasm_chunk_store[index].hash.clone(),
            },
        )
        .unwrap();
        let raw = candid::encode_one(ReadCanisterSnapshotDataResult {
            chunk: chunk.clone(),
        })
        .unwrap();
        assert_eq!(
            IcSnapshotDataReply::decode(&request, &raw).unwrap().chunk(),
            chunk
        );
        let wrong = candid::encode_one(ReadCanisterSnapshotDataResult {
            chunk: vec![17, 255, 0],
        })
        .unwrap();
        assert_eq!(
            IcSnapshotDataReply::decode(&request, &wrong).unwrap_err(),
            IcSnapshotDataError::ChunkHashMismatch
        );
    }
}

#[test]
fn actual_chunk_and_raw_bounds_include_valid_maximum_ranges_and_stored_chunks() {
    let source = metadata_request();
    let chunk = vec![19; MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES];
    let checksum = sha2::Sha256::digest(&chunk);
    let mut values: ReadCanisterSnapshotMetadataResult =
        candid::decode_one(&metadata_bytes()).unwrap();
    values.wasm_module_size = 1024 * 1024;
    values.wasm_chunk_store = vec![ChunkHash {
        hash: checksum.to_vec(),
    }];
    let metadata =
        IcSnapshotMetadataReply::decode(&source, &candid::encode_one(values).unwrap()).unwrap();
    for kind in [
        region(0, 0, 1024 * 1024),
        SnapshotDataKind::WasmChunk {
            hash: checksum.to_vec(),
        },
    ] {
        let request = IcSnapshotDataRequest::new(&metadata, kind).unwrap();
        let raw = candid::encode_one(ReadCanisterSnapshotDataResult {
            chunk: chunk.clone(),
        })
        .unwrap();
        let reply = IcSnapshotDataReply::decode(&request, &raw).unwrap();
        assert_eq!(reply.chunk(), chunk);
        assert_eq!(
            reply.chunk_checksum(),
            &ArtifactChecksumRecord::from_bytes(&chunk)
        );
        let raw = candid::encode_one(ReadCanisterSnapshotDataResult {
            chunk: vec![19; MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES + 1],
        })
        .unwrap();
        assert_eq!(
            IcSnapshotDataReply::decode(&request, &raw).unwrap_err(),
            IcSnapshotDataError::InvalidReply
        );
        assert_eq!(
            IcSnapshotDataReply::decode(&request, &vec![0; MAX_IC_SNAPSHOT_DATA_REPLY_BYTES + 1])
                .unwrap_err(),
            IcSnapshotDataError::ReplyTooLarge
        );
    }
}

#[test]
fn retains_metadata_and_raw_wire_identity_separately_from_matching_data_bytes() {
    let source = metadata_request();
    let raw_metadata = metadata_bytes();
    let original = IcSnapshotMetadataReply::decode(&source, &raw_metadata).unwrap();
    let mut values: ReadCanisterSnapshotMetadataResult = candid::decode_one(&raw_metadata).unwrap();
    values.taken_at_timestamp -= 1;
    let changed =
        IcSnapshotMetadataReply::decode(&source, &candid::encode_one(values).unwrap()).unwrap();
    let request = IcSnapshotDataRequest::new(&original, region(0, 0, 3)).unwrap();
    let other = IcSnapshotDataRequest::new(&changed, region(0, 0, 3)).unwrap();
    assert_eq!(request.arguments(), other.arguments());
    assert_eq!(request.digest(), other.digest());
    let raw = candid::encode_one(ReadCanisterSnapshotDataResult {
        chunk: vec![0, 255, 17],
    })
    .unwrap();
    let first = IcSnapshotDataReply::decode(&request, &raw).unwrap();
    let second = IcSnapshotDataReply::decode(&other, &raw).unwrap();
    assert_eq!(first.chunk_checksum(), second.chunk_checksum());
    assert_eq!(first.payload_checksum(), second.payload_checksum());
    assert_ne!(first.digest(), second.digest());
    // An equivalent record with the two type-table rows in the opposite order
    // carries identical data but keeps a different exact wire identity.
    let alternate_wire = hex_bytes("4449444c026d7b6c01ed8c8bae040001010300ff11");
    let alternate = IcSnapshotDataReply::decode(&request, &alternate_wire).unwrap();
    assert_eq!(first.chunk_checksum(), alternate.chunk_checksum());
    assert_ne!(first.payload_checksum(), alternate.payload_checksum());
    assert_ne!(first.digest(), alternate.digest());
    let mut preimage = b"ic-backup/ic-snapshot-data-reply/v1\0".to_vec();
    preimage.extend(original.digest().hash().as_bytes());
    preimage.extend(request.digest().hash().as_bytes());
    preimage.extend(ArtifactChecksumRecord::from_bytes(&raw).hash().as_bytes());
    assert_eq!(
        first.digest(),
        ArtifactChecksumRecord::from_bytes(&preimage)
    );
    let different = IcSnapshotDataRequest::new(&original, region(0, 1, 3)).unwrap();
    assert_ne!(request.digest(), different.digest());
    assert_ne!(
        first.digest(),
        IcSnapshotDataReply::decode(&different, &raw)
            .unwrap()
            .digest()
    );
    for (target, id) in [
        (source.target(), vec![0, 255, 18]),
        ("2vxsx-fae", source.snapshot_id().to_vec()),
    ] {
        let declaration = IcSnapshotMetadataRequest::new(target, &id).unwrap();
        let associated = IcSnapshotMetadataReply::decode(&declaration, &raw_metadata).unwrap();
        let rebound = IcSnapshotDataRequest::new(&associated, region(0, 0, 3)).unwrap();
        assert_ne!(request.arguments(), rebound.arguments());
        assert_ne!(request.digest(), rebound.digest());
        let view = IcSnapshotDataReply::decode(&rebound, &raw).unwrap();
        assert_eq!(first.chunk_checksum(), view.chunk_checksum());
        assert_ne!(first.digest(), view.digest());
    }
    let text = format!("{first:?}");
    assert!(!text.contains("chunk: ["));
    assert!(!text.contains(first.chunk_checksum().hash()));
    assert!(!text.contains("snapshot_id: ["));
    assert!(!text.contains("arguments: ["));
}

#[test]
fn rejects_truncated_missing_extra_skipped_wrong_type_and_hostile_header_values() {
    #[derive(CandidType)]
    struct Extra {
        chunk: Vec<u8>,
        extra: u64,
    }
    #[derive(CandidType)]
    struct Wrong {
        chunk: Option<Vec<u8>>,
    }
    let source = metadata_request();
    let metadata = IcSnapshotMetadataReply::decode(&source, &metadata_bytes()).unwrap();
    let request = IcSnapshotDataRequest::new(&metadata, region(0, 0, 3)).unwrap();
    let raw = candid::encode_one(ReadCanisterSnapshotDataResult {
        chunk: vec![0, 255, 17],
    })
    .unwrap();
    for length in 0..raw.len() {
        assert_eq!(
            IcSnapshotDataReply::decode(&request, &raw[..length]).unwrap_err(),
            IcSnapshotDataError::InvalidReply
        );
    }
    let invalid = [
        b"bad wire".to_vec(),
        candid::encode_args(()).unwrap(),
        candid::encode_args((
            ReadCanisterSnapshotDataResult {
                chunk: vec![0, 255, 17],
            },
            0_u64,
        ))
        .unwrap(),
        [raw.as_slice(), &[0]].concat(),
        candid::encode_one(Extra {
            chunk: vec![0, 255, 17],
            extra: 0,
        })
        .unwrap(),
        candid::encode_one(Wrong { chunk: None }).unwrap(),
        candid::encode_one(12_u64).unwrap(),
        [
            b"DIDL".as_slice(),
            &[17],
            &[0x6e, 0x7f].repeat(17),
            &[1, 0, 0],
        ]
        .concat(),
        [b"DIDL".as_slice(), &[1, 0x6c, 0xff, 0xff, 0xff, 0xff, 0x0f]].concat(),
    ];
    for bytes in invalid {
        assert_eq!(
            IcSnapshotDataReply::decode(&request, &bytes).unwrap_err(),
            IcSnapshotDataError::InvalidReply
        );
    }
}
