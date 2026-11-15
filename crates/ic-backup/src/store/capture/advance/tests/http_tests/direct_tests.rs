//! Native integration checks; synthetic certificates do not qualify IC effects.
use super::*;
use ic_platform::certification::{fork, labeled, leaf};
use std::sync::atomic::AtomicUsize;

struct Signer {
    envelope: Vec<u8>,
    calls: Arc<AtomicUsize>,
    hang: bool,
    allowed: Arc<AtomicBool>,
    revoke: bool,
}
impl CaptureSigner for Signer {
    type Error = ();
    fn protocol(&self) -> [u8; 32] {
        [71; 32]
    }
    async fn sign_call(&mut self, request: CaptureSigningRequest<'_>) -> Result<Vec<u8>, ()> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(
            request.arguments,
            request.call.arguments(request.plan).unwrap()
        );
        assert_eq!(request.network_requests, 1);
        assert!(request.deadline_ns < request.max_expiry_ns);
        assert_eq!(request.envelope_bytes, 4096 - 44 - 79);
        if self.revoke {
            self.allowed.store(false, Ordering::SeqCst);
        }
        if self.hang {
            pending::<()>().await;
        }
        Ok(self.envelope.clone())
    }
}
struct Guard(Arc<AtomicBool>);
impl CaptureApplicationGuard for Guard {
    type Error = ();
    fn protocol(&self) -> [u8; 32] {
        [72; 32]
    }
    fn admit(&self, _: &CapturePlan, application: &[u8; 32]) -> Result<(), ()> {
        if application == &[2; 32] && self.0.load(Ordering::SeqCst) {
            Ok(())
        } else {
            Err(())
        }
    }
}
type Provider = DirectCaptureProvider<Signer, Guard>;
fn provider(
    server: &Server,
    plan: &CapturePlan,
    envelope: Vec<u8>,
    allowed: &Arc<AtomicBool>,
    calls: &Arc<AtomicUsize>,
    hang: bool,
    revoke: bool,
) -> Provider {
    Provider::new(
        server.client(plan, Duration::from_secs(2)),
        Signer {
            envelope,
            calls: Arc::clone(calls),
            hang,
            allowed: Arc::clone(allowed),
            revoke,
        },
        Guard(Arc::clone(allowed)),
        CaptureTiming {
            certificate_max_age_ns: 5_000_000_000,
            delegation_max_age_ns: 60_000_000_000,
            max_clock_skew_ns: 1_000_000_000,
        },
    )
}
fn body(p: &CapturePlan, r: &PreparedStatus, mut time: u64) -> Vec<u8> {
    let mut encoded = Vec::new();
    loop {
        let byte = (time & 127) as u8;
        time >>= 7;
        encoded.push(byte | if time == 0 { 0 } else { 128 });
        if time == 0 {
            break;
        }
    }
    let response = candid_codec::encode_one(&status(p), 4096).unwrap();
    let tree = fork(
        labeled(
            b"request_status",
            labeled(
                r.request_id.to_vec(),
                fork(
                    labeled(b"reply", leaf(response)),
                    labeled(b"status", leaf(b"replied".to_vec())),
                ),
            ),
        ),
        labeled(b"time", leaf(encoded)),
    );
    let certificate = signed(tree, 1, None);
    let mut body = b"\xa2\x66status\x67replied\x6bcertificate\x59".to_vec();
    body.extend_from_slice(&u16::try_from(certificate.len()).unwrap().to_be_bytes());
    body.extend_from_slice(&certificate);
    body
}
fn setup_direct(
    response: Option<Response>,
) -> (
    CapturePlan,
    StatusAdmissionRecord,
    Server,
    Provider,
    Arc<AtomicBool>,
    Arc<AtomicUsize>,
) {
    let (p, mut r, mut a, _) = input();
    set_expiry(&mut r, a.submit_deadline_ns, Scheme::Ed25519);
    let server = Server::new(
        response.unwrap_or_else(|| Response::Bytes(http(200, "", &body(&p, &r, now())))),
    );
    let allowed = Arc::new(AtomicBool::new(true));
    let calls = Arc::new(AtomicUsize::new(0));
    let provider = provider(&server, &p, r.envelope, &allowed, &calls, false, false);
    a.provider = provider.commitment();
    (p, a, server, provider, allowed, calls)
}

