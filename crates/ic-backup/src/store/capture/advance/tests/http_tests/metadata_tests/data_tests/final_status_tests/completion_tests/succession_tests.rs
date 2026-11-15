//! Native handoff/retained-crash-image tests; original certificates are synthetic.
use super::*;
use crate::store::CaptureSuccessionError;

fn next(f: &Fixture) -> CapturePlan {
    let mut request = f.p.request().clone();
    request.operation_id = "ab".repeat(16);
    CapturePlan::new(request).unwrap()
}
fn owner_path(f: &Fixture) -> PathBuf {
    f.root
        .join("targets")
        .join(f.p.request().target.network.to_string())
        .join(f.p.request().target.canister.to_text())
        .join("owner.json")
}

#[test]
fn succession_preserves_original_receipt_and_creates_fresh_bounded_intent() {
    let (mut f, server, mut provider) = settled();
    let receipt = complete(&mut f, &mut provider).unwrap();
    let new = next(&f);
    let approval = new.digest();
    let new_id = new.request().operation_id.clone();
    let expected_owner = crate::store::capture::owner(&new).unwrap();
    let original_manifest = manifest(&f);
    let original_ledger = f.ledger();
    let bytes = fs::read(&original_manifest).unwrap();
    let ledger = fs::read(&original_ledger).unwrap();
    let owner = owner_path(&f);
    let lock = owner.with_file_name("lock");
    use std::os::unix::fs::MetadataExt;
    let lock_inode = fs::metadata(&lock).unwrap().ino();
    drop(f.op);
    let op = runtime()
        .block_on(
            f.store
                .create_capture_after(new, approval, &f.p, &f.t, &mut provider),
        )
        .unwrap();
    assert_eq!(op.inspect().unwrap().digest(), approval);
    let state = op.check().unwrap();
    let status = op.inspect_status().unwrap();
    assert_eq!(status.phase, CaptureStatusPhase::Created);
    assert_eq!(status.signing_requests_reserved, 0);
    assert_eq!(status.network_requests_reserved, 0);
    assert!(state.data.is_empty());
    assert!(state.completed_at_ns.is_none());
    assert_eq!(fs::read(&owner).unwrap(), expected_owner);
    assert_eq!(fs::metadata(&lock).unwrap().ino(), lock_inode);
    let mut old = f
        .store
        .open_capture(&f.p.request().operation_id, f.p.digest())
        .unwrap();
    assert_eq!(
        runtime()
            .block_on(old.complete_capture(&f.t, &mut provider))
            .unwrap(),
        receipt
    );
    assert_eq!(fs::read(original_manifest).unwrap(), bytes);
    assert_eq!(fs::read(original_ledger).unwrap(), ledger);
    assert_eq!(fs::read(owner).unwrap(), expected_owner);
    assert_eq!(server.count(), 1);
    drop(op);
    assert_eq!(
        f.store
            .open_capture(&new_id, approval)
            .unwrap()
            .inspect()
            .unwrap()
            .digest(),
        approval
    );
}

#[test]
fn held_predecessor_or_target_refuses_without_changing_owner() {
    for target_lock in [false, true] {
        let (mut f, server, mut provider) = settled();
        complete(&mut f, &mut provider).unwrap();
        let new = next(&f);
        let approval = new.digest();
        let path = owner_path(&f);
        let before = fs::read(&path).unwrap();
        let held = target_lock.then(|| crate::store::capture::lock_target(&f.op.target).unwrap().0);
        let refusal = runtime().block_on(f.store.create_capture_after(
            new,
            approval,
            &f.p,
            &f.t,
            &mut provider,
        ));
        assert!(matches!(
            refusal,
            Err(CaptureSuccessionError::Capture(CaptureError::Locked))
        ));
        assert_eq!(fs::read(path).unwrap(), before);
        assert!(!f.root.join("operations").join("ab".repeat(16)).exists());
        assert_eq!(server.count(), 1);
        drop(held);
    }
}

