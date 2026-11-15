//! Local settlement invariants; actual certificates/effects have separate qualification.
use super::*;

fn pending_call(call: CaptureCall) -> (PathBuf, BackupStore, CaptureOperation, SnapshotProvider) {
    if call == CaptureCall::Status {
        let (root, store, mut op) = setup("status-local-settlement");
        let mut provider = SnapshotProvider {
            status: Provider::new(&root, Fault::Submit),
            call,
            fault: Fault::None,
            effects: vec![],
            reuse: false,
            stop_prepared: false,
        };
        assert!(ready(op.advance_status(&admission(), &mut provider)).is_err());
        provider.status.fault = Fault::None;
        (root, store, op, provider)
    } else {
        let (root, store, mut op, mut provider) = fixture();
        provider.call = call;
        provider.fault = Fault::Submit;
        assert!(ready(op.advance_snapshot(&authority(), &mut provider)).is_err());
        provider.fault = Fault::None;
        (root, store, op, provider)
    }
}
fn reply(provider: &SnapshotProvider, call: CaptureCall) -> StatusEvidence {
    if call == CaptureCall::Status {
        evidence()
    } else {
        StatusEvidence {
            rejected: false,
            response: provider.reply(call),
            proof: b"native snapshot fixture".to_vec(),
        }
    }
}
const CALLS: [CaptureCall; 3] = [
    CaptureCall::Status,
    CaptureCall::ListSnapshots,
    CaptureCall::TakeSnapshot,
];

#[test]
fn retained_original_results_settle_each_call_without_remote_effects_or_replacement() {
    for call in CALLS {
        let (root, store, op, mut provider) = pending_call(call);
        let original = reply(&provider, call);
        let before = op.inspect_snapshot().unwrap();
        let effects = provider.effects.clone();
        let status_effects = provider.status.effects();
        drop(op);
        let mut op = reopen(&store);
        provider.fault = Fault::Admission;
        provider.status.fault = Fault::Admission;
        op.settle_capture_result(call, &original, &provider)
            .unwrap();
        let after = op.inspect_snapshot().unwrap();
        assert_eq!(
            after.signing_requests_reserved,
            before.signing_requests_reserved
        );
        assert_eq!(
            after.network_requests_reserved,
            before.network_requests_reserved
        );
        assert_eq!(provider.effects, effects);
        assert_eq!(provider.status.effects(), status_effects);
        let bytes = fs::read(path(&root).join("ledger.bin")).unwrap();
        drop(op);
        let mut op = reopen(&store);
        op.settle_capture_result(call, &original, &provider)
            .unwrap();
        let mut changed = reply(&provider, call);
        changed.proof.push(0);
        assert!(matches!(
            op.settle_capture_result(call, &changed, &provider),
            Err(CaptureSettlementError::EvidenceMismatch)
        ));
        assert_eq!(fs::read(path(&root).join("ledger.bin")).unwrap(), bytes);
        assert_eq!(provider.effects, effects);
        // Settled replay still requires original authentication, not only equal file hashes.
        provider.fault = Fault::Authenticate;
        provider.status.fault = Fault::Authenticate;
        assert!(matches!(
            op.settle_capture_result(call, &original, &provider),
            Err(CaptureSettlementError::Provider(_))
        ));
    }
}

#[test]
fn no_result_can_skip_original_submission_or_settle_another_call() {
    let (root, _, mut op) = setup("unsubmitted-settlement");
    let (_, _, _, provider) = fixture();
    for stage in 0..3 {
        for call in CALLS {
            assert!(matches!(
                op.settle_capture_result(call, &evidence(), &provider),
                Err(CaptureSettlementError::NotSubmitted)
            ));
        }
        match stage {
            0 => reserve(&mut op),
            1 => retain_request(&mut op),
            _ => {}
        }
    }
    assert!(!path(&root).join("status-result.bin").exists());
    for call in [CaptureCall::ListSnapshots, CaptureCall::TakeSnapshot] {
        let (root, _, mut op, provider) = pending_call(call);
        let before = fs::read(path(&root).join("ledger.bin")).unwrap();
        for other in CALLS.into_iter().filter(|other| *other != call) {
            assert!(
                op.settle_capture_result(other, &reply(&provider, call), &provider)
                    .is_err()
            );
        }
        assert_eq!(fs::read(path(&root).join("ledger.bin")).unwrap(), before);
    }
}

#[test]
fn bounds_and_authentication_failures_leave_pending_bytes_unchanged() {
    for call in CALLS {
        let (root, _, mut op, mut provider) = pending_call(call);
        let before = fs::read(path(&root).join("ledger.bin")).unwrap();
        let view = op.inspect_snapshot().unwrap();
        let mut oversized = reply(&provider, call);
        oversized.proof = vec![0; admission().result_bytes as usize];
        assert!(matches!(
            op.settle_capture_result(call, &oversized, &provider),
            Err(CaptureSettlementError::Capture(_))
        ));
        provider.status.fault = Fault::Authenticate;
        provider.fault = Fault::Authenticate;
        assert!(matches!(
            op.settle_capture_result(call, &reply(&provider, call), &provider),
            Err(CaptureSettlementError::Provider(_))
        ));
        assert_eq!(op.inspect_snapshot().unwrap(), view);
        assert_eq!(fs::read(path(&root).join("ledger.bin")).unwrap(), before);
    }
}

#[test]
fn rejection_and_invalid_policy_replies_are_retained_without_dependent_effects() {
    for call in CALLS {
        for rejected in [false, true] {
            let (_, _, mut op, mut provider) = pending_call(call);
            if !rejected {
                provider.fault = Fault::BadStatus;
            }
            let mut original = reply(&provider, call);
            original.rejected = rejected;
            if call == CaptureCall::Status && !rejected {
                original.response = b"bad candid".to_vec();
            }
            let effects = provider.effects.clone();
            op.settle_capture_result(call, &original, &provider)
                .unwrap();
            if call == CaptureCall::Status {
                let outcome = ready(op.advance_status(&admission(), &mut provider));
                if rejected {
                    assert_eq!(outcome.unwrap(), StatusProgress::Rejected);
                } else {
                    assert!(outcome.is_err());
                }
            } else {
                let outcome = ready(op.advance_snapshot(&authority(), &mut provider));
                if rejected {
                    assert_eq!(outcome.unwrap(), SnapshotProgress::Rejected(call));
                } else {
                    assert!(outcome.is_err());
                }
            }
            assert_eq!(provider.effects, effects);
        }
    }
}

#[test]
fn partial_or_orphan_evidence_is_preserved_and_never_adopted_as_a_receipt() {
    for orphan in [false, true] {
        let (root, store, mut op, provider) = pending_call(CaptureCall::TakeSnapshot);
        let original = reply(&provider, CaptureCall::TakeSnapshot);
        let changed = if orphan {
            path(&root).join("snapshot-take-result.bin")
        } else {
            path(&root).join("ledger.bin")
        };
        if orphan {
            write_file(&changed, &original.encode(4096).unwrap());
        } else {
            fs::OpenOptions::new()
                .append(true)
                .open(&changed)
                .unwrap()
                .write_all(&[0])
                .unwrap();
        }
        let bytes = fs::read(&changed).unwrap();
        assert!(
            op.settle_capture_result(CaptureCall::TakeSnapshot, &original, &provider)
                .is_err()
        );
        drop(op);
        assert!(
            store
                .open_capture(&plan().request().operation_id, plan().digest())
                .is_err()
        );
        assert_eq!(fs::read(&changed).unwrap(), bytes);
    }
}
