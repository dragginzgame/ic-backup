use super::*;
use candid::{CandidType, Nat, Reserved};
use ic_management_canister_types::{
    CanisterTimer, ChunkHash, OnLowWasmMemoryHookStatus, SnapshotMetadataGlobal, SnapshotSource,
};
use serde::Deserialize;

#[derive(Deserialize)]
struct GoldenCase {
    name: String,
    target: String,
    snapshot_id: Vec<u8>,
    arguments_hex: String,
    request_digest: String,
    reply_hex: String,
    payload_checksum: String,
    digest: String,
    globals: usize,
    source_present: bool,
}

fn hex_bytes(hex: &str) -> Vec<u8> {
    let (pairs, rest) = hex.as_bytes().as_chunks::<2>();
    assert_eq!(rest, b"");
    pairs
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

#[test]
fn admits_every_registered_independent_wire_and_evidence_digest() {
    let cases: Vec<GoldenCase> = serde_json::from_str(include_str!("golden.json")).unwrap();
    for case in cases {
        let request = IcSnapshotMetadataRequest::new(&case.target, &case.snapshot_id).unwrap();
        assert_eq!(
            request.arguments(),
            hex_bytes(&case.arguments_hex),
            "{}",
            case.name
        );
        assert_eq!(
            request.digest().hash(),
            case.request_digest,
            "{}",
            case.name
        );
        let raw = hex_bytes(&case.reply_hex);
        let reply = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
        let official: ReadCanisterSnapshotMetadataResult = candid::decode_one(&raw).unwrap();
        assert_eq!(
            candid::encode_one(reply.metadata()).unwrap(),
            candid::encode_one(official).unwrap()
        );
        assert_eq!(reply.payload_checksum().hash(), case.payload_checksum);
        assert_eq!(reply.digest().hash(), case.digest);
        assert_eq!(reply.metadata().globals.len(), case.globals);
        assert_eq!(reply.metadata().source.is_some(), case.source_present);
    }
}

fn request() -> IcSnapshotMetadataRequest {
    IcSnapshotMetadataRequest::new("renrk-eyaaa-aaaaa-aaada-cai", &[0, 255, 17]).unwrap()
}

fn metadata() -> ReadCanisterSnapshotMetadataResult {
    ReadCanisterSnapshotMetadataResult {
        source: Some(SnapshotSource::TakenFromCanister(Reserved)),
        taken_at_timestamp: u64::MAX,
        wasm_module_size: u64::MAX,
        globals: vec![
            Some(SnapshotMetadataGlobal::I32(i32::MIN)),
            Some(SnapshotMetadataGlobal::I64(i64::MIN)),
            Some(SnapshotMetadataGlobal::F32(f32::from_bits(0x7fc0_0042))),
            Some(SnapshotMetadataGlobal::F64(f64::from_bits(
                0x8000_0000_0000_0000,
            ))),
            Some(SnapshotMetadataGlobal::V128(Nat::from(u128::MAX))),
            None,
        ],
        wasm_memory_size: 0,
        stable_memory_size: u64::MAX,
        wasm_chunk_store: vec![
            ChunkHash { hash: vec![2; 32] },
            ChunkHash { hash: vec![1; 32] },
        ],
        canister_version: u64::MAX,
        certified_data: vec![17; 32],
        global_timer: Some(CanisterTimer::Active(u64::MAX)),
        on_low_wasm_memory_hook_status: Some(OnLowWasmMemoryHookStatus::Executed),
    }
}

#[test]
fn exact_arguments_normalization_and_independent_digests() {
    let request = request();
    let args: ReadCanisterSnapshotMetadataArgs = candid::decode_one(request.arguments()).unwrap();
    assert_eq!(args.canister_id.to_text(), request.target());
    assert_eq!(args.snapshot_id, request.snapshot_id());
    assert_eq!(request.receiver(), "aaaaa-aa");
    assert_eq!(request.method(), "read_canister_snapshot_metadata");
    let principal = args.canister_id.as_slice();
    let mut preimage = b"ic-backup/ic-management-request/v1\0".to_vec();
    preimage.extend([0, u8::try_from(principal.len()).unwrap()]);
    preimage.extend(principal);
    preimage.push(1);
    preimage.push(31);
    preimage.extend(b"read_canister_snapshot_metadata");
    preimage.extend(
        u32::try_from(request.arguments().len())
            .unwrap()
            .to_be_bytes(),
    );
    preimage.extend(request.arguments());
    assert_eq!(
        request.digest(),
        ArtifactChecksumRecord::from_bytes(&preimage)
    );
    let alias =
        IcSnapshotMetadataRequest::new(&request.target().to_uppercase(), request.snapshot_id())
            .unwrap();
    assert_eq!(alias.arguments(), request.arguments());
    assert_eq!(alias.digest(), request.digest());
    let different = IcSnapshotMetadataRequest::new(request.target(), &[0, 255, 18]).unwrap();
    assert_ne!(different.digest(), request.digest());
    let other_target = IcSnapshotMetadataRequest::new("2vxsx-fae", request.snapshot_id()).unwrap();
    assert_ne!(other_target.digest(), request.digest());
    let raw = candid::encode_one(metadata()).unwrap();
    let reply = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let mut expected = b"ic-backup/ic-snapshot-metadata-reply/v1\0".to_vec();
    expected.extend(request.digest().hash().as_bytes());
    expected.extend(ArtifactChecksumRecord::from_bytes(&raw).hash().as_bytes());
    assert_eq!(
        reply.digest(),
        ArtifactChecksumRecord::from_bytes(&expected)
    );
    assert_eq!(
        reply.payload_checksum(),
        &ArtifactChecksumRecord::from_bytes(&raw)
    );
    assert_eq!(reply.request().arguments(), request.arguments());
    assert_ne!(
        reply.digest(),
        IcSnapshotMetadataReply::decode(&different, &raw)
            .unwrap()
            .digest()
    );
}

#[test]
fn preserves_order_optional_values_and_float_bits_without_completeness_claims() {
    let request = request();
    let raw = candid::encode_one(metadata()).unwrap();
    let reply = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let value = reply.metadata();
    assert_eq!(value.taken_at_timestamp, u64::MAX);
    assert_eq!(value.wasm_module_size, u64::MAX);
    assert_eq!(value.wasm_memory_size, 0);
    assert_eq!(value.stable_memory_size, u64::MAX);
    assert_eq!(value.canister_version, u64::MAX);
    assert_eq!(value.certified_data, vec![17; 32]);
    assert_eq!(value.wasm_chunk_store[0].hash, vec![2; 32]);
    assert_eq!(value.wasm_chunk_store[1].hash, vec![1; 32]);
    assert!(matches!(
        value.source,
        Some(SnapshotSource::TakenFromCanister(_))
    ));
    assert!(matches!(
        value.globals[0],
        Some(SnapshotMetadataGlobal::I32(i32::MIN))
    ));
    assert!(matches!(
        value.globals[1],
        Some(SnapshotMetadataGlobal::I64(i64::MIN))
    ));
    assert!(
        matches!(value.globals[2], Some(SnapshotMetadataGlobal::F32(f)) if f.to_bits() == 0x7fc0_0042)
    );
    assert!(
        matches!(value.globals[3], Some(SnapshotMetadataGlobal::F64(f)) if f.to_bits() == 0x8000_0000_0000_0000)
    );
    let maximum = Nat::from(u128::MAX);
    assert!(matches!(&value.globals[4], Some(SnapshotMetadataGlobal::V128(n)) if *n == maximum));
    assert!(value.globals[5].is_none());
    assert!(matches!(
        value.global_timer,
        Some(CanisterTimer::Active(u64::MAX))
    ));
    assert!(matches!(
        value.on_low_wasm_memory_hook_status,
        Some(OnLowWasmMemoryHookStatus::Executed)
    ));
    for source in [None, Some(SnapshotSource::MetadataUpload(Reserved))] {
        for timer in [None, Some(CanisterTimer::Inactive)] {
            for hook in [
                None,
                Some(OnLowWasmMemoryHookStatus::Ready),
                Some(OnLowWasmMemoryHookStatus::ConditionNotSatisfied),
            ] {
                let mut value = metadata();
                value.source = source.clone();
                value.global_timer = timer.clone();
                value.on_low_wasm_memory_hook_status = hook.clone();
                value.globals = vec![None];
                value.certified_data.clear();
                value.wasm_chunk_store.clear();
                let reply =
                    IcSnapshotMetadataReply::decode(&request, &candid::encode_one(&value).unwrap())
                        .unwrap();
                assert_eq!(reply.metadata().source, source);
                assert_eq!(
                    candid::encode_one(reply.metadata()).unwrap(),
                    candid::encode_one(value).unwrap()
                );
            }
        }
    }
    let diagnostic = format!("{reply:?}");
    assert!(!diagnostic.contains("certified_data"));
    assert!(!diagnostic.contains("globals"));
    assert!(!diagnostic.contains("snapshot_id: ["));
}

#[test]
fn enforces_request_raw_sequence_hash_and_global_bounds() {
    assert_eq!(
        IcSnapshotMetadataRequest::new("bad", &[1]).unwrap_err(),
        IcRequestError::InvalidTarget
    );
    for id in [vec![], vec![1; MAX_IC_SNAPSHOT_ID_BYTES + 1]] {
        assert_eq!(
            IcSnapshotMetadataRequest::new("2vxsx-fae", &id).unwrap_err(),
            IcRequestError::InvalidSnapshotId
        );
    }
    assert!(IcSnapshotMetadataRequest::new("2vxsx-fae", &[1; MAX_IC_SNAPSHOT_ID_BYTES]).is_ok());
    let request = request();
    assert_eq!(
        IcSnapshotMetadataReply::decode(&request, &vec![0; MAX_IC_SNAPSHOT_METADATA_BYTES + 1])
            .unwrap_err(),
        IcSnapshotMetadataError::ReplyTooLarge
    );
    let check = |value: &ReadCanisterSnapshotMetadataResult| {
        IcSnapshotMetadataReply::decode(&request, &candid::encode_one(value).unwrap()).map(|_| ())
    };
    let mut value = metadata();
    value.globals = vec![None; MAX_IC_SNAPSHOT_GLOBALS];
    assert_eq!(check(&value), Ok(()));
    value.globals.push(None);
    assert_eq!(check(&value), Err(IcSnapshotMetadataError::InvalidReply));
    value = metadata();
    value.certified_data.push(0);
    assert_eq!(check(&value), Err(IcSnapshotMetadataError::InvalidReply));
    value = metadata();
    value.wasm_chunk_store[0].hash.pop();
    assert_eq!(
        check(&value),
        Err(IcSnapshotMetadataError::InvalidChunkHash)
    );
    value.wasm_chunk_store[0].hash = vec![1; 32];
    assert_eq!(
        check(&value),
        Err(IcSnapshotMetadataError::InvalidChunkHash)
    );
    value.wasm_chunk_store[0].hash = vec![1; 33];
    assert_eq!(check(&value), Err(IcSnapshotMetadataError::InvalidReply));
    value = metadata();
    value.wasm_chunk_store = (0..MAX_IC_SNAPSHOT_CHUNKS)
        .map(|i| {
            let mut hash = vec![0; 32];
            hash[..8].copy_from_slice(&u64::try_from(i).unwrap().to_be_bytes());
            ChunkHash { hash }
        })
        .collect();
    assert_eq!(check(&value), Ok(()));
    value.wasm_chunk_store.push(ChunkHash { hash: vec![3; 32] });
    assert_eq!(check(&value), Err(IcSnapshotMetadataError::InvalidReply));
    value = metadata();
    value.globals = vec![Some(SnapshotMetadataGlobal::V128(
        Nat::from(u128::MAX) + Nat::from(1_u8),
    ))];
    assert_eq!(check(&value), Err(IcSnapshotMetadataError::InvalidGlobal));
}

#[test]
fn rejects_malformed_truncated_extra_and_skipped_wire_without_payload_diagnostics() {
    #[derive(CandidType)]
    struct Extra {
        value: ReadCanisterSnapshotMetadataResult,
        extra: u64,
    }
    let request = request();
    let raw = candid::encode_one(metadata()).unwrap();
    for length in 0..raw.len() {
        assert_eq!(
            IcSnapshotMetadataReply::decode(&request, &raw[..length]).unwrap_err(),
            IcSnapshotMetadataError::InvalidReply
        );
    }
    for invalid in [
        b"bad wire".to_vec(),
        candid::encode_args(()).unwrap(),
        candid::encode_args((metadata(), 1_u64)).unwrap(),
        [raw.as_slice(), &[0]].concat(),
        candid::encode_one(123_u64).unwrap(),
    ] {
        let error = IcSnapshotMetadataReply::decode(&request, &invalid).unwrap_err();
        assert_eq!(error, IcSnapshotMetadataError::InvalidReply);
        assert_eq!(
            error.to_string(),
            "invalid or unsupported snapshot metadata reply"
        );
    }
    // A nested unexpected record must reject, rather than default missing fields.
    let bytes = candid::encode_one(Extra {
        value: metadata(),
        extra: 0,
    })
    .unwrap();
    assert_eq!(
        IcSnapshotMetadataReply::decode(&request, &bytes).unwrap_err(),
        IcSnapshotMetadataError::InvalidReply
    );
}

#[derive(CandidType)]
struct MetadataShape<S, G, T> {
    source: S,
    taken_at_timestamp: u64,
    wasm_module_size: u64,
    globals: Vec<G>,
    wasm_memory_size: u64,
    stable_memory_size: u64,
    wasm_chunk_store: Vec<ChunkHash>,
    canister_version: u64,
    certified_data: Vec<u8>,
    global_timer: T,
    on_low_wasm_memory_hook_status: Option<OnLowWasmMemoryHookStatus>,
}

fn shape<S, G, T>(source: S, globals: Vec<G>, timer: T) -> MetadataShape<S, G, T> {
    MetadataShape {
        source,
        globals,
        global_timer: timer,
        taken_at_timestamp: 0,
        wasm_module_size: 0,
        wasm_memory_size: 0,
        stable_memory_size: 0,
        wasm_chunk_store: vec![],
        canister_version: 0,
        certified_data: vec![],
        on_low_wasm_memory_hook_status: None,
    }
}

#[test]
fn rejects_unknown_optional_variants_reserved_payloads_and_type_header_work() {
    #[derive(CandidType)]
    enum Unknown {
        Future(u64),
    }
    #[derive(CandidType, Deserialize)]
    enum BadSource {
        #[serde(rename = "taken_from_canister")]
        Taken(u64),
    }
    let request = request();
    let invalid = [
        candid::encode_one(shape(
            Some(Unknown::Future(0)),
            vec![None::<SnapshotMetadataGlobal>],
            None::<CanisterTimer>,
        ))
        .unwrap(),
        candid::encode_one(shape(
            None::<SnapshotSource>,
            vec![Some(Unknown::Future(0))],
            None::<CanisterTimer>,
        ))
        .unwrap(),
        candid::encode_one(shape(
            None::<SnapshotSource>,
            vec![None::<SnapshotMetadataGlobal>],
            Some(Unknown::Future(0)),
        ))
        .unwrap(),
        candid::encode_one(shape(
            Some(BadSource::Taken(0)),
            vec![None::<SnapshotMetadataGlobal>],
            None::<CanisterTimer>,
        ))
        .unwrap(),
        // 65 finite opt entries exceed the admitted table before any value parsing.
        [
            b"DIDL".as_slice(),
            &[65],
            &[0x6e, 0x7f].repeat(65),
            &[1, 0, 0],
        ]
        .concat(),
        // A single record with a hostile field count cannot allocate from that count.
        [b"DIDL".as_slice(), &[1, 0x6c, 0xff, 0xff, 0xff, 0xff, 0x0f]].concat(),
    ];
    for raw in invalid {
        assert_eq!(
            IcSnapshotMetadataReply::decode(&request, &raw).unwrap_err(),
            IcSnapshotMetadataError::InvalidReply
        );
    }
    // Valid optional fields can disappear from the wire without becoming defaults.
    let raw = candid::encode_one(shape(
        None::<SnapshotSource>,
        vec![None::<SnapshotMetadataGlobal>],
        None::<CanisterTimer>,
    ))
    .unwrap();
    let reply = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    assert!(reply.metadata().source.is_none());
    assert!(reply.metadata().globals[0].is_none());
}
