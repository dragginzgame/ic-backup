//! The real shared HTTP path with fixture signing/certificates; no IC effects.
use super::super::procedure_tests::{expected_request, reply, selected};
use super::*;
use crate::store::operation::intent::Intent;
use crate::store::{
    CaptureOperation, CapturePlan,
    capture::procedure::{Next, Procedure},
};
use crate::store::{
    capture::test_fixtures::{
        certificates::signed,
        http::{Response, Server, http, runtime},
        signatures::Scheme,
    },
    tests::root,
};
use ic_host_fs::Directory;
use ic_platform::{
    certificate::construction::{fork, labeled, leaf},
    ingress::{Content as EnvelopeContent, Envelope, decode_envelope, encode_envelope},
};
use std::{
    fs,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

struct ExternalSigner {
    calls: usize,
    refuse: bool,
    cancel: bool,
    orphan: Option<PathBuf>,
    revoke: Option<Arc<AtomicBool>>,
    acknowledge: Option<PathBuf>,
}
impl Signer for ExternalSigner {
    type Error = &'static str;
    fn protocol(&self) -> [u8; 32] {
        [2; 32]
    }
    async fn sign(&mut self, request: SigningRequest<'_>) -> Result<Vec<u8>, Self::Error> {
        self.calls += 1;
        if let Some(path) = &self.acknowledge {
            crate::store::tests::write_file(path, b"signing reserved");
        }
        if self.refuse {
            return Err("lost signature");
        }
        if self.cancel {
            std::future::pending::<()>().await;
        }
        if let Some(path) = &self.orphan {
            fs::write(path, b"retained orphan").unwrap();
        }
        if let Some(allowed) = &self.revoke {
            allowed.store(false, Ordering::SeqCst);
        }
        assert!(now().unwrap() < request.deadline_ns);
        assert_eq!(request.envelope_bytes, 4096);
        assert_eq!(request.network_requests, 0);
        let sender = request.plan.sender();
        let content = match request.signing {
            Signing::Call { method, arguments } => EnvelopeContent::Call {
                canister_id: ic_platform_types::Principal::management_canister()
                    .as_slice()
                    .try_into()
                    .unwrap(),
                method_name: method.into(),
                arg: arguments.into(),
                sender: sender.as_slice().try_into().unwrap(),
                ingress_expiry: request.expires_ns,
                nonce: Some(vec![self.calls as u8]),
                sender_info: None,
            },
            Signing::Observation { request_id } => EnvelopeContent::ReadState {
                sender: sender.as_slice().try_into().unwrap(),
                ingress_expiry: request.expires_ns,
                paths: vec![vec![b"request_status".to_vec(), request_id.to_vec()]],
            },
        };
        let signature = Scheme::Ed25519.sign(&content.signable().unwrap());
        Ok(encode_envelope(
            &Envelope {
                content,
                sender_pubkey: Some(Scheme::Ed25519.key()),
                sender_sig: Some(signature),
                sender_delegation: None,
            },
            request.envelope_bytes as usize,
        )
        .unwrap())
    }
}
struct Application {
    allowed: Arc<AtomicBool>,
    actions: Mutex<Vec<Action>>,
}
impl Guard for Application {
    type Error = &'static str;
    fn protocol(&self) -> [u8; 32] {
        [3; 32]
    }
    fn admit(
        &self,
        _: crate::store::OperationPlan<'_>,
        action: Action,
        witness: &[u8; 32],
    ) -> Result<(), Self::Error> {
        assert_eq!(witness, &[4; 32]);
        self.actions.lock().unwrap().push(action);
        if self.allowed.load(Ordering::SeqCst) {
            Ok(())
        } else {
            Err("revoked")
        }
    }
}
fn fixture(
    response: Response,
) -> (
    PathBuf,
    Directory,
    CaptureOperation,
    Connection<ExternalSigner, Application>,
    Server,
) {
    let path = root("capture-runtime");
    let parent = super::super::custody::tests::store(&path);
    let server = Server::new(response);
    let transport = server.client(selected().plan(), Duration::from_secs(5));
    let intent = selected();
    let mut profile = intent.profile().clone();
    profile.transport = transport.commitment();
    profile.prepare_duration_ns = 10_000_000_000;
    profile.submit_duration_ns = 20_000_000_000;
    profile.certificate_max_age_ns = 2_000_000_000;
    profile.delegation_max_age_ns = 2_000_000_000;
    let mut request = intent.plan().request().clone();
    request.limits.observations = 1;
    let recorder =
        CaptureOperation::create(&parent, CapturePlan::new(request, profile).unwrap()).unwrap();
    let connection = Connection {
        transport,
        signer: ExternalSigner {
            calls: 0,
            refuse: false,
            cancel: false,
            orphan: None,
            revoke: None,
            acknowledge: None,
        },
        guard: Application {
            allowed: Arc::new(AtomicBool::new(true)),
            actions: Mutex::new(Vec::new()),
        },
    };
    (path, parent, recorder, connection, server)
}
fn leb(mut n: u64) -> Vec<u8> {
    let mut bytes = Vec::new();
    loop {
        let next = (n & 127) as u8;
        n >>= 7;
        bytes.push(next | if n > 0 { 128 } else { 0 });
        if n == 0 {
            return bytes;
        }
    }
}
fn terminal_certificate(request_id: &[u8; 32], response: Vec<u8>, status: &[u8]) -> Vec<u8> {
    signed(
        fork(
            labeled(
                b"request_status",
                labeled(
                    request_id.to_vec(),
                    fork(
                        labeled(b"reply", leaf(response)),
                        labeled(b"status", leaf(status.to_vec())),
                    ),
                ),
            ),
            labeled(b"time", leaf(leb(now().unwrap()))),
        ),
        1,
        None,
    )
}
fn certificate_body(certificate: &[u8], call: bool) -> Vec<u8> {
    let mut bytes = if call {
        b"\xa2\x66status\x67replied\x6bcertificate\x59".to_vec()
    } else {
        b"\xa1\x6bcertificate\x59".to_vec()
    };
    bytes.extend_from_slice(&u16::try_from(certificate.len()).unwrap().to_be_bytes());
    bytes.extend_from_slice(certificate);
    http(200, "", &bytes)
}
fn envelope(request: &[u8]) -> &[u8] {
    let end = request.windows(4).position(|b| b == b"\r\n\r\n").unwrap() + 4;
    &request[end..]
}
fn response(request: &[u8], n: usize) -> Vec<u8> {
    let intent = selected();
    let content = decode_envelope(envelope(request), 4096)
        .unwrap()
        .value
        .content;
    let EnvelopeContent::Call {
        method_name, arg, ..
    } = &content
    else {
        panic!("call")
    };
    let step = match method_name.as_str() {
        "canister_status" => {
            if n == 0 {
                Step::InitialStatus
            } else {
                Step::FinalStatus
            }
        }
        "list_canister_snapshots" => Step::ListSnapshots,
        "take_canister_snapshot" => Step::TakeSnapshot,
        "read_canister_snapshot_metadata" => Step::Metadata,
        "read_canister_snapshot_data" => (0..9)
            .map(Step::Data)
            .find(|step| expected_request(&intent, *step).1 == *arg)
            .unwrap(),
        _ => panic!("unexpected method"),
    };
    assert_eq!(arg, &expected_request(&intent, step).1);
    assert!(
        request.starts_with(
            format!(
                "POST /api/v3/canister/{}/call ",
                intent.plan().request().target.canister.to_text()
            )
            .as_bytes()
        )
    );
    certificate_body(
        &terminal_certificate(
            &content.request_id().unwrap(),
            reply(&intent, step),
            b"replied",
        ),
        true,
    )
}

#[test]
fn fixed_runtime_acquires_multi_extent_evidence_and_cold_replays_without_live_inputs() {
    let (path, parent, mut recorder, mut connection, server) =
        fixture(Response::Computed(Box::new(|bytes, n| {
            Some(response(bytes, n))
        })));
    let rt = runtime();
    for _ in 0..14 {
        assert_eq!(
            rt.block_on(recorder.advance(&mut connection)).unwrap(),
            Progress::Advanced
        );
    }
    let Progress::Complete(summary) = rt.block_on(recorder.advance(&mut connection)).unwrap()
    else {
        panic!("complete")
    };
    assert_eq!(server.count(), 14);
    assert_eq!(connection.signer.calls, 14);
    assert_eq!(summary.payload_bytes, 23);
    assert_eq!(
        fs::read(
            path.join("bundles")
                .join(&summary.bundle_id)
                .join("payload/module.wasm")
        )
        .unwrap(),
        b"abcdefghij"
    );
    let actions = connection.guard.actions.lock().unwrap();
    assert!(actions.contains(&Action::TakeSnapshot));
    assert!(actions.contains(&Action::ReadSnapshot));
    drop(actions);
    let spent = recorder.check().unwrap();
    let id = recorder.intent.plan().request().operation_id.clone();
    let original = fs::read(path.join("operations").join(&id).join("journal.bin")).unwrap();
    drop(recorder);
    server.finish();
    connection.guard.allowed.store(false, Ordering::SeqCst);
    connection.signer.refuse = true;
    let mut reopened = CaptureOperation::open(&parent, &id).unwrap();
    assert_eq!(
        rt.block_on(reopened.advance(&mut connection)).unwrap(),
        Progress::Complete(summary)
    );
    assert_eq!(reopened.check().unwrap(), spent);
    assert_eq!(
        fs::read(path.join("operations").join(id).join("journal.bin")).unwrap(),
        original
    );
}

#[test]
fn admission_refuses_before_spending_and_lost_signing_never_repeats() {
    let (_, parent, mut recorder, mut connection, server) = fixture(Response::Disconnect);
    let rt = runtime();
    let initial = recorder.check().unwrap();
    connection.guard.allowed.store(false, Ordering::SeqCst);
    assert!(matches!(
        rt.block_on(recorder.advance(&mut connection)),
        Err(Error::Guard(_))
    ));
    assert_eq!(recorder.check().unwrap(), initial);
    connection.guard.allowed.store(true, Ordering::SeqCst);
    connection.signer.refuse = true;
    assert!(matches!(
        rt.block_on(recorder.advance(&mut connection)),
        Err(Error::Signer(_))
    ));
    assert_eq!(connection.signer.calls, 1);
    let id = recorder.intent.plan().request().operation_id.clone();
    let spent = recorder.check().unwrap();
    drop(recorder);
    let mut reopened = CaptureOperation::open(&parent, &id).unwrap();
    connection.guard.allowed.store(false, Ordering::SeqCst);
    assert_eq!(
        rt.block_on(reopened.advance(&mut connection)).unwrap(),
        Progress::NeedsAttention(Step::InitialStatus)
    );
    assert_eq!(connection.signer.calls, 1);
    assert_eq!(server.count(), 0);
    assert_eq!(reopened.check().unwrap(), spent);
}

#[test]
fn invocation_projection_never_outlives_changed_evidence_or_payload_across_http() {
    for payload in [false, true] {
        let location = Arc::new(Mutex::new(None::<PathBuf>));
        let mutation = Arc::clone(&location);
        let (path, _, mut operation, mut connection, server) =
            fixture(Response::Computed(Box::new(move |request, n| {
                if (!payload && n == 1) || (payload && n == 13) {
                    let file = mutation.lock().unwrap().clone().unwrap();
                    let mut bytes = fs::read(&file).unwrap();
                    *bytes.last_mut().unwrap() ^= 1;
                    fs::write(file, bytes).unwrap();
                }
                Some(response(request, n))
            })));
        let directory = path
            .join("operations")
            .join(&operation.plan().request().operation_id);
        *location.lock().unwrap() = Some(if payload {
            path.join("bundles")
                .join(&operation.plan().request().operation_id)
                .join("payload/module.wasm")
        } else {
            directory.join("initial-status-result.bin")
        });
        let mut refused = false;
        for _ in 0..20 {
            match runtime().block_on(operation.advance(&mut connection)) {
                Ok(Progress::Advanced) => (),
                Err(_) => {
                    refused = true;
                    break;
                }
                other => panic!("changed original must refuse before completion: {other:?}"),
            }
        }
        assert!(refused);
        assert_eq!(server.count(), if payload { 14 } else { 2 });
        assert_eq!(connection.signer.calls, server.count());
        assert_eq!(directory.join("final-status-result.bin").exists(), payload);
        assert!(!directory.join("list-snapshots-result.bin").exists() || payload);
        assert!(
            !path
                .join("bundles")
                .join(&operation.plan().request().operation_id)
                .join("manifest.json")
                .exists()
        );
    }
}

#[test]
fn mutation_and_revoked_guard_after_signing_cannot_dispatch() {
    for orphan in [false, true] {
        let (path, _, mut recorder, mut connection, server) = fixture(Response::Disconnect);
        if orphan {
            connection.signer.orphan = Some(
                path.join("operations")
                    .join(&recorder.intent.plan().request().operation_id)
                    .join("orphan.bin"),
            );
        } else {
            connection.signer.revoke = Some(Arc::clone(&connection.guard.allowed));
        }
        assert!(
            runtime()
                .block_on(recorder.advance(&mut connection))
                .is_err()
        );
        assert_eq!(connection.signer.calls, 1);
        assert_eq!(server.count(), 0);
        if !orphan {
            let journal = recorder.check().unwrap();
            assert_eq!(journal.signatures, 1);
            assert!(journal.calls[0].update.request.is_none());
        }
    }
}

#[test]
fn cancelled_signing_and_uncertain_http_return_attention_without_repeating() {
    for cancel in [false, true] {
        let (_, parent, mut recorder, mut connection, server) = fixture(Response::Disconnect);
        let rt = runtime();
        if cancel {
            connection.signer.cancel = true;
            assert!(
                rt.block_on(async {
                    tokio::time::timeout(
                        Duration::from_millis(50),
                        recorder.advance(&mut connection),
                    )
                    .await
                })
                .is_err()
            );
        } else {
            assert!(matches!(
                rt.block_on(recorder.advance(&mut connection)),
                Err(Error::Http(_))
            ));
        }
        let spent = recorder.check().unwrap();
        assert_eq!(spent.calls[0].update.submitted, !cancel);
        let id = recorder.intent.plan().request().operation_id.clone();
        drop(recorder);
        let mut reopened = CaptureOperation::open(&parent, &id).unwrap();
        assert_eq!(
            rt.block_on(reopened.advance(&mut connection)).unwrap(),
            Progress::NeedsAttention(Step::InitialStatus)
        );
        assert_eq!(reopened.check().unwrap(), spent);
        assert_eq!(connection.signer.calls, 1);
        assert_eq!(server.count(), usize::from(!cancel));
    }
}

#[test]
fn original_reply_import_is_local_after_shutdown_and_revoked_authority() {
    let retained = Arc::new(Mutex::new(None));
    let captured = Arc::clone(&retained);
    let (_, parent, mut recorder, mut connection, server) =
        fixture(Response::Computed(Box::new(move |request, _| {
            let content = decode_envelope(envelope(request), 4096)
                .unwrap()
                .value
                .content;
            let response = reply(&selected(), Step::InitialStatus);
            let proof =
                terminal_certificate(&content.request_id().unwrap(), response.clone(), b"replied");
            *captured.lock().unwrap() = Some(StatusEvidence {
                rejected: false,
                response,
                proof,
            });
            None
        })));
    let rt = runtime();
    assert!(rt.block_on(recorder.advance(&mut connection)).is_err());
    let id = recorder.intent.plan().request().operation_id.clone();
    drop(recorder);
    server.finish();
    connection.guard.allowed.store(false, Ordering::SeqCst);
    let mut reopened = CaptureOperation::open(&parent, &id).unwrap();
    reopened
        .reconcile_reply(&retained.lock().unwrap().take().unwrap().proof)
        .unwrap();
    assert_eq!(
        reopened
            .project(&reopened.check().unwrap(), &reopened.intent)
            .unwrap()
            .next()
            .unwrap(),
        Next::Call(Step::ListSnapshots)
    );
    assert_eq!(connection.signer.calls, 1);
}

#[test]
fn one_observation_resolves_pending_update_and_preserves_original_submission_count() {
    let original_id = Arc::new(Mutex::new(None));
    let captured = Arc::clone(&original_id);
    let (_, parent, mut recorder, mut connection, server) =
        fixture(Response::Computed(Box::new(move |request, _| {
            if let Ok(parsed) = decode_envelope(envelope(request), 4096)
                && let content @ EnvelopeContent::Call { .. } = parsed.value.content
            {
                *captured.lock().unwrap() = Some(content.request_id().unwrap());
                None
            } else {
                let content = decode_envelope(envelope(request), 4096)
                    .unwrap()
                    .value
                    .content;
                let EnvelopeContent::ReadState { paths, .. } = content else {
                    panic!("lookup")
                };
                let id = captured.lock().unwrap().unwrap();
                assert_eq!(paths[0][1].as_slice(), id);
                Some(certificate_body(
                    &terminal_certificate(&id, reply(&selected(), Step::InitialStatus), b"replied"),
                    false,
                ))
            }
        })));
    let rt = runtime();
    assert!(rt.block_on(recorder.advance(&mut connection)).is_err());
    assert_eq!(
        rt.block_on(recorder.advance(&mut connection)).unwrap(),
        Progress::NeedsAttention(Step::InitialStatus)
    );
    assert_eq!(
        rt.block_on(recorder.reconcile_observation(&mut connection))
            .unwrap(),
        Progress::Advanced
    );
    assert_eq!(server.count(), 2);
    assert_eq!(connection.signer.calls, 2);
    let spent = recorder.check().unwrap();
    assert_eq!(
        (spent.signatures, spent.network, spent.observations),
        (2, 2, 1)
    );
    assert!(spent.calls[0].update.result.is_none());
    let id = recorder.intent.plan().request().operation_id.clone();
    drop(recorder);
    server.finish();
    connection.guard.allowed.store(false, Ordering::SeqCst);
    let mut reopened = CaptureOperation::open(&parent, &id).unwrap();
    assert!(matches!(
        rt.block_on(reopened.reconcile_observation(&mut connection)),
        Err(Error::Resolved)
    ));
    assert_eq!(reopened.check().unwrap(), spent);
}

#[test]
fn prepared_call_resumes_exact_bytes_without_another_signature() {
    let (path, parent, mut recorder, mut connection, server) =
        fixture(Response::Computed(Box::new(|bytes, n| {
            Some(response(bytes, n))
        })));
    let reference = now().unwrap();
    let (method, arguments) = Procedure::new(&recorder.intent).unwrap().request().unwrap();
    recorder
        .append_fixture(
            FactRecord::CallReserved {
                step: Step::InitialStatus,
                reference_ns: reference,
            },
            None,
        )
        .unwrap();
    let bytes = runtime()
        .block_on(connection.sign(
            recorder.intent.view(),
            Action::Inspect,
            reference,
            Signing::Call {
                method,
                arguments: &arguments,
            },
        ))
        .unwrap();
    recorder
        .append_fixture(
            FactRecord::CallPrepared {
                step: Step::InitialStatus,
                request: ReferenceRecord::new(&bytes).unwrap(),
            },
            Some(&bytes),
        )
        .unwrap();
    let id = recorder.intent.plan().request().operation_id.clone();
    drop(recorder);
    connection.signer.refuse = true;
    let mut reopened = CaptureOperation::open(&parent, &id).unwrap();
    assert_eq!(
        runtime()
            .block_on(reopened.advance(&mut connection))
            .unwrap(),
        Progress::Advanced
    );
    assert_eq!(connection.signer.calls, 1);
    assert_eq!(server.count(), 1);
    assert_eq!(
        fs::read(
            path.join("operations")
                .join(id)
                .join(Step::InitialStatus.name(Role::Request))
        )
        .unwrap(),
        bytes
    );
}

#[test]
fn remainder_admission_includes_signing_network_evidence_and_local_completion() {
    let intent = selected();
    let bytes = Journal::create(&intent).unwrap();
    let journal = Journal::open(bytes.as_slice(), &intent).unwrap();
    journal.remaining(14).unwrap();
    assert!(journal.remaining(21).is_err());
    for resource in 0..3 {
        let mut journal = journal.clone();
        match resource {
            0 => journal.allowance.signer_network = 1,
            1 => journal.allowance.evidence = 1024 * 1024,
            _ => journal.frames_reserved = FRAMES - 4,
        }
        assert!(journal.remaining(14).is_err());
    }
}

#[test]
fn changed_transport_profile_refuses_before_reservation_or_signing() {
    let (_, _, mut recorder, mut connection, server) = fixture(Response::Disconnect);
    let before = recorder.check().unwrap();
    connection.transport = server.client(selected().plan(), Duration::from_secs(1));
    assert!(matches!(
        runtime().block_on(recorder.advance(&mut connection)),
        Err(Error::Binding)
    ));
    assert_eq!(recorder.check().unwrap(), before);
    assert_eq!(connection.signer.calls, 0);
    assert_eq!(server.count(), 0);
}

#[test]
fn certified_policy_failure_is_retained_and_blocks_cold_continuation() {
    let (_, parent, mut recorder, mut connection, server) =
        fixture(Response::Computed(Box::new(|request, _| {
            let content = decode_envelope(envelope(request), 4096)
                .unwrap()
                .value
                .content;
            let mut running = crate::store::capture::status::tests::status(selected().plan());
            running.status = ic_platform_protocol::management_snapshots::Status::Running;
            let response = crate::store::capture::test_fixtures::wire::encode(&running, 4096);
            Some(certificate_body(
                &terminal_certificate(&content.request_id().unwrap(), response, b"replied"),
                true,
            ))
        })));
    let rt = runtime();
    assert!(matches!(
        rt.block_on(recorder.advance(&mut connection)),
        Err(Error::Replay(_))
    ));
    let spent = recorder.check().unwrap();
    assert!(spent.calls[0].update.result.is_some());
    let id = recorder.intent.plan().request().operation_id.clone();
    drop(recorder);
    let mut reopened = CaptureOperation::open(&parent, &id).unwrap();
    assert!(matches!(
        rt.block_on(reopened.advance(&mut connection)),
        Err(Error::Replay(_))
    ));
    assert_eq!(reopened.check().unwrap(), spent);
    assert_eq!((connection.signer.calls, server.count()), (1, 1));
}

#[test]
fn process_child() {
    let Some(control) = std::env::var_os("IC_BACKUP_RECORDER_CHILD") else {
        return;
    };
    let control = PathBuf::from(control);
    let (path, _, mut operation, mut connection, _server) = fixture(Response::Disconnect);
    fs::write(
        control.join("store-path"),
        path.as_os_str().as_encoded_bytes(),
    )
    .unwrap();
    connection.signer.acknowledge = Some(control.join("ack"));
    connection.signer.cancel = true;
    runtime()
        .block_on(operation.advance(&mut connection))
        .unwrap();
    panic!("reserved signing must remain pending until the parent kills this process");
}

#[test]
fn certified_rejection_remains_settled_without_replacement_or_live_replay() {
    let (path, parent, mut operation, mut connection, server) =
        fixture(Response::Computed(Box::new(|request, _| {
            let content = decode_envelope(envelope(request), 4096)
                .unwrap()
                .value
                .content;
            let certificate = signed(
                fork(
                    labeled(
                        b"request_status",
                        labeled(
                            content.request_id().unwrap().to_vec(),
                            fork(
                                fork(
                                    labeled(b"reject_code", leaf(vec![5])),
                                    labeled(b"reject_message", leaf(b"fixture rejection".to_vec())),
                                ),
                                labeled(b"status", leaf(b"rejected".to_vec())),
                            ),
                        ),
                    ),
                    labeled(b"time", leaf(leb(now().unwrap()))),
                ),
                1,
                None,
            );
            Some(certificate_body(&certificate, true))
        })));
    assert!(matches!(
        runtime().block_on(operation.advance(&mut connection)),
        Err(Error::Replay(_))
    ));
    let spent = operation.check().unwrap();
    assert!(spent.calls[0].update.result.is_some());
    let id = operation.plan().request().operation_id.clone();
    let journal = fs::read(path.join("operations").join(&id).join("journal.bin")).unwrap();
    let evidence = operation.read_role(&spent.calls[0], Role::Result).unwrap();
    assert!(StatusEvidence::decode(&evidence).unwrap().rejected);
    drop(operation);
    drop(server);
    connection.guard.allowed.store(false, Ordering::SeqCst);
    let mut operation = CaptureOperation::open(&parent, &id).unwrap();
    assert!(matches!(
        runtime().block_on(operation.advance(&mut connection)),
        Err(Error::Replay(_))
    ));
    assert_eq!(operation.check().unwrap(), spent);
    assert_eq!(
        fs::read(path.join("operations").join(id).join("journal.bin")).unwrap(),
        journal
    );
    assert_eq!(
        operation.read_role(&spent.calls[0], Role::Result).unwrap(),
        evidence
    );
    assert_eq!(connection.signer.calls, 1);
}

#[test]
fn acknowledged_process_death_keeps_original_intent_claim_and_spending() {
    use std::{
        process::{Child, Command, Stdio},
        thread,
        time::Instant,
    };
    struct ChildGuard(Child);
    impl Drop for ChildGuard {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let control = root("recorder-process-death");
    let mut child = ChildGuard(
        Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "store::operation::runtime::tests::process_child",
                "--nocapture",
            ])
            .env("IC_BACKUP_RECORDER_CHILD", &control)
            .stdout(Stdio::from(
                fs::File::create(control.join("child.stdout")).unwrap(),
            ))
            .stderr(Stdio::from(
                fs::File::create(control.join("child.stderr")).unwrap(),
            ))
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(10);
    while fs::read(control.join("ack")).ok().as_deref() != Some(b"signing reserved".as_slice()) {
        assert!(
            child.0.try_wait().unwrap().is_none(),
            "child exited before signing acknowledgement"
        );
        assert!(
            Instant::now() < deadline,
            "child did not acknowledge the durable reservation"
        );
        thread::sleep(Duration::from_millis(10));
    }
    let path =
        PathBuf::from(String::from_utf8(fs::read(control.join("store-path")).unwrap()).unwrap());
    let store = crate::store::BackupStore::open(&path).unwrap();
    let id = selected().request().operation_id.clone();
    let bytes = fs::read(path.join("operations").join(&id).join("intent.json")).unwrap();
    let intent = CapturePlan::decode(&bytes).unwrap();
    assert!(store.open_capture(&id, intent.digest()).is_err());
    child.0.kill().unwrap();
    child.0.wait().unwrap();
    let mut operation = store.open_capture(&id, intent.digest()).unwrap();
    let before = operation.check().unwrap();
    assert_eq!((before.signatures, before.network), (1, 1));
    assert!(before.calls[0].update.request.is_none());
    let (_, _, _, mut connection, server) = fixture(Response::Disconnect);
    connection.guard.allowed.store(false, Ordering::SeqCst);
    assert_eq!(
        runtime()
            .block_on(operation.advance(&mut connection))
            .unwrap(),
        Progress::NeedsAttention(Step::InitialStatus)
    );
    assert_eq!(operation.check().unwrap(), before);
    assert_eq!(operation.plan().encoded(), bytes);
    assert_eq!((connection.signer.calls, server.count()), (0, 0));
    let mut request = intent.request().clone();
    request.operation_id = "ff".repeat(16);
    let other = CapturePlan::new(request, intent.profile().clone()).unwrap();
    assert!(matches!(
        store.create_capture(other.clone(), other.digest()),
        Err(super::super::replay::Error::Capture(
            OperationStoreError::Occupied
        ))
    ));
}

#[test]
fn orphan_appearing_during_http_refuses_result_retention_without_repeating() {
    let location = Arc::new(Mutex::new(None::<PathBuf>));
    let captured = Arc::clone(&location);
    let (path, parent, mut recorder, mut connection, server) =
        fixture(Response::Computed(Box::new(move |bytes, n| {
            crate::store::tests::write_file(
                captured.lock().unwrap().as_ref().unwrap(),
                b"original orphan",
            );
            Some(response(bytes, n))
        })));
    let id = recorder.intent.plan().request().operation_id.clone();
    let orphan = path.join("operations").join(&id).join("orphan.bin");
    *location.lock().unwrap() = Some(orphan.clone());
    let error = runtime()
        .block_on(recorder.advance(&mut connection))
        .unwrap_err();
    assert!(matches!(error, Error::AfterResponse { .. }));
    assert!(
        !error
            .received_certificate()
            .unwrap()
            .certificate()
            .is_empty()
    );
    drop(recorder);
    assert!(CaptureOperation::open(&parent, &id).is_err());
    assert_eq!(fs::read(orphan).unwrap(), b"original orphan");
    assert_eq!((connection.signer.calls, server.count()), (1, 1));
}

// Legacy AfterReply/retention cases adapted to certificates and the common recorder.
#[test]
fn failed_result_write_returns_original_evidence_for_offline_import_without_new_allowance() {
    use std::os::unix::fs::PermissionsExt;
    for observation in [false, true] {
        let location = Arc::new(Mutex::new(None::<PathBuf>));
        let fault = Arc::clone(&location);
        let original = Arc::new(Mutex::new(None));
        let wire_id = Arc::clone(&original);
        let (path, parent, mut operation, mut connection, server) =
            fixture(Response::Computed(Box::new(move |request, _| {
                if let Ok(parsed) = decode_envelope(envelope(request), 4096)
                    && let content @ EnvelopeContent::Call { .. } = parsed.value.content
                {
                    *wire_id.lock().unwrap() = Some(content.request_id().unwrap());
                    if observation {
                        return None;
                    }
                }
                fs::set_permissions(
                    fault.lock().unwrap().as_ref().unwrap(),
                    fs::Permissions::from_mode(0o500),
                )
                .unwrap();
                Some(certificate_body(
                    &terminal_certificate(
                        &wire_id.lock().unwrap().unwrap(),
                        reply(&selected(), Step::InitialStatus),
                        b"replied",
                    ),
                    !observation,
                ))
            })));
        let id = operation.plan().request().operation_id.clone();
        let directory = path.join("operations").join(&id);
        *location.lock().unwrap() = Some(directory.clone());
        let rt = runtime();
        if observation {
            assert!(rt.block_on(operation.advance(&mut connection)).is_err());
        }
        let error = if observation {
            rt.block_on(operation.reconcile_observation(&mut connection))
        } else {
            rt.block_on(operation.advance(&mut connection))
        }
        .unwrap_err();
        let received = error.received_certificate().unwrap();
        assert_eq!(received.request_id(), original.lock().unwrap().unwrap());
        let proof = received.certificate().to_vec();
        assert_eq!(
            format!("{received:?}"),
            format!("ReceivedCertificate {{ bytes: {}, .. }}", proof.len())
        );
        assert!(
            !directory
                .join(Step::InitialStatus.name(if observation {
                    Role::ObservationResult
                } else {
                    Role::Result
                }))
                .exists()
        );
        drop(operation);
        server.finish();
        connection.guard.allowed.store(false, Ordering::SeqCst);
        // Restore access only; original files, ownership and bytes remain unchanged.
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o700)).unwrap();
        let mut operation = CaptureOperation::open(&parent, &id).unwrap();
        let before = operation.check().unwrap();
        assert!(operation.reconcile_observation_reply(b"invalid").is_err());
        assert_eq!(operation.check().unwrap(), before);
        if observation {
            operation.reconcile_observation_reply(&proof).unwrap();
        } else {
            operation.reconcile_reply(&proof).unwrap();
        }
        assert!(operation.reconcile_observation_reply(&proof).is_err());
        let view = operation.inspect().unwrap();
        assert_eq!(view.next_call, Some(Step::ListSnapshots));
        assert_eq!(view.signing_requests_reserved, before.signatures);
        assert_eq!(view.network_requests_reserved, before.network);
        assert_eq!(view.observations_reserved, u32::from(observation));
        assert_eq!(connection.signer.calls, 1 + usize::from(observation));
    }
}

