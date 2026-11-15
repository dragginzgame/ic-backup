use super::super::*;
use crate::store::{
    capture::{
        advance::tests::{
            admission,
            certificate_tests::{der, fixture, reply_tree, signed},
            fixtures::signatures::{Scheme, change, fixture as request_fixture},
        },
        status::tests::status,
    },
    tests::root,
};
use ic_host_artifacts::artifact::Sha256Digest;
use std::{
    fs,
    io::Read,
    net::{TcpListener, TcpStream},
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub(in crate::store) fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}

pub(in crate::store) fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .try_into()
        .unwrap()
}

pub(in crate::store) fn input() -> (CapturePlan, PreparedStatus, StatusAdmissionRecord, Vec<u8>) {
    input_for(Scheme::Ed25519)
}

pub(in crate::store) fn input_for(
    scheme: Scheme,
) -> (CapturePlan, PreparedStatus, StatusAdmissionRecord, Vec<u8>) {
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

pub(in crate::store) fn set_expiry(request: &mut PreparedStatus, expiry: u64, scheme: Scheme) {
    change(request, |envelope| {
        if let EnvelopeContent::Call { ingress_expiry, .. } = envelope.content.to_mut() {
            *ingress_expiry = expiry;
        }
        envelope.sender_sig = Some(scheme.sign(&envelope.content.to_request_id().signable()));
    });
    request.expires_ns = expiry;
}

pub(in crate::store) fn http(status: u16, headers: &str, body: &[u8]) -> Vec<u8> {
    [format!("HTTP/1.1 {status} Test\r\nContent-Type: application/cbor\r\nContent-Length: {}\r\nConnection: close\r\n{headers}\r\n", body.len()).as_bytes(), body].concat()
}

pub(in crate::store) enum Response {
    Sequence(Vec<Vec<u8>>),
    Bytes(Vec<u8>),
    Disconnect,
    HoldBody,
    ObserveCancellation(Arc<AtomicBool>),
}

pub(in crate::store) struct Server {
    pub(in crate::store) endpoint: String,
    pub(in crate::store) root: PathBuf,
    pub(in crate::store) stop: Arc<AtomicBool>,
    pub(in crate::store) requests: Arc<Mutex<Vec<Vec<u8>>>>,
    pub(in crate::store) thread: Option<thread::JoinHandle<()>>,
}

impl Server {
    pub(in crate::store) fn new(response: Response) -> Self {
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
                        Response::Sequence(replies) => {
                            let n = requests.lock().unwrap().len() - 1;
                            if let Some(bytes) = replies.get(n) {
                                let _ = stream.write_all(bytes);
                            }
                        }
                        Response::Bytes(bytes) => {
                            let _ = stream.write_all(bytes);
                        }
                        Response::Disconnect => {}
                        Response::ObserveCancellation(closed) => {
                            stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/cbor\r\nContent-Length: 10\r\n\r\n").unwrap();
                            let mut byte = [0];
                            let result = stream.read(&mut byte);
                            closed.store(matches!(result, Ok(0)), Ordering::SeqCst);
                        }
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
    pub(in crate::store) fn client(
        &self,
        p: &CapturePlan,
        timeout: Duration,
    ) -> StatusHttpTransport {
        StatusHttpTransport::new(&self.endpoint, p.request().target.network, der(1), timeout)
            .unwrap()
    }
    pub(in crate::store) fn count(&self) -> usize {
        self.requests.lock().unwrap().len()
    }
    pub(in crate::store) fn finish(mut self) -> Vec<Vec<u8>> {
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

pub(in crate::store) fn read_request(stream: &mut TcpStream) -> Vec<u8> {
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

pub(in crate::store) struct HttpProvider {
    pub(in crate::store) transport: StatusHttpTransport,
    pub(in crate::store) prepared: Option<PreparedStatus>,
    pub(in crate::store) replay: bool,
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
