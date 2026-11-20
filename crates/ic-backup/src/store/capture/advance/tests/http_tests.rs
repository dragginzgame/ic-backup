use super::*;
use crate::store::{
    BackupDigest,
    capture::{
        advance::tests::fixtures::{
            certificates::{der, signed},
            http::{Response, Server, http, input, input_for, now, runtime, set_expiry},
            signatures::{Scheme, change},
        },
        status::tests::plan,
    },
    tests::root,
};
use std::{
    fs,
    io::{Read, Write},
    net::TcpListener,
    thread,
    time::Duration,
};

#[test]
fn certificate_survives_closing_deadline_without_relaxing_original_admission() {
    let (p, r, mut a, body) = input();
    let transport = BackupHttpTransport::new(
        "http://127.0.0.1:1",
        p.request().target.network,
        der(1),
        Duration::from_secs(1),
    )
    .unwrap();
    let ic_platform::ingress::CallResponse::Replied { certificate } =
        ic_platform::ingress::decode_call_response(&body, a.result_bytes as usize)
            .unwrap()
            .value
    else {
        panic!("certificate")
    };
    a.submit_deadline_ns = 1;
    let error = transport
        .admit_received_certificate(
            &super::super::request::IngressRequirements {
                target: p.request().target,
                sender: p.request().sender,
                recipient: ic_platform_types::Principal::management_canister(),
                trust_root: p.request().trust_root,
            },
            &a,
            &r,
            certificate.clone(),
        )
        .err()
        .unwrap();
    assert!(
        matches!(&error, BackupHttpError::AfterResponse { source, .. } if matches!(**source, BackupHttpError::Deadline))
    );
    let received = error.received_certificate().unwrap();
    assert_eq!(received.request_id(), r.request_id);
    assert_eq!(received.certificate(), certificate);
}

#[test]
fn origin_admission_preserves_written_path_and_literal_loopback_rules() {
    let p = plan();
    for endpoint in [
        "https://example.com",
        "https://example.com/",
        "http://127.0.0.1/",
        "http://[::1]:8080/",
    ] {
        assert!(
            BackupHttpTransport::new(
                endpoint,
                p.request().target.network,
                der(1),
                Duration::from_secs(1)
            )
            .is_ok()
        );
    }
    for endpoint in [
        " https://example.com/",
        "https://example.com ",
        "https://example.com/path/..",
        "https://example.com\\path\\..",
        "http://localhost/",
        "http://example.com/",
        "https://example.com/?query",
        "https://example.com/#fragment",
        "https://user:password@example.com/",
        "https://@example.com/",
        "https://example.com/%2e",
        "https://example.com/\n",
        "ftp://example.com/",
        "http://127.1",
        "http://2130706433",
        "http://0x7f000001",
    ] {
        assert!(
            matches!(
                BackupHttpTransport::new(
                    endpoint,
                    p.request().target.network,
                    der(1),
                    Duration::from_secs(1)
                ),
                Err(BackupHttpError::Configuration)
            ),
            "{endpoint}"
        );
    }
    for timeout in [Duration::ZERO, Duration::from_secs(301)] {
        assert!(matches!(
            BackupHttpTransport::new(
                "https://example.com",
                p.request().target.network,
                der(1),
                timeout
            ),
            Err(BackupHttpError::Configuration)
        ));
    }
    assert!(matches!(
        BackupHttpTransport::new(
            "https://example.com",
            p.request().target.network,
            vec![],
            Duration::from_secs(1)
        ),
        Err(BackupHttpError::Configuration)
    ));
}

#[test]
fn exact_retained_envelope_and_effective_target_produce_authenticated_evidence() {
    let (p, r, a, body) = input();
    let server = Server::new(Response::Bytes(http(200, "", &body)));
    let transport = server.client(&p, Duration::from_secs(2));
    let evidence = runtime()
        .block_on(transport.submit_once(&p, &a, &r))
        .unwrap();
    authenticate_status_evidence(&p, &a, &r, &evidence, &der(1)).unwrap();
    p.validate_status_reply(&evidence.response).unwrap();
    let requests = server.finish();
    assert_eq!(requests.len(), 1);
    let split = requests[0]
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .unwrap();
    assert_eq!(&requests[0][split + 4..], r.envelope);
    assert!(
        requests[0].starts_with(
            format!(
                "POST /api/v3/canister/{}/call HTTP/1.1\r\n",
                p.request().target.canister.to_text()
            )
            .as_bytes()
        )
    );
}

