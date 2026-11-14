//! Re-authored local-custody regressions; provenance is in capture-intent.md.
use super::*;
use std::{
    process::{Child, Command},
    time::{Duration, Instant},
};

const PLAN: &[u8] = include_bytes!("fixtures/capture-plan.json");
const DIGEST: &str = "f19893fe983ec0abb66d5aa33f05dbd69f07bb89f0ff4026fd03b7451cab9d0d";

pub(super) fn request(id: u64) -> CaptureRequest {
    CaptureRequest {
        operation_id: format!("{id:032x}"),
        target: target(),
        trust_root: "22".repeat(32).parse().unwrap(),
        initiator: Principal::from_text("2vxsx-fae").unwrap(),
        release: "33".repeat(32).parse().unwrap(),
        module_sha256: "44".repeat(32).parse().unwrap(),
        execution: CaptureExecution::Direct {
            sender: Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap(),
        },
        limits: CaptureLimitsRecord {
            payload_bytes: 1_048_576,
            extent_bytes: 4096,
            network_requests: 20,
            observations: 5,
            signing_requests: 20,
            evidence_bytes: 1_048_576,
        },
    }
}
fn plan(id: u64) -> CapturePlan {
    CapturePlan::new(request(id)).unwrap()
}
fn operation_path(root: &Path) -> PathBuf {
    root.join("operations/00000000000000000000000000000001")
}
fn owner_path(root: &Path) -> PathBuf {
    root.join("targets")
        .join(target().network.to_string())
        .join(target().canister.to_text())
}
fn create(store: &BackupStore) -> CaptureOperation {
    let p = plan(1);
    let digest = p.digest();
    store.create_capture(p, digest).unwrap()
}
fn reopen(store: &BackupStore) -> Result<CaptureOperation, CaptureError> {
    store.open_capture(&request(1).operation_id, DIGEST.parse().unwrap())
}

#[test]
fn golden_plan_and_first_frame_bind_exact_intent() {
    let p = plan(1);
    assert_eq!(p.encoded(), PLAN);
    assert_eq!(p.digest().to_string(), DIGEST);
    let root = root("capture-golden");
    let store = BackupStore::initialize(&root).unwrap();
    let operation = create(&store);
    assert_eq!(operation.inspect().unwrap().encoded(), PLAN);
    let ledger = fs::read(operation_path(&root).join("ledger.bin")).unwrap();
    let hex: String = ledger.iter().map(|b| format!("{b:02x}")).collect();
    assert_eq!(hex, include_str!("fixtures/capture-created.hex").trim());
    assert_eq!(
        fs::read(operation_path(&root).join("plan.json")).unwrap(),
        PLAN
    );
    let owner: serde_json::Value =
        serde_json::from_slice(&fs::read(owner_path(&root).join("owner.json")).unwrap()).unwrap();
    assert_eq!(owner["plan_sha256"], DIGEST);
    assert!(matches!(reopen(&store), Err(CaptureError::Locked)));
    drop(operation);
    assert_eq!(reopen(&store).unwrap().inspect().unwrap().encoded(), PLAN);
}