#[test]
fn nonterminal_lookup_is_spent_and_surviving_original_reply_can_still_resolve() {
    let retained = Arc::new(Mutex::new(None));
    let captured = Arc::clone(&retained);
    let (_, parent, mut recorder, mut connection, server) =
        fixture(Response::Computed(Box::new(move |request, _| {
            if let Ok(parsed) = decode_envelope(envelope(request), 4096)
                && let content @ EnvelopeContent::Call { .. } = parsed.value.content
            {
                let response = reply(&selected(), Step::InitialStatus);
                let proof = terminal_certificate(
                    &content.request_id().unwrap(),
                    response.clone(),
                    b"replied",
                );
                *captured.lock().unwrap() = Some((
                    content.request_id().unwrap(),
                    StatusEvidence {
                        rejected: false,
                        response,
                        proof,
                    },
                ));
                None
            } else {
                let id = captured.lock().unwrap().as_ref().unwrap().0;
                Some(certificate_body(
                    &terminal_certificate(&id, vec![], b"processing"),
                    false,
                ))
            }
        })));
    let rt = runtime();
    assert!(rt.block_on(recorder.advance(&mut connection)).is_err());
    assert_eq!(
        rt.block_on(recorder.reconcile_observation(&mut connection))
            .unwrap(),
        Progress::NeedsAttention(Step::InitialStatus)
    );
    let spent = recorder.check().unwrap();
    let id = recorder.intent.plan().request().operation_id.clone();
    drop(recorder);
    let mut reopened = CaptureOperation::open(&parent, &id).unwrap();
    assert_eq!(
        rt.block_on(reopened.reconcile_observation(&mut connection))
            .unwrap(),
        Progress::NeedsAttention(Step::InitialStatus)
    );
    assert_eq!(reopened.check().unwrap(), spent);
    assert_eq!((connection.signer.calls, server.count()), (2, 2));
    server.finish();
    connection.guard.allowed.store(false, Ordering::SeqCst);
    reopened
        .reconcile_reply(&retained.lock().unwrap().take().unwrap().1.proof)
        .unwrap();
    let settled = reopened.check().unwrap();
    assert_eq!(
        (settled.signatures, settled.network, settled.observations),
        (2, 2, 1)
    );
    assert!(
        settled.calls[0]
            .observation
            .as_ref()
            .unwrap()
            .result
            .is_some()
    );
}

