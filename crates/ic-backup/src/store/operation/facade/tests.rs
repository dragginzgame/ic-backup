//! Source and succession regressions at the operation boundary. Native signatures,
//! files and locks do not qualify management effects or Canic authority.
//! Preserves the source/custody lessons from frozen legacy 4ae087fad8c23de712aa60bf569bde4b2b932651
//! and original source-pass binding fix 9f7b886a824eec67dda8f81792a560436419bb2a.
use super::*;
use crate::store::{
    CaptureSourceError, CaptureSuccessionError, OperationReplayError, StoreError,
    capture::test_fixtures::{
        completion::{complete, limits, manifest, replace_owner, settled},
        source::{extent, maximum},
    },
    tests::root,
};
use std::fs;

#[test]
fn exact_source_holds_capture_lock_and_preserves_inventory_and_bounded_reads() {
    let (mut f, server, mut connection) = settled();
    let receipt = complete(&mut f, &mut connection).unwrap();
    assert_eq!(
        f.store
            .latest_completed_backup(&receipt.target, limits())
            .unwrap(),
        Some(receipt.clone())
    );
    let source_hash = f.p.digest();
    let id = f.p.request().operation_id.clone();
    let mut source = f
        .store
        .open_capture_source(&mut f.op, &receipt, maximum())
        .unwrap();
    assert_eq!(source.summary(), &receipt);
    assert_eq!(source.read_data(&extent(&receipt)).unwrap(), b"hello");
    assert!(matches!(
        f.store.open_capture(&id, source_hash),
        Err(OperationReplayError::Capture(OperationStoreError::Locked))
    ));
    drop(source);
    assert_eq!(
        f.op.verify_completion(receipt.manifest_sha256).unwrap(),
        receipt
    );
    assert_eq!(server.count(), 0);
}

#[test]
fn source_rejects_wrong_receipt_store_and_original_payload_limit() {
    let (mut f, _, mut connection) = settled();
    let receipt = complete(&mut f, &mut connection).unwrap();
    let mut wrong = receipt.clone();
    wrong.completed_at_ns += 1;
    assert!(matches!(
        f.store.open_capture_source(&mut f.op, &wrong, maximum()),
        Err(CaptureSourceError::Selection)
    ));
    assert!(matches!(
        f.store
            .open_capture_source(&mut f.op, &receipt, 1.try_into().unwrap()),
        Err(CaptureSourceError::Store(StoreError::PayloadLimit { .. }))
    ));
    let other = BackupStore::initialize(&root("capture-source-other")).unwrap();
    assert!(
        other
            .open_capture_source(&mut f.op, &receipt, maximum())
            .is_err()
    );
    let mut source = f
        .store
        .open_capture_source(&mut f.op, &receipt, maximum())
        .unwrap();
    let file = f
        .root
        .join("bundles")
        .join(&receipt.bundle_id)
        .join("payload/module.wasm");
    fs::write(&file, b"helloworle").unwrap();
    assert!(source.read_data(&extent(&receipt)).is_err());
    assert!(source.read_data(&extent(&receipt)).is_err());
    assert_eq!(fs::read(file).unwrap(), b"helloworle");
}

#[test]
fn authenticated_succession_preserves_old_receipt_source_and_fresh_allowances() {
    let (mut f, server, mut connection) = settled();
    let receipt = complete(&mut f, &mut connection).unwrap();
    let old_journal = fs::read(f.ledger()).unwrap();
    let old_manifest = fs::read(manifest(&f)).unwrap();
    let mut request = f.p.request().clone();
    request.operation_id = "ab".repeat(16);
    let next = f.replan(request);
    let approval = next.digest();
    let original = f.p.clone();
    drop(f.op);
    let next = f
        .store
        .create_capture_after(
            next,
            approval,
            crate::store::OperationPlan::Capture(&original),
        )
        .unwrap();
    let view = next.inspect().unwrap();
    assert_eq!(view.signing_requests_reserved, 0);
    assert_eq!(view.network_requests_reserved, 0);
    assert_eq!(view.next_call, Some(Step::InitialStatus));
    let mut previous = f
        .store
        .open_capture(&original.request().operation_id, original.digest())
        .unwrap();
    assert_eq!(
        previous.verify_completion(receipt.manifest_sha256).unwrap(),
        receipt
    );
    let mut source = f
        .store
        .open_capture_source(&mut previous, &receipt, maximum())
        .unwrap();
    assert_eq!(source.read_data(&extent(&receipt)).unwrap(), b"hello");
    drop(source);
    assert_eq!(
        fs::read(
            f.root
                .join("operations")
                .join(&original.request().operation_id)
                .join("journal.bin")
        )
        .unwrap(),
        old_journal
    );
    assert_eq!(
        fs::read(
            f.root
                .join("bundles")
                .join(&original.request().operation_id)
                .join("manifest.json")
        )
        .unwrap(),
        old_manifest
    );
    assert_eq!(server.count(), 0);
}

#[test]
fn staged_or_missing_receipts_cannot_outlive_original_target_ownership() {
    for published in [false, true] {
        let (mut f, _, mut connection) = settled();
        if published {
            complete(&mut f, &mut connection).unwrap();
            let file = manifest(&f);
            fs::rename(&file, file.with_file_name("retained-manifest")).unwrap();
        }
        replace_owner(&f);
        let original = fs::read(f.ledger()).unwrap();
        assert!(f.op.inspect().is_err());
        assert!(
            crate::store::capture::test_fixtures::http::runtime()
                .block_on(f.op.advance(&mut connection))
                .is_err()
        );
        assert_eq!(fs::read(f.ledger()).unwrap(), original);
        assert!(!manifest(&f).exists());
    }
}

#[test]
fn held_unfinished_or_wrong_approval_predecessor_cannot_change_a_claim() {
    let (mut f, _, mut connection) = settled();
    complete(&mut f, &mut connection).unwrap();
    let mut request = f.p.request().clone();
    request.operation_id = "ab".repeat(16);
    let next = f.replan(request);
    let owner = f
        .root
        .join("targets")
        .join(f.p.request().target.network.to_string())
        .join(f.p.request().target.canister.to_text())
        .join("owner.json");
    let before = fs::read(&owner).unwrap();
    assert!(matches!(
        f.store.create_capture_after(
            next.clone(),
            next.digest(),
            crate::store::OperationPlan::Capture(&f.p)
        ),
        Err(CaptureSuccessionError::Completion(
            OperationReplayError::Capture(OperationStoreError::Locked)
        ))
    ));
    assert!(matches!(
        f.store.create_capture_after(
            next,
            Sha256Digest::compute(b"wrong"),
            crate::store::OperationPlan::Capture(&f.p)
        ),
        Err(CaptureSuccessionError::Capture(
            OperationStoreError::ApprovalMismatch
        ))
    ));
    assert_eq!(fs::read(&owner).unwrap(), before);
}