#[test]
fn redirects_throttling_acceptance_and_server_failures_never_retry() {
    let rt = runtime();

    for status in [202, 301, 302, 307, 308, 400, 429, 500, 503] {
        let (p, r, a, _) = input();
        let server = Server::new(Response::Bytes(http(
            status,
            "Location: /again\r\nRetry-After: 0\r\n",
            b"",
        )));
        let transport = server.client(&p, Duration::from_secs(1));
        assert!(
            matches!(rt.block_on(transport.submit_once(&p,&a,&r)),Err(BackupHttpError::HttpStatus(code)) if code==status)
        );
        assert_eq!(server.finish().len(), 1);
    }
}

#[test]
fn malformed_unproven_and_compressed_responses_remain_uncertain() {
    let rt = runtime();
    let non_replicated=b"\xa3\x66status\x78\x18non_replicated_rejection\x6breject_code\x04\x6ereject_message\x66denied";
    for (body, uncertified) in [
        (b"malformed".as_slice(), false),
        (b"\xa1\x66status\x68accepted".as_slice(), true),
        (non_replicated.as_slice(), true),
    ] {
        let (p, r, a, _) = input();
        let server = Server::new(Response::Bytes(http(200, "", body)));
        let result = rt.block_on(
            server
                .client(&p, Duration::from_secs(1))
                .submit_once(&p, &a, &r),
        );
        if uncertified {
            assert!(matches!(result, Err(BackupHttpError::NoCertificate)));
        } else {
            assert!(matches!(result, Err(BackupHttpError::Response)));
        }
        assert_eq!(server.finish().len(), 1);
    }
    let (p, r, a, _) = input();
    let server=Server::new(Response::Bytes(b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok".to_vec()));
    assert!(matches!(
        rt.block_on(
            server
                .client(&p, Duration::from_secs(1))
                .submit_once(&p, &a, &r)
        ),
        Err(BackupHttpError::Response)
    ));
    assert_eq!(server.finish().len(), 1);
    let (p, r, a, body) = input();
    let server = Server::new(Response::Bytes(http(
        200,
        "Content-Encoding: gzip\r\n",
        &body,
    )));
    assert!(matches!(
        rt.block_on(
            server
                .client(&p, Duration::from_secs(1))
                .submit_once(&p, &a, &r)
        ),
        Err(BackupHttpError::Response)
    ));
    assert_eq!(server.finish().len(), 1);
}

fn chunked_response(chunks: &[&[u8]]) -> Vec<u8> {
    let mut response = b"HTTP/1.1 200 OK\r\nContent-Type: application/cbor\r\n\
        Transfer-Encoding: chunked\r\nConnection: close\r\n\r\n"
        .to_vec();
    for chunk in chunks {
        response.extend_from_slice(format!("{:x}\r\n", chunk.len()).as_bytes());
        response.extend_from_slice(chunk);
        response.extend_from_slice(b"\r\n");
    }
    response.extend_from_slice(b"0\r\n\r\n");

    response
}

#[test]
fn declared_and_streamed_body_limits_refuse_before_decoding() {
    let rt = runtime();

    for chunked in [false, true] {
        let (p, r, a, _) = input();
        let body = vec![0; a.result_bytes as usize + 1];
        let response = if chunked {
            chunked_response(&[&body])
        } else {
            http(200, "", &body)
        };
        let server = Server::new(Response::Bytes(response));
        assert!(matches!(
            rt.block_on(
                server
                    .client(&p, Duration::from_secs(1))
                    .submit_once(&p, &a, &r)
            ),
            Err(BackupHttpError::ResponseLimit)
        ));
        assert_eq!(server.finish().len(), 1);
    }
}

#[test]
fn wrong_binding_changed_envelope_and_expired_admission_send_nothing() {
    let rt = runtime();
    let (p, mut r, mut a, _) = input();
    let server = Server::new(Response::Disconnect);
    let wrong = BackupHttpTransport::new(
        &server.endpoint,
        BackupDigest::compute(b"wrong network"),
        der(1),
        Duration::from_secs(1),
    )
    .unwrap();
    assert!(matches!(
        rt.block_on(wrong.submit_once(&p, &a, &r)),
        Err(BackupHttpError::Binding)
    ));
    let wrong = BackupHttpTransport::new(
        &server.endpoint,
        p.request().target.network,
        der(2),
        Duration::from_secs(1),
    )
    .unwrap();
    assert!(matches!(
        rt.block_on(wrong.submit_once(&p, &a, &r)),
        Err(BackupHttpError::Binding)
    ));
    let transport = server.client(&p, Duration::from_secs(1));
    r.request_id[0] ^= 1;
    assert!(matches!(
        rt.block_on(transport.submit_once(&p, &a, &r)),
        Err(BackupHttpError::Admission(_))
    ));
    r.request_id[0] ^= 1;
    a.prepare_deadline_ns = 1;
    a.submit_deadline_ns = 2;
    assert!(matches!(
        rt.block_on(transport.submit_once(&p, &a, &r)),
        Err(BackupHttpError::Deadline)
    ));
    assert_eq!(server.finish().len(), 0);
}

#[test]
fn disconnect_body_timeout_and_cancellation_never_retry() {
    let rt = runtime();
    for mode in [Response::Disconnect, Response::HoldBody] {
        let (p, r, a, _) = input();
        let server = Server::new(mode);
        assert!(matches!(
            rt.block_on(
                server
                    .client(&p, Duration::from_millis(100))
                    .submit_once(&p, &a, &r)
            ),
            Err(BackupHttpError::Uncertain)
        ));
        assert_eq!(server.finish().len(), 1);
    }
    let (p, r, a, _) = input();
    let server = Server::new(Response::HoldBody);
    let transport = server.client(&p, Duration::from_secs(2));
    rt.block_on(async {
        let future = transport.submit_once(&p, &a, &r);
        assert!(
            tokio::time::timeout(Duration::from_millis(100), future)
                .await
                .is_err()
        );
        tokio::time::sleep(Duration::from_millis(30)).await;
    });
    assert_eq!(server.count(), 1);
    assert_eq!(server.finish().len(), 1);
}

#[test]
fn original_submit_deadline_and_ingress_expiry_bound_body_waits() {
    let rt = runtime();

    for expiry in [false, true] {
        let (p, mut r, mut a, _) = input();
        let server = Server::new(Response::HoldBody);
        let transport = server.client(&p, Duration::from_secs(30));
        // Start after setup and allow a loaded host to dispatch before expiry.
        // The watchdog still precedes the transport's own timeout.
        if expiry {
            set_expiry(&mut r, now() + 2_000_000_000, Scheme::Ed25519);
        } else {
            a.prepare_deadline_ns = now();
            a.submit_deadline_ns = a.prepare_deadline_ns + 2_000_000_000;
        }
        let result = rt.block_on(async {
            tokio::time::timeout(Duration::from_secs(5), transport.submit_once(&p, &a, &r))
                .await
                .expect("original deadline must precede fixture watchdog")
        });
        assert!(matches!(
            result,
            Err(BackupHttpError::Uncertain | BackupHttpError::Deadline)
        ));
        assert_eq!(server.finish().len(), 1);
    }
}

#[test]
fn chunked_certificate_authenticates_but_tampering_and_truncation_do_not() {
    let rt = runtime();
    let (p, r, a, mut body) = input();
    let mid = body.len() / 2;
    let response = chunked_response(&[&body[..mid], &body[mid..]]);
    let server = Server::new(Response::Bytes(response));
    let evidence = rt
        .block_on(
            server
                .client(&p, Duration::from_secs(1))
                .submit_once(&p, &a, &r),
        )
        .unwrap();
    assert!(!evidence.rejected);
    assert_eq!(server.finish().len(), 1);
    *body.last_mut().unwrap() ^= 1;
    let server = Server::new(Response::Bytes(http(200, "", &body)));
    assert!(matches!(
        rt.block_on(
            server
                .client(&p, Duration::from_secs(1))
                .submit_once(&p, &a, &r)
        ),
        Err(BackupHttpError::AfterResponse { source, received })
            if matches!(*source, BackupHttpError::Authentication(_)) && !received.certificate().is_empty()
    ));
    assert_eq!(server.finish().len(), 1);
    let response=b"HTTP/1.1 200 OK\r\nContent-Type: application/cbor\r\nContent-Length: 100\r\nConnection: close\r\n\r\nx".to_vec();
    let server = Server::new(Response::Bytes(response));
    assert!(matches!(
        rt.block_on(
            server
                .client(&p, Duration::from_secs(1))
                .submit_once(&p, &a, &r)
        ),
        Err(BackupHttpError::Uncertain)
    ));
    assert_eq!(server.finish().len(), 1);
}

#[test]
fn http_success_with_certified_rejection_is_a_rejected_result() {
    use ic_platform::certificate::construction::{fork, labeled, leaf};
    let (p, r, a, _) = input();
    let tree = fork(
        labeled(
            b"request_status",
            labeled(
                r.request_id.to_vec(),
                fork(
                    labeled(b"reject_code", leaf(vec![2])),
                    fork(
                        labeled(b"reject_message", leaf(b"transient".to_vec())),
                        labeled(b"status", leaf(b"rejected".to_vec())),
                    ),
                ),
            ),
        ),
        labeled(b"time", leaf(vec![100])),
    );
    let certificate = signed(tree, 1, None);
    let mut body = b"\xa2\x66status\x67replied\x6bcertificate\x59".to_vec();
    body.extend_from_slice(&u16::try_from(certificate.len()).unwrap().to_be_bytes());
    body.extend_from_slice(&certificate);
    let server = Server::new(Response::Bytes(http(200, "", &body)));
    let evidence = runtime()
        .block_on(
            server
                .client(&p, Duration::from_secs(1))
                .submit_once(&p, &a, &r),
        )
        .unwrap();
    assert!(evidence.rejected);
    authenticate_status_evidence(&p, &a, &r, &evidence, &der(1)).unwrap();
    assert_eq!(server.finish().len(), 1);
}

#[test]
fn signature_authentication_precedes_http_for_both_direct_key_profiles() {
    let rt = runtime();

    for scheme in [Scheme::Ed25519, Scheme::Secp256k1] {
        let (p, r, a, body) = input_for(scheme);
        let server = Server::new(Response::Bytes(http(200, "", &body)));
        let transport = server.client(&p, Duration::from_secs(1));
        let evidence = rt.block_on(transport.submit_once(&p, &a, &r)).unwrap();
        authenticate_status_evidence(&p, &a, &r, &evidence, &der(1)).unwrap();
        p.validate_status_reply(&evidence.response).unwrap();
        assert_eq!(server.finish().len(), 1);

        for case in 0..3 {
            let (p, mut r, a, body) = input_for(scheme);
            change(&mut r, |envelope| match case {
                0 => envelope.sender_sig.as_mut().unwrap()[0] ^= 1,
                1 => *envelope.sender_pubkey.as_mut().unwrap().last_mut().unwrap() ^= 1,
                2 => envelope.sender_delegation = Some(vec![]),
                _ => unreachable!(),
            });
            let server = Server::new(Response::Bytes(http(200, "", &body)));
            let transport = server.client(&p, Duration::from_secs(1));
            let Err(error) = rt.block_on(transport.submit_once(&p, &a, &r)) else {
                panic!("invalid signer output reached HTTP");
            };
            assert!(matches!(
                (case, error),
                (0, BackupHttpError::Request(StatusRequestError::Signature))
                    | (1, BackupHttpError::Request(StatusRequestError::Sender))
                    | (2, BackupHttpError::Request(StatusRequestError::Identity))
            ));
            assert!(server.finish().is_empty());
        }
    }
}

#[test]
fn shared_reader_preserves_exact_body_limit_and_empty_body_refusals() {
    let rt = runtime();

    for size in [0, 4096, 4097] {
        let (p, r, a, _) = input();
        assert_eq!(a.result_bytes, 4096);
        let server = Server::new(Response::Bytes(http(200, "", &vec![0; size])));
        let result = rt.block_on(
            server
                .client(&p, Duration::from_secs(1))
                .submit_once(&p, &a, &r),
        );
        if size > a.result_bytes as usize {
            assert!(matches!(result, Err(BackupHttpError::ResponseLimit)));
        } else {
            // Exact/empty body admission reaches the protocol decoder, which refuses it.
            assert!(matches!(result, Err(BackupHttpError::Response)));
        }
        assert_eq!(server.finish().len(), 1);
    }
}

#[test]
fn https_sends_tls_and_refuses_a_plaintext_peer_without_http_fallback() {
    let rt = runtime();
    let (p, r, a, _) = input();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let endpoint = format!("https://{}", listener.local_addr().unwrap());
    let evidence = root("status-https-handshake");
    let peer = thread::spawn(move || {
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(
                        std::time::Instant::now() < deadline,
                        "missing TLS connection"
                    );
                    thread::sleep(Duration::from_millis(2));
                }
                Err(error) => panic!("fixture accept: {error}"),
            }
        };
        // The listener polls; the TLS peer uses bounded blocking IO.
        stream.set_nonblocking(false).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        stream
            .set_write_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut prefix = [0; 5];
        stream.read_exact(&mut prefix).unwrap();
        fs::write(evidence.join("client-prefix.bin"), prefix).unwrap();
        assert_eq!(prefix[0], 22); // TLS handshake record, not a plaintext HTTP request.
        assert_eq!(prefix[1], 3);
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
            .unwrap();
    });
    let transport = BackupHttpTransport::new(
        &endpoint,
        p.request().target.network,
        der(1),
        Duration::from_secs(1),
    )
    .unwrap();
    assert!(matches!(
        rt.block_on(transport.submit_once(&p, &a, &r)),
        Err(BackupHttpError::Uncertain)
    ));
    peer.join().unwrap();
}

