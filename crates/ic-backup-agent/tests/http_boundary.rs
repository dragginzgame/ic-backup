//! Real local HTTP operations with controlled responses; no simulated management effects.
mod fence;
mod http_capture;
mod recovery;
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
use ic_backup_agent::AgentSnapshotUploadProvider;
use ic_backup_agent::{AgentMutationProvider, AgentSnapshotTransferReadProvider, PreparedUpdate};
use ic_backup_agent::{AgentTransport, MAX_HTTP_RESPONSE_BYTES, TransportError};

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

#[tokio::test]
async fn upload_provider_requires_signed_retention_and_keeps_metadata_or_data_pending() {
    use ic_backup::{
        model::{
            artifacts::ArtifactChecksumRecord,
            ic_snapshot_metadata::{IcSnapshotMetadataReply, IcSnapshotMetadataRequest},
            ic_snapshot_upload::{IcSnapshotUploadAttempt, IcSnapshotUploadRequest},
        },
        ports::ic_snapshot_upload::IcSnapshotUploadProvider,
    };
    use ic_management_canister_types::SnapshotDataKind;
    let read = IcSnapshotMetadataRequest::new(TARGET, &[0, 255, 17]).unwrap();
    // Exact SDK wire from Backup's existing "upload-source" codec vector. This HTTP
    // fixture proves transport/refusal, not authentic source or management effects.
    let metadata =
        IcSnapshotMetadataReply::decode(&read, include_bytes!("upload-metadata.candid")).unwrap();
    let source = support::plan(TARGET, &read.digest(), 11);
    let upload = IcSnapshotUploadRequest::metadata(
        &source,
        &metadata,
        &ArtifactChecksumRecord::from_bytes(b"declared tree"),
    )
    .unwrap();
    let data = IcSnapshotUploadRequest::data(
        &upload,
        &[9, 0, 255],
        SnapshotDataKind::WasmModule { offset: 0, size: 3 },
        &[7, 8, 9],
    )
    .unwrap();
    for payload in [&upload, &data] {
        for (label, refuse, discard) in [
            ("upload-retention-refused", true, false),
            ("upload-accepted", false, false),
            ("upload-lost", false, true),
        ] {
            let root = support::root(label);
            let mut server = Server::new(&root, "202 Accepted", vec![], discard, false);
            let plan = support::plan(TARGET, &payload.binding_digest(), 1);
            let layout = BackupLayoutGuard::acquire(&root).unwrap();
            let guard = support::retain(&layout, &plan, 1);
            let original = fs::read(guard.path()).unwrap();
            let request =
                IcSnapshotUploadAttempt::new(&plan, 1, guard.record().unwrap(), payload).unwrap();
            let transport = AgentTransport::new(
                plan.context().clone(),
                &server.endpoint,
                support::identity(),
                vec![1],
                Duration::from_secs(1),
            )
            .unwrap();
            let mut provider = AgentSnapshotUploadProvider::new(
                &transport,
                |actual: &IcSnapshotUploadAttempt<'_, '_>, prepared: &PreparedUpdate<'_>| {
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
                .submit_upload(&request, guard.record().unwrap())
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
            assert_eq!(fs::read(guard.path()).unwrap(), original);
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
}

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
    for (label, status, body, discard, delay) in [
        ("accepted", "202 Accepted", vec![], false, false),
        ("throttled", "429 Too Many Requests", vec![], false, false),
        (
            "unavailable",
            "503 Service Unavailable",
            vec![],
            false,
            false,
        ),
        ("redirect", "307 Temporary Redirect", vec![], false, false),
        ("malformed", "200 OK", vec![255], false, false),
        (
            "oversize",
            "200 OK",
            vec![0; MAX_HTTP_RESPONSE_BYTES + 1],
            false,
            false,
        ),
        ("lost", "200 OK", vec![], true, false),
        ("timeout", "200 OK", vec![], false, true),
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
        let mut envelope = None;
        let result = {
            let mut provider = AgentMutationProvider::new(
                &transport,
                |_: &IcMutationRequest<'_>, prepared: &PreparedUpdate<'_>| {
                    assert!(server.requests.lock().unwrap().is_empty());
                    support::retain_signed(&root, prepared)
                        .map_err(|_| IcMutationProviderError::Indeterminate)?;
                    envelope = Some(prepared.envelope().to_vec());
                    Ok(())
                },
            );
            provider
                .submit_mutation(&request, guard.record().unwrap())
                .await
        };
        let envelope = envelope.unwrap();
        server.finish().unwrap();
        assert!(
            matches!(result, Err(IcMutationProviderError::Indeterminate)),
            "{label}"
        );
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
    let mut provider = AgentMutationProvider::new(
        &transport,
        |_: &IcMutationRequest<'_>,
         _: &PreparedUpdate<'_>|
         -> Result<(), IcMutationProviderError> {
            panic!("changed context must reject before retention")
        },
    );
    assert!(matches!(
        provider
            .submit_mutation(&request, guard.record().unwrap())
            .await,
        Err(IcMutationProviderError::Unsupported)
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
    let mut provider = AgentMutationProvider::new(
        &transport,
        |_: &IcMutationRequest<'_>,
         _: &PreparedUpdate<'_>|
         -> Result<(), IcMutationProviderError> {
            panic!("stale reservation must reject before retention")
        },
    );
    assert!(matches!(
        provider.submit_mutation(&request, &untouched).await,
        Err(IcMutationProviderError::Unsupported)
    ));
    for endpoint in [
        "http://example.com/",
        " https://example.com/",
        "https://example.com ",
        "https://user:secret@example.com/",
        "https://example.com/path",
        "https://example.com/path/..",
        "https://example.com\\path\\..",
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
