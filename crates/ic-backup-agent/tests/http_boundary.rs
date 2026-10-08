//! Real local HTTP operations with controlled responses; no simulated management effects.
mod support;
const TARGET: &str = "renrk-eyaaa-aaaaa-aaada-cai";
use ic_backup::{
    model::{
        ic_mutation::IcMutationRequest,
        ic_request::IcManagementMethodRecord as Method,
        operation_plan::{PlanContextRecord, PlanContextRequest},
    },
    ops::persistence::{AttemptJournalGuard, BackupLayoutGuard},
};
use ic_backup_agent::{
    AgentTransport, MAX_HTTP_RESPONSE_BYTES, ReservedUpdate, TransportError, UpdateOutcome,
};
use std::{
    fs,
    io::{Read, Write},
    net::TcpListener,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

struct Server {
    endpoint: String,
    requests: Arc<Mutex<Vec<Vec<u8>>>>,
    done: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}
impl Server {
    fn new(status: &str, body: Vec<u8>, discard: bool, delay: bool) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let endpoint = format!("http://{}/", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let retained = requests.clone();
        let done = Arc::new(AtomicBool::new(false));
        let stopping = done.clone();
        let status = status.to_owned();
        let thread = thread::spawn(move || {
            while !stopping.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        stream
                            .set_read_timeout(Some(Duration::from_secs(3)))
                            .unwrap();
                        let mut bytes = Vec::new();
                        let mut chunk = [0; 4096];
                        loop {
                            let count = stream.read(&mut chunk).unwrap();
                            assert_ne!(count, 0);
                            bytes.extend_from_slice(&chunk[..count]);
                            if let Some(end) = bytes.windows(4).position(|p| p == b"\r\n\r\n") {
                                let header = String::from_utf8_lossy(&bytes[..end]);
                                let len = header
                                    .lines()
                                    .find_map(|line| {
                                        line.to_ascii_lowercase()
                                            .strip_prefix("content-length: ")
                                            .map(|n| n.parse::<usize>().unwrap())
                                    })
                                    .unwrap();
                                if bytes.len() >= end + 4 + len {
                                    break;
                                }
                            }
                        }
                        retained.lock().unwrap().push(bytes);
                        if discard {
                            continue;
                        }
                        if delay {
                            thread::sleep(Duration::from_millis(1200));
                        }
                        let location = if status.starts_with("307") {
                            "Location: /retry\r\n"
                        } else {
                            ""
                        };
                        let header = format!(
                            "HTTP/1.1 {status}\r\n{location}Content-Length: {}\r\nContent-Type: application/cbor\r\nConnection: close\r\n\r\n",
                            body.len()
                        );
                        let _ = stream.write_all(header.as_bytes());
                        let _ = stream.write_all(&body);
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(e) => panic!("{e}"),
                }
            }
        });
        Self {
            endpoint,
            requests,
            done,
            thread: Some(thread),
        }
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.done.store(true, Ordering::Release);
        self.thread.take().unwrap().join().unwrap();
    }
}

