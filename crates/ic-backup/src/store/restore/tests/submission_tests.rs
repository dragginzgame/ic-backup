//! Native HTTP/custody checks with synthetic real-signature proofs, not IC effects.
use super::fixtures::{admission, declaration, prepared};
use crate::store::{
    CaptureTiming, DirectRestoreStatus, DirectRestoreStatusError, PreparedStatus, RestorePlan,
    RestoreStatusSigner, RestoreStatusSubmissionError, StatusAdmissionRecord, StatusHttpTransport,
    StatusProgress,
    capture::test_fixtures::{
        certificates::{der, signed},
        completion::{complete, settled},
        http::{Response, Server, http, now, runtime},
        signatures::{Scheme, change},
        source::{maximum, status},
    },
    status_certificate_observation,
};
use ic_platform::{
    Principal, candid_codec,
    certification::{
        fork,
        historical::{TimePolicy, TimeWindow},
        labeled, leaf,
    },
    ic_protocol::EnvelopeContent,
};
use std::{
    fs,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32, Ordering},
    },
    time::Duration,
};

struct Signer {
    calls: Arc<AtomicU32>,
    allow: Arc<AtomicBool>,
}
impl RestoreStatusSigner for Signer {
    type Error = ();
    fn protocol(&self) -> [u8; 32] {
        [27; 32]
    }
    fn admit(&self, _: &RestorePlan, a: &StatusAdmissionRecord) -> Result<(), ()> {
        if self.allow.load(Ordering::SeqCst) && a.application == [2; 32] {
            Ok(())
        } else {
            Err(())
        }
    }
    async fn sign_status(
        &mut self,
        p: &RestorePlan,
        a: &StatusAdmissionRecord,
        _: &[u8],
    ) -> Result<PreparedStatus, ()> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let mut r = request(p, a);
        let reference = now();
        r.observation = status_certificate_observation(TimePolicy {
            certificate: TimeWindow {
                reference_ns: reference,
                max_age_ns: 60_000_000_000,
                max_future_skew_ns: a.submit_deadline_ns - reference + 1_000_000_000,
            },
            delegation: TimeWindow {
                reference_ns: reference,
                max_age_ns: 120_000_000_000,
                max_future_skew_ns: a.submit_deadline_ns - reference + 1_000_000_000,
            },
        })
        .unwrap();
        Ok(r)
    }
}
fn request(p: &RestorePlan, a: &StatusAdmissionRecord) -> PreparedStatus {
    let mut r = prepared(p, Scheme::Ed25519);
    r.expires_ns = a.submit_deadline_ns;
    change(&mut r, |e| {
        if let EnvelopeContent::Call { ingress_expiry, .. } = e.content.to_mut() {
            *ingress_expiry = a.submit_deadline_ns;
        }
        e.sender_sig = Some(Scheme::Ed25519.sign(&e.content.to_request_id().signable()));
    });
    r
}
fn provider(
    endpoint: &str,
    p: &RestorePlan,
    calls: &Arc<AtomicU32>,
    allow: &Arc<AtomicBool>,
    timeout: u64,
) -> DirectRestoreStatus<Signer> {
    DirectRestoreStatus::new(
        StatusHttpTransport::new(
            endpoint,
            p.request().target.network,
            der(1),
            Duration::from_secs(timeout),
        )
        .unwrap(),
        Signer {
            calls: Arc::clone(calls),
            allow: Arc::clone(allow),
        },
        CaptureTiming {
            certificate_max_age_ns: 60_000_000_000,
            delegation_max_age_ns: 120_000_000_000,
            max_clock_skew_ns: 1_000_000_000,
        },
    )
}
fn certificate(r: &PreparedStatus, response: &[u8], rejected: bool, key: u8) -> Vec<u8> {
    let fields = if rejected {
        fork(
            labeled(b"reject_code", leaf(vec![4])),
            fork(
                labeled(b"reject_message", leaf(b"no".to_vec())),
                labeled(b"status", leaf(b"rejected".to_vec())),
            ),
        )
    } else {
        fork(
            labeled(b"reply", leaf(response.to_vec())),
            labeled(b"status", leaf(b"replied".to_vec())),
        )
    };
    let mut time = now();
    let mut encoded = Vec::new();
    loop {
        let byte = (time & 127) as u8;
        time >>= 7;
        encoded.push(byte | if time == 0 { 0 } else { 128 });
        if time == 0 {
            break;
        }
    }
    signed(
        fork(
            labeled(b"request_status", labeled(r.request_id.to_vec(), fields)),
            labeled(b"time", leaf(encoded)),
        ),
        key,
        None,
    )
}
fn body(proof: &[u8]) -> Vec<u8> {
    let mut body = b"\xa2\x66status\x67replied\x6bcertificate\x59".to_vec();
    body.extend_from_slice(&u16::try_from(proof.len()).unwrap().to_be_bytes());
    body.extend_from_slice(proof);
    body
}

