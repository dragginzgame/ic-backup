//! Native original-custody regressions; IC effects are not simulated here.
use crate::store::{
    RestorePersistenceError, RestorePlan,
    capture::test_fixtures::{
        completion::{complete, manifest, settled},
        data::Fixture,
        http::{HttpProvider, runtime},
        source::maximum,
    },
    restore::tests::fixtures::declaration,
    *,
};
use ic_host_artifacts::artifact::Sha256Digest;
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

fn plans(f: &mut Fixture, p: &mut HttpProvider, evidence: u64, count: usize) -> Vec<RestorePlan> {
    let receipt = complete(f, p).unwrap();
    let source = runtime()
        .block_on(
            f.store
                .open_capture_source(&mut f.op, &receipt, maximum(), &f.t, p),
        )
        .unwrap();
    let mut request = declaration(&source);
    request.limits.evidence_bytes = evidence;
    (0..count)
        .map(|_| RestorePlan::new(&source, request.clone()).unwrap())
        .collect()
}
fn target_owner(f: &Fixture) -> PathBuf {
    f.root
        .join("targets")
        .join(f.p.request().target.network.to_string())
        .join(f.p.request().target.canister.to_text())
        .join("owner.json")
}

#[test]
fn restore_creation_and_read_only_reopen_retain_original_source_and_locks() {
    let (mut f, server, mut p) = settled();
    let mut plans = plans(&mut f, &mut p, 1024 * 1024, 2);
    let plan = plans.pop().unwrap();
    let approval = plan.digest();
    let expected = plan.encoded().to_vec();
    let source_ledger = f.ledger();
    let source_bytes = fs::read(&source_ledger).unwrap();
    let owner = target_owner(&f);
    let target_lock_inode = fs::metadata(owner.with_file_name("lock")).unwrap().ino();
    let directory = f.root.join("operations").join(&plan.request().operation_id);
    drop(f.op);
    let op = runtime()
        .block_on(f.store.create_restore(&plan, approval, &f.p, &f.t, &mut p))
        .unwrap();
    assert_eq!(op.inspect().unwrap().encoded(), expected);
    assert!(matches!(
        f.store
            .open_capture(&f.p.request().operation_id, f.p.digest()),
        Err(CaptureError::Locked)
    ));
    assert_eq!(fs::read(&source_ledger).unwrap(), source_bytes);
    assert_eq!(
        fs::metadata(owner.with_file_name("lock")).unwrap().ino(),
        target_lock_inode
    );
    for name in ["lock", "plan.json", "ledger.bin"] {
        assert_eq!(
            fs::metadata(directory.join(name))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
    let ledger = fs::read(directory.join("ledger.bin")).unwrap();
    let owned = fs::read(&owner).unwrap();
    drop(op);
    let reopened = runtime()
        .block_on(
            f.store
                .open_restore(&plans.pop().unwrap(), approval, &f.t, &mut p),
        )
        .unwrap();
    assert_eq!(reopened.inspect().unwrap().digest(), approval);
    assert_eq!(fs::read(directory.join("ledger.bin")).unwrap(), ledger);
    assert_eq!(fs::read(directory.join("plan.json")).unwrap(), expected);
    assert_eq!(fs::read(owner).unwrap(), owned);
    assert_eq!(server.count(), 1);
    assert!(!f.root.join("bundles").join("cd".repeat(16)).exists());
}

#[test]
fn restore_admission_refuses_budget_approval_and_held_source_before_owner_change() {
    for fault in ["budget", "approval", "source_lock"] {
        let (mut f, server, mut p) = settled();
        let plan = plans(&mut f, &mut p, if fault == "budget" { 1 } else { 1024 }, 1)
            .pop()
            .unwrap();
        let owner = target_owner(&f);
        let before = fs::read(&owner).unwrap();
        let approval = if fault == "approval" {
            Sha256Digest::compute(b"wrong")
        } else {
            plan.digest()
        };
        let result =
            runtime().block_on(f.store.create_restore(&plan, approval, &f.p, &f.t, &mut p));
        match fault {
            "budget" => assert!(matches!(
                result,
                Err(RestorePersistenceError::Capture(CaptureError::Ledger(
                    crate::store::LedgerError::ByteLimit
                )))
            )),
            "approval" => assert!(matches!(
                result,
                Err(RestorePersistenceError::Capture(
                    CaptureError::ApprovalMismatch
                ))
            )),
            _ => assert!(matches!(
                result,
                Err(RestorePersistenceError::Capture(CaptureError::Locked))
            )),
        }
        assert_eq!(fs::read(owner).unwrap(), before);
        assert!(!f.root.join("operations").join("cd".repeat(16)).exists());
        assert_eq!(server.count(), 1);
    }
}

#[test]
fn restore_reopen_preserves_partial_changed_and_unknown_original_files() {
    for fault in [
        "plan", "partial", "tail", "missing", "owner", "extra", "source",
    ] {
        let (mut f, _, mut p) = settled();
        let mut plans = plans(&mut f, &mut p, 1024, 2);
        let plan = plans.pop().unwrap();
        let approval = plan.digest();
        let owner = target_owner(&f);
        let dir = f.root.join("operations").join(&plan.request().operation_id);
        let source_module = f
            .root
            .join("bundles")
            .join(&f.p.request().operation_id)
            .join("payload/module.wasm");
        drop(f.op);
        let op = runtime()
            .block_on(f.store.create_restore(&plan, approval, &f.p, &f.t, &mut p))
            .unwrap();
        drop(op);
        let damaged = match fault {
            "plan" => {
                let path = dir.join("plan.json");
                fs::write(&path, b"{}").unwrap();
                path
            }
            "partial" => {
                let path = dir.join("ledger.bin");
                let mut b = fs::read(&path).unwrap();
                b.pop();
                fs::write(&path, b).unwrap();
                path
            }
            "tail" => {
                let path = dir.join("ledger.bin");
                let mut b = fs::read(&path).unwrap();
                b.push(0);
                fs::write(&path, b).unwrap();
                path
            }
            "missing" => {
                fs::rename(dir.join("ledger.bin"), dir.join("retained-ledger.bin")).unwrap();
                dir.join("retained-ledger.bin")
            }
            "owner" => {
                fs::write(&owner, b"{}").unwrap();
                owner.clone()
            }
            "extra" => {
                let path = dir.join("unexpected");
                fs::write(&path, b"evidence").unwrap();
                path
            }
            _ => {
                fs::write(&source_module, b"HELLOWORLD").unwrap();
                source_module
            }
        };
        let retained = fs::read(&damaged).unwrap();
        let owner_bytes = fs::read(&owner).unwrap();
        assert!(
            runtime()
                .block_on(
                    f.store
                        .open_restore(&plans.pop().unwrap(), approval, &f.t, &mut p)
                )
                .is_err(),
            "{fault}"
        );
        assert_eq!(fs::read(damaged).unwrap(), retained);
        assert_eq!(fs::read(owner).unwrap(), owner_bytes);
    }
}

#[test]
fn older_source_does_not_release_an_unfinished_current_capture() {
    let (mut f, server, mut p) = settled();
    let plan = plans(&mut f, &mut p, 1024, 1).pop().unwrap();
    let owner = target_owner(&f);
    let mut request = f.p.request().clone();
    request.operation_id = "ab".repeat(16);
    let next = CapturePlan::new(request.clone()).unwrap();
    drop(f.op);
    let next_op = runtime()
        .block_on(f.store.create_capture_after(
            next,
            CapturePlan::new(request.clone()).unwrap().digest(),
            &f.p,
            &f.t,
            &mut p,
        ))
        .unwrap();
    drop(next_op);
    let next = CapturePlan::new(request).unwrap();
    let before = fs::read(&owner).unwrap();
    assert!(
        runtime()
            .block_on(
                f.store
                    .create_restore(&plan, plan.digest(), &next, &f.t, &mut p)
            )
            .is_err()
    );
    assert_eq!(fs::read(owner).unwrap(), before);
    assert!(
        !f.root
            .join("operations")
            .join(&plan.request().operation_id)
            .exists()
    );
    assert_eq!(server.count(), 1);
}

#[test]
fn held_restore_refuses_replaced_lock_plan_and_ledger_identity() {
    for name in ["lock", "plan.json", "ledger.bin"] {
        let (mut f, _, mut p) = settled();
        let plan = plans(&mut f, &mut p, 1024, 1).pop().unwrap();
        let path = f
            .root
            .join("operations")
            .join(&plan.request().operation_id)
            .join(name);
        drop(f.op);
        let op = runtime()
            .block_on(
                f.store
                    .create_restore(&plan, plan.digest(), &f.p, &f.t, &mut p),
            )
            .unwrap();
        let bytes = fs::read(&path).unwrap();
        fs::rename(&path, path.with_extension("retained")).unwrap();
        fs::write(&path, &bytes).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        assert!(matches!(op.inspect(), Err(CaptureError::Changed)));
        assert_eq!(fs::read(path).unwrap(), bytes);
    }
}

// Build a second native capture from the fixture's synthetic call evidence under
// a different original plan. This is filesystem/authentication coverage, not IC
// attribution or proof that two real snapshot-taking calls ran.
fn copy_private_tree(from: &Path, to: &Path) {
    fs::create_dir(to).unwrap();
    fs::set_permissions(to, fs::Permissions::from_mode(0o700)).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let dest = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_private_tree(&entry.path(), &dest);
        } else {
            fs::copy(entry.path(), &dest).unwrap();
            fs::set_permissions(dest, fs::Permissions::from_mode(0o600)).unwrap();
        }
    }
}

