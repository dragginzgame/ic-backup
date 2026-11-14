//! Real loopback HTTP with synthetic certificate bodies; no simulated IC management effects.
use super::certificate_tests::{der, fixture, reply_tree, signed};
use super::request_tests::{Scheme, change, fixture as request_fixture};
use super::*;
use std::{
    io::Read,
    net::{TcpListener, TcpStream},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .try_into()
        .unwrap()
}
fn input() -> (CapturePlan, PreparedStatus, StatusAdmissionRecord, Vec<u8>) {
    input_for(Scheme::Ed25519)
}
fn input_for(scheme: Scheme) -> (CapturePlan, PreparedStatus, StatusAdmissionRecord, Vec<u8>) {
    let (_, old, _) = fixture();
    let (p, mut r) = request_fixture(scheme);
    let mut request = p.request().clone();
    request.trust_root = Sha256Digest::compute(&der(1));
    let p = CapturePlan::new(request).unwrap();
    set_expiry(&mut r, now() + 30_000_000_000, scheme);
    r.observation = old.observation;
    let response = candid_codec::encode_one(&status(&p), 4096).unwrap();
    let mut a = admission();
    a.prepare_deadline_ns = now() + 10_000_000_000;
    a.submit_deadline_ns = a.prepare_deadline_ns + 10_000_000_000;
    let certificate = signed(reply_tree(&r.request_id, &response, b"replied"), 1, None);
    // Fixed IC v3 CBOR map with one bounded certificate byte string; not a production encoder.
    let mut body = b"\xa2\x66status\x67replied\x6bcertificate\x59".to_vec();
    body.extend_from_slice(&u16::try_from(certificate.len()).unwrap().to_be_bytes());
    body.extend_from_slice(&certificate);
    (p, r, a, body)
}
fn set_expiry(request: &mut PreparedStatus, expiry: u64, scheme: Scheme) {
    change(request, |envelope| {
        if let EnvelopeContent::Call { ingress_expiry, .. } = envelope.content.to_mut() {
            *ingress_expiry = expiry;
        }
        envelope.sender_sig = Some(scheme.sign(&envelope.content.to_request_id().signable()));
    });
    request.expires_ns = expiry;
}
fn http(status: u16, headers: &str, body: &[u8]) -> Vec<u8> {
    [format!("HTTP/1.1 {status} Test\r\nContent-Type: application/cbor\r\nContent-Length: {}\r\nConnection: close\r\n{headers}\r\n", body.len()).as_bytes(), body].concat()
}

enum Response {
    Bytes(Vec<u8>),
    Disconnect,
    HoldBody,
}
struct Server {
    endpoint: String,
    root: PathBuf,
    stop: Arc<AtomicBool>,
    requests: Arc<Mutex<Vec<Vec<u8>>>>,
    thread: Option<thread::JoinHandle<()>>,
}
impl Server {
    fn new(response: Response) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let endpoint = format!("http://{}", listener.local_addr().unwrap());
        let root = root("status-http");
        let requests = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let thread = {
            let stop = Arc::clone(&stop);
            let requests = Arc::clone(&requests);
            let root = root.clone();
            thread::spawn(move || {
                while !stop.load(Ordering::SeqCst) {
                    let mut stream = match listener.accept() {
                        Ok((stream, _)) => stream,
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(2));
                            continue;
                        }
                        Err(error) => panic!("fixture accept: {error}"),
                    };
                    stream
                        .set_read_timeout(Some(Duration::from_secs(2)))
                        .unwrap();
                    stream
                        .set_write_timeout(Some(Duration::from_secs(2)))
                        .unwrap();
                    let request = read_request(&mut stream);
                    let mut captured = requests.lock().unwrap();
                    fs::write(
                        root.join(format!("request-{}.bin", captured.len())),
                        &request,
                    )
                    .unwrap();
                    captured.push(request);
                    drop(captured);
                    match &response {
                        Response::Bytes(bytes) => {
                            let _ = stream.write_all(bytes);
                        }
                        Response::Disconnect => {}
                        Response::HoldBody => {
                            stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/cbor\r\nContent-Length: 10\r\n\r\n").unwrap();
                            while !stop.load(Ordering::SeqCst) {
                                thread::sleep(Duration::from_millis(2));
                            }
                        }
                    }
                }
            })
        };
        Self {
            endpoint,
            root,
            stop,
            requests,
            thread: Some(thread),
        }
    }
    fn client(&self, p: &CapturePlan, timeout: Duration) -> StatusHttpTransport {
        StatusHttpTransport::new(&self.endpoint, p.request().target.network, der(1), timeout)
            .unwrap()
    }
    fn count(&self) -> usize {
        self.requests.lock().unwrap().len()
    }
    fn finish(mut self) -> Vec<Vec<u8>> {
        self.stop.store(true, Ordering::SeqCst);
        self.thread.take().unwrap().join().unwrap();
        let requests = self.requests.lock().unwrap().clone();
        assert!(self.root.is_dir());
        requests
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
fn read_request(stream: &mut TcpStream) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut buffer = [0; 4096];
    loop {
        let n = stream.read(&mut buffer).unwrap();
        assert_ne!(n, 0, "incomplete fixture request");
        bytes.extend_from_slice(&buffer[..n]);
        assert!(bytes.len() <= 1024 * 1024 + 16384);
        if let Some(offset) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
            let headers = std::str::from_utf8(&bytes[..offset]).unwrap();
            let length: usize = headers
                .lines()
                .find_map(|line| {
                    line.split_once(':')
                        .filter(|(name, _)| name.eq_ignore_ascii_case("content-length"))
                        .map(|(_, value)| value.trim().parse().unwrap())
                })
                .unwrap();
            if bytes.len() >= offset + 4 + length {
                assert_eq!(bytes.len(), offset + 4 + length);
                return bytes;
            }
        }
    }
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
            StatusHttpTransport::new(
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
                StatusHttpTransport::new(
                    endpoint,
                    p.request().target.network,
                    der(1),
                    Duration::from_secs(1)
                ),
                Err(StatusHttpError::Configuration)
            ),
            "{endpoint}"
        );
    }
    for timeout in [Duration::ZERO, Duration::from_secs(301)] {
        assert!(matches!(
            StatusHttpTransport::new(
                "https://example.com",
                p.request().target.network,
                der(1),
                timeout
            ),
            Err(StatusHttpError::Configuration)
        ));
    }
    assert!(matches!(
        StatusHttpTransport::new(
            "https://example.com",
            p.request().target.network,
            vec![],
            Duration::from_secs(1)
        ),
        Err(StatusHttpError::Configuration)
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
            matches!(rt.block_on(transport.submit_once(&p,&a,&r)),Err(StatusHttpError::HttpStatus(code)) if code==status)
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
            assert!(matches!(result, Err(StatusHttpError::NoCertificate)));
        } else {
            assert!(matches!(result, Err(StatusHttpError::Response)));
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
        Err(StatusHttpError::Response)
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
        Err(StatusHttpError::Response)
    ));
    assert_eq!(server.finish().len(), 1);
}

