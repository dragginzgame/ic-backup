//! Re-authored legacy extent/metadata regressions; no IC effects or wire decoding.
use super::*;
use ic_host_artifacts::artifact::Sha256Digest;
use ic_platform::{
    candid_codec::{Nat, Reserved},
    management_canister_types::{CanisterTimer, ChunkHash, OnLowWasmMemoryHookStatus},
};

pub(super) fn plan(extent: u32) -> CapturePlan {
    let original =
        CapturePlan::decode(include_bytes!("../../tests/fixtures/capture-plan.json").to_vec())
            .unwrap();
    let mut request = original.request().clone();
    request.limits.extent_bytes = extent;
    request.limits.payload_bytes = 1 << 30;
    request.limits.network_requests = 8192;
    request.limits.signing_requests = 8192;
    CapturePlan::new(request).unwrap()
}
pub(super) fn metadata(module: u64, heap: u64, stable: u64) -> ReadCanisterSnapshotMetadataResult {
    ReadCanisterSnapshotMetadataResult {
        source: Some(SnapshotSource::TakenFromCanister(Reserved)),
        taken_at_timestamp: 100,
        wasm_module_size: module,
        wasm_memory_size: heap,
        stable_memory_size: stable,
        globals: vec![],
        wasm_chunk_store: vec![],
        canister_version: 1,
        certified_data: vec![],
        global_timer: Some(CanisterTimer::Inactive),
        on_low_wasm_memory_hook_status: Some(OnLowWasmMemoryHookStatus::ConditionNotSatisfied),
    }
}
pub(super) fn chunk(bytes: &[u8]) -> ChunkHash {
    ChunkHash {
        hash: Sha256Digest::compute(bytes).as_bytes().to_vec(),
    }
}
fn kinds(transfer: &CaptureTransfer<'_>) -> Vec<String> {
    transfer
        .requests()
        .map(|request| match request.kind {
            SnapshotDataKind::WasmModule { offset, size } => format!("module:{offset}:{size}"),
            SnapshotDataKind::WasmMemory { offset, size } => format!("heap:{offset}:{size}"),
            SnapshotDataKind::StableMemory { offset, size } => format!("stable:{offset}:{size}"),
            SnapshotDataKind::WasmChunk { hash } => format!("chunk:{hash:?}"),
        })
        .collect()
}

#[test]
fn exact_region_order_extents_and_original_request_identity() {
    let p = plan(4);
    let m = metadata(9, 0, 5);
    let id = [0, 255, 17];
    let transfer = p.plan_transfer(&id, &m).unwrap();
    assert_eq!(
        kinds(&transfer),
        [
            "module:0:4",
            "module:4:4",
            "module:8:1",
            "stable:0:4",
            "stable:4:1"
        ]
    );
    assert_eq!(
        (
            transfer.data_calls(),
            transfer.capture_calls(),
            transfer.minimum_network_requests()
        ),
        (5, 10, 10)
    );
    assert_eq!(transfer.payload_ceiling(), 14);
    assert_eq!(transfer.requests().len(), 5);
    for request in transfer.requests() {
        assert_eq!(request.canister_id, p.request().target.canister);
        assert_eq!(request.snapshot_id, id);
    }
    for (module, heap, stable, extent) in [(1, 1, 1, 4), (8, 4, 4, 4), (17, 9, 3, 8)] {
        let p = plan(extent);
        let m = metadata(module, heap, stable);
        let transfer = p.plan_transfer(&id, &m).unwrap();
        let mut covered = [0; 3];
        for request in transfer.requests() {
            let (region, offset, size) = match request.kind {
                SnapshotDataKind::WasmModule { offset, size } => (0, offset, size),
                SnapshotDataKind::WasmMemory { offset, size } => (1, offset, size),
                SnapshotDataKind::StableMemory { offset, size } => (2, offset, size),
                SnapshotDataKind::WasmChunk { .. } => unreachable!(),
            };
            assert_eq!(offset, covered[region]);
            assert!((1..=u64::from(extent)).contains(&size));
            covered[region] += size;
        }
        assert_eq!(covered, [module, heap, stable]);
    }
}

