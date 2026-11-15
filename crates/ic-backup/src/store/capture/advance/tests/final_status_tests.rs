//! Native original-journal tests; certificates are synthetic, not IC qualification.
use super::*;
use crate::store::{
    CaptureFinalStatusError, CaptureStatusPhase, StatusProgress,
    capture::{
        advance::tests::fixtures::{
            data::Fixture,
            final_status::{advance, assembled, provider, request, result},
        },
        ledger::{EventRecord, EvidenceRecord},
    },
};
use std::fs;

#[test]
fn final_status_uses_distinct_evidence_and_cold_replay_is_effect_free() {
    let mut f = assembled(20);
    let r = request(&f, false);
    let id = r.request_id;
    let e = result(&f, &r, false);
    let (server, mut p) = provider(&f, r, &e);
    let a = f.a.call.clone();
    let initial = f.op.inspect_status().unwrap();
    assert_eq!(
        f.op.inspect_final_status().unwrap().phase,
        CaptureStatusPhase::Created
    );
    assert_eq!(
        advance(&mut f, &a, &mut p).unwrap(),
        StatusProgress::Verified
    );
    assert_eq!(server.count(), 1);
    let final_view = f.op.inspect_final_status().unwrap();
    assert_eq!(final_view.phase, CaptureStatusPhase::Settled);
    assert_eq!(final_view.request_id, Some(id));
    assert_eq!(final_view.signing_requests_reserved, 1);
    assert_ne!(initial.request_id, final_view.request_id);
    let bytes = fs::read(f.ledger()).unwrap();
    let mut f = f.reopen();
    p.replay = true;
    assert_eq!(
        advance(&mut f, &a, &mut p).unwrap(),
        StatusProgress::Verified
    );
    assert_eq!(fs::read(f.ledger()).unwrap(), bytes);
    assert_eq!(f.op.inspect_status().unwrap(), initial);
    assert_eq!(f.op.inspect_final_status().unwrap(), final_view);
    assert_eq!(server.count(), 1);
}

#[test]
fn old_status_request_and_invalid_final_state_cannot_satisfy_completion() {
    for reuse in [true, false] {
        let mut f = assembled(20);
        let r = request(&f, reuse);
        let e = result(&f, &r, !reuse);
        let (server, mut p) = provider(&f, r, &e);
        let a = f.a.call.clone();
        let refusal = advance(&mut f, &a, &mut p).unwrap_err();
        if reuse {
            assert!(matches!(
                refusal,
                CaptureFinalStatusError::Status(StatusAdvanceError::Capture(
                    CaptureError::StatusAdmission
                ))
            ));
        } else {
            assert!(matches!(
                refusal,
                CaptureFinalStatusError::Status(StatusAdvanceError::Status(_))
            ));
        }
        let expected = if reuse {
            CaptureStatusPhase::PreparationReserved
        } else {
            CaptureStatusPhase::Settled
        };
        assert_eq!(f.op.inspect_final_status().unwrap().phase, expected);
        let mut f = f.reopen();
        p.replay = true;
        if reuse {
            assert_eq!(
                advance(&mut f, &a, &mut p).unwrap(),
                StatusProgress::PreparationPending
            );
        } else {
            assert!(advance(&mut f, &a, &mut p).is_err());
        }
        assert_eq!(server.count(), usize::from(!reuse));
        assert!(
            !f.root
                .join("bundles")
                .join(&f.p.request().operation_id)
                .join("manifest.json")
                .exists()
        );
    }
}

#[test]
fn assembly_original_bindings_and_cumulative_budget_precede_new_signing() {
    for fault in 0..4 {
        let mut f = if fault == 0 {
            Fixture::new(false, false)
        } else {
            assembled(if fault == 3 { 7 } else { 20 })
        };
        let r = request(&f, false);
        let e = result(&f, &r, false);
        let (server, mut p) = provider(&f, r, &e);
        let mut a = f.a.call.clone();
        if fault == 1 {
            a.provider[0] ^= 1;
        }
        if fault == 2 {
            a.application[0] ^= 1;
        }
        if fault == 3 {
            a.prepare_requests = 1;
        }
        let before = fs::read(f.ledger()).unwrap();
        assert!(advance(&mut f, &a, &mut p).is_err());
        assert_eq!(fs::read(f.ledger()).unwrap(), before);
        assert!(p.prepared.is_some());
        assert_eq!(server.count(), 0);
    }
}

#[test]
fn final_status_uncertain_frames_retain_spending_and_exact_prepared_bytes() {
    for stage in 0..4 {
        for partial in [false, true] {
            let mut f = assembled(20);
            let r = request(&f, false);
            let e = result(&f, &r, false);
            let a = f.a.call.clone();
            let rb = r.encode(a.request_bytes).unwrap();
            let eb = e.encode(a.result_bytes).unwrap();
            let events = [
                EventRecord::FinalStatusStarted {
                    admission: a.clone(),
                },
                EventRecord::FinalStatusPrepared {
                    request: EvidenceRecord::new(&rb),
                },
                EventRecord::FinalStatusSubmissionReserved,
                EventRecord::FinalStatusSettled {
                    result: EvidenceRecord::new(&eb),
                },
            ];
            let evidence = [
                None,
                Some(("final-status-request.bin", rb.as_slice())),
                None,
                Some(("final-status-result.bin", eb.as_slice())),
            ];
            for n in 0..stage {
                f.op.commit(&events[n], evidence[n]).unwrap();
            }
            assert!(
                f.op.commit_with(&events[stage], evidence[stage], |file, bytes| {
                    file.write_all(if partial {
                        &bytes[..bytes.len() / 2]
                    } else {
                        bytes
                    })?;
                    file.sync_all()?;
                    Err(std::io::Error::other("uncertain final status"))
                })
                .is_err()
            );
            if partial {
                let before = fs::read(f.ledger()).unwrap();
                let path = f.ledger();
                let Fixture { op, store, p, .. } = f;
                drop(op);
                assert!(
                    store
                        .open_capture(&p.request().operation_id, p.digest())
                        .is_err()
                );
                assert_eq!(fs::read(path).unwrap(), before);
            } else {
                let mut f = f.reopen();
                let (server, mut p) = provider(&f, r, &e);
                p.prepared = None; // Any attempted re-sign panics in the provider.
                p.replay = stage != 1;
                let expected = match stage {
                    0 => StatusProgress::PreparationPending,
                    2 => StatusProgress::SubmissionPending,
                    _ => StatusProgress::Verified,
                };
                assert_eq!(advance(&mut f, &a, &mut p).unwrap(), expected);
                assert_eq!(server.count(), usize::from(stage == 1));
            }
        }
    }
}
