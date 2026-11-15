//! Actual management ingress through PocketIC's async HTTP gateway.
//! The fixture key and maintenance guard are test-only, not a Canic/process adapter.
use ic_backup::store::{
    BackupStore, BackupTarget, CaptureApplicationGuard, CaptureExecution, CaptureLimitsRecord,
    CapturePlan, CaptureRequest, CaptureSigner, CaptureSigningRequest, CaptureTiming,
    DirectCaptureProvider, SnapshotAdmissionRecord, SnapshotProgress, StatusAdmissionRecord,
    StatusHttpTransport, StatusProgress,
};
use ic_host_artifacts::artifact::Sha256Digest;
use ic_platform::{
    Principal,
    ed25519::{Signer as _, SigningKey},
    ic_protocol::{Envelope, EnvelopeContent, encode_envelope},
};
use ic_testkit::{
    pic::{PocketIcStartupConfig, server_assets},
    pocket_ic::{CanisterSettings, PocketIcBuilder},
};
use std::{
    borrow::Cow,
    fs,
    os::unix::fs::DirBuilderExt,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .try_into()
        .unwrap()
}
fn der() -> Vec<u8> {
    [
        b"\x30\x2a\x30\x05\x06\x03\x2b\x65\x70\x03\x21\x00".as_slice(),
        SigningKey::from_bytes(&[17; 32]).verifying_key().as_bytes(),
    ]
    .concat()
}
struct FixtureSigner(Arc<AtomicUsize>);
impl CaptureSigner for FixtureSigner {
    type Error = ();
    fn protocol(&self) -> [u8; 32] {
        [1; 32]
    }
    async fn sign_call(&mut self, request: CaptureSigningRequest<'_>) -> Result<Vec<u8>, ()> {
        self.0.fetch_add(1, Ordering::SeqCst);
        assert_eq!(request.network_requests, 0);
        assert!(now() < request.deadline_ns);
        let CaptureExecution::Direct { sender } = request.plan.request().execution else {
            panic!("direct");
        };
        let content = EnvelopeContent::Call {
            nonce: None,
            ingress_expiry: request.max_expiry_ns,
            sender: sender.as_slice().try_into().unwrap(),
            canister_id: [].as_slice().try_into().unwrap(),
            method_name: request.call.method_name().into(),
            arg: request.arguments.to_vec(),
            sender_info: None,
        };
        let signature = SigningKey::from_bytes(&[17; 32]).sign(&content.to_request_id().signable());
        encode_envelope(
            &Envelope {
                content: Cow::Owned(content),
                sender_pubkey: Some(der()),
                sender_sig: Some(signature.to_bytes().to_vec()),
                sender_delegation: None,
            },
            request.envelope_bytes as usize,
        )
        .map_err(|_| ())
    }
}
struct FixtureGuard(Arc<AtomicBool>);
impl CaptureApplicationGuard for FixtureGuard {
    type Error = ();
    fn protocol(&self) -> [u8; 32] {
        [2; 32]
    }
    fn admit(&self, _: &CapturePlan, application: &[u8; 32]) -> Result<(), ()> {
        if application == &[3; 32] && self.0.load(Ordering::SeqCst) {
            Ok(())
        } else {
            Err(())
        }
    }
}

#[test]
fn stopped_canister_snapshot_and_offline_cold_replay() {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let assets = std::env::var_os("IC_BACKUP_TESTKIT_DIRECTORY")
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace.join("../.tools/testkit"));
    // Never downloads. The Make setup target explicitly provisions this Testkit release.
    let binary = server_assets::check(&assets).expect("run make install-testkit-server");
    let evidence = workspace.join("target/fixtures").join(format!(
        "direct-status-{}-{}",
        std::process::id(),
        now()
    ));
    fs::create_dir_all(&evidence).unwrap();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(status_fixture(&evidence, binary));
}

