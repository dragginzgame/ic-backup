//! Coverage tests over real local codecs; no management behavior is simulated.

use super::*;
use crate::model::{
    ic_snapshot_data::IcSnapshotDataRequest, ic_snapshot_metadata::IcSnapshotMetadataRequest,
};
use ic_management_canister_types::{
    ChunkHash, ReadCanisterSnapshotDataResult, ReadCanisterSnapshotMetadataResult,
};
use sha2::{Digest, Sha256};

fn source() -> ReadCanisterSnapshotMetadataResult {
    let cases: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../../ic_snapshot_metadata/tests/golden.json")).unwrap();
    let case = cases
        .iter()
        .find(|case| case["name"] == "data-source")
        .unwrap();
    let raw = case["reply_hex"]
        .as_str()
        .unwrap()
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect::<Vec<_>>();
    candid::decode_one(&raw).unwrap()
}

fn region(index: usize, offset: u64, size: u64) -> SnapshotDataKind {
    match index {
        0 => SnapshotDataKind::WasmModule { offset, size },
        1 => SnapshotDataKind::WasmMemory { offset, size },
        2 => SnapshotDataKind::StableMemory { offset, size },
        _ => unreachable!(),
    }
}

fn admit(
    coverage: &mut IcSnapshotDataCoverage<'_>,
    kind: SnapshotDataKind,
    chunk: Vec<u8>,
) -> Result<(), IcSnapshotDataCoverageError> {
    let request = IcSnapshotDataRequest::new(coverage.metadata(), kind).unwrap();
    let raw = candid::encode_one(ReadCanisterSnapshotDataResult { chunk }).unwrap();
    let reply = IcSnapshotDataReply::decode(&request, &raw).unwrap();
    coverage.admit(&reply)
}

#[test]
fn interleaved_regions_and_unordered_chunks_complete_only_after_exact_coverage() {
    let request =
        IcSnapshotMetadataRequest::new("renrk-eyaaa-aaaaa-aaada-cai", &[0, 255, 17]).unwrap();
    let mut values = source();
    values.wasm_module_size = 5;
    values.wasm_memory_size = 3;
    values.stable_memory_size = 1;
    let raw = candid::encode_one(values).unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let mut coverage = IcSnapshotDataCoverage::new(&metadata);
    assert!(coverage.complete().is_none());
    admit(&mut coverage, region(1, 0, 3), vec![7; 3]).unwrap();
    admit(&mut coverage, region(0, 0, 2), vec![8; 2]).unwrap();
    admit(
        &mut coverage,
        SnapshotDataKind::WasmChunk {
            hash: metadata.metadata().wasm_chunk_store[1].hash.clone(),
        },
        vec![],
    )
    .unwrap();
    admit(&mut coverage, region(2, 0, 1), vec![9]).unwrap();
    admit(&mut coverage, region(0, 2, 3), vec![10; 3]).unwrap();
    assert_eq!(coverage.covered_region_bytes(), [5, 3, 1]);
    assert_eq!(coverage.covered_chunks(), 1);
    assert!(coverage.complete().is_none());
    admit(
        &mut coverage,
        SnapshotDataKind::WasmChunk {
            hash: metadata.metadata().wasm_chunk_store[0].hash.clone(),
        },
        vec![0, 255, 17],
    )
    .unwrap();
    assert_eq!(
        coverage.complete().unwrap().metadata().digest(),
        metadata.digest()
    );
    assert_eq!(coverage.covered_chunks(), 2);
    // All individual replies and their byte buffers have already been dropped.
    let reopened = IcSnapshotDataCoverage::new(&metadata);
    assert_eq!(reopened.covered_region_bytes(), [0; 3]);
    assert_eq!(reopened.covered_chunks(), 0);
    assert!(reopened.complete().is_none());
    let debug = format!("{coverage:?}");
    assert!(!debug.contains(request.target()));
    assert!(!debug.contains(metadata.digest().hash()));
}

