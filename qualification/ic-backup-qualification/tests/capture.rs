//! All keys, application permissions, fixture memory and proxying remain test-only.
//! Actual-IC public-API tests, including a separately built normal-dependency consumer.

mod consumer;
mod proxy;
mod server;

use authority::{FixtureGuard, FixtureSigner, der, now};
use ic_backup::store::{
    BackupConnection, BackupDigest, BackupHttpTransport, BackupStore, BackupSummary, BackupTarget,
    CaptureRequest, CaptureTiming, OperationLimits, OperationOptions, OperationPlan,
    OperationProgress, OperationStep, RestoreRequest, ScanLimits,
};
use ic_backup_consumer::authority;
use ic_platform_protocol::management_snapshots::{ReadDataArgs, ReadKind};
use ic_platform_types::Principal;
use ic_testkit::{
    pocket_ic::{CanisterSettings, PocketIcBuilder, common::rest::BlobCompression},
    server::PreparedServer,
};
use std::{
    fs,
    os::unix::fs::DirBuilderExt,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::{Duration, SystemTime},
};

type Connection = BackupConnection<FixtureSigner, FixtureGuard>;

///
/// Mode
/// Selects the capture or recovery journey to qualify.
///

#[derive(Clone, Copy, Debug)]
enum Mode {
    Complete,
    Capacity,
    Isolated,
    Boundary,
    WriteFailure,
    TakeObservation,
    DataReply,
    RestoreAllocation,
    RestoreAllocationObservation,
    RestoreUpload,
    RestoreLoad,
}

#[test]
fn twenty_mib_capture_restore_interrupted_transfer_and_offline_replay() {
    run(Mode::Capacity);
}

#[test]
fn isolated_normal_consumer_continues_capture_restore_and_replays_in_new_processes() {
    run(Mode::Isolated);
}

#[test]
fn full_mib_chunk_captures_and_restores_through_complete_envelopes_and_proofs() {
    run(Mode::Boundary);
}

#[test]
fn load_certificate_survives_local_write_failure_and_settles_offline() {
    run(Mode::WriteFailure);
}

#[test]
fn complete_multi_extent_capture_restore_succession_and_offline_replay() {
    run(Mode::Complete);
}

#[test]
fn interrupted_take_observes_once_then_replays_without_server_or_live_authority() {
    run(Mode::TakeObservation);
}

#[test]
fn surviving_payload_certificate_settles_original_call_after_shutdown() {
    run(Mode::DataReply);
}

#[test]
fn original_allocation_reply_settles_offline_without_reallocating() {
    run(Mode::RestoreAllocation);
}

#[test]
fn original_mid_upload_reply_settles_offline_without_uploading_again() {
    run(Mode::RestoreUpload);
}

#[test]
fn original_load_reply_settles_offline_without_rewinding_again() {
    run(Mode::RestoreLoad);
}

#[test]
fn interrupted_allocation_observes_once_then_replays_offline() {
    run(Mode::RestoreAllocationObservation);
}

fn run(mode: Mode) {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()
        .unwrap();
    let assets = std::env::var_os("IC_BACKUP_TESTKIT_DIRECTORY")
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace.parent().unwrap().join(".tools/testkit-server"));
    let prepared = PreparedServer::check(&assets).expect("run make install-testkit-server");
    let evidence = workspace.join("target/fixtures").join(format!(
        "capture-{mode:?}-{}-{}",
        std::process::id(),
        now()
    ));
    fs::create_dir_all(&evidence).unwrap();
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(exercise(&evidence, prepared, mode));
}

fn leb(mut value: u64) -> Vec<u8> {
    let mut out = Vec::new();

    loop {
        let byte = (value & 127) as u8;
        value >>= 7;
        out.push(byte | if value == 0 { 0 } else { 128 });

        if value == 0 {
            return out;
        }
    }
}

fn wasm(heap: &[u8]) -> Vec<u8> {
    let mut bytes = b"\0asm\x01\0\0\0".to_vec();
    bytes.extend_from_slice(&[5, 3, 1, 0, 1]); // one memory, one page
    bytes.extend_from_slice(b"\x07\x0a\x01\x06memory\x02\0");
    let mut data = vec![1, 0, 0x41, 0, 0x0b];
    data.extend_from_slice(&leb(heap.len() as u64));
    data.extend_from_slice(heap);
    bytes.push(11);
    bytes.extend_from_slice(&leb(data.len() as u64));
    bytes.extend_from_slice(&data);

    bytes
}

