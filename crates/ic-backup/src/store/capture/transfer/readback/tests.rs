//! Source/readback regressions derived from the frozen metadata/upload contracts.
use super::*;
use crate::store::capture::transfer::tests::{chunk, metadata, plan};
use ic_platform::{
    candid_codec::{self, Nat, Reserved, decoder::Error as DecodeError},
    management_canister_types::ChunkHash,
};

fn encode(value: &ReadCanisterSnapshotMetadataResult) -> Vec<u8> {
    candid_codec::encode_one(value, 1024 * 1024).unwrap()
}
fn uploaded(source: &ReadCanisterSnapshotMetadataResult) -> ReadCanisterSnapshotMetadataResult {
    let mut value = source.clone();
    value.source = Some(SnapshotSource::MetadataUpload(Reserved));
    value.taken_at_timestamp = 0;
    value.canister_version = u64::MAX;
    value
}
fn check(
    source: &ReadCanisterSnapshotMetadataResult,
    destination: &ReadCanisterSnapshotMetadataResult,
) -> Result<(), ReadbackMetadataError> {
    plan(4)
        .plan_transfer(&[1], source)
        .unwrap()
        .decode_uploaded_metadata(&encode(destination))
        .map(|_| ())
}

#[test]
fn fresh_provenance_and_chunk_order_are_preserved_without_mutating_the_source() {
    let p = plan(4);
    let mut source = metadata(3, 4, 5);
    source.wasm_chunk_store = vec![chunk(b""), chunk(b"chunk")];
    source.globals = vec![
        Some(SnapshotMetadataGlobal::I32(i32::MIN)),
        Some(SnapshotMetadataGlobal::I64(i64::MAX)),
        Some(SnapshotMetadataGlobal::F32(f32::from_bits(0x7fc0_0042))),
        Some(SnapshotMetadataGlobal::F64(-0.0)),
        Some(SnapshotMetadataGlobal::V128(Nat::from(u128::MAX))),
    ];
    source.certified_data = vec![7; 32];
    source.global_timer = Some(CanisterTimer::Active(u64::MAX));
    source.on_low_wasm_memory_hook_status = Some(OnLowWasmMemoryHookStatus::Executed);
    let before = encode(&source);
    let transfer = p.plan_transfer(&[0, 255, 17], &source).unwrap();
    let mut destination = uploaded(&source);
    destination.wasm_chunk_store.reverse();
    let raw = encode(&destination);
    let admitted = transfer.decode_uploaded_metadata(&raw).unwrap();
    assert_eq!(encode(&admitted), raw);
    assert_eq!(encode(transfer.metadata()), before);
    assert_eq!(admitted.taken_at_timestamp, 0);
    assert_eq!(admitted.canister_version, u64::MAX);
    assert!(
        matches!(admitted.globals[2], Some(SnapshotMetadataGlobal::F32(value))
        if value.to_bits() == 0x7fc0_0042)
    );
    assert!(
        matches!(admitted.globals[3], Some(SnapshotMetadataGlobal::F64(value))
        if value.to_bits() == (-0.0_f64).to_bits())
    );
}

#[test]
fn float_comparison_keeps_nan_payload_signaling_sign_and_signed_zero_bits() {
    let mut source = metadata(1, 0, 0);
    for bits in [
        0,
        0x8000_0000,
        0x7f80_0000,
        0xff80_0000,
        0x7fc0_0042,
        0x7f80_0001,
        0xffc0_0042,
    ] {
        source.globals = vec![Some(SnapshotMetadataGlobal::F32(f32::from_bits(bits)))];
        let mut destination = uploaded(&source);
        assert_eq!(check(&source, &destination), Ok(()));
        destination.globals[0] = Some(SnapshotMetadataGlobal::F32(f32::from_bits(bits ^ 1)));
        assert_eq!(
            check(&source, &destination),
            Err(ReadbackMetadataError::Globals)
        );
    }
    for bits in [
        0,
        0x8000_0000_0000_0000,
        0x7ff0_0000_0000_0000,
        0xfff0_0000_0000_0000,
        0x7ff8_0000_0000_0042,
        0x7ff0_0000_0000_0001,
        0xfff8_0000_0000_0042,
    ] {
        source.globals = vec![Some(SnapshotMetadataGlobal::F64(f64::from_bits(bits)))];
        let mut destination = uploaded(&source);
        assert_eq!(check(&source, &destination), Ok(()));
        destination.globals[0] = Some(SnapshotMetadataGlobal::F64(f64::from_bits(bits ^ 1)));
        assert_eq!(
            check(&source, &destination),
            Err(ReadbackMetadataError::Globals)
        );
    }
    source.globals = vec![
        Some(SnapshotMetadataGlobal::F32(0.0)),
        Some(SnapshotMetadataGlobal::F64(0.0)),
    ];
    for index in 0..2 {
        let mut destination = uploaded(&source);
        destination.globals[index] = Some(if index == 0 {
            SnapshotMetadataGlobal::F32(-0.0)
        } else {
            SnapshotMetadataGlobal::F64(-0.0)
        });
        assert_eq!(
            check(&source, &destination),
            Err(ReadbackMetadataError::Globals)
        );
    }
}