#[test]
fn every_plan_binding_changes_approval_and_invalid_limits_refuse() {
    for change in [
        |r: &mut CaptureRequest| r.operation_id = format!("{:032x}", 2),
        |r: &mut CaptureRequest| r.target.network = Sha256Digest::compute(b"other network"),
        |r: &mut CaptureRequest| r.target.canister = Principal::from_text("aaaaa-aa").unwrap(),
        |r: &mut CaptureRequest| r.trust_root = Sha256Digest::compute(b"other root"),
        |r: &mut CaptureRequest| r.initiator = Principal::from_text("aaaaa-aa").unwrap(),
        |r: &mut CaptureRequest| r.release = Sha256Digest::compute(b"other release"),
        |r: &mut CaptureRequest| r.module_sha256 = Sha256Digest::compute(b"other module"),
        |r: &mut CaptureRequest| {
            r.execution = CaptureExecution::Direct {
                sender: Principal::from_text("aaaaa-aa").unwrap(),
            }
        },
        |r: &mut CaptureRequest| r.limits.payload_bytes += 1,
        |r: &mut CaptureRequest| r.limits.extent_bytes += 1,
        |r: &mut CaptureRequest| r.limits.network_requests += 1,
        |r: &mut CaptureRequest| r.limits.observations += 1,
        |r: &mut CaptureRequest| r.limits.signing_requests += 1,
        |r: &mut CaptureRequest| r.limits.evidence_bytes += 1,
    ] {
        let mut r = request(1);
        change(&mut r);
        assert_ne!(CapturePlan::new(r).unwrap().digest(), plan(1).digest());
    }
    for change in [
        |r: &mut CaptureRequest| r.operation_id = "../escape".into(),
        |r: &mut CaptureRequest| r.limits.payload_bytes = 0,
        |r: &mut CaptureRequest| r.limits.payload_bytes = (1 << 30) + 1,
        |r: &mut CaptureRequest| r.limits.extent_bytes = 0,
        |r: &mut CaptureRequest| r.limits.extent_bytes = (1 << 20) + 1,
        |r: &mut CaptureRequest| r.limits.payload_bytes = 1,
        |r: &mut CaptureRequest| r.limits.network_requests = 0,
        |r: &mut CaptureRequest| r.limits.network_requests = 8193,
        |r: &mut CaptureRequest| r.limits.observations = 21,
        |r: &mut CaptureRequest| r.limits.signing_requests = 0,
        |r: &mut CaptureRequest| r.limits.signing_requests = 8193,
        |r: &mut CaptureRequest| r.limits.evidence_bytes = 0,
        |r: &mut CaptureRequest| r.limits.evidence_bytes = (1 << 30) + 1,
    ] {
        let mut r = request(1);
        change(&mut r);
        assert!(matches!(
            CapturePlan::new(r),
            Err(CaptureError::InvalidPlan)
        ));
    }
}

#[test]
fn wrong_approval_writes_nothing_and_does_not_replace_original_plan() {
    let root = root("capture-approval");
    let store = BackupStore::initialize(&root).unwrap();
    assert!(matches!(
        store.create_capture(plan(1), Sha256Digest::compute(b"wrong")),
        Err(CaptureError::ApprovalMismatch)
    ));
    for dir in ["targets", "operations", "bundles"] {
        assert_eq!(fs::read_dir(root.join(dir)).unwrap().count(), 0);
    }
    drop(create(&store));
    let mut changed = request(1);
    changed.limits.network_requests += 1;
    let changed = CapturePlan::new(changed).unwrap();
    assert!(matches!(
        store.open_capture(&request(1).operation_id, changed.digest()),
        Err(CaptureError::ApprovalMismatch)
    ));
    assert_eq!(reopen(&store).unwrap().inspect().unwrap().encoded(), PLAN);
}

#[test]
fn durable_owner_blocks_same_target_but_independent_target_can_progress() {
    let root = root("capture-owners");
    let store = BackupStore::initialize(&root).unwrap();
    let operation = create(&store);
    let other = plan(2);
    let digest = other.digest();
    assert!(matches!(
        store.create_capture(other, digest),
        Err(CaptureError::Occupied)
    ));
    let mut r = request(3);
    r.target.canister = Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap();
    let other = CapturePlan::new(r).unwrap();
    let digest = other.digest();
    let independent = store.create_capture(other, digest).unwrap();
    assert_eq!(independent.inspect().unwrap().digest(), digest);
    drop(operation);
    let other = plan(2);
    let digest = other.digest();
    assert!(matches!(
        store.create_capture(other, digest),
        Err(CaptureError::Occupied)
    ));
    assert!(
        !root
            .join("operations")
            .join(request(2).operation_id)
            .exists()
    );
    assert_eq!(reopen(&store).unwrap().inspect().unwrap().encoded(), PLAN);
}

#[test]
fn missing_truncated_changed_or_extended_ledger_never_resets_authority() {
    for mutation in 0..5 {
        let root = root("capture-ledger-refusal");
        let store = BackupStore::initialize(&root).unwrap();
        drop(create(&store));
        let path = operation_path(&root).join("ledger.bin");
        let original = fs::read(&path).unwrap();
        fs::rename(&path, path.with_extension("original")).unwrap();
        match mutation {
            0 => {}
            1 => write_file(&path, &original[..original.len() - 1]),
            2 => {
                let mut bytes = original.clone();
                bytes[10] ^= 1;
                write_file(&path, &bytes);
            }
            3 => {
                let mut bytes = original.clone();
                bytes.extend_from_slice(b"future event");
                write_file(&path, &bytes);
            }
            _ => write_file(&path, &vec![0; 16 * 1024 + 37]),
        }
        assert!(reopen(&store).is_err());
        assert_eq!(fs::read(path.with_extension("original")).unwrap(), original);
        assert_eq!(path.exists(), mutation != 0);
        let other = plan(2);
        let digest = other.digest();
        assert!(matches!(
            store.create_capture(other, digest),
            Err(CaptureError::Occupied)
        ));
    }
}