#[test]
fn restore_status_retains_terminal_evidence_before_policy_and_replays_without_effects() {
    for mode in ["success", "policy", "rejected"] {
        let (mut f, capture_server, mut p) = settled();
        let receipt = complete(&mut f, &mut p).unwrap();
        let source = runtime()
            .block_on(
                f.store
                    .open_capture_source(&mut f.op, &receipt, maximum(), &f.t, &mut p),
            )
            .unwrap();
        let mut decl = declaration(&source);
        decl.sender = Principal::self_authenticating(Scheme::Ed25519.key());
        let plan = RestorePlan::new(&source, decl).unwrap();
        drop(source);
        let mut observed = status(&f.p);
        observed.settings.controllers = vec![plan.request().sender];
        let response = if mode == "policy" {
            vec![7]
        } else {
            candid_codec::encode_one(&observed, 4096).unwrap()
        };
        let mut a = StatusAdmissionRecord {
            prepare_deadline_ns: now() + 60_000_000_000,
            submit_deadline_ns: now() + 120_000_000_000,
            prepare_requests: 1,
            ..admission()
        };
        let proof = certificate(&request(&plan, &a), &response, mode == "rejected", 1);
        let mut server = Server::new(Response::Bytes(http(200, "", &body(&proof))));
        let calls = Arc::new(AtomicU32::new(0));
        let allow = Arc::new(AtomicBool::new(true));
        let mut direct = provider(
            &server.endpoint,
            &plan,
            &calls,
            &allow,
            if mode == "timeout" { 1 } else { 5 },
        );
        a.provider = direct.commitment();
        let directory = f.root.join("operations").join(&plan.request().operation_id);
        drop(f.op);
        let mut op = runtime()
            .block_on(
                f.store
                    .create_restore(&plan, plan.digest(), &f.p, &f.t, &mut p),
            )
            .unwrap();
        runtime()
            .block_on(op.prepare_status(&a, &mut direct))
            .unwrap();
        let request_bytes = fs::read(directory.join("status-request.bin")).unwrap();
        let before = fs::read(directory.join("ledger.bin")).unwrap();
        allow.store(false, Ordering::SeqCst);
        assert!(matches!(
            runtime().block_on(op.submit_status(&direct)),
            Err(RestoreStatusSubmissionError::Direct(
                DirectRestoreStatusError::Signer(())
            ))
        ));
        allow.store(true, Ordering::SeqCst);
        let changed = provider(&server.endpoint, &plan, &calls, &allow, 6);
        assert!(matches!(
            runtime().block_on(op.submit_status(&changed)),
            Err(RestoreStatusSubmissionError::Direct(
                DirectRestoreStatusError::Binding
            ))
        ));
        assert_eq!(server.count(), 0);
        assert!(!op.inspect_status().unwrap().submission_reserved);
        assert_eq!(fs::read(directory.join("ledger.bin")).unwrap(), before);
        let progress = runtime().block_on(op.submit_status(&direct));
        if mode == "policy" {
            assert!(matches!(
                progress,
                Err(RestoreStatusSubmissionError::Status(_))
            ));
        } else {
            assert_eq!(
                progress.unwrap(),
                if mode == "rejected" {
                    StatusProgress::Rejected
                } else {
                    StatusProgress::Verified
                }
            );
        }
        let view = op.inspect_status().unwrap();
        assert!(view.submission_reserved && view.result_retained);
        assert_eq!(view.network_requests_reserved, 2);
        assert_eq!(server.count(), 1);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(capture_server.count(), 1);
        let requests = server.requests.lock().unwrap();
        let raw = &requests[0];
        let split = raw.windows(4).position(|b| b == b"\r\n\r\n").unwrap() + 4;
        assert_eq!(
            &raw[split..],
            PreparedStatus::decode(&request_bytes).unwrap().envelope
        );
        assert!(
            raw.starts_with(
                format!(
                    "POST /api/v3/canister/{}/call ",
                    plan.request().target.canister.to_text()
                )
                .as_bytes()
            )
        );
        drop(requests);
        let ledger = fs::read(directory.join("ledger.bin")).unwrap();
        let result = fs::read(directory.join("status-result.bin")).unwrap();
        assert_eq!(
            crate::store::StatusEvidence::decode(&result).unwrap().proof,
            proof
        );
        server.stop.store(true, Ordering::SeqCst);
        server.thread.take().unwrap().join().unwrap();
        allow.store(false, Ordering::SeqCst);
        drop(op);
        let mut op = runtime()
            .block_on(f.store.open_restore(&plan, plan.digest(), &f.t, &mut p))
            .unwrap();
        let replay = runtime().block_on(op.submit_status(&direct));
        if mode == "policy" {
            assert!(matches!(
                replay,
                Err(RestoreStatusSubmissionError::Status(_))
            ));
        } else {
            assert_eq!(
                replay.unwrap(),
                if mode == "rejected" {
                    StatusProgress::Rejected
                } else {
                    StatusProgress::Verified
                }
            );
        }
        assert_eq!(op.inspect_status().unwrap(), view);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(server.count(), 1);
        assert_eq!(fs::read(directory.join("ledger.bin")).unwrap(), ledger);
        assert_eq!(
            fs::read(directory.join("status-request.bin")).unwrap(),
            request_bytes
        );
        assert_eq!(
            fs::read(directory.join("status-result.bin")).unwrap(),
            result
        );
        if mode == "success" {
            let mut changed = result;
            changed[5] ^= 1;
            fs::write(directory.join("status-result.bin"), &changed).unwrap();
            assert!(op.inspect_status().is_err());
            assert!(runtime().block_on(op.submit_status(&direct)).is_err());
            assert_eq!(
                fs::read(directory.join("status-result.bin")).unwrap(),
                changed
            );
            assert_eq!(server.count(), 1);
        }
    }
}