fn options() -> OperationOptions {
    OperationOptions {
        application: [3; 32],
        snapshot_capacity: 4,
        prepare_duration_ns: 30_000_000_000,
        submit_duration_ns: 120_000_000_000,
        timing: CaptureTiming {
            certificate_max_age_ns: 60_000_000_000,
            delegation_max_age_ns: 86_400_000_000_000,
            max_clock_skew_ns: 1_000_000_000,
        },
        signer_network: 0,
        request_bytes: 8192,
        result_bytes: 2 * 1024 * 1024,
    }
}

fn scan() -> ScanLimits {
    ScanLimits {
        entries: 10.try_into().unwrap(),
        bytes: (4 * 1024 * 1024).try_into().unwrap(),
        results: 10.try_into().unwrap(),
    }
}

async fn complete(
    operation: &mut ic_backup::store::CaptureOperation,
    connection: &mut Connection,
) -> BackupSummary {
    for _ in 0..128 {
        match operation.advance(connection).await.unwrap() {
            OperationProgress::Advanced => (),
            OperationProgress::Complete(receipt) => return *receipt,
            OperationProgress::NeedsAttention(step) => panic!("unexpected pending {step:?}"),
        }
    }

    panic!("fixed capture did not complete within original call allowance")
}

async fn exercise(evidence: &Path, prepared: PreparedServer, mode: Mode) {
    let capacity = matches!(mode, Mode::Capacity);
    let mut server = server::RetainedServer::start(
        prepared,
        evidence,
        Duration::from_secs(if capacity { 1800 } else { 600 }),
    );
    let mut owned = server
        .server()
        .instance(
            PocketIcBuilder::new()
                .with_nns_subnet()
                .with_application_subnet()
                .with_initial_time(SystemTime::now().into()),
            Duration::from_secs(30),
        )
        .await
        .unwrap();
    let sender = Principal::self_authenticating(der());
    let native_sender = sender.as_slice().try_into().unwrap();
    let pic = owned.client_mut();
    let canister = pic
        .create_canister_with_settings(
            Some(native_sender),
            Some(CanisterSettings {
                controllers: Some(vec![native_sender]),
                ..CanisterSettings::default()
            }),
        )
        .await;
    let heap: Vec<_> = (0..65536).map(|n| (n % 251) as u8).collect();
    let stable: Vec<_> = (0..if capacity { 19 * 1024 * 1024 } else { 131072 })
        .map(|n| ((n * 3) % 251) as u8)
        .collect();
    let wasm = wasm(&heap);
    pic.install_canister(canister, wasm.clone(), vec![], Some(native_sender))
        .await;
    pic.set_stable_memory(canister, stable.clone(), BlobCompression::NoCompression)
        .await;
    let boundary = matches!(mode, Mode::Boundary | Mode::Capacity);
    let chunks = [
        vec![],
        if capacity {
            vec![0xa7; 20 * 1024 * 1024 - wasm.len() - heap.len() - stable.len()]
        } else if boundary {
            vec![0xa7; 1024 * 1024]
        } else {
            vec![0, 255, 17]
        },
    ];
    for bytes in &chunks {
        assert_eq!(
            pic.upload_chunk(canister, Some(native_sender), bytes.clone())
                .await
                .unwrap(),
            BackupDigest::compute(bytes).as_bytes()
        );
    }
    pic.stop_canister(canister, Some(native_sender))
        .await
        .unwrap();
    let existing = pic
        .take_canister_snapshot(canister, Some(native_sender), None)
        .await
        .unwrap();
    assert_eq!(pic.get_stable_memory(canister).await, stable);
    let trust = pic.root_key().await.unwrap();
    fs::write(evidence.join("root.der"), &trust).unwrap();
    let backend = pic
        .make_live_with_params(
            Some("127.0.0.1".parse().unwrap()),
            None,
            Some(vec!["127.0.0.1".into()]),
            None,
        )
        .await;
    let proxy = proxy::Proxy::start(
        backend.as_str(),
        evidence.join("wire"),
        match mode {
            Mode::Complete | Mode::Boundary | Mode::WriteFailure | Mode::Isolated => None,
            Mode::Capacity => Some(("read_canister_snapshot_data", 2)),
            Mode::TakeObservation => Some(("take_canister_snapshot", 0)),
            Mode::DataReply => Some(("read_canister_snapshot_data", 0)),
            Mode::RestoreAllocation | Mode::RestoreAllocationObservation => {
                Some(("upload_canister_snapshot_metadata", 0))
            }
            Mode::RestoreUpload => Some(("upload_canister_snapshot_data", 1)),
            Mode::RestoreLoad => Some(("load_canister_snapshot", 0)),
        },
        matches!(mode, Mode::WriteFailure).then(|| {
            (
                "load_canister_snapshot",
                evidence.join("store/operations").join("cd".repeat(16)),
            )
        }),
    );
    let calls = Arc::new(AtomicUsize::new(0));
    let allowed = Arc::new(AtomicBool::new(true));
    let network = BackupDigest::compute(b"ic-backup/private-pocketic-fixture/1");
    let consumer = matches!(mode, Mode::Isolated).then(|| {
        consumer::Consumer::build(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".."),
            evidence,
            &proxy.endpoint,
            network,
        )
    });
    let mut connection = BackupConnection::new(
        BackupHttpTransport::new(
            &proxy.endpoint,
            network,
            trust.clone(),
            Duration::from_secs(30),
        )
        .unwrap(),
        FixtureSigner(Arc::clone(&calls)),
        FixtureGuard(Arc::clone(&allowed)),
    );
    let request = CaptureRequest {
        operation_id: "01".repeat(16),
        target: BackupTarget {
            network,
            canister: Principal::from_slice(canister.as_slice()),
        },
        trust_root: BackupDigest::compute(&trust),
        initiator: sender,
        release: BackupDigest::compute(b"memory-fixture-release"),
        module_sha256: BackupDigest::compute(&wasm),
        sender,
        limits: OperationLimits {
            // Metadata lacks chunk lengths: allow 1 MiB per chunk, including empty.
            payload_bytes: if capacity { 22 } else { 4 } * 1024 * 1024,
            extent_bytes: if boundary { 1024 * 1024 } else { 16 * 1024 },
            network_requests: 128,
            observations: 1,
            signing_requests: 128,
            evidence_bytes: if capacity { 1024 } else { 256 } * 1024 * 1024,
        },
    };
    let mut capture_options = options();
    if boundary {
        capture_options.result_bytes = 4 * 1024 * 1024;
    }
    let plan = connection.plan(request, capture_options).unwrap();
    let approval = plan.digest();
    fs::write(evidence.join("intent.json"), plan.encoded()).unwrap();
    fs::DirBuilder::new()
        .mode(0o700)
        .create(evidence.join("store"))
        .unwrap();
    let store = BackupStore::initialize(&evidence.join("store")).unwrap();
    let mut operation = store.create_capture(plan.clone(), approval).unwrap();
    if capacity {
        loop {
            match operation.advance(&mut connection).await {
                Ok(OperationProgress::Advanced) => (),
                Err(_) => break,
                other => panic!("expected interrupted capacity transfer, got {other:?}"),
            }
        }
        let before = operation.inspect().unwrap();
        assert_eq!(before.next_call, Some(OperationStep::Data(2)));
        let certificate = proxy.lost.lock().unwrap().clone().unwrap();
        let signatures = calls.load(Ordering::SeqCst);
        let requests = proxy.calls.load(Ordering::SeqCst);
        drop(operation);
        operation = store
            .open_capture(&plan.request().operation_id, approval)
            .unwrap();
        let reopened = operation.inspect().unwrap();
        assert_eq!(reopened.next_call, before.next_call);
        assert_eq!(reopened.request_id, before.request_id);
        assert_eq!(
            reopened.signing_requests_reserved,
            before.signing_requests_reserved
        );
        assert_eq!(
            reopened.network_requests_reserved,
            before.network_requests_reserved
        );
        assert_eq!(reopened.observations_reserved, before.observations_reserved);
        allowed.store(false, Ordering::SeqCst);
        operation.reconcile_reply(&certificate).unwrap();
        assert!(operation.reconcile_reply(&certificate).is_err());
        assert_eq!(calls.load(Ordering::SeqCst), signatures);
        assert_eq!(proxy.calls.load(Ordering::SeqCst), requests);
        allowed.store(true, Ordering::SeqCst);
    }
    if let Some(child) = &consumer {
        drop(operation);
        child.call("capture-step", &plan.request().operation_id, approval, true);
        child.call(
            "capture-finish",
            &plan.request().operation_id,
            approval,
            true,
        );
        operation = store
            .open_capture(&plan.request().operation_id, approval)
            .unwrap();
    }
    if matches!(mode, Mode::TakeObservation | Mode::DataReply) {
        for _ in 0..128 {
            match operation.advance(&mut connection).await {
                Ok(OperationProgress::Advanced) => (),
                Err(error) => {
                    eprintln!("interrupted capture: {error:?}");
                    break;
                }
                other => panic!("expected original lost reply, got {other:?}"),
            }
        }
        let before = operation.inspect().unwrap();
        let lost_step = if matches!(mode, Mode::TakeObservation) {
            OperationStep::TakeSnapshot
        } else {
            OperationStep::Data(0)
        };
        assert_eq!(before.next_call, Some(lost_step));
        assert_eq!(
            before.signing_requests_reserved as usize,
            calls.load(Ordering::SeqCst)
        );
        assert_eq!(
            operation.advance(&mut connection).await.unwrap(),
            OperationProgress::NeedsAttention(lost_step)
        );
        let certificate = proxy.lost.lock().unwrap().clone().unwrap();
        if matches!(mode, Mode::TakeObservation) {
            assert_eq!(
                operation
                    .reconcile_observation(&mut connection)
                    .await
                    .unwrap(),
                OperationProgress::Advanced
            );
            assert_eq!(operation.inspect().unwrap().observations_reserved, 1);
        }
        let snapshots = owned
            .client_mut()
            .list_canister_snapshots(canister, Some(native_sender))
            .await
            .unwrap();
        assert_eq!(snapshots.len(), 2);
        assert!(snapshots.iter().any(|s| s.id == existing.id));
        let signatures = calls.load(Ordering::SeqCst);
        let requests = proxy.calls.load(Ordering::SeqCst);
        let ledger_path = evidence
            .join("store/operations")
            .join(&plan.request().operation_id)
            .join("journal.bin");
        drop(operation);
        proxy.finish();
        server::close(owned, &mut server).await;
        allowed.store(false, Ordering::SeqCst);
        let mut operation = store
            .open_capture(&plan.request().operation_id, approval)
            .unwrap();
        if matches!(mode, Mode::DataReply) {
            assert_eq!(
                operation.advance(&mut connection).await.unwrap(),
                OperationProgress::NeedsAttention(lost_step)
            );
            operation.reconcile_reply(&certificate).unwrap();
            assert!(operation.reconcile_reply(&certificate).is_err());
        }
        let replay = operation.inspect().unwrap();
        assert_eq!(
            replay.next_call,
            Some(if matches!(mode, Mode::DataReply) {
                OperationStep::Data(1)
            } else {
                OperationStep::Metadata
            })
        );
        let original = fs::read(&ledger_path).unwrap();
        assert!(operation.advance(&mut connection).await.is_err());
        assert_eq!(fs::read(&ledger_path).unwrap(), original);
        assert_eq!(calls.load(Ordering::SeqCst), signatures);
        assert_eq!(replay.network_requests_reserved as usize, requests);
        fs::write(
            evidence.join("qualified.txt"),
            concat!(
                "actual original management ingress; ",
                "explicit authenticated recovery; ",
                "cold replay without server or live authority; ",
                "no resubmission\n",
            ),
        )
        .unwrap();

        return;
    }
    let receipt = complete(&mut operation, &mut connection).await;
    if capacity {
        assert_eq!(receipt.payload_bytes, 20 * 1024 * 1024);
    }
    assert_eq!(
        receipt.payload_bytes,
        (wasm.len() + heap.len() + stable.len() + chunks[1].len()) as u64
    );
    let payload = evidence
        .join("store/bundles")
        .join(&receipt.bundle_id)
        .join("payload");
    for (name, expected) in [
        ("module.wasm", wasm.as_slice()),
        ("heap.bin", heap.as_slice()),
        ("stable.bin", stable.as_slice()),
    ] {
        assert_eq!(fs::read(payload.join(name)).unwrap(), expected);
    }
    for chunk in &chunks {
        assert_eq!(
            fs::read(payload.join(format!("chunk-{}.bin", BackupDigest::compute(chunk)))).unwrap(),
            *chunk
        );
    }
    assert_eq!(
        store
            .verify_payload(
                &receipt.bundle_id,
                receipt.manifest_sha256,
                receipt.payload_bytes.try_into().unwrap()
            )
            .unwrap(),
        receipt
    );
    assert_eq!(
        store
            .latest_completed_backup(&receipt.target, scan())
            .unwrap(),
        Some(receipt.clone())
    );
    let original_journal = evidence
        .join("store/operations")
        .join(&receipt.bundle_id)
        .join("journal.bin");
    let original_bytes = fs::read(&original_journal).unwrap();
    drop(operation);
    let (next, second) = if matches!(
        mode,
        Mode::Complete | Mode::Boundary | Mode::Capacity | Mode::Isolated
    ) {
        let mut request = plan.request().clone();
        request.operation_id = "02".repeat(16);
        let next = connection.plan(request, capture_options).unwrap();
        let mut successor = store
            .create_capture_after(next.clone(), next.digest(), OperationPlan::Capture(&plan))
            .unwrap();
        let second = complete(&mut successor, &mut connection).await;
        drop(successor);

        (next, second)
    } else {
        (plan.clone(), receipt.clone())
    };
    assert_eq!(fs::read(&original_journal).unwrap(), original_bytes);
    let mut operation = store.open_capture(&receipt.bundle_id, approval).unwrap();
    let mut source = store
        .open_capture_source(
            &mut operation,
            &receipt,
            receipt.payload_bytes.try_into().unwrap(),
        )
        .unwrap();
    let snapshot_id = receipt
        .snapshot_id_hex
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|hex| u8::from_str_radix(std::str::from_utf8(hex).unwrap(), 16).unwrap())
        .collect();
    assert_eq!(
        source
            .read_data(&ReadDataArgs {
                canister_id: receipt.target.canister,
                snapshot_id,
                kind: ReadKind::WasmModule(ic_platform_protocol::management_snapshots::Extent {
                    offset: 0,
                    size: wasm.len() as u64
                })
            })
            .unwrap(),
        wasm
    );
    let mut restore_options = options();
    // Original-result lookup must also work under the admitted 2 MiB upload profile.
    restore_options.request_bytes =
        if boundary || matches!(mode, Mode::RestoreAllocationObservation) {
            2 * 1024 * 1024
        } else {
            64 * 1024
        };
    restore_options.result_bytes = if boundary {
        4 * 1024 * 1024
    } else {
        128 * 1024
    };
    let restore = connection
        .plan_restore(
            &source,
            RestoreRequest {
                operation_id: "cd".repeat(16),
                target: receipt.target,
                trust_root: plan.request().trust_root,
                initiator: sender,
                release: receipt.release,
                sender,
                limits: OperationLimits {
                    payload_bytes: if capacity { 22 } else { 4 } * 1024 * 1024,
                    extent_bytes: if boundary { 1024 * 1024 } else { 16 * 1024 },
                    network_requests: 128,
                    observations: 1,
                    signing_requests: 128,
                    evidence_bytes: if capacity { 1024 } else { 256 } * 1024 * 1024,
                },
            },
            restore_options,
        )
        .unwrap();
    let restore_id = restore.request().operation_id.clone();
    let restore_approval = restore.digest();
    let restore_intent = restore.encoded().to_vec();
    drop(source);
    drop(operation);
    let mut restoring = store
        .create_restore(&restore, restore.digest(), &next)
        .unwrap();
    let restore_path = evidence
        .join("store/operations")
        .join(&restore.request().operation_id)
        .join("journal.bin");
    // Change actual stable bytes independently so successful restore must rewind state.
    owned
        .client_mut()
        .set_stable_memory(
            canister,
            vec![0xa5; stable.len()],
            BlobCompression::NoCompression,
        )
        .await;
    assert_ne!(owned.client_mut().get_stable_memory(canister).await, stable);
    if let Some(child) = &consumer {
        drop(restoring);
        child.call("restore-step", &restore_id, restore_approval, true);
        child.call("restore-finish", &restore_id, restore_approval, true);
        restoring = store.open_restore(&restore_id, restore_approval).unwrap();
    }
    if matches!(
        mode,
        Mode::RestoreAllocation
            | Mode::RestoreAllocationObservation
            | Mode::RestoreUpload
            | Mode::RestoreLoad
            | Mode::WriteFailure
    ) {
        let mut returned_certificate = None;
        loop {
            match restoring.advance(&mut connection).await {
                Ok(OperationProgress::Advanced) => (),
                Err(error) => {
                    eprintln!("interrupted restore: {error:?}");
                    if matches!(mode, Mode::WriteFailure) {
                        returned_certificate =
                            Some(error.received_certificate().unwrap().certificate().to_vec());
                        use std::os::unix::fs::PermissionsExt;
                        fs::set_permissions(
                            restore_path.parent().unwrap(),
                            fs::Permissions::from_mode(0o700),
                        )
                        .unwrap();
                        drop(restoring);
                        restoring = store.open_restore(&restore_id, restore_approval).unwrap();
                    }
                    break;
                }
                other => panic!("expected lost original restore reply, got {other:?}"),
            }
        }
        let step = match mode {
            Mode::RestoreAllocation | Mode::RestoreAllocationObservation => {
                OperationStep::AllocateSnapshot
            }
            Mode::RestoreUpload => OperationStep::Upload(1),
            _ => OperationStep::LoadSnapshot,
        };
        let before = restoring.inspect().unwrap();
        assert_eq!(before.next_call, Some(step));
        let mut signatures = calls.load(Ordering::SeqCst);
        let mut requests = proxy.calls.load(Ordering::SeqCst);
        let pending = fs::read(&restore_path).unwrap();
        assert_eq!(
            restoring.advance(&mut connection).await.unwrap(),
            OperationProgress::NeedsAttention(step)
        );
        assert_eq!(fs::read(&restore_path).unwrap(), pending);
        assert_eq!(
            owned
                .client_mut()
                .list_canister_snapshots(canister, Some(native_sender))
                .await
                .unwrap()
                .len(),
            3
        );
        let actual = owned.client_mut().get_stable_memory(canister).await;
        if matches!(mode, Mode::RestoreLoad | Mode::WriteFailure) {
            assert_eq!(actual, stable);
        } else {
            assert_ne!(actual, stable);
        }
        let observation = matches!(mode, Mode::RestoreAllocationObservation);
        if observation {
            assert_eq!(
                restoring
                    .reconcile_observation(&mut connection)
                    .await
                    .unwrap(),
                OperationProgress::Advanced
            );
            signatures += 1;
            requests += 1;
        }
        let certificate = if let Some(proof) = returned_certificate {
            assert!(proxy.lost.lock().unwrap().is_none());
            assert!(
                !restore_path
                    .parent()
                    .unwrap()
                    .join("load-snapshot-result.bin")
                    .exists()
            );
            fs::write(evidence.join("returned-load-certificate.bin"), &proof).unwrap();

            proof
        } else {
            proxy.lost.lock().unwrap().clone().unwrap()
        };
        let observed_requests = Arc::clone(&proxy.calls);
        drop(restoring);
        drop(restore);
        proxy.finish();
        server::close(owned, &mut server).await;
        allowed.store(false, Ordering::SeqCst);
        let mut restoring = store.open_restore(&restore_id, restore_approval).unwrap();
        assert_eq!(restoring.plan().encoded(), restore_intent);
        if !observation {
            restoring.reconcile_reply(&certificate).unwrap();
        }
        assert!(restoring.reconcile_reply(&certificate).is_err());
        let settled = restoring.inspect().unwrap();
        assert_eq!(
            settled.signing_requests_reserved,
            before.signing_requests_reserved + u32::from(observation)
        );
        assert_eq!(
            settled.network_requests_reserved,
            before.network_requests_reserved + u32::from(observation)
        );
        assert_eq!(settled.observations_reserved, u32::from(observation));
        assert_eq!(
            settled.next_call,
            Some(match step {
                OperationStep::AllocateSnapshot => OperationStep::Upload(0),
                OperationStep::Upload(1) => OperationStep::Upload(2),
                _ => OperationStep::AfterLoadStatus,
            })
        );
        let closed = fs::read(&restore_path).unwrap();
        assert!(restoring.advance(&mut connection).await.is_err());
        assert_eq!(fs::read(&restore_path).unwrap(), closed);
        assert_eq!(calls.load(Ordering::SeqCst), signatures);
        assert_eq!(observed_requests.load(Ordering::SeqCst), requests);
        fs::write(
            evidence.join("qualified.txt"),
            concat!(
                "actual original allocation/mid-upload/load effect; ",
                "exact original terminal certificate settled after shutdown; ",
                "no renewed signing/submission after cold reopen or revoked authority\n",
            ),
        )
        .unwrap();

        return;
    }
    let loaded = loop {
        match restoring.advance(&mut connection).await.unwrap() {
            OperationProgress::Advanced => (),
            OperationProgress::NeedsAttention(step) => {
                panic!("unexpected pending restore {step:?}")
            }
            OperationProgress::Complete(receipt) => break *receipt,
        }
    };
    let restored_journal = fs::read(&restore_path).unwrap();
    assert_eq!(loaded.source_operation_id, receipt.bundle_id);
    assert_eq!(restoring.inspect().unwrap().completed, Some(loaded.clone()));
    let status = owned
        .client_mut()
        .canister_status(canister, Some(native_sender))
        .await
        .unwrap();
    assert_eq!(format!("{:?}", status.status), "Stopped");
    assert_eq!(
        status.module_hash.unwrap(),
        BackupDigest::compute(&wasm).as_bytes()
    );
    assert_eq!(owned.client_mut().get_stable_memory(canister).await, stable);
    assert_eq!(
        owned
            .client_mut()
            .list_canister_snapshots(canister, Some(native_sender))
            .await
            .unwrap()
            .len(),
        4
    );
    drop(restoring);
    let mut follow_request = plan.request().clone();
    follow_request.operation_id = "03".repeat(16);
    let follow = connection.plan(follow_request, options()).unwrap();
    let mut successor = store
        .create_capture_after(
            follow.clone(),
            follow.digest(),
            OperationPlan::Restore(&restore),
        )
        .unwrap();
    assert_eq!(
        successor.advance(&mut connection).await.unwrap(),
        OperationProgress::Advanced
    );
    assert_eq!(
        successor.inspect().unwrap().next_call,
        Some(OperationStep::ListSnapshots)
    );
    drop(successor);
    let signatures = calls.load(Ordering::SeqCst);
    drop(restore);
    proxy.finish();
    server::close(owned, &mut server).await;
    allowed.store(false, Ordering::SeqCst);
    let mut operation = store.open_capture(&receipt.bundle_id, approval).unwrap();
    assert_eq!(
        operation.advance(&mut connection).await.unwrap(),
        OperationProgress::Complete(Box::new(receipt.clone()))
    );
    assert_eq!(
        operation
            .verify_completion(receipt.manifest_sha256)
            .unwrap(),
        receipt
    );
    assert_eq!(fs::read(&original_journal).unwrap(), original_bytes);
    drop(operation);
    let mut restoring = store.open_restore(&restore_id, restore_approval).unwrap();
    assert_eq!(
        restoring.advance(&mut connection).await.unwrap(),
        OperationProgress::Complete(Box::new(loaded))
    );
    assert_eq!(fs::read(&restore_path).unwrap(), restored_journal);
    drop(restoring);
    assert_eq!(
        store
            .list_backups(Some(&receipt.target), scan())
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        store
            .latest_completed_backup(&receipt.target, scan())
            .unwrap(),
        Some(second)
    );
    assert_eq!(calls.load(Ordering::SeqCst), signatures);
    if let Some(child) = &consumer {
        child.call(
            "capture-inspect",
            &plan.request().operation_id,
            approval,
            false,
        );
        child.call("restore-inspect", &restore_id, restore_approval, false);
    }
    fs::write(
        evidence.join("qualified.txt"),
        concat!(
            "actual multi-extent module/heap/stable/chunks capture; ",
            "exact publication; ",
            "authenticated target succession; ",
            "older held source; ",
            "actual source-bound allocation/upload/metadata and byte readback/load/stopped completion; ",
            "offline terminal replay and history\n",
        ),
    )
    .unwrap();
}
