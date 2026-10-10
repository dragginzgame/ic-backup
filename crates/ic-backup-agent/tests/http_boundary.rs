//! Real local HTTP operations with controlled responses; no simulated management effects.
mod http_capture;
mod support;
const TARGET: &str = "renrk-eyaaa-aaaaa-aaada-cai";
use ic_backup::ports::ic_mutation::{IcMutationProvider, IcMutationProviderError};
use ic_backup::{
    model::{
        ic_mutation::IcMutationRequest,
        ic_request::IcManagementMethodRecord as Method,
        operation_plan::{PlanContextRecord, PlanContextRequest},
    },
    ops::persistence::{AttemptJournalGuard, BackupLayoutGuard},
};
use ic_backup_agent::{AgentMutationProvider, AgentSnapshotTransferReadProvider, PreparedUpdate};
use ic_backup_agent::{
    AgentTransport, MAX_HTTP_RESPONSE_BYTES, ReservedUpdate, TransportError, UpdateOutcome,
};

#[tokio::test]
async fn async_provider_requires_retention_before_one_call_and_keeps_accepted_or_lost_pending() {
    for (label, refuse, discard) in [
        ("retention-refused", true, false),
        ("accepted", false, false),
        ("lost", false, true),
    ] {
        let root = support::root(label);
        let mut server = Server::new(&root, "202 Accepted", vec![], discard, false);
        let payload = support::payload(Method::StopCanister, TARGET, None);
        let plan = support::plan(TARGET, &payload.digest(), 1);
        let layout = BackupLayoutGuard::acquire(&root).unwrap();
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
        let mut provider = AgentMutationProvider::new(
            &transport,
            |actual: &IcMutationRequest<'_>, prepared: &PreparedUpdate<'_>| {
                assert_eq!(actual.authority().digest(), request.authority().digest());
                assert!(AttemptJournalGuard::open(&layout, actual.authority()).is_err());
                assert!(server.requests.lock().unwrap().is_empty());
                if refuse {
                    return Err(IcMutationProviderError::Unavailable);
                }
                support::retain_signed(&root, prepared).unwrap();
                Ok(())
            },
        );
        let result = provider
            .submit_mutation(&request, guard.record().unwrap())
            .await;
        assert_eq!(
            result.unwrap_err(),
            if refuse {
                IcMutationProviderError::Unavailable
            } else {
                IcMutationProviderError::Indeterminate
            }
        );
        server.finish().unwrap();
        assert_eq!(server.requests.lock().unwrap().len(), usize::from(!refuse));
        assert_eq!(fs::read(guard.path()).unwrap(), before);
        if !refuse {
            let requests = server.requests.lock().unwrap();
            let bytes = requests[0].complete_bytes(&root);
            let end = bytes.windows(4).position(|p| p == b"\r\n\r\n").unwrap();
            assert_eq!(
                &bytes[end + 4..],
                fs::read(root.join("signed-ingress.cbor")).unwrap()
            );
        }
    }
}
#[tokio::test]
async fn transfer_provider_retains_exact_ingress_before_one_call_and_keeps_failures_pending() {
    use ic_backup::{
        model::{
            ic_snapshot_metadata::IcSnapshotMetadataRequest,
            ic_snapshot_transfer_read::{
                IcSnapshotTransferReadPayload, IcSnapshotTransferReadRequest,
            },
        },
        ports::{
            ic_observation::IcObservationProviderError,
            ic_snapshot_transfer_read::IcSnapshotTransferReadProvider,
        },
    };
    for (label, refuse, discard) in [
        ("transfer-retention-refused", true, false),
        ("transfer-accepted", false, false),
        ("transfer-lost", false, true),
    ] {
        let root = support::root(label);
        let mut server = Server::new(&root, "202 Accepted", vec![], discard, false);
        let payload = IcSnapshotMetadataRequest::new(TARGET, &[0, 255, 17]).unwrap();
        let plan = support::plan(TARGET, &payload.digest(), 1);
        let layout = BackupLayoutGuard::acquire(&root).unwrap();
        let guard = support::retain(&layout, &plan, 1);
        let before = fs::read(guard.path()).unwrap();
        let request = IcSnapshotTransferReadRequest::new(
            &plan,
            1,
            guard.record().unwrap(),
            IcSnapshotTransferReadPayload::Metadata(&payload),
        )
        .unwrap();
        let transport = AgentTransport::new(
            plan.context().clone(),
            &server.endpoint,
            support::identity(),
            vec![1],
            Duration::from_secs(1),
        )
        .unwrap();
        let mut provider = AgentSnapshotTransferReadProvider::new(
            &transport,
            |actual: &IcSnapshotTransferReadRequest<'_, '_>, prepared: &PreparedUpdate<'_>| {
                assert_eq!(actual.authority().digest(), request.authority().digest());
                assert!(AttemptJournalGuard::open(&layout, actual.authority()).is_err());
                assert!(server.requests.lock().unwrap().is_empty());
                if refuse {
                    return Err(IcObservationProviderError::Unavailable);
                }
                support::retain_signed(&root, prepared).unwrap();
                Ok(())
            },
        );
        let result = provider
            .read_snapshot(&request, guard.record().unwrap())
            .await;
        assert_eq!(
            result.unwrap_err(),
            if refuse {
                IcObservationProviderError::Unavailable
            } else {
                IcObservationProviderError::Indeterminate
            }
        );
        server.finish().unwrap();
        assert_eq!(server.requests.lock().unwrap().len(), usize::from(!refuse));
        assert_eq!(fs::read(guard.path()).unwrap(), before);
        if !refuse {
            let requests = server.requests.lock().unwrap();
            let bytes = requests[0].complete_bytes(&root);
            let end = bytes.windows(4).position(|p| p == b"\r\n\r\n").unwrap();
            assert_eq!(
                &bytes[end + 4..],
                fs::read(root.join("signed-ingress.cbor")).unwrap()
            );
        }
    }
}

