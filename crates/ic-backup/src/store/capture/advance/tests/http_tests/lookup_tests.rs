//! Real loopback read-state with signed synthetic certificates; no IC effect simulation.
use super::*;
use ic_platform::{
    certification::historical::{TimePolicy, TimeWindow},
    ic_protocol::{Envelope, encode_envelope},
};

pub(super) fn policy() -> CaptureLookupHttpPolicy {
    CaptureLookupHttpPolicy {
        request: CaptureLookupPolicy {
            expires_ns: now() + 30_000_000_000,
            envelope_bytes: 4096,
        },
        observation: CaptureObservationPolicy {
            evidence_bytes: 4096,
            time: TimePolicy {
                certificate: TimeWindow {
                    reference_ns: 100,
                    max_age_ns: 0,
                    max_future_skew_ns: 0,
                },
                delegation: TimeWindow {
                    reference_ns: 10,
                    max_age_ns: 0,
                    max_future_skew_ns: 0,
                },
            },
        },
        deadline_ns: now() + 20_000_000_000,
    }
}
pub(super) fn lookup(p: &CapturePlan, r: &PreparedStatus, scheme: Scheme, expiry: u64) -> Vec<u8> {
    let CaptureExecution::Direct { sender } = p.request().execution else {
        panic!("direct")
    };
    let content = EnvelopeContent::ReadState {
        sender: sender.as_slice().try_into().unwrap(),
        ingress_expiry: expiry,
        paths: vec![vec![
            b"request_status".to_vec().into(),
            r.request_id.to_vec().into(),
        ]],
    };
    let signature = scheme.sign(&content.to_request_id().signable());
    encode_envelope(
        &Envelope {
            content: Cow::Owned(content),
            sender_pubkey: Some(scheme.key()),
            sender_sig: Some(signature),
            sender_delegation: None,
        },
        4096,
    )
    .unwrap()
}
pub(super) fn body(cert: &[u8]) -> Vec<u8> {
    let mut bytes = b"\xa1\x6bcertificate\x59".to_vec();
    bytes.extend_from_slice(&u16::try_from(cert.len()).unwrap().to_be_bytes());
    bytes.extend_from_slice(cert);
    bytes
}
fn transport(s: &Server, p: &CapturePlan) -> StatusHttpTransport {
    StatusHttpTransport::new(
        &s.endpoint,
        p.request().target.network,
        der(1),
        Duration::from_secs(1),
    )
    .unwrap()
}

#[test]
fn read_state_uses_exact_original_route_and_bytes_for_all_calls_and_direct_keys() {
    for scheme in [Scheme::Ed25519, Scheme::Secp256k1] {
        for call in [
            CaptureCall::Status,
            CaptureCall::ListSnapshots,
            CaptureCall::TakeSnapshot,
        ] {
            let (p, mut original, a, _) = input_for(scheme);
            change(&mut original, |e| {
                if let EnvelopeContent::Call {
                    method_name,
                    arg,
                    ingress_expiry,
                    ..
                } = e.content.to_mut()
                {
                    *method_name = call.method_name().into();
                    *arg = call.arguments(&p).unwrap();
                    *ingress_expiry = 21;
                }
                e.sender_sig = Some(scheme.sign(&e.content.to_request_id().signable()));
            });
            original.expires_ns = 21; // Valid historical original, expired for submission.
            let cert = signed(
                reply_tree(&original.request_id, b"exact original reply", b"replied"),
                1,
                None,
            );
            let s = Server::new(Response::Bytes(http(200, "", &body(&cert))));
            let policy = policy();
            let bytes = lookup(&p, &original, scheme, policy.request.expires_ns);
            let before = original.encode(4096).unwrap();
            let proof = runtime()
                .block_on(transport(&s, &p).lookup_once(&p, &a, &original, call, &bytes, policy))
                .unwrap();
            assert_eq!(proof.certificate, cert);
            assert!(
                matches!(proof.outcome, CaptureObservation::Replied(ref r) if r == b"exact original reply")
            );
            let sent = s.requests.lock().unwrap();
            assert_eq!(sent.len(), 1);
            assert!(
                sent[0].starts_with(
                    format!(
                        "POST /api/v2/canister/{}/read_state HTTP/1.1\r\n",
                        p.request().target.canister
                    )
                    .as_bytes()
                )
            );
            assert!(sent[0].ends_with(&bytes));
            assert_eq!(original.encode(4096).unwrap(), before);
        }
    }
}

#[test]
fn read_state_refuses_policy_signature_binding_and_expiry_before_connection() {
    let (p, r, a, _) = input();
    let s = Server::new(Response::Bytes(http(200, "", b"unused")));
    for case in 0..5 {
        let mut policy = policy();
        let mut bytes = lookup(&p, &r, Scheme::Ed25519, policy.request.expires_ns);
        match case {
            0 => policy.observation.evidence_bytes = a.result_bytes + 1,
            1 => policy.observation.time.certificate.max_age_ns = 101,
            2 => policy.deadline_ns = 1,
            3 => policy.request.expires_ns += 1,
            4 => *bytes.last_mut().unwrap() ^= 1,
            _ => unreachable!(),
        }
        let result = runtime().block_on(transport(&s, &p).lookup_once(
            &p,
            &a,
            &r,
            CaptureCall::Status,
            &bytes,
            policy,
        ));
        assert!(result.is_err(), "case {case}");
        assert!(s.requests.lock().unwrap().is_empty());
    }
    let wrong = StatusHttpTransport::new(
        &s.endpoint,
        Sha256Digest::compute(b"wrong network"),
        der(1),
        Duration::from_secs(1),
    )
    .unwrap();
    let policy = policy();
    let bytes = lookup(&p, &r, Scheme::Ed25519, policy.request.expires_ns);
    assert!(matches!(
        runtime().block_on(wrong.lookup_once(&p, &a, &r, CaptureCall::Status, &bytes, policy)),
        Err(CaptureLookupHttpError::Http(StatusHttpError::Binding))
    ));
    assert!(s.requests.lock().unwrap().is_empty());
}

