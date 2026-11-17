//! Re-authored cases from the frozen legacy metadata tests; fixture provenance
//! is recorded in docs/architecture/capture-transfer.md.
use super::super::MAX_GLOBALS;
use super::super::tests::{chunk, metadata, plan};
use super::*;
use crate::store::BackupDigest;
use candid::Nat;
use candid::{CandidType, Reserved};
use ic_platform_protocol::management_snapshots::{Float32Bits, Float64Bits};
use ic_platform_protocol::management_snapshots::{Global, SnapshotSource, fixtures};
use ic_platform_protocol::management_snapshots::{LowMemoryHook, Timer};
use serde::Deserialize;

fn encode(value: &SnapshotMetadata) -> Vec<u8> {
    fixtures::snapshot_metadata(value, MAX_REPLY_BYTES).unwrap()
}
fn schema(value: &impl CandidType) -> Vec<u8> {
    candid::encode_one(value).unwrap()
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
fn round_trip_metadata() -> SnapshotMetadata {
    let mut m = metadata(3, 5, 7);
    m.globals = vec![
        Some(Global::I32(i32::MIN)),
        Some(Global::I64(i64::MIN)),
        Some(Global::F32(Float32Bits(
            (f32::from_bits(0x7fc0_0042)).to_bits(),
        ))),
        Some(Global::F64(Float64Bits(
            (f64::from_bits(0x8000_0000_0000_0000)).to_bits(),
        ))),
        Some(Global::V128(u128::MAX)),
    ];
    m.wasm_chunk_store = vec![chunk(b"second"), chunk(b"")];
    m.canister_version = 11;
    m.taken_at_timestamp = 13;
    m.certified_data = vec![17; 32];
    m.global_timer = Some(Timer::Active(19));
    m.on_low_wasm_memory_hook_status = Some(LowMemoryHook::Executed);
    m
}

#[test]
fn independent_legacy_vectors_preserve_promotions_absence_and_every_field() {
    #[derive(Deserialize)]
    struct Case {
        name: String,
        reply_hex: String,
        payload_checksum: String,
    }
    let cases: Vec<Case> = serde_json::from_str(include_str!("golden.json")).unwrap();
    let maximum = u128::MAX;
    for case in cases {
        let raw = hex(&case.reply_hex);
        assert_eq!(
            BackupDigest::compute(&raw).to_string(),
            case.payload_checksum,
            "{}",
            case.name
        );
        let value = decode(&raw).unwrap();
        assert_eq!(value.canister_version, u64::MAX);
        assert_eq!(value.taken_at_timestamp, u64::MAX);
        assert_eq!(value.wasm_module_size, u64::MAX);
        assert_eq!(value.wasm_memory_size, 0);
        assert_eq!(value.stable_memory_size, u64::MAX);
        assert_eq!(value.certified_data, [17; 32]);
        assert_eq!(value.wasm_chunk_store[0].hash, [2; 32]);
        assert_eq!(value.wasm_chunk_store[1].hash, [1; 32]);
        assert!(matches!(
            value.source,
            Some(SnapshotSource::TakenFromCanister(_))
        ));
        assert!(matches!(value.global_timer, Some(Timer::Active(u64::MAX))));
        assert!(matches!(
            value.on_low_wasm_memory_hook_status,
            Some(LowMemoryHook::Executed)
        ));
        assert!(matches!(value.globals[0], Some(Global::I32(i32::MIN))));
        assert!(matches!(value.globals[1], Some(Global::I64(i64::MIN))));
        assert!(matches!(value.globals[2], Some(Global::F32(v)) if v.0 == 0x7fc0_0042));
        assert!(matches!(value.globals[3], Some(Global::F64(v)) if v.0 == 0x8000_0000_0000_0000));
        assert!(matches!(&value.globals[4], Some(Global::V128(v)) if *v == maximum));
        assert_eq!(
            value.globals.len(),
            if case.name == "sdk-optional" { 6 } else { 5 }
        );
        if value.globals.len() == 6 {
            assert!(value.globals[5].is_none());
        }
    }
}

#[test]
fn public_decoder_preserves_bytes_and_flows_into_original_schedule() {
    let p = plan(8);
    let m = round_trip_metadata();
    let raw = encode(&m);
    let decoded = p.decode_metadata(&[0, 255, 17], &raw).unwrap();
    assert_eq!(encode(&decoded), raw);
    let transfer = p.plan_transfer(&[0, 255, 17], &decoded).unwrap();
    assert_eq!(transfer.data_calls(), 5);
    for request in transfer.requests() {
        assert_eq!(request.snapshot_id, [0, 255, 17]);
        assert_eq!(request.canister_id, p.request().target.canister);
    }
    assert_eq!(
        transfer
            .decode_data(
                0,
                &crate::store::capture::test_fixtures::wire::encode(
                    &ic_platform_protocol::management_snapshots::SnapshotData {
                        chunk: b"abc".to_vec()
                    },
                    4096
                )
            )
            .unwrap(),
        b"abc"
    );
}

#[test]
fn valid_absence_is_unsupported_capture_instead_of_fabricated_state() {
    let p = plan(8);
    for alter in [
        |m: &mut SnapshotMetadata| m.source = None,
        |m: &mut SnapshotMetadata| {
            m.source = Some(SnapshotSource::MetadataUpload(
                ic_platform_protocol::management_snapshots::Reserved,
            ))
        },
        |m: &mut SnapshotMetadata| m.global_timer = None,
        |m: &mut SnapshotMetadata| m.on_low_wasm_memory_hook_status = None,
        |m: &mut SnapshotMetadata| m.globals.push(None),
    ] {
        let mut m = round_trip_metadata();
        alter(&mut m);
        let raw = encode(&m);
        assert_eq!(encode(&decode(&raw).unwrap()), raw);
        assert!(matches!(
            p.decode_metadata(&[1], &raw),
            Err(TransferMetadataError::Transfer(
                TransferError::UnsupportedProfile
            ))
        ));
    }
}

#[test]
fn original_limits_and_invalid_snapshot_ids_are_not_bypassed_by_decoding() {
    let mut request = plan(8).request().clone();
    request.limits.payload_bytes = 8;
    let p = CapturePolicy::new(request).unwrap();
    assert!(matches!(
        p.decode_metadata(&[1], &encode(&metadata(9, 0, 0))),
        Err(TransferMetadataError::Transfer(
            TransferError::PayloadBudget {
                required: 9,
                original: 8
            }
        ))
    ));
    assert!(matches!(
        p.decode_metadata(&[], &encode(&metadata(1, 0, 0))),
        Err(TransferMetadataError::Transfer(TransferError::SnapshotId))
    ));
    assert!(p.decode_metadata(&[1], &encode(&metadata(8, 0, 0))).is_ok());
}

#[test]
fn admitted_maxima_and_oversized_collections_hashes_and_globals() {
    let p = plan(64);
    let mut m = metadata(1, 0, 0);
    m.globals = vec![Some(Global::I32(1)); MAX_GLOBALS];
    m.certified_data = vec![0; 32];
    m.wasm_chunk_store = (0..1024_u32).map(|n| chunk(&n.to_be_bytes())).collect();
    assert!(p.decode_metadata(&[1], &encode(&m)).is_ok());
    for alter in [
        |m: &mut SnapshotMetadata| m.globals.push(None),
        |m: &mut SnapshotMetadata| m.certified_data.push(0),
        |m: &mut SnapshotMetadata| m.wasm_chunk_store.push(chunk(b"extra")),
        |m: &mut SnapshotMetadata| m.wasm_chunk_store[0].hash.push(0),
    ] {
        let mut oversized = m.clone();
        alter(&mut oversized);
        assert!(matches!(
            p.decode_metadata(&[1], &encode(&oversized)),
            Err(TransferMetadataError::Decode(DecodeError::Limit))
        ));
    }
    m.wasm_chunk_store[0].hash.pop();
    assert!(matches!(
        p.decode_metadata(&[1], &encode(&m)),
        Err(TransferMetadataError::Transfer(TransferError::ChunkHash))
    ));
    m.wasm_chunk_store[0] = m.wasm_chunk_store[1].clone();
    assert!(matches!(
        p.decode_metadata(&[1], &encode(&m)),
        Err(TransferMetadataError::Transfer(TransferError::ChunkHash))
    ));
    m.wasm_chunk_store.clear();
    // The typed API cannot hold 129 bits; preserve the hostile original wire case.
    let raw = schema(&shape(
        ic_management_canister_types::SnapshotSource::TakenFromCanister(Reserved),
        vec![Some(
            ic_management_canister_types::SnapshotMetadataGlobal::V128(Nat(
                Nat::from(u128::MAX).0 + 1_u8
            )),
        )],
        ic_management_canister_types::CanisterTimer::Inactive,
        ic_management_canister_types::OnLowWasmMemoryHookStatus::Ready,
    ));
    assert!(matches!(
        p.decode_metadata(&[1], &raw),
        Err(TransferMetadataError::Decode(DecodeError::Limit))
    ));
}

#[test]
fn malformed_truncated_trailing_arity_and_header_limits_refuse() {
    let raw = encode(&round_trip_metadata());
    for end in 0..raw.len() {
        assert!(decode(&raw[..end]).is_err(), "prefix {end}");
    }
    let mut trailing = raw;
    trailing.push(0);
    assert!(matches!(
        decode(&trailing),
        Err(TransferMetadataError::Decode(DecodeError::Trailing))
    ));
    for raw in [b"DIDL\0\0".as_slice(), b"DIDL\0\x02\x7f\x7f"] {
        assert!(matches!(
            decode(raw),
            Err(TransferMetadataError::Decode(DecodeError::Arity))
        ));
    }
    assert!(matches!(
        decode(&vec![0; MAX_REPLY_BYTES + 1]),
        Err(TransferMetadataError::ReplyTooLarge)
    ));
    for raw in [b"DIDL\x41".as_slice(), b"DIDL\x01\x6c\xff\xff\xff\xff\x0f"] {
        assert!(matches!(
            decode(raw),
            Err(TransferMetadataError::Decode(DecodeError::Limit))
        ));
    }
}

// Test-only wire alternatives; no replacement management declarations in production.
#[derive(CandidType, serde::Deserialize)]
struct Shape<S, G, T, H> {
    source: S,
    globals: Vec<G>,
    #[serde(rename = "global_timer")]
    timer: T,
    #[serde(rename = "on_low_wasm_memory_hook_status")]
    hook: H,
    canister_version: u64,
    certified_data: Vec<u8>,
    wasm_module_size: u64,
    stable_memory_size: u64,
    wasm_chunk_store: Vec<ic_management_canister_types::ChunkHash>,
    taken_at_timestamp: u64,
    wasm_memory_size: u64,
}
fn shape<S, G, T, H>(source: S, globals: Vec<G>, timer: T, hook: H) -> Shape<S, G, T, H> {
    Shape {
        source,
        globals,
        timer,
        hook,
        canister_version: 2,
        certified_data: vec![],
        wasm_module_size: 1,
        stable_memory_size: 0,
        wasm_chunk_store: vec![],
        taken_at_timestamp: 3,
        wasm_memory_size: 0,
    }
}

#[test]
fn null_and_reserved_absence_and_bare_promotion_remain_valid_wire() {
    for raw in [
        schema(&shape((), vec![()], (), ())),
        schema(&shape(Reserved, vec![Reserved], Reserved, Reserved)),
    ] {
        let m = decode(&raw).unwrap();
        assert!(m.source.is_none());
        assert_eq!(m.globals.len(), 1);
        assert!(m.globals[0].is_none());
        assert!(m.global_timer.is_none());
        assert!(m.on_low_wasm_memory_hook_status.is_none());
        assert!(matches!(
            plan(4).decode_metadata(&[1], &raw),
            Err(TransferMetadataError::Transfer(
                TransferError::UnsupportedProfile
            ))
        ));
    }
    let raw = schema(&shape(
        ic_management_canister_types::SnapshotSource::TakenFromCanister(Reserved),
        vec![ic_management_canister_types::SnapshotMetadataGlobal::I32(7)],
        ic_management_canister_types::CanisterTimer::Inactive,
        ic_management_canister_types::OnLowWasmMemoryHookStatus::Ready,
    ));
    assert!(plan(4).decode_metadata(&[1], &raw).is_ok());
}

#[test]
fn unknown_optional_variants_and_non_reserved_source_payloads_cannot_disappear() {
    #[derive(candid::CandidType, serde::Deserialize)]
    enum Future {
        Value(()),
    }

    #[derive(candid::CandidType, serde::Deserialize)]

    enum BadSource {
        #[serde(rename = "taken_from_canister")]
        Taken(()),
    }

    for raw in [
        schema(&shape(
            Some(Future::Value(())),
            Vec::<Option<ic_management_canister_types::SnapshotMetadataGlobal>>::new(),
            None::<ic_management_canister_types::CanisterTimer>,
            None::<ic_management_canister_types::OnLowWasmMemoryHookStatus>,
        )),
        schema(&shape(
            None::<ic_management_canister_types::SnapshotSource>,
            vec![Some(Future::Value(()))],
            None::<ic_management_canister_types::CanisterTimer>,
            None::<ic_management_canister_types::OnLowWasmMemoryHookStatus>,
        )),
        schema(&shape(
            None::<ic_management_canister_types::SnapshotSource>,
            Vec::<Option<ic_management_canister_types::SnapshotMetadataGlobal>>::new(),
            Some(Future::Value(())),
            None::<ic_management_canister_types::OnLowWasmMemoryHookStatus>,
        )),
        schema(&shape(
            None::<ic_management_canister_types::SnapshotSource>,
            Vec::<Option<ic_management_canister_types::SnapshotMetadataGlobal>>::new(),
            None::<ic_management_canister_types::CanisterTimer>,
            Some(Future::Value(())),
        )),
        schema(&shape(
            Some(BadSource::Taken(())),
            Vec::<Option<ic_management_canister_types::SnapshotMetadataGlobal>>::new(),
            None::<ic_management_canister_types::CanisterTimer>,
            None::<ic_management_canister_types::OnLowWasmMemoryHookStatus>,
        )),
    ] {
        assert!(matches!(
            decode(&raw),
            Err(TransferMetadataError::Decode(_))
        ));
    }
}

#[test]
fn declared_global_limit_wins_before_elements_are_read() {
    #[derive(CandidType, serde::Deserialize)]
    struct GlobalsOnly {
        globals: Vec<()>,
    }
    assert!(matches!(
        decode(&schema(&GlobalsOnly {
            globals: vec![(); MAX_GLOBALS + 1]
        })),
        Err(TransferMetadataError::Decode(DecodeError::Limit))
    ));
}

#[test]
fn unknown_zero_byte_field_and_missing_required_field_refuse() {
    #[derive(candid::CandidType, serde::Deserialize)]
    struct Extra {}

    // The empty record cannot default required metadata fields.
    assert!(matches!(
        decode(&schema(&Extra {})),
        Err(TransferMetadataError::Decode(DecodeError::Type))
    ));
    #[derive(candid::CandidType, serde::Deserialize)]
    struct Unknown {
        #[serde(rename = "")]
        extra: (),
    }

    assert!(matches!(
        decode(&schema(&Unknown { extra: () })),
        Err(TransferMetadataError::Decode(DecodeError::Limit))
    ));
}

#[test]
fn missing_optional_fields_are_absent_but_never_capture_defaults() {
    #[derive(CandidType, serde::Deserialize)]
    struct RequiredOnly {
        globals: Vec<Option<ic_management_canister_types::SnapshotMetadataGlobal>>,
        canister_version: u64,
        certified_data: Vec<u8>,
        wasm_module_size: u64,
        stable_memory_size: u64,
        wasm_chunk_store: Vec<ic_management_canister_types::ChunkHash>,
        taken_at_timestamp: u64,
        wasm_memory_size: u64,
    }
    let raw = schema(&RequiredOnly {
        globals: vec![],
        canister_version: 1,
        certified_data: vec![],
        wasm_module_size: 2,
        stable_memory_size: 3,
        wasm_chunk_store: vec![],
        taken_at_timestamp: 4,
        wasm_memory_size: 5,
    });
    let value = decode(&raw).unwrap();
    assert!(value.source.is_none());
    assert!(value.global_timer.is_none());
    assert!(value.on_low_wasm_memory_hook_status.is_none());
    assert!(matches!(
        plan(8).decode_metadata(&[1], &raw),
        Err(TransferMetadataError::Transfer(
            TransferError::UnsupportedProfile
        ))
    ));
}

#[test]
fn restore_upload_arguments_match_independent_sdk_float_bits_and_full_integer_range() {
    use ic_management_canister_types::{
        CanisterTimer, OnLowWasmMemoryHookStatus, SnapshotMetadataGlobal,
        UploadCanisterSnapshotMetadataArgs,
    };
    let metadata = round_trip_metadata();
    let target = plan(8).request().target.canister;
    let bytes = crate::store::restore::procedure::upload_metadata_arguments(
        target,
        &metadata,
        MAX_REPLY_BYTES,
    )
    .unwrap();
    let independent = UploadCanisterSnapshotMetadataArgs {
        canister_id: candid::Principal::from_slice(target.as_slice()),
        replace_snapshot: None,
        wasm_module_size: 3,
        wasm_memory_size: 5,
        stable_memory_size: 7,
        globals: vec![
            SnapshotMetadataGlobal::I32(i32::MIN),
            SnapshotMetadataGlobal::I64(i64::MIN),
            SnapshotMetadataGlobal::F32(f32::from_bits(0x7fc0_0042)),
            SnapshotMetadataGlobal::F64(f64::from_bits(0x8000_0000_0000_0000)),
            SnapshotMetadataGlobal::V128(Nat::from(u128::MAX)),
        ],
        certified_data: vec![17; 32],
        global_timer: Some(CanisterTimer::Active(19)),
        on_low_wasm_memory_hook_status: Some(OnLowWasmMemoryHookStatus::Executed),
    };
    assert_eq!(bytes, candid::encode_one(independent).unwrap());
}