#[test]
fn restore_uncertain_status_stays_spent_without_a_second_http_request_after_reopen() {
    for mode in [
        "accepted",
        "redirect",
        "throttled",
        "unavailable",
        "disconnect",
        "proof",
        "cancel",
        "timeout",
        "orphan",
    ] {
        let (mut f, _, mut p) = settled();
        let receipt = complete(&mut f, &mut p).unwrap();
        let source = runtime()
            .block_on(
                f.store
                    .open_capture_source(&mut f.op, &receipt, maximum(), &f.t, &mut p),
            )
            .unwrap();
        let mut decl = declaration(&source);
        decl.sender = Principal::self_authenticating(Scheme::Ed25519.key());
        let plan = RestorePlan::new(&source, decl).unwrap();
        drop(source);
        let mut a = StatusAdmissionRecord {
            prepare_deadline_ns: now() + 60_000_000_000,
            submit_deadline_ns: now() + 120_000_000_000,
            prepare_requests: 1,
            ..admission()
        };
        let response = match mode {
            "disconnect" => Response::Disconnect,
            "cancel" | "timeout" => Response::HoldBody,
            "orphan" => Response::OrphanBeforeReply {
                file: f
                    .root
                    .join("operations")
                    .join(&plan.request().operation_id)
                    .join("status-result.bin"),
                response: http(
                    200,
                    "",
                    &body(&certificate(&request(&plan, &a), &[7], false, 1)),
                ),
            },
            "proof" => Response::Bytes(http(
                200,
                "",
                &body(&certificate(&request(&plan, &a), &[7], false, 2)),
            )),
            _ => Response::Bytes(http(
                match mode {
                    "accepted" => 202,
                    "redirect" => 307,
                    "throttled" => 429,
                    _ => 503,
                },
                "Location: /retry\r\n",
                &[],
            )),
        };
        let server = Server::new(response);
        let calls = Arc::new(AtomicU32::new(0));
        let allow = Arc::new(AtomicBool::new(true));
        let mut direct = provider(
            &server.endpoint,
            &plan,
            &calls,
            &allow,
            if mode == "timeout" { 1 } else { 5 },
        );
        a.provider = direct.commitment();
        let directory = f.root.join("operations").join(&plan.request().operation_id);
        drop(f.op);
        let mut op = runtime()
            .block_on(
                f.store
                    .create_restore(&plan, plan.digest(), &f.p, &f.t, &mut p),
            )
            .unwrap();
        runtime()
            .block_on(op.prepare_status(&a, &mut direct))
            .unwrap();
        if mode == "cancel" {
            runtime().block_on(async {
                assert!(
                    tokio::time::timeout(Duration::from_millis(200), op.submit_status(&direct))
                        .await
                        .is_err()
                );
                let bytes = fs::read(directory.join("ledger.bin")).unwrap();
                assert!(
                    bytes
                        .windows(b"status_submission_reserved".len())
                        .any(|b| b == b"status_submission_reserved")
                );
            });
        } else {
            let outcome = runtime().block_on(op.submit_status(&direct));
            if mode == "orphan" {
                assert!(matches!(
                    outcome,
                    Err(RestoreStatusSubmissionError::Capture(_))
                ));
                let ledger = fs::read(directory.join("ledger.bin")).unwrap();
                assert!(
                    ledger
                        .windows(b"status_submission_reserved".len())
                        .any(|b| b == b"status_submission_reserved")
                );
                assert!(op.inspect_status().is_err());
                assert_eq!(server.count(), 1);
                drop(op);
                assert!(
                    runtime()
                        .block_on(f.store.open_restore(&plan, plan.digest(), &f.t, &mut p))
                        .is_err()
                );
                assert_eq!(fs::read(directory.join("ledger.bin")).unwrap(), ledger);
                assert_eq!(
                    fs::read(directory.join("status-result.bin")).unwrap(),
                    b"orphan-result"
                );
                assert_eq!(calls.load(Ordering::SeqCst), 1);
                continue;
            }
            assert!(
                matches!(outcome, Err(RestoreStatusSubmissionError::Direct(_))),
                "{mode}"
            );
        }
        let view = op.inspect_status().unwrap();
        assert!(view.submission_reserved && !view.result_retained);
        assert_eq!(view.network_requests_reserved, 2);
        assert_eq!(server.count(), 1);
        let ledger = fs::read(directory.join("ledger.bin")).unwrap();
        let request = fs::read(directory.join("status-request.bin")).unwrap();
        allow.store(false, Ordering::SeqCst);
        drop(op);
        let mut op = runtime()
            .block_on(f.store.open_restore(&plan, plan.digest(), &f.t, &mut p))
            .unwrap();
        assert_eq!(
            runtime().block_on(op.submit_status(&direct)).unwrap(),
            StatusProgress::SubmissionPending
        );
        assert_eq!(op.inspect_status().unwrap(), view);
        assert_eq!(server.count(), 1);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(fs::read(directory.join("ledger.bin")).unwrap(), ledger);
        assert_eq!(
            fs::read(directory.join("status-request.bin")).unwrap(),
            request
        );
        assert!(!directory.join("status-result.bin").exists());
        if mode == "accepted" {
            let partial = &ledger[..ledger.len() - 1];
            fs::write(directory.join("ledger.bin"), partial).unwrap();
            assert!(runtime().block_on(op.submit_status(&direct)).is_err());
            assert_eq!(server.count(), 1);
            assert_eq!(fs::read(directory.join("ledger.bin")).unwrap(), partial);
        }
    }
}