#[test]
fn global_variant_order_length_absence_and_integer_changes_refuse() {
    let mut source = metadata(1, 0, 0);
    source.globals = vec![
        Some(SnapshotMetadataGlobal::I32(7)),
        Some(SnapshotMetadataGlobal::I64(7)),
        Some(SnapshotMetadataGlobal::V128(Nat::from(u128::MAX))),
    ];
    for (index, replacement) in [
        (0, None),
        (0, Some(SnapshotMetadataGlobal::I64(7))),
        (0, Some(SnapshotMetadataGlobal::I32(8))),
        (1, Some(SnapshotMetadataGlobal::I64(8))),
        (
            2,
            Some(SnapshotMetadataGlobal::V128(Nat::from(u128::MAX - 1))),
        ),
        (
            2,
            Some(SnapshotMetadataGlobal::V128(Nat(
                Nat::from(u128::MAX).0 + 1_u8
            ))),
        ),
    ] {
        let mut destination = uploaded(&source);
        destination.globals[index] = replacement;
        assert_eq!(
            check(&source, &destination),
            Err(ReadbackMetadataError::Globals)
        );
    }
    let mut destination = uploaded(&source);
    destination.globals.swap(0, 1);
    assert_eq!(
        check(&source, &destination),
        Err(ReadbackMetadataError::Globals)
    );
    destination = uploaded(&source);
    destination.globals.pop();
    assert_eq!(
        check(&source, &destination),
        Err(ReadbackMetadataError::Globals)
    );
    destination = uploaded(&source);
    destination
        .globals
        .push(Some(SnapshotMetadataGlobal::I32(7)));
    assert_eq!(
        check(&source, &destination),
        Err(ReadbackMetadataError::Globals)
    );
}

#[test]
fn source_origin_each_region_and_certified_bytes_are_checked() {
    let mut source = metadata(1, 2, 3);
    source.certified_data = vec![0, 255, 17];
    for origin in [None, Some(SnapshotSource::TakenFromCanister(Reserved))] {
        let mut destination = uploaded(&source);
        destination.source = origin;
        assert_eq!(
            check(&source, &destination),
            Err(ReadbackMetadataError::Origin)
        );
    }
    for region in 0..3 {
        let mut destination = uploaded(&source);
        match region {
            0 => destination.wasm_module_size = 0,
            1 => destination.wasm_memory_size = u64::MAX,
            _ => destination.stable_memory_size = 4,
        }
        assert_eq!(
            check(&source, &destination),
            Err(ReadbackMetadataError::RegionSizes)
        );
    }
    for bytes in [vec![], vec![0, 255], vec![0, 255, 18]] {
        let mut destination = uploaded(&source);
        destination.certified_data = bytes;
        assert_eq!(
            check(&source, &destination),
            Err(ReadbackMetadataError::CertifiedData)
        );
    }
}

