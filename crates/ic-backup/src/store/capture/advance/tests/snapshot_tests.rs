//! Native original-byte/spending tests. Management effects are qualified in PocketIC.
use super::*;
use crate::store::{CaptureSnapshotPhase as Phase, SnapshotReplyError};
use ic_platform::management_canister_types::Snapshot;

fn authority() -> SnapshotAdmissionRecord {
    SnapshotAdmissionRecord {
        capacity: 2,
        list: admission(),
        take: admission(),
    }
}
fn snapshot(id: u8) -> Snapshot {
    Snapshot {
        id: vec![id],
        taken_at_timestamp: 17,
        total_size: 1024,
    }
}
struct SnapshotProvider {
    status: Provider,
    call: CaptureCall,
    fault: Fault,
    effects: Vec<(CaptureCall, &'static str)>,
    reuse: bool,
    stop_prepared: bool,
}
impl StatusProvider for SnapshotProvider {
    type Error = &'static str;
    fn admit(
        &self,
        p: &CapturePlan,
        a: &StatusAdmissionRecord,
        args: &[u8],
        r: Option<&PreparedStatus>,
    ) -> Result<(), Self::Error> {
        self.status.admit(p, a, args, r)
    }
    async fn prepare(
        &mut self,
        p: &CapturePlan,
        a: &StatusAdmissionRecord,
        args: &[u8],
    ) -> Result<PreparedStatus, Self::Error> {
        self.status.prepare(p, a, args).await
    }
    async fn submit_once(
        &mut self,
        p: &CapturePlan,
        a: &StatusAdmissionRecord,
        r: &PreparedStatus,
    ) -> Result<StatusEvidence, Self::Error> {
        self.status.submit_once(p, a, r).await
    }
    fn authenticate(
        &self,
        p: &CapturePlan,
        a: &StatusAdmissionRecord,
        r: &PreparedStatus,
        e: &StatusEvidence,
    ) -> Result<(), Self::Error> {
        self.status.authenticate(p, a, r, e)
    }
}
impl SnapshotProvider {
    fn active(&self, call: CaptureCall, fault: Fault) -> bool {
        self.call == call && self.fault == fault
    }
    fn reply(&self, call: CaptureCall) -> Vec<u8> {
        if self.active(call, Fault::BadStatus) {
            return b"malformed authenticated reply".to_vec();
        }
        match call {
            CaptureCall::ListSnapshots => {
                candid_codec::encode_one(&vec![snapshot(1)], 4096).unwrap()
            }
            CaptureCall::TakeSnapshot => {
                candid_codec::encode_one(&snapshot(if self.reuse { 1 } else { 2 }), 4096).unwrap()
            }
            CaptureCall::Status => panic!("not a snapshot call"),
        }
    }
}
impl CaptureProvider for SnapshotProvider {
    fn admit_call(
        &self,
        _: &CapturePlan,
        a: &StatusAdmissionRecord,
        call: CaptureCall,
        prepared: Option<&PreparedStatus>,
    ) -> Result<(), Self::Error> {
        assert_eq!(a, &admission());
        if self.active(call, Fault::Admission)
            || (self.stop_prepared
                && self.call == call
                && prepared.is_some()
                && path(&self.status.root)
                    .join(snapshots::names(call).unwrap().0)
                    .exists())
        {
            Err("authority lost")
        } else {
            Ok(())
        }
    }
    async fn prepare_call(
        &mut self,
        p: &CapturePlan,
        _: &StatusAdmissionRecord,
        call: CaptureCall,
    ) -> Result<PreparedStatus, Self::Error> {
        let last = events(&self.status.root).pop().unwrap();
        assert_eq!(
            last["kind"],
            if call == CaptureCall::ListSnapshots {
                "snapshot_started"
            } else {
                "snapshot_take_reserved"
            }
        );
        self.effects.push((call, "sign"));
        if self.active(call, Fault::WaitPrepare) {
            pending::<()>().await;
        }
        if self.active(call, Fault::Prepare) {
            return Err("signer lost");
        }
        let mut content = call_content();
        if let EnvelopeContent::Call {
            method_name, arg, ..
        } = &mut content
        {
            *method_name = call.method_name().into();
            *arg = call.arguments(p).unwrap();
        }
        let mut r = prepared_content(content);
        if self.active(call, Fault::BadRequest) {
            r.request_id[0] ^= 1;
        }
        Ok(r)
    }
    async fn submit_call(
        &mut self,
        _: &CapturePlan,
        _: &StatusAdmissionRecord,
        call: CaptureCall,
        r: &PreparedStatus,
    ) -> Result<StatusEvidence, Self::Error> {
        let last = events(&self.status.root).pop().unwrap();
        assert_eq!(last["kind"], "snapshot_submission_reserved");
        assert_eq!(
            fs::read(path(&self.status.root).join(snapshots::names(call).unwrap().0)).unwrap(),
            r.encode(4096).unwrap()
        );
        self.effects.push((call, "submit"));
        if self.active(call, Fault::WaitSubmit) {
            pending::<()>().await;
        }
        if self.active(call, Fault::Submit) {
            return Err("reply lost");
        }
        Ok(StatusEvidence {
            rejected: self.active(call, Fault::Reject),
            response: self.reply(call),
            proof: b"native snapshot fixture".to_vec(),
        })
    }
    fn authenticate_call(
        &self,
        p: &CapturePlan,
        _: &StatusAdmissionRecord,
        call: CaptureCall,
        r: &PreparedStatus,
        e: &StatusEvidence,
    ) -> Result<(), Self::Error> {
        r.validate_call_for(p, call).map_err(|_| "request")?;
        if self.active(call, Fault::Authenticate) {
            return Err("proof refused");
        }
        if e.response != self.reply(call) || e.proof != b"native snapshot fixture" {
            return Err("original evidence changed");
        }
        Ok(())
    }
}
fn fixture() -> (PathBuf, BackupStore, CaptureOperation, SnapshotProvider) {
    let (root, store, mut op) = setup("snapshot-continuation");
    let mut provider = SnapshotProvider {
        status: Provider::new(&root, Fault::None),
        call: CaptureCall::ListSnapshots,
        fault: Fault::None,
        effects: vec![],
        reuse: false,
        stop_prepared: false,
    };
    assert_eq!(
        ready(op.advance_status(&admission(), &mut provider)).unwrap(),
        StatusProgress::Verified
    );
    (root, store, op, provider)
}

#[test]
fn list_then_take_retains_original_bytes_and_cold_settled_replay_is_effect_free() {
    let (root, store, mut op, mut provider) = fixture();
    assert_eq!(
        ready(op.advance_snapshot(&authority(), &mut provider)).unwrap(),
        SnapshotProgress::Captured(snapshot(2))
    );
    let view = op.inspect_snapshot().unwrap();
    assert_eq!(view.phase, Phase::Settled);
    assert_eq!(view.signing_requests_reserved, 3);
    assert_eq!(view.network_requests_reserved, 6);
    let files: Vec<_> = [
        "ledger.bin",
        "snapshot-list-request.bin",
        "snapshot-list-result.bin",
        "snapshot-take-request.bin",
        "snapshot-take-result.bin",
    ]
    .into_iter()
    .map(|name| (name, fs::read(path(&root).join(name)).unwrap()))
    .collect();
    assert_eq!(
        provider.effects,
        [
            (CaptureCall::ListSnapshots, "sign"),
            (CaptureCall::ListSnapshots, "submit"),
            (CaptureCall::TakeSnapshot, "sign"),
            (CaptureCall::TakeSnapshot, "submit")
        ]
    );
    drop(op);
    let mut op = reopen(&store);
    provider.fault = Fault::Admission;
    assert_eq!(
        ready(op.advance_snapshot(&authority(), &mut provider)).unwrap(),
        SnapshotProgress::Captured(snapshot(2))
    );
    assert_eq!(provider.effects.len(), 4);
    assert_eq!(op.inspect_snapshot().unwrap(), view);
    for (name, bytes) in files {
        assert_eq!(fs::read(path(&root).join(name)).unwrap(), bytes);
    }
}

#[test]
fn pending_list_and_take_never_repeat_after_error_or_cancellation() {
    for call in [CaptureCall::ListSnapshots, CaptureCall::TakeSnapshot] {
        for fault in [
            Fault::Prepare,
            Fault::WaitPrepare,
            Fault::Submit,
            Fault::WaitSubmit,
            Fault::Authenticate,
            Fault::BadRequest,
        ] {
            let (_, store, mut op, mut provider) = fixture();
            provider.call = call;
            provider.fault = fault;
            if matches!(fault, Fault::WaitPrepare | Fault::WaitSubmit) {
                let a = authority();
                let mut future = pin!(op.advance_snapshot(&a, &mut provider));
                assert!(
                    future
                        .as_mut()
                        .poll(&mut Context::from_waker(Waker::noop()))
                        .is_pending()
                );
            } else {
                assert!(ready(op.advance_snapshot(&authority(), &mut provider)).is_err());
            }
            let before = provider.effects.clone();
            let view = op.inspect_snapshot().unwrap();
            drop(op);
            let mut op = reopen(&store);
            provider.fault = Fault::None;
            let result = ready(op.advance_snapshot(&authority(), &mut provider)).unwrap();
            let preparing = matches!(
                fault,
                Fault::Prepare | Fault::WaitPrepare | Fault::BadRequest
            );
            assert_eq!(
                result,
                if preparing {
                    SnapshotProgress::PreparationPending(call)
                } else {
                    SnapshotProgress::SubmissionPending(call)
                }
            );
            assert_eq!(provider.effects, before);
            assert_eq!(op.inspect_snapshot().unwrap(), view);
        }
    }
}

#[test]
fn changed_authority_and_cumulative_budgets_refuse_before_list_spending() {
    for case in 0..6 {
        let (root, _, mut op, mut provider) = fixture();
        let before = fs::read(path(&root).join("ledger.bin")).unwrap();
        let mut a = authority();
        match case {
            0 => a.capacity = 0,
            1 => a.take.application[0] ^= 1,
            2 => a.list.provider[0] ^= 1,
            3 => a.list.prepare_requests = 20,
            4 => a.take.result_bytes = 1024 * 1024,
            _ => a.capacity = 1025,
        }
        assert!(ready(op.advance_snapshot(&a, &mut provider)).is_err());
        assert!(provider.effects.is_empty());
        assert_eq!(fs::read(path(&root).join("ledger.bin")).unwrap(), before);
    }
}

#[test]
fn authenticated_full_inventory_or_bad_list_never_reaches_take() {
    for malformed in [false, true] {
        let (_, store, mut op, mut provider) = fixture();
        let mut a = authority();
        if malformed {
            provider.fault = Fault::BadStatus;
        } else {
            a.capacity = 1;
        }
        assert!(matches!(
            ready(op.advance_snapshot(&a, &mut provider)),
            Err(SnapshotAdvanceError::Reply(_))
        ));
        assert_eq!(op.inspect_snapshot().unwrap().phase, Phase::ListSettled);
        assert_eq!(provider.effects.len(), 2);
        drop(op);
        let mut op = reopen(&store);
        assert!(ready(op.advance_snapshot(&a, &mut provider)).is_err());
        assert_eq!(provider.effects.len(), 2);
    }
}

#[test]
fn original_rejections_and_reused_take_ids_stay_consumed() {
    for call in [CaptureCall::ListSnapshots, CaptureCall::TakeSnapshot] {
        let (_, store, mut op, mut provider) = fixture();
        provider.call = call;
        provider.fault = Fault::Reject;
        assert_eq!(
            ready(op.advance_snapshot(&authority(), &mut provider)).unwrap(),
            SnapshotProgress::Rejected(call)
        );
        let before = provider.effects.clone();
        drop(op);
        let mut op = reopen(&store);
        assert_eq!(
            ready(op.advance_snapshot(&authority(), &mut provider)).unwrap(),
            SnapshotProgress::Rejected(call)
        );
        assert_eq!(provider.effects, before);
    }
    let (_, store, mut op, mut provider) = fixture();
    provider.reuse = true;
    assert!(matches!(
        ready(op.advance_snapshot(&authority(), &mut provider)),
        Err(SnapshotAdvanceError::Reply(
            SnapshotReplyError::AlreadyExisted
        ))
    ));
    drop(op);
    let mut op = reopen(&store);
    assert!(matches!(
        ready(op.advance_snapshot(&authority(), &mut provider)),
        Err(SnapshotAdvanceError::Reply(
            SnapshotReplyError::AlreadyExisted
        ))
    ));
    assert_eq!(provider.effects.len(), 4);
}

#[test]
fn changed_or_missing_snapshot_evidence_and_partial_ledger_refuse_without_repair() {
    for case in 0..4 {
        let (root, store, mut op, mut provider) = fixture();
        ready(op.advance_snapshot(&authority(), &mut provider)).unwrap();
        drop(op);
        let file = path(&root).join("snapshot-take-result.bin");
        match case {
            0 => {
                fs::remove_file(&file).unwrap();
            }
            1 => {
                fs::write(&file, b"changed").unwrap();
            }
            2 => {
                std::fs::OpenOptions::new()
                    .append(true)
                    .open(path(&root).join("ledger.bin"))
                    .unwrap()
                    .write_all(&[1])
                    .unwrap();
            }
            _ => {
                write_file(&path(&root).join("unreferenced.bin"), b"retained");
            }
        }
        assert!(
            store
                .open_capture(&plan().request().operation_id, plan().digest())
                .is_err()
        );
        assert_eq!(provider.effects.len(), 4);
    }
}

#[test]
fn snapshot_transitions_cannot_skip_status_list_or_prepared_evidence() {
    let (root, _, mut op) = setup("snapshot-illegal-transition");
    let event = EventRecord::SnapshotStarted {
        sequence: 5,
        admission: authority(),
        arguments_sha256: *Sha256Digest::compute(
            &CaptureCall::ListSnapshots.arguments(&plan()).unwrap(),
        )
        .as_bytes(),
    };
    assert!(op.commit(&event, None).is_err());
    assert_eq!(events(&root).len(), 1);
    let (_, _, mut op, mut provider) = fixture();
    assert!(
        op.commit(
            &EventRecord::SnapshotTakeReserved {
                sequence: 9,
                arguments_sha256: *Sha256Digest::compute(
                    &CaptureCall::TakeSnapshot.arguments(&plan()).unwrap()
                )
                .as_bytes()
            },
            None
        )
        .is_err()
    );
    provider.call = CaptureCall::TakeSnapshot;
    provider.fault = Fault::Admission;
    assert!(ready(op.advance_snapshot(&authority(), &mut provider)).is_err());
    assert_eq!(op.inspect_snapshot().unwrap().phase, Phase::ListSettled);
    assert!(
        op.commit(
            &EventRecord::SnapshotSubmissionReserved {
                sequence: 11,
                call: CaptureCall::TakeSnapshot
            },
            None
        )
        .is_err()
    );
    assert_eq!(provider.effects.len(), 2);
}

#[test]
fn prepared_list_and_take_reopen_with_original_request_without_resigning() {
    for call in [CaptureCall::ListSnapshots, CaptureCall::TakeSnapshot] {
        let (root, store, mut op, mut provider) = fixture();
        provider.call = call;
        provider.stop_prepared = true;
        assert!(ready(op.advance_snapshot(&authority(), &mut provider)).is_err());
        assert_eq!(
            op.inspect_snapshot().unwrap().phase,
            match call {
                CaptureCall::ListSnapshots => Phase::ListPrepared,
                CaptureCall::TakeSnapshot => Phase::TakePrepared,
                _ => unreachable!(),
            }
        );
        let request = path(&root).join(snapshots::names(call).unwrap().0);
        let bytes = fs::read(&request).unwrap();
        drop(op);
        provider.stop_prepared = false;
        let mut op = reopen(&store);
        assert_eq!(
            ready(op.advance_snapshot(&authority(), &mut provider)).unwrap(),
            SnapshotProgress::Captured(snapshot(2))
        );
        assert_eq!(fs::read(&request).unwrap(), bytes);
        assert_eq!(
            provider
                .effects
                .iter()
                .filter(|effect| **effect == (call, "sign"))
                .count(),
            1
        );
        let view = op.inspect_snapshot().unwrap();
        assert_eq!(view.signing_requests_reserved, 3);
        assert_eq!(view.network_requests_reserved, 6);
    }
}