#[test]
fn changed_owner_and_missing_target_lock_are_not_repaired() {
    for name in ["owner.json", "lock"] {
        let root = root("capture-target-originals");
        let store = BackupStore::initialize(&root).unwrap();
        let operation = create(&store);
        let path = owner_path(&root).join(name);
        fs::rename(&path, root.with_extension(format!("{name}-original"))).unwrap();
        assert!(matches!(operation.inspect(), Err(CaptureError::Incomplete)));
        drop(operation);
        assert!(matches!(reopen(&store), Err(CaptureError::Incomplete)));
        assert!(!path.exists());
        let other = plan(2);
        let digest = other.digest();
        assert!(store.create_capture(other, digest).is_err());
        assert!(!path.exists(), "missing original must not be recreated");
    }
}

#[test]
fn replaced_locks_and_operation_directory_refuse_held_inspection() {
    for kind in 0..3 {
        let root = root("capture-custody");
        let store = BackupStore::initialize(&root).unwrap();
        let operation = create(&store);
        let path = match kind {
            0 => owner_path(&root).join("lock"),
            1 => operation_path(&root).join("lock"),
            _ => operation_path(&root),
        };
        fs::rename(&path, path.with_extension("original")).unwrap();
        if kind == 2 {
            DirBuilder::new().mode(0o700).create(&path).unwrap();
        } else {
            write_file(&path, b"");
        }
        assert!(matches!(operation.inspect(), Err(CaptureError::Changed)));
    }
}

#[test]
fn malformed_original_plan_is_refused_without_reencoding() {
    let raw = std::str::from_utf8(PLAN).unwrap();
    for bad in [
        raw.replacen('{', "{\"unknown\":1,", 1),
        raw.replacen('{', "{\"format\":\"duplicate\",", 1),
        raw.replace("local-capture-plan/1", "local-capture-plan/2"),
        raw.replace("2vxsx-fae", "2VXSX-FAE"),
        raw.replace(&"22".repeat(32), &"AA".repeat(32)),
        raw.replace("\"network_requests\":20", "\"network_requests\":21"),
    ] {
        let root = root("capture-invalid-plan");
        let store = BackupStore::initialize(&root).unwrap();
        drop(create(&store));
        let path = operation_path(&root).join("plan.json");
        fs::write(&path, bad.as_bytes()).unwrap();
        assert!(reopen(&store).is_err());
        assert_eq!(fs::read(path).unwrap(), bad.as_bytes());
    }
}

#[test]
fn symlinked_and_hardlinked_ledger_refuse_without_following_or_repair() {
    for hard in [false, true] {
        let root = root("capture-unsafe-ledger");
        let store = BackupStore::initialize(&root).unwrap();
        drop(create(&store));
        let path = operation_path(&root).join("ledger.bin");
        let retained = root.with_extension("ledger-original");
        fs::rename(&path, &retained).unwrap();
        if hard {
            fs::hard_link(&retained, &path).unwrap();
        } else {
            symlink(&retained, &path).unwrap();
        }
        assert!(matches!(reopen(&store), Err(CaptureError::Store(_))));
        assert!(retained.exists());
    }
}

#[test]
fn canic_route_declaration_round_trips_with_bound_generation() {
    let mut r = request(1);
    r.execution = CaptureExecution::Canic {
        executor: target().canister,
        protocol: Sha256Digest::compute(b"test-only protocol"),
        generation: 7,
    };
    let p = CapturePlan::new(r.clone()).unwrap();
    let digest = p.digest();
    let root = root("capture-canic-declaration");
    let store = BackupStore::initialize(&root).unwrap();
    drop(store.create_capture(p, digest).unwrap());
    let opened = store.open_capture(&r.operation_id, digest).unwrap();
    assert_eq!(opened.inspect().unwrap().request().execution, r.execution);
    if let CaptureExecution::Canic { generation, .. } = &mut r.execution {
        *generation += 1;
    }
    assert_ne!(CapturePlan::new(r).unwrap().digest(), digest);
}