#[test]
fn concrete_provider_settles_and_cold_replay_never_reacquires_application_or_signs() {
    let rt = runtime();
    let (p, a, server, mut provider, allowed, calls) = setup_direct(None);
    let store = BackupStore::initialize(&root("direct-provider-settled")).unwrap();
    let mut op = store
        .create_capture(CapturePlan::new(p.request().clone()).unwrap(), p.digest())
        .unwrap();
    assert_eq!(
        rt.block_on(op.advance_status(&a, &mut provider)).unwrap(),
        StatusProgress::Verified
    );
    assert_eq!(op.inspect_status().unwrap().network_requests_reserved, 2);
    drop(op);
    allowed.store(false, Ordering::SeqCst);
    let mut op = store
        .open_capture(&p.request().operation_id, p.digest())
        .unwrap();
    assert_eq!(
        rt.block_on(op.advance_status(&a, &mut provider)).unwrap(),
        StatusProgress::Verified
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(server.finish().len(), 1);
}

#[test]
fn wrong_original_provider_application_arguments_and_cost_refuse_before_signing() {
    let (p, a, server, provider, _, calls) = setup_direct(None);
    for case in 0..5 {
        let mut changed = a.clone();
        let mut args = arguments(&p).unwrap();
        match case {
            0 => changed.provider[0] ^= 1,
            1 => changed.application[0] ^= 1,
            2 => args.push(0),
            3 => changed.submit_requests = 2,
            _ => changed.request_bytes = 44 + 79,
        }
        assert!(provider.admit(&p, &changed, &args, None).is_err());
    }
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(server.finish().len(), 0);
}

#[test]
fn bad_signed_output_and_lost_application_authority_preserve_spent_preparation() {
    let rt = runtime();
    for case in 0..4 {
        let (p, a, server, _, allowed, calls) = setup_direct(None);
        let (_, mut r) = request_fixture(Scheme::Ed25519);
        set_expiry(&mut r, a.submit_deadline_ns, Scheme::Ed25519);
        let envelope = match case {
            0 => {
                change(&mut r, |e| e.sender_sig.as_mut().unwrap()[0] ^= 1);
                r.envelope
            }
            1 => vec![0; 4096],
            2 => {
                set_expiry(&mut r, a.submit_deadline_ns + 1, Scheme::Ed25519);
                r.envelope
            }
            _ => r.envelope,
        };
        let mut provider = provider(&server, &p, envelope, &allowed, &calls, false, case == 3);
        let store = BackupStore::initialize(&root("direct-provider-refusal")).unwrap();
        let mut op = store
            .create_capture(CapturePlan::new(p.request().clone()).unwrap(), p.digest())
            .unwrap();
        assert!(rt.block_on(op.advance_status(&a, &mut provider)).is_err());
        assert_eq!(
            op.inspect_status().unwrap().phase,
            CaptureStatusPhase::PreparationReserved
        );
        drop(op);
        let mut op = store
            .open_capture(&p.request().operation_id, p.digest())
            .unwrap();
        assert_eq!(
            rt.block_on(op.advance_status(&a, &mut provider)).unwrap(),
            StatusProgress::PreparationPending
        );
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(server.finish().len(), 0);
    }
}

#[test]
fn preparation_timeout_and_cancellation_cannot_sign_again_after_reopen() {
    let rt = runtime();
    for cancel in [false, true] {
        let (p, mut a, server, _, allowed, calls) = setup_direct(None);
        let mut provider = provider(&server, &p, vec![], &allowed, &calls, true, false);
        if !cancel {
            a.prepare_deadline_ns = now() + 1_000_000_000;
        }
        let store = BackupStore::initialize(&root("direct-provider-timeout")).unwrap();
        let mut op = store
            .create_capture(CapturePlan::new(p.request().clone()).unwrap(), p.digest())
            .unwrap();
        rt.block_on(async {
            if cancel {
                let mut future = Box::pin(op.advance_status(&a, &mut provider));
                std::future::poll_fn(|cx| {
                    assert!(future.as_mut().poll(cx).is_pending());
                    assert_eq!(calls.load(Ordering::SeqCst), 1);
                    Poll::Ready(())
                })
                .await;
                drop(future);
            } else {
                assert!(matches!(
                    op.advance_status(&a, &mut provider).await,
                    Err(StatusAdvanceError::Provider(DirectCaptureError::Deadline))
                ));
            }
        });
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        drop(op);
        let mut op = store
            .open_capture(&p.request().operation_id, p.digest())
            .unwrap();
        assert_eq!(
            rt.block_on(op.advance_status(&a, &mut provider)).unwrap(),
            StatusProgress::PreparationPending
        );
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(server.finish().len(), 0);
    }
}

#[test]
fn cancelled_http_is_pending_without_second_submission_on_cold_replay() {
    let rt = runtime();
    let (p, a, server, mut provider, _, calls) = setup_direct(Some(Response::HoldBody));
    let store = BackupStore::initialize(&root("direct-provider-cancel-submit")).unwrap();
    let mut op = store
        .create_capture(CapturePlan::new(p.request().clone()).unwrap(), p.digest())
        .unwrap();
    rt.block_on(async {
        assert!(
            tokio::time::timeout(
                Duration::from_millis(200),
                op.advance_status(&a, &mut provider)
            )
            .await
            .is_err()
        );
    });
    assert_eq!(
        op.inspect_status().unwrap().phase,
        CaptureStatusPhase::SubmissionReserved
    );
    drop(op);
    let mut op = store
        .open_capture(&p.request().operation_id, p.digest())
        .unwrap();
    assert_eq!(
        rt.block_on(op.advance_status(&a, &mut provider)).unwrap(),
        StatusProgress::SubmissionPending
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(server.finish().len(), 1);
}

#[test]
fn prepared_bytes_time_policy_and_continuity_are_rechecked_before_http() {
    let rt = runtime();
    let (p, a, server, mut provider, allowed, _) = setup_direct(None);
    let mut r = rt
        .block_on(provider.prepare(&p, &a, &arguments(&p).unwrap()))
        .unwrap();
    for index in [39, 47, 55, 63, 71] {
        r.observation[index] ^= 1;
        assert!(
            provider
                .admit(&p, &a, &arguments(&p).unwrap(), Some(&r))
                .is_err()
        );
        r.observation[index] ^= 1;
    }
    allowed.store(false, Ordering::SeqCst);
    assert!(matches!(
        rt.block_on(provider.submit_once(&p, &a, &r)),
        Err(DirectCaptureError::Application(()))
    ));
    assert_eq!(server.finish().len(), 0);
}

#[test]
fn historical_authentication_uses_original_window_without_live_freshness_or_guard() {
    let (p, mut a, server, provider, allowed, _) = setup_direct(None);
    a.prepare_deadline_ns = 100_000_000_001;
    a.submit_deadline_ns = 101_000_000_000;
    let (_, mut r) = request_fixture(Scheme::Ed25519);
    set_expiry(&mut r, a.submit_deadline_ns, Scheme::Ed25519);
    r.observation =
        status_certificate_observation(ic_platform::certification::historical::TimePolicy {
            certificate: ic_platform::certification::historical::TimeWindow {
                reference_ns: 100_000_000_000,
                max_age_ns: 5_000_000_000,
                max_future_skew_ns: 2_000_000_000,
            },
            delegation: ic_platform::certification::historical::TimeWindow {
                reference_ns: 100_000_000_000,
                max_age_ns: 60_000_000_000,
                max_future_skew_ns: 2_000_000_000,
            },
        })
        .unwrap();
    let bytes = body(&p, &r, 100_500_000_000);
    let ic_platform::ic_protocol::TransportCallResponse::Replied { certificate } =
        ic_platform::ic_protocol::decode_call_response(&bytes, 4096).unwrap()
    else {
        panic!("certificate");
    };
    let evidence = status_certificate_evidence(&p, &a, &r, &certificate, &der(1)).unwrap();
    allowed.store(false, Ordering::SeqCst);
    provider.authenticate(&p, &a, &r, &evidence).unwrap();
    assert!(matches!(
        provider.admit(&p, &a, &arguments(&p).unwrap(), Some(&r)),
        Err(DirectCaptureError::Deadline)
    ));
    assert_eq!(server.finish().len(), 0);
}

#[test]
fn live_deadlines_and_observation_arithmetic_refuse_before_signing() {
    let (p, a, server, provider, _, calls) = setup_direct(None);
    for case in 0..3 {
        let mut a = a.clone();
        match case {
            0 => a.prepare_deadline_ns = now() - 1,
            1 => a.submit_deadline_ns = now() + 301_000_000_000,
            _ => {
                a.prepare_deadline_ns = u64::MAX - 1;
                a.submit_deadline_ns = u64::MAX;
            }
        }
        assert!(matches!(
            provider.admit(&p, &a, &arguments(&p).unwrap(), None),
            Err(DirectCaptureError::Deadline)
        ));
    }
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(server.finish().len(), 0);
}

#[test]
fn changed_route_timeout_or_timing_cannot_reuse_original_provider_commitment() {
    let (p, a, server, _, allowed, calls) = setup_direct(None);
    for case in 0..4 {
        let endpoint = if case == 0 {
            "http://127.0.0.1:1"
        } else {
            &server.endpoint
        };
        let transport = StatusHttpTransport::new(
            endpoint,
            p.request().target.network,
            der(1),
            Duration::from_secs(if case == 1 { 1 } else { 2 }),
        )
        .unwrap();
        let timing = CaptureTiming {
            certificate_max_age_ns: if case == 2 { 0 } else { 5_000_000_000 },
            delegation_max_age_ns: 60_000_000_000,
            max_clock_skew_ns: if case == 3 { u64::MAX } else { 1_000_000_000 },
        };
        let provider = DirectCaptureProvider::new(
            transport,
            Signer {
                envelope: vec![],
                calls: Arc::clone(&calls),
                hang: false,
                allowed: Arc::clone(&allowed),
                revoke: false,
            },
            Guard(Arc::clone(&allowed)),
            timing,
        );
        assert_ne!(provider.commitment(), a.provider);
        assert!(matches!(
            provider.admit(&p, &a, &arguments(&p).unwrap(), None),
            Err(DirectCaptureError::Binding)
        ));
        if case == 3 {
            let mut exact = a.clone();
            exact.provider = provider.commitment();
            assert!(matches!(
                provider.admit(&p, &exact, &arguments(&p).unwrap(), None),
                Err(DirectCaptureError::Deadline)
            ));
        }
    }
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(server.finish().len(), 0);
}

#[test]
fn finite_snapshot_calls_authenticate_exact_arguments_without_widening_status_api() {
    let rt = runtime();
    for call in [CaptureCall::ListSnapshots, CaptureCall::TakeSnapshot] {
        let (p, mut r, mut a, _) = input();
        change(&mut r, |envelope| {
            if let EnvelopeContent::Call {
                method_name, arg, ..
            } = envelope.content.to_mut()
            {
                *method_name = call.method_name().into();
                *arg = call.arguments(&p).unwrap();
            }
        });
        set_expiry(&mut r, a.submit_deadline_ns, Scheme::Ed25519);
        let server = Server::new(Response::Disconnect);
        let allowed = Arc::new(AtomicBool::new(true));
        let calls = Arc::new(AtomicUsize::new(0));
        let mut provider = provider(
            &server,
            &p,
            r.envelope.clone(),
            &allowed,
            &calls,
            false,
            false,
        );
        a.provider = provider.commitment();
        let prepared = rt.block_on(provider.prepare_call(&p, &a, call)).unwrap();
        assert_eq!(prepared.envelope, r.envelope);
        provider.admit_call(&p, &a, call, Some(&prepared)).unwrap();
        assert!(
            provider
                .admit_call(&p, &a, CaptureCall::Status, Some(&prepared))
                .is_err()
        );
        assert!(authenticate_status_request(&p, &a, &prepared).is_err());
        assert!(
            rt.block_on(
                server
                    .client(&p, Duration::from_secs(2))
                    .submit_once(&p, &a, &prepared)
            )
            .is_err()
        );
        if call == CaptureCall::TakeSnapshot {
            for case in 0..3 {
                let mut args = p.take_snapshot_arguments(&[], 1).unwrap();
                match case {
                    0 => args.replace_snapshot = Some(vec![1]),
                    1 => args.uninstall_code = Some(true),
                    _ => args.canister_id = ic_platform::Principal::anonymous(),
                }
                let mut changed =
                    PreparedStatus::decode(&prepared.encode(a.request_bytes).unwrap()).unwrap();
                change(&mut changed, |envelope| {
                    if let EnvelopeContent::Call { arg, .. } = envelope.content.to_mut() {
                        *arg = candid_codec::encode_one(&args, 512).unwrap();
                    }
                });
                set_expiry(&mut changed, a.submit_deadline_ns, Scheme::Ed25519);
                assert!(provider.admit_call(&p, &a, call, Some(&changed)).is_err());
            }
        }
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(server.finish().is_empty());
    }
}

#[test]
fn offline_result_settlement_requires_exact_original_certificate_and_does_not_send() {
    use ic_platform::ic_protocol::{TransportCallResponse, decode_call_response};
    let rt = runtime();
    let (p, a, server, mut provider, allowed, calls) = setup_direct(Some(Response::Disconnect));
    let root = root("direct-result-settlement");
    let store = BackupStore::initialize(&root).unwrap();
    let mut op = store
        .create_capture(CapturePlan::new(p.request().clone()).unwrap(), p.digest())
        .unwrap();
    assert!(rt.block_on(op.advance_status(&a, &mut provider)).is_err());
    let directory = root.join("operations").join(&p.request().operation_id);
    let prepared =
        PreparedStatus::decode(&fs::read(directory.join("status-request.bin")).unwrap()).unwrap();
    let TransportCallResponse::Replied { certificate } =
        decode_call_response(&body(&p, &prepared, now()), 4096).unwrap()
    else {
        panic!("certificate");
    };
    let original = status_certificate_evidence(&p, &a, &prepared, &certificate, &der(1)).unwrap();
    let before = fs::read(directory.join("ledger.bin")).unwrap();
    allowed.store(false, Ordering::SeqCst);
    drop(op);
    let mut op = store
        .open_capture(&p.request().operation_id, p.digest())
        .unwrap();
    for case in 0..5 {
        let mut changed = StatusEvidence {
            rejected: original.rejected,
            response: original.response.clone(),
            proof: original.proof.clone(),
        };
        match case {
            0 => changed.response[0] ^= 1,
            1 => changed.proof[0] ^= 1,
            2 => changed.rejected = true,
            _ => {
                let mut other = PreparedStatus::decode(&prepared.encode(4096).unwrap()).unwrap();
                let time = if case == 3 {
                    other.request_id[0] ^= 1;
                    now()
                } else {
                    a.submit_deadline_ns + 2_000_000_000
                };
                let TransportCallResponse::Replied { certificate } =
                    decode_call_response(&body(&p, &other, time), 4096).unwrap()
                else {
                    panic!("certificate");
                };
                changed.proof = certificate;
            }
        }
        assert!(matches!(
            op.settle_capture_result(CaptureCall::Status, &changed, &provider),
            Err(CaptureSettlementError::Provider(_))
        ));
        assert_eq!(fs::read(directory.join("ledger.bin")).unwrap(), before);
    }
    op.settle_capture_result(CaptureCall::Status, &original, &provider)
        .unwrap();
    assert_eq!(
        rt.block_on(op.advance_status(&a, &mut provider)).unwrap(),
        StatusProgress::Verified
    );
    let settled = fs::read(directory.join("ledger.bin")).unwrap();
    op.settle_capture_result(CaptureCall::Status, &original, &provider)
        .unwrap();
    assert_eq!(fs::read(directory.join("ledger.bin")).unwrap(), settled);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(server.finish().len(), 1);
}