#[test]
fn timer_and_hook_require_exact_presence_variant_and_value() {
    let timers = [
        None,
        Some(CanisterTimer::Inactive),
        Some(CanisterTimer::Active(0)),
        Some(CanisterTimer::Active(u64::MAX)),
    ];
    for (source_index, timer) in timers.iter().enumerate().skip(1) {
        let mut source = metadata(1, 0, 0);
        source.global_timer = timer.clone();
        for (destination_index, timer) in timers.iter().enumerate() {
            let mut destination = uploaded(&source);
            destination.global_timer = timer.clone();
            assert_eq!(
                check(&source, &destination),
                if source_index == destination_index {
                    Ok(())
                } else {
                    Err(ReadbackMetadataError::GlobalTimer)
                }
            );
        }
    }
    let hooks = [
        None,
        Some(OnLowWasmMemoryHookStatus::ConditionNotSatisfied),
        Some(OnLowWasmMemoryHookStatus::Ready),
        Some(OnLowWasmMemoryHookStatus::Executed),
    ];
    for (source_index, hook) in hooks.iter().enumerate().skip(1) {
        let mut source = metadata(1, 0, 0);
        source.on_low_wasm_memory_hook_status = hook.clone();
        for (destination_index, hook) in hooks.iter().enumerate() {
            let mut destination = uploaded(&source);
            destination.on_low_wasm_memory_hook_status = hook.clone();
            assert_eq!(
                check(&source, &destination),
                if source_index == destination_index {
                    Ok(())
                } else {
                    Err(ReadbackMetadataError::LowMemoryHook)
                }
            );
        }
    }
}

#[test]
fn complete_chunk_set_includes_empty_chunks_and_rejects_duplicates_and_bad_ids() {
    let mut source = metadata(1, 0, 0);
    source.wasm_chunk_store = vec![chunk(b""), chunk(b"x")];
    for chunks in [
        vec![],
        vec![chunk(b"x")],
        vec![chunk(b"x"), chunk(b"x")],
        vec![chunk(b""), chunk(b"different")],
        vec![chunk(b""), chunk(b"x"), chunk(b"y")],
        vec![chunk(b""), ChunkHash { hash: vec![0; 31] }],
        vec![chunk(b""), ChunkHash { hash: vec![] }],
    ] {
        let mut destination = uploaded(&source);
        destination.wasm_chunk_store = chunks;
        assert_eq!(
            check(&source, &destination),
            Err(ReadbackMetadataError::ChunkStore)
        );
    }
    source.wasm_chunk_store.clear();
    assert_eq!(check(&source, &uploaded(&source)), Ok(()));
}

#[test]
fn exact_collection_caps_pass_and_excess_counts_refuse_through_shared_decoder() {
    let mut source = metadata(1, 0, 0);
    source.certified_data = vec![0; 32];
    source.globals = vec![Some(SnapshotMetadataGlobal::V128(Nat::from(u128::MAX))); 4096];
    source.wasm_chunk_store = (0..1024_u32)
        .map(|i| {
            let mut hash = vec![0; 32];
            hash[..4].copy_from_slice(&i.to_le_bytes());
            ChunkHash { hash }
        })
        .collect();
    assert_eq!(check(&source, &uploaded(&source)), Ok(()));
    for field in 0..3 {
        let mut destination = uploaded(&source);
        match field {
            0 => destination.globals.push(None),
            1 => destination.wasm_chunk_store.push(chunk(b"extra")),
            _ => destination.certified_data.push(0),
        }
        assert_eq!(
            check(&source, &destination),
            Err(ReadbackMetadataError::Metadata(
                TransferMetadataError::Decode(DecodeError::Limit)
            ))
        );
    }
}

#[test]
fn truncated_wrong_shape_extra_and_oversized_replies_never_compare_as_success() {
    let p = plan(4);
    let source = metadata(1, 0, 0);
    let transfer = p.plan_transfer(&[1], &source).unwrap();
    let destination = uploaded(&source);
    let raw = encode(&destination);
    for length in 0..raw.len() {
        assert!(matches!(
            transfer.decode_uploaded_metadata(&raw[..length]),
            Err(ReadbackMetadataError::Metadata(
                TransferMetadataError::Decode(_)
            ))
        ));
    }
    for invalid in [
        [raw.as_slice(), &[0]].concat(),
        candid_codec::arguments::encode_args(&(destination, 0_u64), 1024 * 1024).unwrap(),
        candid_codec::encode_one(&0_u64, 100).unwrap(),
    ] {
        assert!(matches!(
            transfer.decode_uploaded_metadata(&invalid),
            Err(ReadbackMetadataError::Metadata(
                TransferMetadataError::Decode(_)
            ))
        ));
    }
    assert!(matches!(
        transfer.decode_uploaded_metadata(&vec![0; 1024 * 1024 + 1]),
        Err(ReadbackMetadataError::Metadata(
            TransferMetadataError::ReplyTooLarge
        ))
    ));
}
