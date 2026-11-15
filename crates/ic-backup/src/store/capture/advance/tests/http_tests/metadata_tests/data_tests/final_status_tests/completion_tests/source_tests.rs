//! Composition tests retain original capture authority and reuse the held payload reader.
use super::*;
use crate::store::{CaptureSourceError, PayloadReadError, StoreError};
use ic_platform::management_canister_types::{ReadCanisterSnapshotDataArgs, SnapshotDataKind};
use std::num::NonZeroU64;

fn extent(receipt: &BackupSummary) -> ReadCanisterSnapshotDataArgs {
    ReadCanisterSnapshotDataArgs {
        canister_id: receipt.target.canister,
        snapshot_id: receipt
            .snapshot_id_hex
            .as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|hex| u8::from_str_radix(std::str::from_utf8(hex).unwrap(), 16).unwrap())
            .collect(),
        kind: SnapshotDataKind::WasmModule { offset: 0, size: 5 },
    }
}
fn maximum() -> NonZeroU64 {
    NonZeroU64::new(10).unwrap()
}

#[test]
fn authenticated_source_retains_original_lock_receipt_and_bounded_bytes() {
    let (mut f, server, mut p) = settled();
    let receipt = complete(&mut f, &mut p).unwrap();
    let ledger_path = f.ledger();
    let manifest_path = manifest(&f);
    let ledger = fs::read(&ledger_path).unwrap();
    let manifest_bytes = fs::read(&manifest_path).unwrap();
    let (owner_path, owner) = replace_owner(&f);
    let mut source = runtime()
        .block_on(
            f.store
                .open_capture_source(&mut f.op, &receipt, maximum(), &f.t, &mut p),
        )
        .unwrap();
    assert_eq!(source.summary(), &receipt);
    assert!(matches!(
        f.store
            .open_capture(&f.p.request().operation_id, f.p.digest()),
        Err(CaptureError::Locked)
    ));
    let mut request = extent(&receipt);
    request.snapshot_id.push(0);
    assert!(matches!(
        source.read_data(&request),
        Err(PayloadReadError::Request)
    ));
    assert_eq!(source.read_data(&extent(&receipt)).unwrap(), b"hello");
    drop(source);
    assert_eq!(fs::read(ledger_path).unwrap(), ledger);
    assert_eq!(fs::read(manifest_path).unwrap(), manifest_bytes);
    assert_eq!(fs::read(owner_path).unwrap(), owner);
    assert_eq!(server.count(), 1);
    drop(f.op);
    assert!(
        f.store
            .open_capture(&f.p.request().operation_id, f.p.digest())
            .is_ok()
    );
}

#[test]
fn wrong_receipt_store_or_payload_budget_never_returns_a_source() {
    let (mut f, server, mut p) = settled();
    let receipt = complete(&mut f, &mut p).unwrap();
    for field in [
        "network",
        "canister",
        "release",
        "module",
        "bundle",
        "timestamp",
    ] {
        let mut selected = receipt.clone();
        match field {
            "network" => selected.target.network = Sha256Digest::compute(b"other network"),
            "canister" => {
                selected.target.canister = ic_platform::Principal::from_text("aaaaa-aa").unwrap()
            }
            "release" => selected.release = Sha256Digest::compute(b"other release"),
            "module" => selected.module_sha256 = Sha256Digest::compute(b"other module"),
            "bundle" => selected.bundle_id = "ef".repeat(16),
            "timestamp" => selected.completed_at_ns += 1,
            _ => unreachable!(),
        }
        assert!(matches!(
            runtime().block_on(f.store.open_capture_source(
                &mut f.op,
                &selected,
                maximum(),
                &f.t,
                &mut p
            )),
            Err(CaptureSourceError::Selection)
        ));
    }
    assert!(matches!(
        runtime().block_on(f.store.open_capture_source(
            &mut f.op,
            &receipt,
            NonZeroU64::new(9).unwrap(),
            &f.t,
            &mut p
        )),
        Err(CaptureSourceError::Store(StoreError::PayloadLimit { .. }))
    ));
    let other = BackupStore::initialize(&root("capture-source-other-store")).unwrap();
    assert!(matches!(
        runtime().block_on(other.open_capture_source(&mut f.op, &receipt, maximum(), &f.t, &mut p)),
        Err(CaptureSourceError::Capture(CaptureError::Changed))
    ));
    assert_eq!(server.count(), 1);
}

#[test]
fn changed_held_bytes_refuse_and_invalidate_without_repair() {
    let (mut f, server, mut p) = settled();
    let receipt = complete(&mut f, &mut p).unwrap();
    let path = manifest(&f).parent().unwrap().join("payload/module.wasm");
    let ledger_path = f.ledger();
    let ledger = fs::read(&ledger_path).unwrap();
    let mut source = runtime()
        .block_on(
            f.store
                .open_capture_source(&mut f.op, &receipt, maximum(), &f.t, &mut p),
        )
        .unwrap();
    fs::write(&path, b"HelloWorld").unwrap();
    assert!(matches!(
        source.read_data(&extent(&receipt)),
        Err(PayloadReadError::Store(_))
    ));
    assert!(matches!(
        source.read_data(&extent(&receipt)),
        Err(PayloadReadError::Inactive)
    ));
    assert_eq!(fs::read(path).unwrap(), b"HelloWorld");
    assert_eq!(fs::read(ledger_path).unwrap(), ledger);
    assert_eq!(server.count(), 1);
}