#[test]
fn read_state_transport_failures_are_single_attempt_and_bounded() {
    let (p, r, a, _) = input();
    for case in 0..7 {
        let response = match case {
            0 => Response::Disconnect,
            1 => Response::Bytes(http(307, "Location: /elsewhere\r\n", b"")),
            2 => Response::Bytes(http(200, "", &[0; 4097])),
            3 => Response::Bytes(http(200, "Content-Encoding: gzip\r\n", b"bad")),
            4 => Response::Bytes(http(200, "", b"bad")),
            5 => Response::HoldBody,
            6 => Response::Bytes(http(
                200,
                "",
                &body(&signed(
                    reply_tree(&[99; 32], b"different", b"replied"),
                    2,
                    None,
                )),
            )),
            _ => unreachable!(),
        };
        let s = Server::new(response);
        let policy = policy();
        let bytes = lookup(&p, &r, Scheme::Ed25519, policy.request.expires_ns);
        let result = runtime().block_on(transport(&s, &p).lookup_once(
            &p,
            &a,
            &r,
            CaptureCall::Status,
            &bytes,
            policy,
        ));
        match case {
            0 | 5 => assert!(matches!(
                result,
                Err(CaptureLookupHttpError::Http(StatusHttpError::Uncertain))
            )),
            1 => assert!(matches!(
                result,
                Err(CaptureLookupHttpError::Http(StatusHttpError::HttpStatus(
                    307
                )))
            )),
            2 => assert!(matches!(
                result,
                Err(CaptureLookupHttpError::Http(StatusHttpError::ResponseLimit))
            )),
            3 | 4 => assert!(matches!(
                result,
                Err(CaptureLookupHttpError::Http(StatusHttpError::Response))
            )),
            6 => assert!(matches!(
                result,
                Err(CaptureLookupHttpError::Observation(_))
            )),
            _ => unreachable!(),
        }
        assert_eq!(s.requests.lock().unwrap().len(), 1);
    }
}

#[test]
fn read_state_returns_processing_without_polling_or_settlement() {
    let (p, r, a, _) = input();
    let cert = signed(reply_tree(&r.request_id, b"", b"processing"), 1, None);
    let s = Server::new(Response::Bytes(http(200, "", &body(&cert))));
    let policy = policy();
    let bytes = lookup(&p, &r, Scheme::Ed25519, policy.request.expires_ns);
    let evidence = runtime()
        .block_on(transport(&s, &p).lookup_once(&p, &a, &r, CaptureCall::Status, &bytes, policy))
        .unwrap();
    assert!(matches!(evidence.outcome, CaptureObservation::Processing));
    assert_eq!(evidence.certificate, cert);
    assert_eq!(s.requests.lock().unwrap().len(), 1);
}

#[test]
fn cancelled_read_state_closes_the_connection_without_detached_retries() {
    let (p, r, a, _) = input();
    let closed = Arc::new(AtomicBool::new(false));
    let s = Server::new(Response::ObserveCancellation(Arc::clone(&closed)));
    let transport = s.client(&p, Duration::from_secs(30));
    let policy = policy();
    let bytes = lookup(&p, &r, Scheme::Ed25519, policy.request.expires_ns);
    let original = r.encode(4096).unwrap();
    runtime().block_on(async {
        // Keep polling until the real server acknowledges receiving this request.
        let future = transport.lookup_once(&p, &a, &r, CaptureCall::Status, &bytes, policy);
        let mut future = Box::pin(future);
        tokio::time::timeout(
            Duration::from_secs(3),
            std::future::poll_fn(|cx| {
                assert!(future.as_mut().poll(cx).is_pending());
                if s.count() == 1 {
                    Poll::Ready(())
                } else {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            }),
        )
        .await
        .expect("server must receive the one lookup");
        drop(future);
        tokio::time::timeout(Duration::from_secs(3), async {
            while !closed.load(Ordering::SeqCst) {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("cancelled body read must close its connection");
        // The listener continues accepting while the runtime remains active.
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(s.count(), 1);
    });
    assert_eq!(r.encode(4096).unwrap(), original);
    assert_eq!(s.finish().len(), 1);
}

#[test]
fn lookup_deadline_and_lookup_expiry_each_bound_body_waiting() {
    let (p, r, a, _) = input();
    for expiry in [false, true] {
        let s = Server::new(Response::HoldBody);
        let transport = s.client(&p, Duration::from_secs(30));
        let mut policy = policy();
        if expiry {
            policy.request.expires_ns = now() + 500_000_000;
        } else {
            policy.deadline_ns = now() + 500_000_000;
        }
        let bytes = lookup(&p, &r, Scheme::Ed25519, policy.request.expires_ns);
        let result = runtime().block_on(async {
            tokio::time::timeout(
                Duration::from_secs(3),
                transport.lookup_once(&p, &a, &r, CaptureCall::Status, &bytes, policy),
            )
            .await
            .expect("lookup bound must precede the configured HTTP timeout")
        });
        assert!(matches!(
            result,
            Err(CaptureLookupHttpError::Http(
                StatusHttpError::Uncertain | StatusHttpError::Deadline
            ))
        ));
        assert_eq!(s.finish().len(), 1);
    }
}
