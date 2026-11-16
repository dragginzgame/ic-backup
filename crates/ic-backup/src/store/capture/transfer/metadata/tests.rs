//! Re-authored cases from the frozen legacy metadata tests; fixture provenance
//! is recorded in docs/architecture/capture-transfer.md.
use super::super::tests::{chunk, metadata, plan};
use super::*;
use ic_host_artifacts::artifact::Sha256Digest;
use ic_platform::{
    candid_codec::Nat,
    management_canister_types::{CanisterTimer, OnLowWasmMemoryHookStatus},
};
use serde::Deserialize;

fn encode(value: &impl Encode) -> Vec<u8> {
    candid_codec::encode_one(value, MAX_REPLY_BYTES).unwrap()
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
fn round_trip_metadata() -> ReadCanisterSnapshotMetadataResult {
    let mut m = metadata(3, 5, 7);
    m.globals = vec![
        Some(SnapshotMetadataGlobal::I32(i32::MIN)),
        Some(SnapshotMetadataGlobal::I64(i64::MIN)),
        Some(SnapshotMetadataGlobal::F32(f32::from_bits(0x7fc0_0042))),
        Some(SnapshotMetadataGlobal::F64(f64::from_bits(
            0x8000_0000_0000_0000,
        ))),
        Some(SnapshotMetadataGlobal::V128(Nat::from(u128::MAX))),
    ];
    m.wasm_chunk_store = vec![chunk(b"second"), chunk(b"")];
    m.canister_version = 11;
    m.taken_at_timestamp = 13;
    m.certified_data = vec![17; 32];
    m.global_timer = Some(CanisterTimer::Active(19));
    m.on_low_wasm_memory_hook_status = Some(OnLowWasmMemoryHookStatus::Executed);
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
    let maximum = Nat::from(u128::MAX);
    for case in cases {
        let raw = hex(&case.reply_hex);
        assert_eq!(
            Sha256Digest::compute(&raw).to_string(),
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
        assert!(matches!(
            value.global_timer,
            Some(CanisterTimer::Active(u64::MAX))
        ));
        assert!(matches!(
            value.on_low_wasm_memory_hook_status,
            Some(OnLowWasmMemoryHookStatus::Executed)
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
            matches!(value.globals[2], Some(SnapshotMetadataGlobal::F32(v)) if v.to_bits() == 0x7fc0_0042)
        );
        assert!(
            matches!(value.globals[3], Some(SnapshotMetadataGlobal::F64(v)) if v.to_bits() == 0x8000_0000_0000_0000)
        );
        assert!(
            matches!(&value.globals[4], Some(SnapshotMetadataGlobal::V128(v)) if *v == maximum)
        );
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
                &encode(
                    &ic_platform::management_canister_types::ReadCanisterSnapshotDataResult {
                        chunk: b"abc".to_vec()
                    }
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
        |m: &mut ReadCanisterSnapshotMetadataResult| m.source = None,
        |m: &mut ReadCanisterSnapshotMetadataResult| {
            m.source = Some(SnapshotSource::MetadataUpload(Reserved))
        },
        |m: &mut ReadCanisterSnapshotMetadataResult| m.global_timer = None,
        |m: &mut ReadCanisterSnapshotMetadataResult| m.on_low_wasm_memory_hook_status = None,
        |m: &mut ReadCanisterSnapshotMetadataResult| m.globals.push(None),
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
    m.globals = vec![Some(SnapshotMetadataGlobal::I32(1)); MAX_GLOBALS];
    m.certified_data = vec![0; 32];
    m.wasm_chunk_store = (0..1024_u32).map(|n| chunk(&n.to_be_bytes())).collect();
    assert!(p.decode_metadata(&[1], &encode(&m)).is_ok());
    for alter in [
        |m: &mut ReadCanisterSnapshotMetadataResult| m.globals.push(None),
        |m: &mut ReadCanisterSnapshotMetadataResult| m.certified_data.push(0),
        |m: &mut ReadCanisterSnapshotMetadataResult| m.wasm_chunk_store.push(chunk(b"extra")),
        |m: &mut ReadCanisterSnapshotMetadataResult| m.wasm_chunk_store[0].hash.push(0),
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
    m.globals = vec![Some(SnapshotMetadataGlobal::V128(Nat(Nat::from(
        u128::MAX,
    )
    .0 + 1_u8)))];
    assert!(matches!(
        p.decode_metadata(&[1], &encode(&m)),
        Err(TransferMetadataError::Transfer(TransferError::GlobalRange))
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
struct Shape<S, G, T, H> {
    source: S,
    globals: Vec<G>,
    timer: T,
    hook: H,
}
impl<S: Encode, G: Encode, T: Encode, H: Encode> Encode for Shape<S, G, T, H> {
    const TYPE: &'static Type = &Type::Record(&[
        (339281328, <Vec<G>>::TYPE),
        (783802816, u64::TYPE),
        (842117339, S::TYPE),
        (864574658, <Vec<u8>>::TYPE),
        (903275689, T::TYPE),
        (1282538240, H::TYPE),
        (1299009433, u64::TYPE),
        (1710030651, u64::TYPE),
        (1767008372, <Vec<ChunkHash>>::TYPE),
        (3569147778, u64::TYPE),
        (3598099748, u64::TYPE),
    ]);
    fn write(&self, out: &mut Writer) -> Result<(), candid_codec::Error> {
        self.globals.write(out)?;
        2_u64.write(out)?;
        self.source.write(out)?;
        Vec::<u8>::new().write(out)?;
        self.timer.write(out)?;
        self.hook.write(out)?;
        1_u64.write(out)?;
        0_u64.write(out)?;
        Vec::<ChunkHash>::new().write(out)?;
        3_u64.write(out)?;
        0_u64.write(out)
    }
}
fn shape<S, G, T, H>(source: S, globals: Vec<G>, timer: T, hook: H) -> Shape<S, G, T, H> {
    Shape {
        source,
        globals,
        timer,
        hook,
    }
}

#[test]
fn null_and_reserved_absence_and_bare_promotion_remain_valid_wire() {
    for raw in [
        encode(&shape((), vec![()], (), ())),
        encode(&shape(Reserved, vec![Reserved], Reserved, Reserved)),
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
    let raw = encode(&shape(
        SnapshotSource::TakenFromCanister(Reserved),
        vec![SnapshotMetadataGlobal::I32(7)],
        CanisterTimer::Inactive,
        OnLowWasmMemoryHookStatus::Ready,
    ));
    assert!(plan(4).decode_metadata(&[1], &raw).is_ok());
}

#[test]
fn unknown_optional_variants_and_non_reserved_source_payloads_cannot_disappear() {
    enum Future {
        Value(()),
    }
    ic_platform::candid_variant!(Future { 1 => Value(()) });
    enum BadSource {
        Taken(()),
    }
    ic_platform::candid_variant!(BadSource { 2359638948 => Taken(()) });
    for raw in [
        encode(&shape(
            Some(Future::Value(())),
            Vec::<Option<SnapshotMetadataGlobal>>::new(),
            None::<CanisterTimer>,
            None::<OnLowWasmMemoryHookStatus>,
        )),
        encode(&shape(
            None::<SnapshotSource>,
            vec![Some(Future::Value(()))],
            None::<CanisterTimer>,
            None::<OnLowWasmMemoryHookStatus>,
        )),
        encode(&shape(
            None::<SnapshotSource>,
            Vec::<Option<SnapshotMetadataGlobal>>::new(),
            Some(Future::Value(())),
            None::<OnLowWasmMemoryHookStatus>,
        )),
        encode(&shape(
            None::<SnapshotSource>,
            Vec::<Option<SnapshotMetadataGlobal>>::new(),
            None::<CanisterTimer>,
            Some(Future::Value(())),
        )),
        encode(&shape(
            Some(BadSource::Taken(())),
            Vec::<Option<SnapshotMetadataGlobal>>::new(),
            None::<CanisterTimer>,
            None::<OnLowWasmMemoryHookStatus>,
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
    struct GlobalsOnly;
    impl Encode for GlobalsOnly {
        const TYPE: &'static Type = &Type::Record(&[(339281328, <Vec<()>>::TYPE)]);
        fn write(&self, out: &mut Writer) -> Result<(), candid_codec::Error> {
            // 4097 null elements consume no payload bytes.
            vec![(); MAX_GLOBALS + 1].write(out)
        }
    }
    assert!(matches!(
        decode(&encode(&GlobalsOnly)),
        Err(TransferMetadataError::Decode(DecodeError::Limit))
    ));
}

#[test]
fn unknown_zero_byte_field_and_missing_required_field_refuse() {
    struct Extra;
    ic_platform::candid_record!(Extra {});
    // The empty record cannot default required metadata fields.
    assert!(matches!(
        decode(&encode(&Extra)),
        Err(TransferMetadataError::Decode(DecodeError::Type))
    ));
    struct Unknown {
        extra: (),
    }
    ic_platform::candid_record!(Unknown { 0 => extra: () });
    assert!(matches!(
        decode(&encode(&Unknown { extra: () })),
        Err(TransferMetadataError::Decode(DecodeError::Limit))
    ));
}

#[test]
fn missing_optional_fields_are_absent_but_never_capture_defaults() {
    struct RequiredOnly;
    impl Encode for RequiredOnly {
        const TYPE: &'static Type = &Type::Record(&[
            (339281328, <Vec<Option<SnapshotMetadataGlobal>>>::TYPE),
            (783802816, u64::TYPE),
            (864574658, <Vec<u8>>::TYPE),
            (1299009433, u64::TYPE),
            (1710030651, u64::TYPE),
            (1767008372, <Vec<ChunkHash>>::TYPE),
            (3569147778, u64::TYPE),
            (3598099748, u64::TYPE),
        ]);
        fn write(&self, out: &mut Writer) -> Result<(), candid_codec::Error> {
            Vec::<Option<SnapshotMetadataGlobal>>::new().write(out)?;
            1_u64.write(out)?;
            Vec::<u8>::new().write(out)?;
            2_u64.write(out)?;
            3_u64.write(out)?;
            Vec::<ChunkHash>::new().write(out)?;
            4_u64.write(out)?;
            5_u64.write(out)
        }
    }
    let raw = encode(&RequiredOnly);
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