#[test]
fn older_source_and_distinct_completed_owner_are_admitted_without_replacing_source() {
    let (mut f, server, mut p) = settled();
    let plan = plans(&mut f, &mut p, 1024, 1).pop().unwrap();
    let old_ledger_path = f.ledger();
    let old_ledger = fs::read(&old_ledger_path).unwrap();
    let source_manifest_path = manifest(&f);
    let source_manifest = fs::read(&source_manifest_path).unwrap();
    let mut request = f.p.request().clone();
    request.operation_id = "ab".repeat(16);
    let next = CapturePlan::new(request.clone()).unwrap();
    let next_digest = next.digest();
    let binding = crate::store::capture::test_fixtures::completion::rebound_transfer_event(
        &mut f, &mut p, &next,
    );
    drop(f.op);
    let next_op = runtime()
        .block_on(
            f.store
                .create_capture_after(next, next_digest, &f.p, &f.t, &mut p),
        )
        .unwrap();
    drop(next_op);
    let next = CapturePlan::new(request).unwrap();
    let next_dir = f.root.join("operations").join(&next.request().operation_id);
    for entry in fs::read_dir(old_ledger_path.parent().unwrap()).unwrap() {
        let entry = entry.unwrap();
        if !matches!(
            entry.file_name().to_str(),
            Some("lock" | "plan.json" | "ledger.bin")
        ) {
            fs::copy(entry.path(), next_dir.join(entry.file_name())).unwrap();
        }
    }
    let mut ledger = fs::read(next_dir.join("ledger.bin")).unwrap();
    let mut rest = old_ledger.as_slice();
    while !rest.is_empty() {
        let len = u32::from_le_bytes(rest[..4].try_into().unwrap()) as usize;
        let body = &rest[4..4 + len];
        let event: serde_json::Value = serde_json::from_slice(body).unwrap();
        if event["kind"] != "created" && event["kind"] != "publish_ready" {
            let body = if event["kind"] == "transfer_bound" {
                binding.clone()
            } else {
                body.to_vec()
            };
            let length = (body.len() as u32).to_le_bytes();
            let previous = &ledger[ledger.len() - 32..];
            let digest = Sha256Digest::compute(&[previous, &length, body.as_slice()].concat());
            ledger.extend_from_slice(&length);
            ledger.extend_from_slice(&body);
            ledger.extend_from_slice(digest.as_bytes());
        }
        rest = &rest[4 + len + 32..];
    }
    fs::write(next_dir.join("ledger.bin"), ledger).unwrap();
    let new_bundle = f.root.join("bundles").join(&next.request().operation_id);
    fs::create_dir(&new_bundle).unwrap();
    fs::set_permissions(&new_bundle, fs::Permissions::from_mode(0o700)).unwrap();
    copy_private_tree(
        &source_manifest_path.parent().unwrap().join("payload"),
        &new_bundle.join("payload"),
    );
    let mut next_op = f
        .store
        .open_capture(&next.request().operation_id, next.digest())
        .unwrap();
    runtime()
        .block_on(next_op.complete_capture(&f.t, &mut p))
        .unwrap();
    drop(next_op);
    let next_original = fs::read(next_dir.join("ledger.bin")).unwrap();
    let op = runtime()
        .block_on(
            f.store
                .create_restore(&plan, plan.digest(), &next, &f.t, &mut p),
        )
        .unwrap();
    assert_eq!(op.inspect().unwrap().source(), plan.source());
    assert_eq!(fs::read(old_ledger_path).unwrap(), old_ledger);
    assert_eq!(fs::read(source_manifest_path).unwrap(), source_manifest);
    assert_eq!(
        fs::read(next_dir.join("ledger.bin")).unwrap(),
        next_original
    );
    assert_eq!(server.count(), 1);
}