#[test]
fn chunk_order_is_canonical_without_rewriting_metadata_and_empty_chunk_counts() {
    let p = plan(4);
    let mut m = metadata(1, 0, 0);
    m.wasm_chunk_store = vec![chunk(b""), chunk(b"x")];
    let original: Vec<_> = m.wasm_chunk_store.iter().map(|c| c.hash.clone()).collect();
    let transfer = p.plan_transfer(&[1], &m).unwrap();
    let mut expected = original.clone();
    expected.sort();
    let chunks: Vec<_> = transfer
        .requests()
        .skip(1)
        .map(|r| match r.kind {
            SnapshotDataKind::WasmChunk { hash } => hash,
            _ => unreachable!(),
        })
        .collect();
    assert_eq!(chunks, expected);
    assert_eq!(transfer.data_calls(), 3);
    assert_eq!(transfer.payload_ceiling(), 9);
    assert_eq!(
        transfer
            .metadata()
            .wasm_chunk_store
            .iter()
            .map(|c| c.hash.clone())
            .collect::<Vec<_>>(),
        original
    );
    let first = kinds(&transfer);
    m.wasm_chunk_store.reverse();
    assert_eq!(kinds(&p.plan_transfer(&[1], &m).unwrap()), first);
}

#[test]
fn conservative_chunk_payload_budget_is_not_reduced_for_known_empty_hash() {
    let mut r = plan(4).request().clone();
    r.limits.payload_bytes = 4;
    let p = CapturePlan::new(r).unwrap();
    let mut m = metadata(1, 0, 0);
    m.wasm_chunk_store.push(chunk(b""));
    assert!(matches!(
        p.plan_transfer(&[1], &m),
        Err(TransferError::PayloadBudget {
            required: 5,
            original: 4
        })
    ));
    let mut r = p.request().clone();
    r.limits.payload_bytes = 5;
    let p = CapturePlan::new(r).unwrap();
    assert_eq!(p.plan_transfer(&[1], &m).unwrap().payload_ceiling(), 5);
}

#[test]
fn checked_size_overflow_and_large_fragmented_input_refuse_before_expansion() {
    let p = plan(1);
    assert!(matches!(
        p.plan_transfer(&[1], &metadata(u64::MAX, 1, 0)),
        Err(TransferError::Overflow)
    ));
    assert!(matches!(
        p.plan_transfer(&[1], &metadata(u64::MAX, 0, 0)),
        Err(TransferError::PayloadBudget { .. })
    ));
    assert!(matches!(
        p.plan_transfer(&[1], &metadata(1 << 30, 0, 0)),
        Err(TransferError::NetworkBudget {
            required: 1_073_741_829,
            original: 8192
        })
    ));
    let p = plan(1 << 20);
    let m = metadata(1 << 30, 0, 0);
    let transfer = p.plan_transfer(&[1], &m).unwrap();
    assert_eq!(transfer.data_calls(), 1024);
    assert!(matches!(transfer.requests().last().unwrap().kind,
        SnapshotDataKind::WasmModule { offset, size }
            if offset == (1 << 30) - (1 << 20) && size == 1 << 20));
}

#[test]
fn exact_network_and_signing_limits_include_all_five_capture_calls() {
    let p = plan(1);
    let m = metadata(8187, 0, 0);
    assert_eq!(p.plan_transfer(&[1], &m).unwrap().capture_calls(), 8192);
    assert!(matches!(
        p.plan_transfer(&[1], &metadata(8188, 0, 0)),
        Err(TransferError::NetworkBudget {
            required: 8193,
            original: 8192
        })
    ));
    let mut r = p.request().clone();
    r.limits.signing_requests = 8191;
    assert!(matches!(
        CapturePlan::new(r).unwrap().plan_transfer(&[1], &m),
        Err(TransferError::SigningBudget {
            required: 8192,
            original: 8191
        })
    ));
}

#[test]
fn canic_counts_both_hops_without_claiming_receipt_or_authentication_costs() {
    let mut r = plan(1).request().clone();
    r.execution = CaptureExecution::Canic {
        executor: r.target.canister,
        protocol: Sha256Digest::compute(b"protocol"),
        generation: 7,
    };
    let p = CapturePlan::new(r).unwrap();
    let m = metadata(4091, 0, 0);
    let transfer = p.plan_transfer(&[1], &m).unwrap();
    assert_eq!(transfer.capture_calls(), 4096);
    assert_eq!(transfer.minimum_network_requests(), 8192);
    assert!(matches!(
        p.plan_transfer(&[1], &metadata(4092, 0, 0)),
        Err(TransferError::NetworkBudget {
            required: 8194,
            original: 8192
        })
    ));
}