#[test]
fn competing_target_creation_retains_exactly_one_owner() {
    let root = root("capture-race");
    BackupStore::initialize(&root).unwrap();
    let barrier = Arc::new(Barrier::new(2));
    let threads: Vec<_> = (1..=2)
        .map(|id| {
            let root = root.clone();
            let barrier = barrier.clone();
            thread::spawn(move || {
                let store = BackupStore::open(&root).unwrap();
                let p = plan(id);
                let digest = p.digest();
                barrier.wait();
                store.create_capture(p, digest)
            })
        })
        .collect();
    let results: Vec<_> = threads.into_iter().map(|t| t.join().unwrap()).collect();
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
    let winner = results.iter().find_map(|r| r.as_ref().ok()).unwrap();
    let original = winner.inspect().unwrap();
    let owner: serde_json::Value =
        serde_json::from_slice(&fs::read(owner_path(&root).join("owner.json")).unwrap()).unwrap();
    assert_eq!(owner["operation_id"], original.request().operation_id);
    assert_eq!(owner["plan_sha256"], original.digest().to_string());
}

struct ChildGuard(Child);
impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn process_child() {
    let Some(path) = std::env::var_os("IC_BACKUP_CAPTURE_CHILD") else {
        return;
    };
    let path = PathBuf::from(path);
    let store = BackupStore::open(&path.join("store")).unwrap();
    let operation = create(&store);
    assert_eq!(operation.inspect().unwrap().digest().to_string(), DIGEST);
    write_file(&path.join("ack"), b"ready");
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}

#[test]
fn acknowledged_process_death_keeps_intent_and_target_claim() {
    let root = root("capture-process-death");
    let path = root.join("store");
    DirBuilder::new().mode(0o700).create(&path).unwrap();
    let store = BackupStore::initialize(&path).unwrap();
    let mut child = ChildGuard(
        Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "store::tests::capture::process_child",
                "--nocapture",
            ])
            .env("IC_BACKUP_CAPTURE_CHILD", &root)
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    while fs::read(root.join("ack")).ok().as_deref() != Some(b"ready".as_slice()) {
        assert!(
            child.0.try_wait().unwrap().is_none(),
            "child exited before acknowledgement"
        );
        assert!(
            Instant::now() < deadline,
            "child failed to acknowledge held intent"
        );
        thread::sleep(Duration::from_millis(10));
    }
    assert!(matches!(reopen(&store), Err(CaptureError::Locked)));
    child.0.kill().unwrap();
    child.0.wait().unwrap();
    let original = reopen(&store).unwrap();
    assert_eq!(original.inspect().unwrap().encoded(), PLAN);
    let other = plan(2);
    let digest = other.digest();
    assert!(matches!(
        store.create_capture(other, digest),
        Err(CaptureError::Occupied)
    ));
}

#[test]
fn root_rename_keeps_original_operation_custody() {
    let path = root("capture-root-rename");
    let store = BackupStore::initialize(&path).unwrap();
    let operation = create(&store);
    let moved = path.with_extension("retained");
    fs::rename(&path, &moved).unwrap();
    DirBuilder::new().mode(0o700).create(&path).unwrap();
    let replacement = BackupStore::initialize(&path).unwrap();
    assert_eq!(operation.inspect().unwrap().encoded(), PLAN);
    assert!(matches!(
        reopen(&replacement),
        Err(CaptureError::Incomplete)
    ));
    drop(operation);
    assert_eq!(reopen(&store).unwrap().inspect().unwrap().encoded(), PLAN);
}

#[test]
fn same_byte_ledger_replacement_refuses_held_inspection() {
    let path = root("capture-ledger-replaced");
    let store = BackupStore::initialize(&path).unwrap();
    let operation = create(&store);
    let ledger = operation_path(&path).join("ledger.bin");
    let original = fs::read(&ledger).unwrap();
    let retained = path.with_extension("original-ledger");
    fs::rename(&ledger, &retained).unwrap();
    write_file(&ledger, &original);
    assert!(matches!(operation.inspect(), Err(CaptureError::Changed)));
    assert_eq!(fs::read(retained).unwrap(), original);
}

#[test]
fn initial_ledger_must_fit_original_evidence_budget_before_claiming_target() {
    let path = root("capture-evidence-budget");
    let store = BackupStore::initialize(&path).unwrap();
    let mut r = request(1);
    r.limits.evidence_bytes = 1;
    let p = CapturePlan::new(r).unwrap();
    let digest = p.digest();
    assert!(matches!(
        store.create_capture(p, digest),
        Err(CaptureError::Ledger(LedgerError::ByteLimit))
    ));
    assert_eq!(fs::read_dir(path.join("targets")).unwrap().count(), 0);
    assert_eq!(fs::read_dir(path.join("operations")).unwrap().count(), 0);
    let original = create(&store);
    assert_eq!(original.inspect().unwrap().encoded(), PLAN);
}