#[test]
fn gaps_overlaps_duplicates_and_failed_admission_leave_every_region_unchanged() {
    let request = IcSnapshotMetadataRequest::new("renrk-eyaaa-aaaaa-aaada-cai", &[0]).unwrap();
    let mut values = source();
    values.wasm_module_size = 5;
    values.wasm_memory_size = 5;
    values.stable_memory_size = 5;
    let metadata =
        IcSnapshotMetadataReply::decode(&request, &candid::encode_one(values).unwrap()).unwrap();
    let mut coverage = IcSnapshotDataCoverage::new(&metadata);
    for index in 0..3 {
        let before = coverage.covered_region_bytes();
        assert_eq!(
            admit(&mut coverage, region(index, 1, 1), vec![0]),
            Err(IcSnapshotDataCoverageError::NoncontiguousRange)
        );
        assert_eq!(coverage.covered_region_bytes(), before);
        admit(&mut coverage, region(index, 0, 2), vec![0; 2]).unwrap();
        let before = coverage.covered_region_bytes();
        for (offset, size) in [(0, 2), (1, 2), (3, 1)] {
            assert_eq!(
                admit(
                    &mut coverage,
                    region(index, offset, size),
                    vec![0; usize::try_from(size).unwrap()]
                ),
                Err(IcSnapshotDataCoverageError::NoncontiguousRange)
            );
            assert_eq!(coverage.covered_region_bytes(), before);
        }
        admit(&mut coverage, region(index, 2, 3), vec![0; 3]).unwrap();
    }
    let kind = SnapshotDataKind::WasmChunk {
        hash: metadata.metadata().wasm_chunk_store[1].hash.clone(),
    };
    admit(&mut coverage, kind.clone(), vec![]).unwrap();
    assert_eq!(
        admit(&mut coverage, kind, vec![]),
        Err(IcSnapshotDataCoverageError::DuplicateChunk)
    );
    assert_eq!(coverage.covered_chunks(), 1);
    assert_eq!(coverage.covered_region_bytes(), [5; 3]);
    assert!(coverage.complete().is_none());
}

#[test]
fn changed_target_id_or_raw_metadata_cannot_mix_but_exact_redecode_can() {
    let request = IcSnapshotMetadataRequest::new("renrk-eyaaa-aaaaa-aaada-cai", &[0]).unwrap();
    let original = source();
    let raw = candid::encode_one(&original).unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let mut coverage = IcSnapshotDataCoverage::new(&metadata);
    for (target, id, timestamp) in [
        (
            "ryjl3-tyaaa-aaaaa-aaaba-cai",
            vec![0],
            original.taken_at_timestamp,
        ),
        (request.target(), vec![1], original.taken_at_timestamp),
        (request.target(), vec![0], original.taken_at_timestamp ^ 1),
    ] {
        let changed_request = IcSnapshotMetadataRequest::new(target, &id).unwrap();
        let mut changed = original.clone();
        changed.taken_at_timestamp = timestamp;
        let changed_metadata = IcSnapshotMetadataReply::decode(
            &changed_request,
            &candid::encode_one(changed).unwrap(),
        )
        .unwrap();
        let data = IcSnapshotDataRequest::new(&changed_metadata, region(0, 0, 1)).unwrap();
        let reply = IcSnapshotDataReply::decode(
            &data,
            &candid::encode_one(ReadCanisterSnapshotDataResult { chunk: vec![0] }).unwrap(),
        )
        .unwrap();
        assert_eq!(
            coverage.admit(&reply),
            Err(IcSnapshotDataCoverageError::MetadataMismatch)
        );
        assert_eq!(coverage.covered_region_bytes(), [0; 3]);
        assert_eq!(coverage.covered_chunks(), 0);
    }
    let same = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let data = IcSnapshotDataRequest::new(&same, region(0, 0, 1)).unwrap();
    let reply = IcSnapshotDataReply::decode(
        &data,
        &candid::encode_one(ReadCanisterSnapshotDataResult { chunk: vec![0] }).unwrap(),
    )
    .unwrap();
    coverage.admit(&reply).unwrap();
    assert_eq!(coverage.covered_region_bytes(), [1, 0, 0]);
    // Equivalent metadata values with a longer valid type-count LEB128 still
    // have different raw evidence. Coverage cannot combine those observations.
    let mut alternate = raw.clone();
    assert!(alternate[4] < 128);
    alternate.splice(4..5, [raw[4] | 128, 0]);
    let equivalent = IcSnapshotMetadataReply::decode(&request, &alternate).unwrap();
    assert_eq!(
        candid::encode_one(equivalent.metadata()).unwrap(),
        candid::encode_one(metadata.metadata()).unwrap()
    );
    let data = IcSnapshotDataRequest::new(&equivalent, region(0, 1, 1)).unwrap();
    let reply = IcSnapshotDataReply::decode(
        &data,
        &candid::encode_one(ReadCanisterSnapshotDataResult { chunk: vec![0] }).unwrap(),
    )
    .unwrap();
    assert_eq!(
        coverage.admit(&reply),
        Err(IcSnapshotDataCoverageError::MetadataMismatch)
    );
    assert_eq!(coverage.covered_region_bytes(), [1, 0, 0]);
}