#[test]
fn interrupted_observation_signing_never_renews_after_reopen() {
    let (_, parent, mut recorder, mut connection, server) = fixture(Response::Disconnect);
    let rt = runtime();
    assert!(rt.block_on(recorder.advance(&mut connection)).is_err());
    connection.signer.cancel = true;
    assert!(
        rt.block_on(async {
            tokio::time::timeout(
                Duration::from_millis(50),
                recorder.reconcile_observation(&mut connection),
            )
            .await
        })
        .is_err()
    );
    let spent = recorder.check().unwrap();
    assert!(
        spent.calls[0]
            .observation
            .as_ref()
            .unwrap()
            .request
            .is_none()
    );
    let id = recorder.intent.plan().request().operation_id.clone();
    drop(recorder);
    let mut reopened = CaptureOperation::open(&parent, &id).unwrap();
    assert_eq!(
        rt.block_on(reopened.reconcile_observation(&mut connection))
            .unwrap(),
        Progress::NeedsAttention(Step::InitialStatus)
    );
    assert_eq!(reopened.check().unwrap(), spent);
    assert_eq!((connection.signer.calls, server.count()), (2, 1));
}

#[test]
fn expired_prepared_request_cannot_receive_a_later_time_or_new_signature() {
    let (_, parent, mut recorder, mut connection, server) = fixture(Response::Disconnect);
    let reference = now().unwrap() - 25_000_000_000;
    let admission = recorder.intent.view().admission(reference).unwrap();
    let (method, arguments) = Procedure::new(&recorder.intent).unwrap().request().unwrap();
    let bytes = runtime()
        .block_on(connection.signer.sign(SigningRequest {
            plan: recorder.intent.view(),
            signing: Signing::Call {
                method,
                arguments: &arguments,
            },
            deadline_ns: now().unwrap() + 1_000_000_000,
            expires_ns: admission.submit_deadline_ns,
            envelope_bytes: 4096,
            network_requests: 0,
        }))
        .unwrap();
    recorder
        .append_fixture(
            FactRecord::CallReserved {
                step: Step::InitialStatus,
                reference_ns: reference,
            },
            None,
        )
        .unwrap();
    recorder
        .append_fixture(
            FactRecord::CallPrepared {
                step: Step::InitialStatus,
                request: ReferenceRecord::new(&bytes).unwrap(),
            },
            Some(&bytes),
        )
        .unwrap();
    let original = recorder.check().unwrap();
    let id = recorder.intent.plan().request().operation_id.clone();
    drop(recorder);
    let mut reopened = CaptureOperation::open(&parent, &id).unwrap();
    assert!(matches!(
        runtime().block_on(reopened.advance(&mut connection)),
        Err(Error::Deadline)
    ));
    assert_eq!(reopened.check().unwrap(), original);
    assert_eq!((connection.signer.calls, server.count()), (1, 0));
}