#[tokio::test]
async fn accepted_errors_disconnect_timeout_and_bounds_make_one_request_and_keep_spending() {
    for (label, status, body, discard, delay, pending) in [
        ("accepted", "202 Accepted", vec![], false, false, true),
        (
            "throttled",
            "429 Too Many Requests",
            vec![],
            false,
            false,
            false,
        ),
        (
            "unavailable",
            "503 Service Unavailable",
            vec![],
            false,
            false,
            false,
        ),
        (
            "redirect",
            "307 Temporary Redirect",
            vec![],
            false,
            false,
            false,
        ),
        ("malformed", "200 OK", vec![255], false, false, false),
        (
            "oversize",
            "200 OK",
            vec![0; MAX_HTTP_RESPONSE_BYTES + 1],
            false,
            false,
            false,
        ),
        ("lost", "200 OK", vec![], true, false, false),
        ("timeout", "200 OK", vec![], false, true, false),
    ] {
        let root = support::root(label);
        let server = Server::new(status, body, discard, delay);
        let payload = support::payload(Method::StopCanister, TARGET, None);
        let plan = support::plan(TARGET, &payload.digest(), 1);
        fs::create_dir(root.join("layout")).unwrap();
        let layout = BackupLayoutGuard::acquire(&root.join("layout")).unwrap();
        let guard = support::retain(&layout, &plan, 1);
        let before = fs::read(guard.path()).unwrap();
        let request = IcMutationRequest::new(&plan, 1, guard.record().unwrap(), &payload).unwrap();
        let transport = AgentTransport::new(
            plan.context().clone(),
            &server.endpoint,
            support::identity(),
            vec![1],
            Duration::from_secs(1),
        )
        .unwrap();
        let prepared = transport
            .prepare(ReservedUpdate::Mutation(&request), guard.record().unwrap())
            .unwrap();
        assert!(server.requests.lock().unwrap().is_empty());
        let envelope = prepared.envelope().to_vec();
        let id = *prepared.request_id();
        fs::write(root.join("signed-ingress.cbor"), &envelope).unwrap();
        fs::write(root.join("request-id.txt"), id.to_string()).unwrap();
        let result = prepared.submit().await;
        if pending {
            assert!(matches!(result,Ok(UpdateOutcome::Pending{request_id}) if request_id==id));
        } else {
            assert!(
                matches!(result, Err(TransportError::Indeterminate)),
                "{label}"
            );
        }
        assert_eq!(fs::read(guard.path()).unwrap(), before);
        let view = guard.record().unwrap().view();
        assert_eq!(
            (
                view.mutations_used,
                view.mutations_remaining,
                view.pending_mutation
            ),
            (1, 0, Some(1))
        );
        drop(guard);
        let reopened =
            AttemptJournalGuard::open(&layout, &plan.attempt_authority(1).unwrap()).unwrap();
        assert_eq!(fs::read(reopened.path()).unwrap(), before);
        let requests = server.requests.lock().unwrap();
        assert_eq!(requests.len(), 1, "{label}");
        let bytes = &requests[0];
        let end = bytes.windows(4).position(|p| p == b"\r\n\r\n").unwrap();
        assert_eq!(&bytes[end + 4..], envelope);
        assert!(
            bytes.starts_with(format!("POST /api/v4/canister/{TARGET}/call HTTP/1.1").as_bytes())
        );
        fs::write(root.join("http-request.bin"), bytes).unwrap();
        drop(requests);
    }
}

#[tokio::test]
async fn context_and_stale_reservations_reject_before_network() {
    let server = Server::new("202 Accepted", vec![], false, false);
    let payload = support::payload(Method::StopCanister, TARGET, None);
    let plan = support::plan(TARGET, &payload.digest(), 1);
    let root = support::root("admission");
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let guard = support::retain(&layout, &plan, 1);
    let request = IcMutationRequest::new(&plan, 1, guard.record().unwrap(), &payload).unwrap();
    let changed = PlanContextRecord::new(&PlanContextRequest {
        network: "ef".repeat(32),
        caller: plan.context().caller().into(),
        release: plan.context().release().into(),
    })
    .unwrap();
    let transport = AgentTransport::new(
        changed,
        &server.endpoint,
        support::identity(),
        vec![1],
        Duration::from_secs(1),
    )
    .unwrap();
    assert!(matches!(
        transport.prepare(ReservedUpdate::Mutation(&request), guard.record().unwrap()),
        Err(TransportError::Context)
    ));
    let transport = AgentTransport::new(
        plan.context().clone(),
        &server.endpoint,
        support::identity(),
        vec![1],
        Duration::from_secs(1),
    )
    .unwrap();
    let untouched = ic_backup::model::attempt_journal::AttemptJournalRecord::new(
        plan.attempt_authority(1).unwrap(),
    );
    assert!(matches!(
        transport.prepare(ReservedUpdate::Mutation(&request), &untouched),
        Err(TransportError::Reservation)
    ));
    for endpoint in [
        "http://example.com/",
        "https://user:secret@example.com/",
        "https://example.com/path",
        "https://example.com/?query",
        "https://example.com/#fragment",
    ] {
        assert!(matches!(
            AgentTransport::new(
                plan.context().clone(),
                endpoint,
                support::identity(),
                vec![1],
                Duration::from_secs(1)
            ),
            Err(TransportError::Configuration)
        ));
    }
    assert!(server.requests.lock().unwrap().is_empty());
}