#[test]
fn malformed_duplicate_and_excessive_chunks_refuse() {
    let p = plan(1);
    let mut m = metadata(1, 0, 0);
    for size in [0, 31, 33] {
        m.wasm_chunk_store = vec![ChunkHash {
            hash: vec![0; size],
        }];
        assert!(matches!(
            p.plan_transfer(&[1], &m),
            Err(TransferError::ChunkHash)
        ));
    }
    m.wasm_chunk_store = vec![chunk(b"a"), chunk(b"b"), chunk(b"a")];
    assert!(matches!(
        p.plan_transfer(&[1], &m),
        Err(TransferError::ChunkHash)
    ));
    m.wasm_chunk_store = (0..1024_u32).map(|i| chunk(&i.to_be_bytes())).collect();
    assert_eq!(p.plan_transfer(&[1], &m).unwrap().data_calls(), 1025);
    m.wasm_chunk_store.push(chunk(b"extra"));
    assert!(matches!(
        p.plan_transfer(&[1], &m),
        Err(TransferError::MetadataBounds)
    ));
}

#[test]
fn missing_round_trip_fields_are_unsupported_and_never_defaulted() {
    let p = plan(4);
    for change in [
        |m: &mut ReadCanisterSnapshotMetadataResult| m.source = None,
        |m: &mut ReadCanisterSnapshotMetadataResult| {
            m.source = Some(SnapshotSource::MetadataUpload(Reserved))
        },
        |m: &mut ReadCanisterSnapshotMetadataResult| m.wasm_module_size = 0,
        |m: &mut ReadCanisterSnapshotMetadataResult| m.global_timer = None,
        |m: &mut ReadCanisterSnapshotMetadataResult| m.on_low_wasm_memory_hook_status = None,
        |m: &mut ReadCanisterSnapshotMetadataResult| m.globals = vec![None],
    ] {
        let mut m = metadata(1, 0, 0);
        change(&mut m);
        assert!(matches!(
            p.plan_transfer(&[1], &m),
            Err(TransferError::UnsupportedProfile)
        ));
    }
}

#[test]
fn metadata_bounds_and_v128_range_are_explicit() {
    let p = plan(4);
    let mut m = metadata(1, 0, 0);
    m.globals = vec![Some(SnapshotMetadataGlobal::I32(0)); 4096];
    m.certified_data = vec![0; 32];
    p.plan_transfer(&[1], &m).unwrap();
    m.globals.push(Some(SnapshotMetadataGlobal::I32(0)));
    assert!(matches!(
        p.plan_transfer(&[1], &m),
        Err(TransferError::MetadataBounds)
    ));
    m.globals.clear();
    m.certified_data.push(0);
    assert!(matches!(
        p.plan_transfer(&[1], &m),
        Err(TransferError::MetadataBounds)
    ));
    m.certified_data.clear();
    m.globals = vec![Some(SnapshotMetadataGlobal::V128(Nat::from(u128::MAX)))];
    p.plan_transfer(&[1], &m).unwrap();
    m.globals = vec![Some(SnapshotMetadataGlobal::V128(Nat(Nat::from(
        u128::MAX,
    )
    .0 + 1_u8)))];
    assert!(matches!(
        p.plan_transfer(&[1], &m),
        Err(TransferError::GlobalRange)
    ));
}

#[test]
fn float_bits_and_source_provenance_remain_borrowed_and_unchanged() {
    let p = plan(4);
    let mut m = metadata(1, 0, 0);
    m.globals = vec![
        Some(SnapshotMetadataGlobal::F32(f32::from_bits(0x7fc0_0042))),
        Some(SnapshotMetadataGlobal::F64(f64::from_bits(
            0x8000_0000_0000_0000,
        ))),
    ];
    m.taken_at_timestamp = u64::MAX;
    m.canister_version = u64::MAX;
    let transfer = p.plan_transfer(&[0, 255], &m).unwrap();
    assert!(std::ptr::eq(transfer.metadata(), &m));
    assert!(
        matches!(transfer.metadata().globals[0], Some(SnapshotMetadataGlobal::F32(v)) if v.to_bits() == 0x7fc0_0042)
    );
    assert!(
        matches!(transfer.metadata().globals[1], Some(SnapshotMetadataGlobal::F64(v)) if v.to_bits() == 0x8000_0000_0000_0000)
    );
    assert_eq!(transfer.metadata().taken_at_timestamp, u64::MAX);
    assert_eq!(transfer.metadata().canister_version, u64::MAX);
    let debug = format!("{transfer:?}");
    assert!(!debug.contains("globals"));
    assert!(!debug.contains("snapshot_id"));
}

#[test]
fn snapshot_identity_bounds_preserve_exact_maximum_bytes() {
    let p = plan(4);
    let m = metadata(1, 0, 0);
    for id in [vec![], vec![0; 257]] {
        assert!(matches!(
            p.plan_transfer(&id, &m),
            Err(TransferError::SnapshotId)
        ));
    }
    let id = vec![255; 256];
    assert_eq!(
        p.plan_transfer(&id, &m)
            .unwrap()
            .requests()
            .next()
            .unwrap()
            .snapshot_id,
        id
    );
}