#[test]
fn incomplete_changed_or_staged_predecessor_never_releases_claim() {
    for image in ["unfinished", "pending", "changed", "orphan"] {
        let (mut f, server, mut provider) = settled();
        if image != "unfinished" {
            complete(&mut f, &mut provider).unwrap();
            let path = manifest(&f);
            match image {
                "pending" => fs::rename(&path, path.with_file_name("manifest.pending")).unwrap(),
                "changed" => fs::write(path, b"changed").unwrap(),
                "orphan" => {
                    fs::write(owner_path(&f).with_file_name("orphan-stage"), b"retained").unwrap()
                }
                _ => unreachable!(),
            }
        }
        let new = next(&f);
        let approval = new.digest();
        let path = owner_path(&f);
        let before = fs::read(&path).unwrap();
        let ledger_path = f.ledger();
        let ledger = fs::read(&ledger_path).unwrap();
        drop(f.op);
        assert!(
            runtime()
                .block_on(
                    f.store
                        .create_capture_after(new, approval, &f.p, &f.t, &mut provider)
                )
                .is_err()
        );
        assert_eq!(fs::read(path).unwrap(), before);
        assert_eq!(fs::read(ledger_path).unwrap(), ledger);
        assert!(!f.root.join("operations").join("ab".repeat(16)).exists());
        assert_eq!(server.count(), 1);
    }
}

#[test]
fn approval_and_target_mismatch_refuse_before_touching_any_claim() {
    let (f, server, mut provider) = settled();
    for wrong_target in [false, true] {
        let mut request = next(&f).request().clone();
        if wrong_target {
            request.target.network = "ff".repeat(32).parse().unwrap();
        }
        let new = CapturePlan::new(request).unwrap();
        let approval = if wrong_target {
            new.digest()
        } else {
            f.p.digest()
        };
        let refusal = runtime().block_on(f.store.create_capture_after(
            new,
            approval,
            &f.p,
            &f.t,
            &mut provider,
        ));
        assert!(matches!(
            refusal,
            Err(CaptureSuccessionError::Capture(
                CaptureError::ApprovalMismatch | CaptureError::InvalidPlan
            ))
        ));
    }
    assert!(!f.root.join("operations").join("ab".repeat(16)).exists());
    assert_eq!(server.count(), 1);
}

#[test]
fn interrupted_successor_keeps_claim_and_old_receipt_without_reset() {
    for partial in [false, true] {
        let (mut f, server, mut provider) = settled();
        let receipt = complete(&mut f, &mut provider).unwrap();
        let new = next(&f);
        let approval = new.digest();
        let id = new.request().operation_id.clone();
        let path = owner_path(&f);
        let expected_owner = crate::store::capture::owner(&new).unwrap();
        drop(f.op);
        let op = runtime()
            .block_on(
                f.store
                    .create_capture_after(new, approval, &f.p, &f.t, &mut provider),
            )
            .unwrap();
        drop(op);
        let journal = f.root.join("operations").join(&id).join("ledger.bin");
        // Retained crash images during initial successor intent persistence.
        if partial {
            fs::write(&journal, [0u8]).unwrap();
        } else {
            fs::remove_file(&journal).unwrap();
        }
        let before = fs::read(&journal).ok();
        assert!(f.store.open_capture(&id, approval).is_err());
        let mut request = f.p.request().clone();
        request.operation_id = "cd".repeat(16);
        let retry = CapturePlan::new(request).unwrap();
        let retry_approval = retry.digest();
        assert!(matches!(
            runtime().block_on(f.store.create_capture_after(
                retry,
                retry_approval,
                &f.p,
                &f.t,
                &mut provider
            )),
            Err(CaptureSuccessionError::Capture(CaptureError::Changed))
        ));
        let mut old = f
            .store
            .open_capture(&f.p.request().operation_id, f.p.digest())
            .unwrap();
        assert_eq!(
            runtime()
                .block_on(old.complete_capture(&f.t, &mut provider))
                .unwrap(),
            receipt
        );
        assert_eq!(fs::read(path).unwrap(), expected_owner);
        assert_eq!(fs::read(journal).ok(), before);
        assert!(!f.root.join("operations").join("cd".repeat(16)).exists());
        assert_eq!(server.count(), 1);
    }
}

#[test]
fn successor_initial_budget_refuses_before_claim_replacement() {
    let (f, server, mut provider) = settled();
    let mut request = next(&f).request().clone();
    request.limits.evidence_bytes = 1;
    let new = CapturePlan::new(request).unwrap();
    let approval = new.digest();
    let path = owner_path(&f);
    let before = fs::read(&path).unwrap();
    // Held predecessor would refuse Locked if budget admission happened too late.
    assert!(matches!(
        runtime().block_on(
            f.store
                .create_capture_after(new, approval, &f.p, &f.t, &mut provider)
        ),
        Err(CaptureSuccessionError::Capture(CaptureError::Ledger(_)))
    ));
    assert_eq!(fs::read(path).unwrap(), before);
    assert!(!f.root.join("operations").join("ab".repeat(16)).exists());
    assert_eq!(server.count(), 1);
}