#[test]
fn partial_materialization_is_retained_and_cannot_unlock_final_status() {
    let (path, parent, mut recorder, mut connection, server) =
        fixture(Response::Computed(Box::new(|bytes, n| {
            Some(response(bytes, n))
        })));
    let rt = runtime();
    for _ in 0..13 {
        assert_eq!(
            rt.block_on(recorder.advance(&mut connection)).unwrap(),
            Progress::Advanced
        );
    }
    recorder
        .append_fixture(FactRecord::MaterializationStarted, None)
        .unwrap();
    let bundles = crate::store::capture::child(&parent, "bundles", false).unwrap();
    let id = recorder.intent.plan().request().operation_id.clone();
    let bundle = crate::store::capture::child(&bundles, &id, true).unwrap();
    let payload = crate::store::capture::child(&bundle, "payload", true).unwrap();
    crate::store::capture::retain(&payload, "module.wasm", b"abc").unwrap();
    assert!(rt.block_on(recorder.advance(&mut connection)).is_err());
    assert!(matches!(
        recorder.check(),
        Err(OperationStoreError::Unusable)
    ));
    drop(recorder);
    let mut reopened = CaptureOperation::open(&parent, &id).unwrap();
    assert!(rt.block_on(reopened.advance(&mut connection)).is_err());
    assert_eq!(
        fs::read(path.join("bundles").join(id).join("payload/module.wasm")).unwrap(),
        b"abc"
    );
    assert_eq!((connection.signer.calls, server.count()), (13, 13));
}