#[test]
fn empty_regions_require_no_reads_but_all_maximum_chunk_identities_need_replies() {
    let request = IcSnapshotMetadataRequest::new("renrk-eyaaa-aaaaa-aaada-cai", &[0]).unwrap();
    let mut values = source();
    values.wasm_module_size = 0;
    values.wasm_memory_size = 0;
    values.stable_memory_size = 0;
    values.wasm_chunk_store.clear();
    let empty =
        IcSnapshotMetadataReply::decode(&request, &candid::encode_one(&values).unwrap()).unwrap();
    assert!(IcSnapshotDataCoverage::new(&empty).complete().is_some());
    let chunks = (0_u16..1024)
        .map(|index| index.to_be_bytes().to_vec())
        .collect::<Vec<_>>();
    values.wasm_chunk_store = chunks
        .iter()
        .map(|chunk| ChunkHash {
            hash: Sha256::digest(chunk).to_vec(),
        })
        .collect();
    let metadata =
        IcSnapshotMetadataReply::decode(&request, &candid::encode_one(values).unwrap()).unwrap();
    let mut coverage = IcSnapshotDataCoverage::new(&metadata);
    for (index, chunk) in chunks.into_iter().enumerate().rev() {
        assert!(coverage.complete().is_none());
        admit(
            &mut coverage,
            SnapshotDataKind::WasmChunk {
                hash: metadata.metadata().wasm_chunk_store[index].hash.clone(),
            },
            chunk,
        )
        .unwrap();
    }
    assert_eq!(coverage.covered_chunks(), 1024);
    assert!(coverage.complete().is_some());
}

#[test]
fn maximum_nat64_regions_stay_independent_and_streamed_maximum_replies_drop() {
    let request = IcSnapshotMetadataRequest::new("renrk-eyaaa-aaaaa-aaada-cai", &[0]).unwrap();
    let mut values = source();
    values.wasm_module_size = u64::MAX;
    values.wasm_memory_size = u64::MAX;
    values.stable_memory_size = u64::MAX;
    values.wasm_chunk_store.clear();
    let metadata =
        IcSnapshotMetadataReply::decode(&request, &candid::encode_one(values).unwrap()).unwrap();
    let mut coverage = IcSnapshotDataCoverage::new(&metadata);
    for index in 0..3 {
        for offset in [0, 1024 * 1024] {
            admit(
                &mut coverage,
                region(index, offset, 1024 * 1024),
                vec![42; 1024 * 1024],
            )
            .unwrap();
        }
    }
    assert_eq!(coverage.covered_region_bytes(), [2 * 1024 * 1024; 3]);
    assert!(coverage.complete().is_none());
}