#[test]
fn declared_and_streamed_body_limits_refuse_before_decoding() {
    let rt = runtime();
    for chunked in [false, true] {
        let (p, r, a, _) = input();
        let body = vec![0; a.result_bytes as usize + 1];
        let response = if chunked {
            [format!("HTTP/1.1 200 OK\r\nContent-Type: application/cbor\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n{:x}\r\n",body.len()).as_bytes(),body.as_slice(),b"\r\n0\r\n\r\n"].concat()
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
            Err(StatusHttpError::ResponseLimit)
        ));
        assert_eq!(server.finish().len(), 1);
    }
}

#[test]
fn wrong_binding_changed_envelope_and_expired_admission_send_nothing() {
    let rt = runtime();
    let (p, mut r, mut a, _) = input();
    let server = Server::new(Response::Disconnect);
    let wrong = StatusHttpTransport::new(
        &server.endpoint,
        Sha256Digest::compute(b"wrong network"),
        der(1),
        Duration::from_secs(1),
    )
    .unwrap();
    assert!(matches!(
        rt.block_on(wrong.submit_once(&p, &a, &r)),
        Err(StatusHttpError::Binding)
    ));
    let wrong = StatusHttpTransport::new(
        &server.endpoint,
        p.request().target.network,
        der(2),
        Duration::from_secs(1),
    )
    .unwrap();
    assert!(matches!(
        rt.block_on(wrong.submit_once(&p, &a, &r)),
        Err(StatusHttpError::Binding)
    ));
    let transport = server.client(&p, Duration::from_secs(1));
    r.request_id[0] ^= 1;
    assert!(matches!(
        rt.block_on(transport.submit_once(&p, &a, &r)),
        Err(StatusHttpError::Admission(_))
    ));
    r.request_id[0] ^= 1;
    a.prepare_deadline_ns = 1;
    a.submit_deadline_ns = 2;
    assert!(matches!(
        rt.block_on(transport.submit_once(&p, &a, &r)),
        Err(StatusHttpError::Deadline)
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
            Err(StatusHttpError::Uncertain)
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

struct HttpProvider {
    transport: StatusHttpTransport,
    prepared: Option<PreparedStatus>,
    replay: bool,
}
impl StatusProvider for HttpProvider {
    type Error = StatusHttpError;
    fn admit(
        &self,
        _: &CapturePlan,
        _: &StatusAdmissionRecord,
        _: &[u8],
        _: Option<&PreparedStatus>,
    ) -> Result<(), Self::Error> {
        assert!(!self.replay);
        Ok(())
    }
    async fn prepare(
        &mut self,
        _: &CapturePlan,
        _: &StatusAdmissionRecord,
        _: &[u8],
    ) -> Result<PreparedStatus, Self::Error> {
        Ok(self.prepared.take().unwrap())
    }
    async fn submit_once(
        &mut self,
        p: &CapturePlan,
        a: &StatusAdmissionRecord,
        r: &PreparedStatus,
    ) -> Result<StatusEvidence, Self::Error> {
        assert!(!self.replay);
        self.transport.submit_once(p, a, r).await
    }
    fn authenticate(
        &self,
        p: &CapturePlan,
        a: &StatusAdmissionRecord,
        r: &PreparedStatus,
        e: &StatusEvidence,
    ) -> Result<(), Self::Error> {
        authenticate_status_request(p, a, r).map_err(StatusHttpError::Request)?;
        authenticate_status_evidence(p, a, r, e, &der(1)).map_err(StatusHttpError::Authentication)
    }
}
#[test]
fn workflow_replay_preserves_settlement_or_pending_reservation_without_http() {
    let rt = runtime();
    for success in [false, true] {
        let (p, r, a, body) = input();
        let server = Server::new(Response::Bytes(http(
            if success { 200 } else { 503 },
            "",
            &body,
        )));
        let root = root("status-http-operation");
        let store = BackupStore::initialize(&root).unwrap();
        let mut operation = store
            .create_capture(CapturePlan::new(p.request().clone()).unwrap(), p.digest())
            .unwrap();
        let mut provider = HttpProvider {
            transport: server.client(&p, Duration::from_secs(2)),
            prepared: Some(r),
            replay: false,
        };
        let result = rt.block_on(operation.advance_status(&a, &mut provider));
        if success {
            assert_eq!(result.unwrap(), StatusProgress::Verified);
        } else {
            assert!(matches!(
                result,
                Err(StatusAdvanceError::Provider(StatusHttpError::HttpStatus(
                    503
                )))
            ));
        }
        let before = operation.inspect_status().unwrap();
        assert_eq!(before.network_requests_reserved, 2);
        let ledger = root
            .join("operations")
            .join(&p.request().operation_id)
            .join("ledger.bin");
        let original = fs::read(&ledger).unwrap();
        drop(operation);
        let mut operation = store
            .open_capture(&p.request().operation_id, p.digest())
            .unwrap();
        provider.replay = true;
        assert_eq!(
            rt.block_on(operation.advance_status(&a, &mut provider))
                .unwrap(),
            if success {
                StatusProgress::Verified
            } else {
                StatusProgress::SubmissionPending
            }
        );
        assert_eq!(operation.inspect_status().unwrap(), before);
        assert_eq!(fs::read(&ledger).unwrap(), original);
        assert_eq!(server.finish().len(), 1);
    }
}

#[test]
fn original_submit_deadline_and_ingress_expiry_bound_body_waits() {
    let rt = runtime();
    for expiry in [false, true] {
        let (p, mut r, mut a, _) = input();
        if expiry {
            set_expiry(&mut r, now() + 150_000_000, Scheme::Ed25519);
        } else {
            a.prepare_deadline_ns = now();
            a.submit_deadline_ns = a.prepare_deadline_ns + 150_000_000;
        }
        let server = Server::new(Response::HoldBody);
        let transport = server.client(&p, Duration::from_secs(2));
        let result = rt.block_on(async {
            tokio::time::timeout(Duration::from_secs(1), transport.submit_once(&p, &a, &r))
                .await
                .expect("original deadline must precede fixture watchdog")
        });
        assert!(matches!(
            result,
            Err(StatusHttpError::Uncertain | StatusHttpError::Deadline)
        ));
        assert_eq!(server.finish().len(), 1);
    }
}

#[test]
fn chunked_certificate_authenticates_but_tampering_and_truncation_do_not() {
    let rt = runtime();
    let (p, r, a, mut body) = input();
    let mid = body.len() / 2;
    let response=[b"HTTP/1.1 200 OK\r\nContent-Type: application/cbor\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n".as_slice(),format!("{mid:x}\r\n").as_bytes(),&body[..mid],format!("\r\n{:x}\r\n",body.len()-mid).as_bytes(),&body[mid..],b"\r\n0\r\n\r\n"].concat();
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
        Err(StatusHttpError::Authentication(_))
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
        Err(StatusHttpError::Uncertain)
    ));
    assert_eq!(server.finish().len(), 1);
}

#[test]
fn http_success_with_certified_rejection_is_a_rejected_result() {
    use ic_platform::certification::{fork, labeled, leaf};
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
                (0, StatusHttpError::Request(StatusRequestError::Signature))
                    | (1, StatusHttpError::Request(StatusRequestError::Sender))
                    | (2, StatusHttpError::Request(StatusRequestError::Identity))
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
            assert!(matches!(result, Err(StatusHttpError::ResponseLimit)));
        } else {
            // Exact/empty body admission reaches the protocol decoder, which refuses it.
            assert!(matches!(result, Err(StatusHttpError::Response)));
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
    let transport = StatusHttpTransport::new(
        &endpoint,
        p.request().target.network,
        der(1),
        Duration::from_secs(1),
    )
    .unwrap();
    assert!(matches!(
        rt.block_on(transport.submit_once(&p, &a, &r)),
        Err(StatusHttpError::Uncertain)
    ));
    peer.join().unwrap();
}