#[test]
fn interrupted_publication_reconciles_exact_readiness_without_new_effects() {
    let (path, parent, mut recorder, mut connection, server) =
        fixture(Response::Computed(Box::new(|bytes, n| {
            Some(response(bytes, n))
        })));
    let rt = runtime();
    for _ in 0..14 {
        assert_eq!(
            rt.block_on(recorder.advance(&mut connection)).unwrap(),
            Progress::Advanced
        );
    }
    recorder
        .append_fixture(
            FactRecord::PublishReady {
                completed_at_ns: 987,
            },
            None,
        )
        .unwrap();
    let journal = recorder.check().unwrap();
    let flow = recorder.project(&journal, &recorder.intent).unwrap();
    let bytes = recorder.manifest_bytes(&journal, &flow).unwrap();
    let id = recorder.intent.plan().request().operation_id.clone();
    let bundle = path.join("bundles").join(&id);
    crate::store::tests::write_file(&bundle.join("manifest.pending"), &bytes);
    let original_manifest = fs::File::open(bundle.join("manifest.pending")).unwrap();
    let original_identity = ic_host_fs::FileId::from_file(&original_manifest).unwrap();
    drop(recorder);
    server.finish();
    connection.guard.allowed.store(false, Ordering::SeqCst);
    connection.signer.refuse = true;
    let mut reopened = CaptureOperation::open(&parent, &id).unwrap();
    let Progress::Complete(summary) = rt.block_on(reopened.advance(&mut connection)).unwrap()
    else {
        panic!("completion")
    };
    assert_eq!(summary.completed_at_ns, 987);
    assert_eq!(fs::read(bundle.join("manifest.json")).unwrap(), bytes);
    assert!(!bundle.join("manifest.pending").exists());
    assert_eq!(
        ic_host_fs::FileId::from_file(&fs::File::open(bundle.join("manifest.json")).unwrap())
            .unwrap(),
        original_identity,
    );
    assert_eq!(reopened.check().unwrap(), journal);
    assert_eq!(connection.signer.calls, 14);
    assert_eq!(
        rt.block_on(reopened.advance(&mut connection)).unwrap(),
        Progress::Complete(summary)
    );
    assert_eq!(reopened.check().unwrap(), journal);
}