use std::{
    fs,
    io::{self, Write},
    net::TcpListener,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

struct Server {
    endpoint: String,
    requests: Arc<Mutex<Vec<http_capture::CapturedRequest>>>,
    done: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<io::Result<()>>>,
    root: PathBuf,
}
impl Server {
    fn new(root: &Path, status: &str, body: Vec<u8>, discard: bool, delay: bool) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let endpoint = format!("http://{}/", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let retained = requests.clone();
        let done = Arc::new(AtomicBool::new(false));
        let stopping = done.clone();
        let status = status.to_owned();
        let evidence = root.to_path_buf();
        let thread = thread::spawn(move || {
            while !stopping.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let capture = http_capture::capture(&mut stream, Duration::from_secs(3));
                        let complete = capture.failure.is_none();
                        let mut requests = retained
                            .lock()
                            .map_err(|_| io::Error::other("fixture request lock poisoned"))?;
                        capture.retain(&evidence, requests.len())?;
                        requests.push(capture);
                        drop(requests);
                        if !complete || discard {
                            continue;
                        }
                        if delay {
                            thread::sleep(Duration::from_millis(1200));
                        }
                        stream.set_write_timeout(Some(Duration::from_secs(3)))?;
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
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                    Err(error) => return Err(error),
                }
            }
            Ok(())
        });
        Self {
            endpoint,
            requests,
            done,
            thread: Some(thread),
            root: root.to_path_buf(),
        }
    }
    fn finish(&mut self) -> io::Result<()> {
        self.done.store(true, Ordering::Release);
        let result = match self.thread.take() {
            Some(thread) => thread
                .join()
                .map_err(|_| io::Error::other("fixture server thread panicked"))
                .and_then(|result| result),
            None => Ok(()),
        };
        if let Err(error) = &result {
            let _ = fs::write(
                self.root.join("http-server-cleanup-error.txt"),
                error.to_string(),
            );
        }
        result
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        // Cleanup may report a worker failure, but must preserve an original test
        // assertion while unwinding instead of causing a second panic/abort.
        if let Err(error) = self.finish() {
            eprintln!(
                "HTTP fixture cleanup failed: {error}; retained at {}",
                self.root.display()
            );
        }
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
        let mut server = Server::new(&root, status, body, discard, delay);
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
        support::retain_signed(&root, &prepared).unwrap();
        let result = prepared.submit().await;
        server.finish().unwrap();
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
        let bytes = requests[0].complete_bytes(&root);
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
    let root = support::root("admission");
    let mut server = Server::new(&root, "202 Accepted", vec![], false, false);
    let payload = support::payload(Method::StopCanister, TARGET, None);
    let plan = support::plan(TARGET, &payload.digest(), 1);
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
    server.finish().unwrap();
    assert!(server.requests.lock().unwrap().is_empty());
}

#[test]
fn worker_cleanup_failure_preserves_the_primary_unwind_and_retains_diagnosis() {
    let root = support::root("cleanup-unwind");
    let server = Server {
        endpoint: String::new(),
        requests: Arc::new(Mutex::new(vec![])),
        done: Arc::new(AtomicBool::new(false)),
        thread: Some(thread::spawn(|| panic!("controlled worker failure"))),
        root: root.clone(),
    };
    let primary = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _server = server;
        panic!("primary test assertion");
    }))
    .unwrap_err();
    assert_eq!(
        primary.downcast_ref::<&str>(),
        Some(&"primary test assertion")
    );
    assert_eq!(
        fs::read_to_string(root.join("http-server-cleanup-error.txt")).unwrap(),
        "fixture server thread panicked"
    );
}