#[test]
fn root_updates_route_and_verify_delegation_for_root() {
    use super::super::IngressRequirements;
    use crate::store::capture::tests::certificates::delegated;
    use ic_platform_types::Principal;

    let root = Principal::from_slice(&[8, 1]);

    for method in ["snapshot_execute", "snapshot_lookup"] {
        let (plan, mut prepared, admission, _) = input();
        let child = plan.request().target.canister;
        assert_ne!(root, child);
        change(&mut prepared, |envelope| {
            if let EnvelopeContent::Call {
                canister_id,
                method_name,
                arg,
                ..
            } = &mut envelope.content
            {
                *canister_id = root.as_slice().try_into().unwrap();
                *method_name = method.into();
                *arg = vec![42];
            }
            envelope.sender_sig = Some(Scheme::Ed25519.sign(&envelope.content.signable().unwrap()));
        });

        for authority in [root, child] {
            let mut evidence = StatusEvidence {
                rejected: false,
                response: vec![17],
                proof: Vec::new(),
            };
            evidence.proof = delegated(authority.as_slice(), &prepared, &evidence);
            let bytes = fixtures::http::call_body(&evidence.proof);
            let server = Server::new(Response::Bytes(http(200, "", &bytes)));
            let requirements = IngressRequirements {
                target: plan.request().target,
                sender: plan.request().sender,
                recipient: root,
                trust_root: plan.request().trust_root,
            };
            let result = runtime().block_on(
                server
                    .client(&plan, Duration::from_secs(2))
                    .submit_expected(requirements, &admission, &prepared, method, &[42]),
            );
            if authority == root {
                assert_eq!(result.unwrap().proof, evidence.proof);
            } else {
                let Err(error) = result else {
                    panic!("accepted child-only certificate")
                };
                assert_eq!(
                    error.received_certificate().unwrap().request_id(),
                    prepared.request_id
                );
            }
            let sent = server.finish();
            assert_eq!(sent.len(), 1);
            assert!(sent[0].starts_with(format!("POST /api/v3/canister/{root}/call ").as_bytes()));
        }
    }
}