#[test]
fn partial_manifest_and_changed_completed_payload_refuse_without_repair() {
    for partial_manifest in [false, true] {
        let (path, parent, mut recorder, mut connection, server) =
            fixture(Response::Computed(Box::new(|bytes, n| {
                Some(response(bytes, n))
            })));
        let rt = runtime();
        for _ in 0..14 {
            assert_eq!(
                rt.block_on(recorder.advance(&mut connection)).unwrap(),
                Progress::Advanced
            );
        }
        let id = recorder.intent.plan().request().operation_id.clone();
        let bundle = path.join("bundles").join(&id);
        let damaged = if partial_manifest {
            recorder
                .append_fixture(
                    FactRecord::PublishReady {
                        completed_at_ns: 999,
                    },
                    None,
                )
                .unwrap();
            crate::store::tests::write_file(&bundle.join("manifest.pending"), b"partial");
            bundle.join("manifest.pending")
        } else {
            assert!(matches!(
                rt.block_on(recorder.advance(&mut connection)).unwrap(),
                Progress::Complete(_)
            ));
            fs::write(bundle.join("payload/module.wasm"), b"abcdefghik").unwrap();
            bundle.join("payload/module.wasm")
        };
        let original = fs::read(&damaged).unwrap();
        let journal = recorder.check().unwrap();
        drop(recorder);
        server.finish();
        connection.guard.allowed.store(false, Ordering::SeqCst);
        let mut reopened = CaptureOperation::open(&parent, &id).unwrap();
        assert!(rt.block_on(reopened.advance(&mut connection)).is_err());
        assert_eq!(reopened.check().unwrap(), journal);
        assert_eq!(fs::read(damaged).unwrap(), original);
        assert_eq!(connection.signer.calls, 14);
    }
}