async fn status_fixture(evidence: &std::path::Path, binary: PathBuf) {
    let mut owned = PocketIcStartupConfig::Spawn {
        server_binary: binary,
        timeout: Duration::from_secs(30),
        server_hard_ttl: Some(Duration::from_secs(180)),
        server_idle_ttl: None,
        server_output_files: Some((
            evidence.join("server.stdout"),
            evidence.join("server.stderr"),
        )),
    }
    .build_instance(
        PocketIcBuilder::new()
            .with_nns_subnet()
            .with_application_subnet()
            .with_initial_time(SystemTime::now().into())
            .with_max_request_time_ms(Some(30_000)),
    )
    .await
    .unwrap();
    let sender = Principal::self_authenticating(der());
    let native_sender = sender.as_slice().try_into().unwrap();
    let pic = owned.instance_mut();
    let canister = pic
        .create_canister_with_settings(
            Some(native_sender),
            Some(CanisterSettings {
                controllers: Some(vec![native_sender]),
                ..CanisterSettings::default()
            }),
        )
        .await;
    // Minimal valid Wasm, no application methods or external obligations.
    let wasm = b"\0asm\x01\0\0\0";
    pic.install_canister(canister, wasm.to_vec(), vec![], Some(native_sender))
        .await;
    pic.stop_canister(canister, Some(native_sender))
        .await
        .unwrap();
    // Fixture setup owns this pre-existing snapshot, outside Backup's plan.
    let existing = pic
        .take_canister_snapshot(canister, Some(native_sender), None)
        .await
        .unwrap();
    let stopped = pic
        .canister_status(canister, Some(native_sender))
        .await
        .unwrap()
        .status;
    let root = pic.root_key().await.unwrap();
    let endpoint = pic
        .make_live_with_params(
            Some("127.0.0.1".parse().unwrap()),
            None,
            Some(vec!["127.0.0.1".into()]),
            None,
        )
        .await;
    let network = Sha256Digest::compute(b"ic-backup/private-pocketic-fixture/1");
    let plan = CapturePlan::new(CaptureRequest {
        operation_id: "00000000000000000000000000000001".into(),
        target: BackupTarget {
            network,
            canister: Principal::from_slice(canister.as_slice()),
        },
        trust_root: Sha256Digest::compute(&root),
        initiator: sender,
        release: Sha256Digest::compute(b"empty-fixture-release"),
        module_sha256: Sha256Digest::compute(wasm),
        execution: CaptureExecution::Direct { sender },
        limits: CaptureLimitsRecord {
            payload_bytes: 1024 * 1024,
            extent_bytes: 4096,
            network_requests: 4,
            observations: 1,
            signing_requests: 3,
            evidence_bytes: 1024 * 1024,
        },
    })
    .unwrap();
    fs::write(evidence.join("root.der"), &root).unwrap();
    let allowed = Arc::new(AtomicBool::new(true));
    let calls = Arc::new(AtomicUsize::new(0));
    let mut provider = direct_provider(endpoint.as_str(), network, &root, &calls, &allowed);
    let reference = now();
    let admission = StatusAdmissionRecord {
        provider: provider.commitment(),
        application: [3; 32],
        prepare_deadline_ns: reference + 30_000_000_000,
        submit_deadline_ns: reference + 60_000_000_000,
        prepare_requests: 0,
        submit_requests: 1,
        request_bytes: 4096,
        result_bytes: 256 * 1024,
    };
    let digest = plan.digest();
    fs::DirBuilder::new()
        .mode(0o700)
        .create(evidence.join("store"))
        .unwrap();
    let store = BackupStore::initialize(&evidence.join("store")).unwrap();
    let mut operation = store.create_capture(plan, digest).unwrap();
    assert_eq!(
        operation
            .advance_status(&admission, &mut provider)
            .await
            .unwrap(),
        StatusProgress::Verified
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    let view = operation.inspect_status().unwrap();
    assert_eq!(view.signing_requests_reserved, 1);
    assert_eq!(view.network_requests_reserved, 1);
    assert!(view.request_id.is_some());
    let snapshot_admission = SnapshotAdmissionRecord {
        capacity: 2,
        list: admission.clone(),
        take: admission.clone(),
    };
    let SnapshotProgress::Captured(taken) = operation
        .advance_snapshot(&snapshot_admission, &mut provider)
        .await
        .unwrap()
    else {
        panic!("capture must settle");
    };
    let snapshot_view = operation.inspect_snapshot().unwrap();
    assert_eq!(snapshot_view.signing_requests_reserved, 3);
    assert_eq!(snapshot_view.network_requests_reserved, 3);
    assert_eq!(calls.load(Ordering::SeqCst), 3);
    assert_ne!(taken.id, existing.id);
    let inventory = pic
        .list_canister_snapshots(canister, Some(native_sender))
        .await
        .unwrap();
    assert_eq!(inventory.len(), 2);
    assert!(inventory.iter().any(|entry| entry == &existing));
    assert!(inventory.iter().any(|entry| entry.id == taken.id
        && entry.taken_at_timestamp == taken.taken_at_timestamp
        && entry.total_size == taken.total_size));
    let observed = pic
        .canister_status(canister, Some(native_sender))
        .await
        .unwrap();
    assert_eq!(observed.status, stopped);
    assert_eq!(
        observed.module_hash,
        Some(Sha256Digest::compute(wasm).as_bytes().to_vec())
    );
    drop(operation);
    // Remove all live authority and the gateway; replay must use only retained proof.
    allowed.store(false, Ordering::SeqCst);
    drop(provider);
    owned.close().await;
    let mut provider = direct_provider(endpoint.as_str(), network, &root, &calls, &allowed);
    let mut reopened = store
        .open_capture("00000000000000000000000000000001", digest)
        .unwrap();
    assert_eq!(
        reopened
            .advance_status(&admission, &mut provider)
            .await
            .unwrap(),
        StatusProgress::Verified
    );
    assert_eq!(reopened.inspect_status().unwrap(), view);
    assert_eq!(
        reopened
            .advance_snapshot(&snapshot_admission, &mut provider)
            .await
            .unwrap(),
        SnapshotProgress::Captured(taken)
    );
    assert_eq!(reopened.inspect_snapshot().unwrap(), snapshot_view);
    assert_eq!(calls.load(Ordering::SeqCst), 3);
    fs::write(evidence.join("verified.txt"), b"certified status, list and take; original snapshot and stopped module preserved; three signatures and reserved HTTP requests; cold replay with server absent\n").unwrap();
    eprintln!("retained qualification evidence: {}", evidence.display());
}

fn direct_provider(
    endpoint: &str,
    network: Sha256Digest,
    root: &[u8],
    calls: &Arc<AtomicUsize>,
    allowed: &Arc<AtomicBool>,
) -> DirectCaptureProvider<FixtureSigner, FixtureGuard> {
    DirectCaptureProvider::new(
        StatusHttpTransport::new(endpoint, network, root.to_vec(), Duration::from_secs(30))
            .unwrap(),
        FixtureSigner(Arc::clone(calls)),
        FixtureGuard(Arc::clone(allowed)),
        CaptureTiming {
            certificate_max_age_ns: 60_000_000_000,
            delegation_max_age_ns: 86_400_000_000_000,
            max_clock_skew_ns: 1_000_000_000,
        },
    )
}
