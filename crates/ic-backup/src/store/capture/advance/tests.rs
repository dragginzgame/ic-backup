//! Native ordering/recovery tests. This provider does not simulate or qualify the IC.
use super::super::{
    ledger::{CaptureStatusPhase, EventRecord, EvidenceRecord},
    status::tests::{plan, status},
};
use super::*;
use crate::store::{
    BackupStore,
    tests::{root, write_file},
};
use ic_host_artifacts::artifact::Sha256Digest;
use std::{
    borrow::Cow,
    cell::Cell,
    fs,
    future::{Future, pending},
    io::Write,
    path::{Path, PathBuf},
    pin::pin,
    task::{Context, Poll, Waker},
};

fn admission() -> StatusAdmissionRecord {
    StatusAdmissionRecord {
        provider: [1; 32],
        application: [2; 32],
        prepare_deadline_ns: 10,
        submit_deadline_ns: 20,
        prepare_requests: 1,
        submit_requests: 1,
        request_bytes: 4096,
        result_bytes: 4096,
    }
}
fn ready<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    match future
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()))
    {
        Poll::Ready(result) => result,
        Poll::Pending => panic!("unexpected asynchronous wait in native fixture"),
    }
}
fn prepared() -> PreparedStatus {
    prepared_content(call_content())
}
fn call_content() -> EnvelopeContent {
    let p = plan();
    let CaptureExecution::Direct { sender } = p.request().execution else {
        panic!("direct fixture")
    };
    EnvelopeContent::Call {
        nonce: Some(vec![0, 255, 7]),
        ingress_expiry: 21,
        sender: sender.as_slice().try_into().unwrap(),
        canister_id: [].as_slice().try_into().unwrap(),
        method_name: "canister_status".into(),
        arg: arguments(&p).unwrap(),
        sender_info: None,
    }
}
fn prepared_content(content: EnvelopeContent) -> PreparedStatus {
    let request_id = *content.to_request_id();
    let expires_ns = content.ingress_expiry();
    // Correct wire shape, deliberately unauthenticated. No IC effect is simulated.
    let envelope = ic_platform::ic_protocol::encode_envelope(
        &ic_platform::ic_protocol::Envelope {
            content: Cow::Owned(content),
            sender_pubkey: Some(vec![1]),
            sender_sig: Some(vec![2]),
            sender_delegation: None,
        },
        4096,
    )
    .unwrap();
    PreparedStatus {
        request_id,
        expires_ns,
        envelope,
        observation: b"original observation binding".to_vec(),
    }
}
fn evidence() -> StatusEvidence {
    StatusEvidence {
        rejected: false,
        response: candid_codec::encode_one(&status(&plan()), 4096).unwrap(),
        proof: b"native ordering fixture; not a certificate".to_vec(),
    }
}
fn path(root: &Path) -> PathBuf {
    root.join("operations").join(&plan().request().operation_id)
}
fn setup(label: &str) -> (PathBuf, BackupStore, CaptureOperation) {
    let root = root(label);
    let store = BackupStore::initialize(&root).unwrap();
    let operation = store.create_capture(plan(), plan().digest()).unwrap();
    (root, store, operation)
}
fn reopen(store: &BackupStore) -> CaptureOperation {
    store
        .open_capture(&plan().request().operation_id, plan().digest())
        .unwrap()
}
fn events(root: &Path) -> Vec<serde_json::Value> {
    let bytes = fs::read(path(root).join("ledger.bin")).unwrap();
    let mut tail = bytes.as_slice();
    let mut result = Vec::new();
    while !tail.is_empty() {
        let n = u32::from_le_bytes(tail[..4].try_into().unwrap()) as usize;
        result.push(serde_json::from_slice(&tail[4..4 + n]).unwrap());
        tail = &tail[n + 36..];
    }
    result
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Fault {
    None,
    Admission,
    Prepare,
    Submit,
    Authenticate,
    WaitPrepare,
    WaitSubmit,
    RequestSize,
    BadRequest,
    ResultSize,
    Reject,
    BadStatus,
}
struct Provider {
    root: PathBuf,
    fault: Fault,
    admissions: Cell<u32>,
    preparations: u32,
    submissions: u32,
    authentications: Cell<u32>,
}
impl Provider {
    fn new(root: &Path, fault: Fault) -> Self {
        Self {
            root: root.to_owned(),
            fault,
            admissions: Cell::new(0),
            preparations: 0,
            submissions: 0,
            authentications: Cell::new(0),
        }
    }
    fn effects(&self) -> (u32, u32) {
        (self.preparations, self.submissions)
    }
}
impl StatusProvider for Provider {
    type Error = &'static str;
    fn admit(
        &self,
        p: &CapturePlan,
        a: &StatusAdmissionRecord,
        args: &[u8],
        request: Option<&PreparedStatus>,
    ) -> Result<(), Self::Error> {
        self.admissions.set(self.admissions.get() + 1);
        assert_eq!(p.digest(), plan().digest());
        assert_eq!(a, &admission());
        assert_eq!(args, arguments(&plan()).unwrap());
        if let Some(request) = request {
            assert_eq!(
                request.encode(a.request_bytes).unwrap(),
                prepared().encode(a.request_bytes).unwrap()
            );
        }
        if self.fault == Fault::Admission {
            return Err("admission");
        }
        Ok(())
    }
    async fn prepare(
        &mut self,
        _: &CapturePlan,
        _: &StatusAdmissionRecord,
        _: &[u8],
    ) -> Result<PreparedStatus, Self::Error> {
        self.preparations += 1;
        assert_eq!(
            events(&self.root).last().unwrap()["kind"],
            "preparation_reserved"
        );
        if self.fault == Fault::WaitPrepare {
            pending::<()>().await;
        }
        if self.fault == Fault::Prepare {
            return Err("signer outcome lost");
        }
        let mut value = prepared();
        if self.fault == Fault::RequestSize {
            value.envelope = vec![0; 4096];
        }
        if self.fault == Fault::BadRequest {
            value.request_id[0] ^= 1;
        }
        Ok(value)
    }
    async fn submit_once(
        &mut self,
        _: &CapturePlan,
        _: &StatusAdmissionRecord,
        request: &PreparedStatus,
    ) -> Result<StatusEvidence, Self::Error> {
        self.submissions += 1;
        assert_eq!(
            events(&self.root).last().unwrap()["kind"],
            "submission_reserved"
        );
        assert_eq!(
            fs::read(path(&self.root).join("status-request.bin")).unwrap(),
            request.encode(4096).unwrap()
        );
        if self.fault == Fault::WaitSubmit {
            pending::<()>().await;
        }
        if self.fault == Fault::Submit {
            return Err("submit outcome lost");
        }
        let mut result = evidence();
        if self.fault == Fault::ResultSize {
            result.proof = vec![0; 4096];
        }
        if self.fault == Fault::Reject {
            result.rejected = true;
        }
        if self.fault == Fault::BadStatus {
            result.response = b"not candid".to_vec();
        }
        Ok(result)
    }
    fn authenticate(
        &self,
        _: &CapturePlan,
        _: &StatusAdmissionRecord,
        request: &PreparedStatus,
        _: &StatusEvidence,
    ) -> Result<(), Self::Error> {
        self.authentications.set(self.authentications.get() + 1);
        assert_eq!(request.request_id, prepared().request_id);
        if self.fault == Fault::Authenticate {
            return Err("proof");
        }
        Ok(())
    }
}
fn reserve(operation: &mut CaptureOperation) {
    operation
        .commit(
            &EventRecord::PreparationReserved {
                sequence: 1,
                admission: admission(),
                arguments_sha256: *Sha256Digest::compute(&arguments(&plan()).unwrap()).as_bytes(),
            },
            None,
        )
        .unwrap();
}
fn retain_request(operation: &mut CaptureOperation) {
    let bytes = prepared().encode(4096).unwrap();
    operation
        .commit(
            &EventRecord::Prepared {
                sequence: 2,
                request: EvidenceRecord::new(&bytes),
            },
            Some(("status-request.bin", &bytes)),
        )
        .unwrap();
}

#[test]
fn successful_status_retains_exact_evidence_and_cold_replay_has_no_effects() {
    let (root, store, mut op) = setup("status-success");
    let mut p = Provider::new(&root, Fault::None);
    assert_eq!(
        ready(op.advance_status(&admission(), &mut p)).unwrap(),
        StatusProgress::Verified
    );
    assert_eq!(p.effects(), (1, 1));
    assert_eq!(
        fs::read(path(&root).join("status-request.bin")).unwrap(),
        prepared().encode(4096).unwrap()
    );
    assert_eq!(
        fs::read(path(&root).join("status-result.bin")).unwrap(),
        evidence().encode(4096).unwrap()
    );
    let original = fs::read(path(&root).join("ledger.bin")).unwrap();
    assert_eq!(
        events(&root)
            .iter()
            .map(|v| v["sequence"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![0, 1, 2, 3, 4]
    );
    drop(op);
    let mut op = reopen(&store);
    let mut cold = Provider::new(&root, Fault::Admission);
    assert_eq!(
        ready(op.advance_status(&admission(), &mut cold)).unwrap(),
        StatusProgress::Verified
    );
    assert_eq!(cold.effects(), (0, 0));
    assert_eq!(cold.admissions.get(), 0);
    assert_eq!(cold.authentications.get(), 1);
    assert_eq!(fs::read(path(&root).join("ledger.bin")).unwrap(), original);
    let mut changed = plan().request().clone();
    changed.operation_id = "02".repeat(16);
    let changed = CapturePlan::new(changed).unwrap();
    let digest = changed.digest();
    assert!(matches!(
        store.create_capture(changed, digest),
        Err(CaptureError::Occupied)
    ));
}

#[test]
fn provider_failure_and_cancelled_futures_never_recreate_live_permission() {
    for fault in [
        Fault::Prepare,
        Fault::Submit,
        Fault::Authenticate,
        Fault::RequestSize,
        Fault::BadRequest,
        Fault::ResultSize,
        Fault::WaitPrepare,
        Fault::WaitSubmit,
    ] {
        let (root, store, mut op) = setup("status-pending");
        let mut p = Provider::new(&root, fault);
        if matches!(fault, Fault::WaitPrepare | Fault::WaitSubmit) {
            let a = admission();
            let mut future = pin!(op.advance_status(&a, &mut p));
            assert!(
                future
                    .as_mut()
                    .poll(&mut Context::from_waker(Waker::noop()))
                    .is_pending()
            );
        } else {
            let result = ready(op.advance_status(&admission(), &mut p));
            if fault == Fault::BadRequest {
                assert!(matches!(
                    result,
                    Err(StatusAdvanceError::Capture(CaptureError::StatusAdmission))
                ));
                assert_eq!(p.effects(), (1, 0));
                assert_eq!(p.admissions.get(), 1);
            } else {
                assert!(result.is_err());
            }
        }
        let pending = if matches!(
            fault,
            Fault::Prepare | Fault::WaitPrepare | Fault::RequestSize | Fault::BadRequest
        ) {
            StatusProgress::PreparationPending
        } else {
            StatusProgress::SubmissionPending
        };
        let spent = p.effects();
        let original = fs::read(path(&root).join("ledger.bin")).unwrap();
        assert_eq!(
            ready(op.advance_status(&admission(), &mut p)).unwrap(),
            pending
        );
        assert_eq!(p.effects(), spent);
        drop(op);
        let mut op = reopen(&store);
        let mut cold = Provider::new(&root, Fault::None);
        assert_eq!(
            ready(op.advance_status(&admission(), &mut cold)).unwrap(),
            pending
        );
        assert_eq!(cold.effects(), (0, 0));
        assert_eq!(cold.admissions.get(), 0);
        assert_eq!(fs::read(path(&root).join("ledger.bin")).unwrap(), original);
    }
}

#[test]
fn prepared_reopen_uses_original_bytes_and_current_admission_without_signing() {
    let (root, store, mut op) = setup("status-prepared");
    reserve(&mut op);
    retain_request(&mut op);
    drop(op);
    let mut op = reopen(&store);
    let original = fs::read(path(&root).join("ledger.bin")).unwrap();
    let mut refuse = Provider::new(&root, Fault::Admission);
    assert!(matches!(
        ready(op.advance_status(&admission(), &mut refuse)),
        Err(StatusAdvanceError::Provider("admission"))
    ));
    assert_eq!(refuse.effects(), (0, 0));
    assert_eq!(fs::read(path(&root).join("ledger.bin")).unwrap(), original);
    let mut p = Provider::new(&root, Fault::None);
    assert_eq!(
        ready(op.advance_status(&admission(), &mut p)).unwrap(),
        StatusProgress::Verified
    );
    assert_eq!(p.effects(), (0, 1));
}

#[test]
fn bound_and_admission_refusals_precede_any_spending_and_bind_reopen() {
    let (root, store, mut op) = setup("status-admission");
    let original = fs::read(path(&root).join("ledger.bin")).unwrap();
    let mut p = Provider::new(&root, Fault::Admission);
    assert!(ready(op.advance_status(&admission(), &mut p)).is_err());
    for alter in [
        |a: &mut StatusAdmissionRecord| a.prepare_requests = u32::MAX,
        |a: &mut StatusAdmissionRecord| a.submit_requests = 0,
        |a: &mut StatusAdmissionRecord| a.request_bytes = 1024 * 1024,
        |a: &mut StatusAdmissionRecord| a.prepare_deadline_ns = 0,
        |a: &mut StatusAdmissionRecord| a.submit_deadline_ns = 1,
    ] {
        let mut a = admission();
        alter(&mut a);
        assert!(matches!(
            ready(op.advance_status(&a, &mut p)),
            Err(StatusAdvanceError::Capture(CaptureError::StatusAdmission))
        ));
    }
    assert_eq!(p.admissions.get(), 1);
    assert_eq!(p.effects(), (0, 0));
    assert_eq!(fs::read(path(&root).join("ledger.bin")).unwrap(), original);
    reserve(&mut op);
    drop(op);
    let mut op = reopen(&store);
    let mut a = admission();
    a.application[0] ^= 1;
    assert!(matches!(
        ready(op.advance_status(&a, &mut p)),
        Err(StatusAdvanceError::Capture(CaptureError::StatusAdmission))
    ));
    assert_eq!(p.effects(), (0, 0));
}

#[test]
fn rejection_and_failed_status_are_settled_consumed_results_on_replay() {
    for fault in [Fault::Reject, Fault::BadStatus] {
        let (root, store, mut op) = setup("status-refused");
        let mut p = Provider::new(&root, fault);
        let first = ready(op.advance_status(&admission(), &mut p));
        if fault == Fault::Reject {
            assert_eq!(first.unwrap(), StatusProgress::Rejected);
        } else {
            assert!(matches!(first, Err(StatusAdvanceError::Status(_))));
        }
        assert_eq!(op.check().unwrap().phase, CaptureStatusPhase::Settled);
        drop(op);
        let mut op = reopen(&store);
        let mut p = Provider::new(&root, Fault::None);
        let replay = ready(op.advance_status(&admission(), &mut p));
        if fault == Fault::Reject {
            assert_eq!(replay.unwrap(), StatusProgress::Rejected);
        } else {
            assert!(matches!(replay, Err(StatusAdvanceError::Status(_))));
        }
        assert_eq!(p.effects(), (0, 0));
    }
}

#[test]
fn partial_frames_and_uncertain_full_writes_invalidate_live_handle_and_preserve_bytes() {
    for offset in [0, 1, 3, 4, 5, 37, usize::MAX - 1, usize::MAX] {
        let (root, store, mut op) = setup("status-append-failure");
        let event = EventRecord::PreparationReserved {
            sequence: 1,
            admission: admission(),
            arguments_sha256: *Sha256Digest::compute(&arguments(&plan()).unwrap()).as_bytes(),
        };
        let initial = fs::read(path(&root).join("ledger.bin")).unwrap();
        let frame = op.check().unwrap().candidate(&event, &plan()).unwrap();
        let count = if offset == usize::MAX - 1 {
            frame.len() - 1
        } else {
            offset.min(frame.len())
        };
        let result = op.commit_with(&event, None, |file, bytes| {
            file.write_all(&bytes[..count])?;
            if offset == usize::MAX {
                file.sync_all()?;
            }
            Err(std::io::Error::other(
                "injected write/sync/lost-return failure",
            ))
        });
        assert!(matches!(result, Err(CaptureError::Io(_))));
        assert!(matches!(op.inspect(), Err(CaptureError::Unusable)));
        assert!(matches!(op.inspect_status(), Err(CaptureError::Unusable)));
        let expected = [initial.as_slice(), &frame[..count]].concat();
        assert_eq!(fs::read(path(&root).join("ledger.bin")).unwrap(), expected);
        drop(op);
        let reopened = store.open_capture(&plan().request().operation_id, plan().digest());
        if count == 0 {
            assert_eq!(
                reopened.unwrap().check().unwrap().phase,
                CaptureStatusPhase::Created
            );
        } else if count == frame.len() {
            let mut op = reopened.unwrap();
            let mut p = Provider::new(&root, Fault::None);
            assert_eq!(
                ready(op.advance_status(&admission(), &mut p)).unwrap(),
                StatusProgress::PreparationPending
            );
            assert_eq!(p.effects(), (0, 0));
        } else {
            assert!(matches!(
                reopened,
                Err(CaptureError::Ledger(ledger::LedgerError::Truncated))
            ));
        }
        assert_eq!(fs::read(path(&root).join("ledger.bin")).unwrap(), expected);
    }
}

#[test]
fn orphan_evidence_and_missing_or_altered_referenced_evidence_block_all_effects() {
    for case in 0..4 {
        let (root, store, mut op) = setup("status-evidence-custody");
        reserve(&mut op);
        let file = path(&root).join("status-request.bin");
        if case == 0 {
            write_file(&file, &prepared().encode(4096).unwrap());
        } else {
            retain_request(&mut op);
            if case == 1 {
                fs::rename(&file, root.join("retained-request")).unwrap();
            }
            if case == 2 {
                fs::rename(&file, root.join("original-request")).unwrap();
                write_file(&file, b"changed");
            }
            if case == 3 {
                fs::hard_link(&file, root.join("extra-link")).unwrap();
            }
        }
        let mut p = Provider::new(&root, Fault::None);
        assert!(ready(op.advance_status(&admission(), &mut p)).is_err());
        assert_eq!(p.effects(), (0, 0));
        assert!(op.inspect_status().is_err());
        drop(op);
        assert!(
            store
                .open_capture(&plan().request().operation_id, plan().digest())
                .is_err()
        );
    }
}

#[test]
fn held_root_rename_keeps_append_in_original_store() {
    let (root, store, mut op) = setup("status-root-rename");
    let moved = root.with_extension("retained");
    fs::rename(&root, &moved).unwrap();
    fs::DirBuilder::new().create(&root).unwrap();
    let mut p = Provider::new(&moved, Fault::None);
    assert_eq!(
        ready(op.advance_status(&admission(), &mut p)).unwrap(),
        StatusProgress::Verified
    );
    assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
    drop(op);
    assert_eq!(
        reopen(&store).check().unwrap().phase,
        CaptureStatusPhase::Settled
    );
}

#[test]
fn process_child() {
    let Some(base) = std::env::var_os("IC_BACKUP_STATUS_CHILD") else {
        return;
    };
    let base = PathBuf::from(base);
    let root = base.join("store");
    let store = BackupStore::open(&root).unwrap();
    let mut op = store.create_capture(plan(), plan().digest()).unwrap();
    let boundary = std::env::var("IC_BACKUP_STATUS_BOUNDARY").unwrap();
    match boundary.as_str() {
        "reserved_prepare" => reserve(&mut op),
        "prepared" => {
            reserve(&mut op);
            retain_request(&mut op);
        }
        "reserved_submit" => {
            reserve(&mut op);
            retain_request(&mut op);
            op.commit(&EventRecord::SubmissionReserved { sequence: 3 }, None)
                .unwrap();
        }
        "invoked_prepare" | "invoked_submit" => {
            let fault = if boundary == "invoked_prepare" {
                Fault::WaitPrepare
            } else {
                Fault::WaitSubmit
            };
            let mut p = Provider::new(&root, fault);
            let a = admission();
            let mut future = pin!(op.advance_status(&a, &mut p));
            assert!(
                future
                    .as_mut()
                    .poll(&mut Context::from_waker(Waker::noop()))
                    .is_pending()
            );
            acknowledge_and_wait(&base, &boundary);
        }
        "settled" => {
            let mut p = Provider::new(&root, Fault::None);
            ready(op.advance_status(&admission(), &mut p)).unwrap();
        }
        _ => panic!("unknown boundary"),
    }
    acknowledge_and_wait(&base, &boundary);
}

fn acknowledge_and_wait(base: &Path, boundary: &str) -> ! {
    write_file(&base.join("ack"), boundary.as_bytes());
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

#[test]
fn acknowledged_owner_death_at_each_boundary_preserves_spending_and_independent_targets() {
    use std::{
        os::unix::fs::DirBuilderExt,
        process::{Child, Command},
        thread,
        time::{Duration, Instant},
    };
    struct Guard(Child);
    impl Drop for Guard {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    for boundary in [
        "reserved_prepare",
        "invoked_prepare",
        "prepared",
        "reserved_submit",
        "invoked_submit",
        "settled",
    ] {
        let base = root("status-process-death");
        let root = base.join("store");
        fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
        let store = BackupStore::initialize(&root).unwrap();
        let mut child = Guard(
            Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "store::capture::advance::tests::process_child",
                    "--nocapture",
                ])
                .env("IC_BACKUP_STATUS_CHILD", &base)
                .env("IC_BACKUP_STATUS_BOUNDARY", boundary)
                .spawn()
                .unwrap(),
        );
        let deadline = Instant::now() + Duration::from_secs(10);
        while fs::read(base.join("ack")).ok().as_deref() != Some(boundary.as_bytes()) {
            assert!(
                child.0.try_wait().unwrap().is_none(),
                "child exited before acknowledged boundary"
            );
            assert!(
                Instant::now() < deadline,
                "child did not reach acknowledged boundary"
            );
            thread::sleep(Duration::from_millis(10));
        }
        assert!(matches!(
            store.open_capture(&plan().request().operation_id, plan().digest()),
            Err(CaptureError::Locked)
        ));
        let mut independent = plan().request().clone();
        independent.operation_id = "02".repeat(16);
        independent.target.canister = ic_platform::Principal::from_slice(&[7, 1]);
        let independent = CapturePlan::new(independent).unwrap();
        let digest = independent.digest();
        let other = store.create_capture(independent, digest).unwrap();
        other.inspect().unwrap();
        let before = fs::read(path(&root).join("ledger.bin")).unwrap();
        child.0.kill().unwrap();
        child.0.wait().unwrap();
        let mut op = reopen(&store);
        let mut p = Provider::new(&root, Fault::None);
        let result = ready(op.advance_status(&admission(), &mut p)).unwrap();
        let expected = match boundary {
            "reserved_prepare" | "invoked_prepare" => StatusProgress::PreparationPending,
            "reserved_submit" | "invoked_submit" => StatusProgress::SubmissionPending,
            _ => StatusProgress::Verified,
        };
        assert_eq!(result, expected);
        assert_eq!(
            p.effects(),
            if boundary == "prepared" {
                (0, 1)
            } else {
                (0, 0)
            }
        );
        if boundary != "prepared" {
            assert_eq!(fs::read(path(&root).join("ledger.bin")).unwrap(), before);
        }
    }
}

#[test]
fn failed_request_and_settlement_frames_preserve_orphans_instead_of_adopting_them() {
    for result_file in [false, true] {
        for full in [false, true] {
            let (root, store, mut op) = setup("status-evidence-append");
            reserve(&mut op);
            let (name, bytes, event) = if result_file {
                retain_request(&mut op);
                op.commit(&EventRecord::SubmissionReserved { sequence: 3 }, None)
                    .unwrap();
                let bytes = evidence().encode(4096).unwrap();
                let event = EventRecord::Settled {
                    sequence: 4,
                    result: EvidenceRecord::new(&bytes),
                };
                ("status-result.bin", bytes, event)
            } else {
                let bytes = prepared().encode(4096).unwrap();
                let event = EventRecord::Prepared {
                    sequence: 2,
                    request: EvidenceRecord::new(&bytes),
                };
                ("status-request.bin", bytes, event)
            };
            let before = fs::read(path(&root).join("ledger.bin")).unwrap();
            let frame = op.check().unwrap().candidate(&event, &plan()).unwrap();
            assert!(
                op.commit_with(&event, Some((name, &bytes)), |file, frame| {
                    if full {
                        file.write_all(frame)?;
                    } // Full frame but sync reports uncertainty.
                    Err(std::io::Error::other("injected append/sync failure"))
                })
                .is_err()
            );
            assert!(matches!(op.inspect(), Err(CaptureError::Unusable)));
            assert!(matches!(op.inspect_status(), Err(CaptureError::Unusable)));
            assert_eq!(fs::read(path(&root).join(name)).unwrap(), bytes);
            drop(op);
            if full {
                let mut op = reopen(&store);
                let mut p = Provider::new(&root, Fault::None);
                assert_eq!(
                    ready(op.advance_status(&admission(), &mut p)).unwrap(),
                    StatusProgress::Verified
                );
                assert_eq!(p.effects(), if result_file { (0, 0) } else { (0, 1) });
                assert!(
                    fs::read(path(&root).join("ledger.bin"))
                        .unwrap()
                        .starts_with(&[before.as_slice(), &frame].concat())
                );
            } else {
                assert!(
                    store
                        .open_capture(&plan().request().operation_id, plan().digest())
                        .is_err()
                );
                assert_eq!(fs::read(path(&root).join("ledger.bin")).unwrap(), before);
            }
        }
    }
}

#[test]
fn closing_custody_failure_never_returns_a_live_reservation() {
    let (root, store, mut op) = setup("status-closing-drift");
    let event = EventRecord::PreparationReserved {
        sequence: 1,
        admission: admission(),
        arguments_sha256: *Sha256Digest::compute(&arguments(&plan()).unwrap()).as_bytes(),
    };
    let ledger_path = path(&root).join("ledger.bin");
    let saved = root.join("original-ledger");
    assert!(matches!(
        op.commit_with(&event, None, |file, frame| {
            file.write_all(frame)?;
            file.sync_all()?;
            fs::rename(&ledger_path, &saved)?;
            write_file(&ledger_path, &fs::read(&saved)?);
            Ok(())
        }),
        Err(CaptureError::Changed)
    ));
    assert!(matches!(op.inspect(), Err(CaptureError::Unusable)));
    assert!(matches!(op.inspect_status(), Err(CaptureError::Unusable)));
    assert_eq!(fs::read(&ledger_path).unwrap(), fs::read(&saved).unwrap());
    drop(op);
    let mut op = reopen(&store);
    let mut p = Provider::new(&root, Fault::None);
    assert_eq!(
        ready(op.advance_status(&admission(), &mut p)).unwrap(),
        StatusProgress::PreparationPending
    );
    assert_eq!(p.effects(), (0, 0));
}

#[test]
fn canic_is_refused_and_exact_evidence_capacity_is_admitted() {
    let a = admission();
    let original = plan();
    let mut r = original.request().clone();
    r.execution = CaptureExecution::Canic {
        executor: ic_platform::Principal::from_slice(&[1, 1]),
        protocol: Sha256Digest::compute(b"protocol"),
        generation: 1,
    };
    assert!(matches!(
        a.validate(&CapturePlan::new(r).unwrap()),
        Err(CaptureError::StatusAdmission)
    ));
    let required = u64::from(a.request_bytes)
        + u64::from(a.result_bytes)
        + ledger::initial(&original).unwrap().len() as u64
        + 4 * (16 * 1024 + 36);
    for (capacity, accepted) in [(required - 1, false), (required, true)] {
        let mut r = original.request().clone();
        r.limits.evidence_bytes = capacity;
        assert_eq!(a.validate(&CapturePlan::new(r).unwrap()).is_ok(), accepted);
    }
}

fn retained_files(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut files: Vec<_> = fs::read_dir(path(root))
        .unwrap()
        .map(|entry| {
            let path = entry.unwrap().path();
            let bytes = fs::read(&path).unwrap();
            (path, bytes)
        })
        .collect();
    files.sort_by(|first, second| first.0.cmp(&second.0));
    files
}

#[test]
fn inspection_reconstructs_original_admission_and_spending_at_every_cold_stage() {
    let (root, store, mut op) = setup("status-inspection-stages");
    for (phase, signing, network, has_request) in [
        (CaptureStatusPhase::Created, 0, 0, false),
        (CaptureStatusPhase::PreparationReserved, 1, 1, false),
        (CaptureStatusPhase::Prepared, 1, 1, true),
        (CaptureStatusPhase::SubmissionReserved, 1, 2, true),
        (CaptureStatusPhase::Settled, 1, 2, true),
    ] {
        match phase {
            CaptureStatusPhase::Created => {}
            CaptureStatusPhase::PreparationReserved => reserve(&mut op),
            CaptureStatusPhase::Prepared => retain_request(&mut op),
            CaptureStatusPhase::SubmissionReserved => op
                .commit(&EventRecord::SubmissionReserved { sequence: 3 }, None)
                .unwrap(),
            CaptureStatusPhase::Settled => {
                let bytes = evidence().encode(4096).unwrap();
                op.commit(
                    &EventRecord::Settled {
                        sequence: 4,
                        result: EvidenceRecord::new(&bytes),
                    },
                    Some(("status-result.bin", &bytes)),
                )
                .unwrap();
            }
        }
        let before = retained_files(&root);
        let expected = CaptureStatusView {
            phase,
            admission: if signing == 0 {
                None
            } else {
                Some(admission())
            },
            signing_requests_reserved: signing,
            network_requests_reserved: network,
            request_id: has_request.then_some(prepared().request_id),
        };
        assert_eq!(op.inspect_status().unwrap(), expected);
        drop(op);
        op = reopen(&store);
        assert_eq!(op.inspect_status().unwrap(), expected);
        assert_eq!(retained_files(&root), before);
        if let Some(mut detached) = op.inspect_status().unwrap().admission {
            detached.provider[0] ^= 1;
            detached.submit_requests += 1;
            assert_ne!(op.inspect_status().unwrap().admission.unwrap(), detached);
        }
    }
}

#[test]
fn recovered_admission_resumes_only_original_prepared_bytes() {
    let (root, store, mut op) = setup("status-inspection-resume");
    reserve(&mut op);
    retain_request(&mut op);
    drop(op);
    let mut op = reopen(&store);
    let original_admission = op.inspect_status().unwrap().admission.unwrap();
    let mut provider = Provider::new(&root, Fault::None);
    assert_eq!(
        ready(op.advance_status(&original_admission, &mut provider)).unwrap(),
        StatusProgress::Verified
    );
    assert_eq!(provider.effects(), (0, 1));
    let view = op.inspect_status().unwrap();
    assert_eq!(view.phase, CaptureStatusPhase::Settled);
    assert_eq!(
        (
            view.signing_requests_reserved,
            view.network_requests_reserved
        ),
        (1, 2)
    );
}

#[test]
fn settled_inspection_does_not_claim_authentication_or_success_or_refund() {
    for fault in [Fault::Reject, Fault::BadStatus] {
        let (root, store, mut op) = setup("status-inspection-refusal");
        let mut provider = Provider::new(&root, fault);
        let result = ready(op.advance_status(&admission(), &mut provider));
        if fault == Fault::Reject {
            assert_eq!(result.unwrap(), StatusProgress::Rejected);
        } else {
            assert!(matches!(result, Err(StatusAdvanceError::Status(_))));
        }
        drop(op);
        let mut op = reopen(&store);
        let before = retained_files(&root);
        let view = op.inspect_status().unwrap();
        assert_eq!(view.phase, CaptureStatusPhase::Settled);
        assert_eq!(
            (
                view.signing_requests_reserved,
                view.network_requests_reserved
            ),
            (1, 2)
        );
        let mut unavailable = Provider::new(&root, Fault::Authenticate);
        assert!(matches!(
            ready(op.advance_status(&view.admission.unwrap(), &mut unavailable)),
            Err(StatusAdvanceError::Provider("proof"))
        ));
        assert_eq!(unavailable.effects(), (0, 0));
        assert_eq!(
            op.inspect_status().unwrap().phase,
            CaptureStatusPhase::Settled
        );
        assert_eq!(retained_files(&root), before);
    }
}

#[test]
fn inspection_refuses_malformed_request_framing_even_with_matching_file_hash() {
    let (root, store, mut op) = setup("status-inspection-framing");
    reserve(&mut op);
    let mut bytes = prepared().encode(4096).unwrap();
    bytes[40..44].copy_from_slice(&u32::MAX.to_le_bytes());
    op.commit(
        &EventRecord::Prepared {
            sequence: 2,
            request: EvidenceRecord::new(&bytes),
        },
        Some(("status-request.bin", &bytes)),
    )
    .unwrap();
    let before = retained_files(&root);
    assert!(matches!(op.inspect_status(), Err(CaptureError::Changed)));
    drop(op);
    assert!(matches!(
        reopen(&store).inspect_status(),
        Err(CaptureError::Changed)
    ));
    assert_eq!(retained_files(&root), before);
}

#[test]
fn call_binding_refuses_wrong_sender_receiver_method_arguments_and_identity() {
    let original = plan();
    for field in 0..7 {
        let mut content = call_content();
        let EnvelopeContent::Call {
            sender,
            canister_id,
            method_name,
            arg,
            ..
        } = &mut content
        else {
            unreachable!()
        };
        match field {
            0 => *sender = original.request().initiator.as_slice().try_into().unwrap(),
            1 => {
                *canister_id = original
                    .request()
                    .target
                    .canister
                    .as_slice()
                    .try_into()
                    .unwrap()
            }
            2 => *method_name = "stop_canister".into(),
            3 => {
                *arg = candid_codec::encode_one(
                    &CanisterIdRecord {
                        canister_id: original.request().initiator,
                    },
                    256,
                )
                .unwrap()
            }
            4 => arg.push(0),
            _ => {}
        }
        // Even a correctly recomputed request ID cannot authorize a different call.
        let mut request = prepared_content(content);
        if field == 5 {
            request.request_id[0] ^= 1;
        }
        if field == 6 {
            request.expires_ns += 1;
        }
        assert!(
            matches!(
                request.validate_call(&original),
                Err(CaptureError::StatusAdmission)
            ),
            "field {field}"
        );
    }
    let request = prepared();
    let mut changed = original.request().clone();
    changed.execution = CaptureExecution::Canic {
        executor: original.request().target.canister,
        protocol: Sha256Digest::compute(b"executor"),
        generation: 1,
    };
    assert!(matches!(
        request.validate_call(&CapturePlan::new(changed).unwrap()),
        Err(CaptureError::StatusAdmission)
    ));
}

#[test]
fn call_binding_refuses_query_read_state_unknown_duplicate_and_trailing_content() {
    let EnvelopeContent::Call {
        sender,
        canister_id,
        method_name,
        arg,
        nonce,
        sender_info,
        ingress_expiry,
    } = call_content()
    else {
        unreachable!()
    };
    let query = prepared_content(EnvelopeContent::Query {
        sender,
        canister_id,
        method_name,
        arg,
        nonce,
        sender_info,
        ingress_expiry,
    });
    let read_state = prepared_content(EnvelopeContent::ReadState {
        sender,
        ingress_expiry,
        paths: vec![],
    });
    let mut malformed = vec![query, read_state];
    let mut trailing = prepared();
    trailing.envelope.push(0);
    malformed.push(trailing);
    for field in [
        b"\x65extra\x00".as_slice(),
        b"\x6bmethod_name\x6fcanister_status",
    ] {
        let mut request = prepared();
        // Insert one field into this test envelope's seven-field content map.
        let marker = b"\x67content\xa7";
        let offset = request
            .envelope
            .windows(marker.len())
            .position(|part| part == marker)
            .unwrap()
            + marker.len();
        request.envelope[offset - 1] = 0xa8;
        request
            .envelope
            .splice(offset..offset, field.iter().copied());
        malformed.push(request);
    }
    for request in malformed {
        assert!(matches!(
            request.validate_call(&plan()),
            Err(CaptureError::StatusAdmission)
        ));
    }
    let original = prepared();
    for end in 0..original.envelope.len() {
        let mut prefix = prepared();
        prefix.envelope.truncate(end);
        assert!(matches!(
            prefix.validate_call(&plan()),
            Err(CaptureError::StatusAdmission)
        ));
    }
}

#[test]
fn call_binding_preserves_exact_bytes_and_does_not_claim_authentication_or_freshness() {
    let request = prepared();
    let original = request.encode(4096).unwrap();
    // Independently recomputed using Node SHA-256 and the IC representation-independent
    // map hash; this is an unsigned test vector, not authentication evidence.
    assert_eq!(
        Sha256Digest::from_bytes(request.request_id).to_string(),
        "9c937904f6a13378c9d9c96339a2ef6b0c987aa4f92f95bbe98f0e1a044532a2"
    );
    assert_eq!(
        Sha256Digest::compute(&request.envelope).to_string(),
        "cfce1de81495739f6c13d0705b3b66e3e3601da870104b8f0c2cc6f37440482a"
    );
    // Expiry 21 is historical and the signature is intentionally invalid.
    // These are provider checks, independent of immutable content binding.
    request.validate_call(&plan()).unwrap();
    assert_eq!(request.encode(4096).unwrap(), original);
    let mut content = call_content();
    let EnvelopeContent::Call { nonce, .. } = &mut content else {
        unreachable!()
    };
    *nonce = None;
    let no_nonce = prepared_content(content);
    assert_ne!(no_nonce.request_id, request.request_id);
    no_nonce.validate_call(&plan()).unwrap();
}

#[test]
fn mismatched_retained_request_refuses_dispatch_and_settled_replay_without_repair() {
    for settled in [false, true] {
        let (root, store, mut op) = setup("status-request-binding");
        reserve(&mut op);
        let mut request = prepared();
        request.request_id[0] ^= 1;
        let bytes = request.encode(4096).unwrap();
        // Correct file/ledger hashes do not establish a matching ingress identity.
        op.commit(
            &EventRecord::Prepared {
                sequence: 2,
                request: EvidenceRecord::new(&bytes),
            },
            Some(("status-request.bin", &bytes)),
        )
        .unwrap();
        if settled {
            op.commit(&EventRecord::SubmissionReserved { sequence: 3 }, None)
                .unwrap();
            let bytes = evidence().encode(4096).unwrap();
            op.commit(
                &EventRecord::Settled {
                    sequence: 4,
                    result: EvidenceRecord::new(&bytes),
                },
                Some(("status-result.bin", &bytes)),
            )
            .unwrap();
        }
        let before = retained_files(&root);
        drop(op);
        let mut op = reopen(&store);
        let mut provider = Provider::new(&root, Fault::None);
        assert!(matches!(
            op.inspect_status(),
            Err(CaptureError::StatusAdmission)
        ));
        assert!(matches!(
            ready(op.advance_status(&admission(), &mut provider)),
            Err(StatusAdvanceError::Capture(CaptureError::StatusAdmission))
        ));
        assert_eq!(provider.effects(), (0, 0));
        assert_eq!(provider.admissions.get(), 0);
        assert_eq!(provider.authentications.get(), 0);
        assert_eq!(retained_files(&root), before);
        assert_eq!(
            op.check().unwrap().phase,
            if settled {
                CaptureStatusPhase::Settled
            } else {
                CaptureStatusPhase::Prepared
            }
        );
    }
}

mod certificate_tests;
mod http_tests;
mod request_tests;

mod snapshot_tests;

mod observation_tests;

mod lookup_tests;

mod metadata_tests;

mod data_tests;

mod assembly_tests;
mod completion_tests;
mod data_acquisition_tests;
mod durable_lookup_tests;
mod final_status_tests;
pub(in crate::store) mod fixtures;
mod http_direct_tests;
mod http_lookup_tests;
mod metadata_acquisition_tests;
mod schedule_tests;
mod source_tests;
mod succession_tests;
