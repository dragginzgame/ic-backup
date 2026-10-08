//! Actual agent HTTP gateway effects on an isolated no-external-effects fixture.
mod support;
use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        ic_mutation::IcMutationRequest,
        ic_request::IcManagementMethodRecord as Method,
        ic_snapshot_data::{IcSnapshotDataReply, IcSnapshotDataRequest},
        ic_snapshot_metadata::{IcSnapshotMetadataReply, IcSnapshotMetadataRequest},
        ic_snapshot_reply::IcSnapshotReply,
        ic_snapshot_transfer_read::{IcSnapshotTransferReadPayload, IcSnapshotTransferReadRequest},
        ic_snapshot_upload::{
            IcSnapshotUploadAttempt, IcSnapshotUploadReply, IcSnapshotUploadReplyKind,
            IcSnapshotUploadRequest,
        },
        operation_plan::OperationPlanRecord,
    },
    ops::persistence::BackupLayoutGuard,
};
use ic_backup_agent::{AgentTransport, ReservedUpdate, TransportError, UpdateOutcome};
use ic_management_canister_types::{CanisterStatusType, SnapshotDataKind};
use ic_testkit::{
    pic::{PocketIcManagedServer, PocketIcStartupConfig},
    pocket_ic::{PocketIcBuilder, nonblocking::PocketIc},
};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

struct Fixture {
    pic: PocketIc,
    // Preserve raw server output and retain cleanup custody until the fixture drops.
    _server: PocketIcManagedServer,
    target: String,
    root: PathBuf,
    endpoint: String,
    key: Vec<u8>,
    sequence: u64,
    release: ArtifactChecksumRecord,
    identity: Arc<dyn ic_agent::Identity>,
}
impl Fixture {
    async fn new(identity: Arc<dyn ic_agent::Identity>) -> Self {
        let root = support::root("pocketic");
        let binary = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../.tools/ic/bin/pocket-ic")
            .canonicalize()
            .expect("prepare pinned PocketIC explicitly");
        let checksums = fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.tools/ic/files.sha256"),
        )
        .unwrap();
        let expected = checksums
            .lines()
            .find_map(|line| line.strip_suffix("  bin/pocket-ic"))
            .unwrap();
        assert_eq!(
            ArtifactChecksumRecord::from_bytes(&fs::read(&binary).unwrap()).hash(),
            expected
        );
        fs::write(root.join("server.sha256"), expected).unwrap();
        let server = PocketIcStartupConfig::spawn(binary, Duration::from_secs(60))
            .with_server_output_files(root.join("server.stdout"), root.join("server.stderr"))
            .start_managed_server()
            .unwrap();
        let mut pic = PocketIcBuilder::new()
            .with_server_url(server.url().parse().unwrap())
            .with_application_subnet()
            .with_nns_subnet()
            .with_state_dir(root.join("simulator"))
            .with_max_request_time_ms(Some(60_000))
            .build_async()
            .await;
        let target = pic.create_canister().await;
        pic.add_cycles(target, 100_000_000_000_000).await;
        let wasm = wat::parse_str(include_str!("state.wat")).unwrap();
        let release = ArtifactChecksumRecord::from_bytes(&wasm);
        fs::write(root.join("fixture.wasm"), &wasm).unwrap();
        pic.install_canister(target, wasm, vec![], None).await;
        pic.update_call(
            target,
            ic_agent::export::Principal::anonymous(),
            "write",
            42_u64.to_le_bytes().to_vec(),
        )
        .await
        .unwrap();
        pic.set_controllers(target, None, vec![identity.sender().unwrap()])
            .await
            .unwrap();
        let key = pic.root_key().await.unwrap();
        let endpoint = pic
            .make_live_with_params(None, None, Some(vec!["127.0.0.1".into()]), None)
            .await
            .to_string();
        fs::write(root.join("trusted-root.der"), &key).unwrap();
        Self {
            pic,
            _server: server,
            target: target.to_text(),
            root,
            endpoint,
            key,
            sequence: 0,
            release,
            identity,
        }
    }
    fn caller(&self) -> ic_agent::export::Principal {
        self.identity.sender().unwrap()
    }
    fn principal(&self) -> ic_agent::export::Principal {
        ic_agent::export::Principal::from_text(&self.target).unwrap()
    }
    fn original(
        &mut self,
        digest: &ArtifactChecksumRecord,
    ) -> (OperationPlanRecord, BackupLayoutGuard, PathBuf) {
        self.sequence += 1;
        let sequence = self.sequence;
        let mut value =
            serde_json::to_value(support::plan(&self.target, digest, sequence)).unwrap();
        value["context"]["caller"] = self.caller().to_text().into();
        value["context"]["network"] = ArtifactChecksumRecord::from_bytes(&self.key).hash().into();
        value["context"]["release"] = self.release.hash().into();
        let plan: OperationPlanRecord = serde_json::from_value(value).unwrap();
        let root = self.root.join(format!("call-{sequence}"));
        fs::create_dir(&root).unwrap();
        let layout = BackupLayoutGuard::acquire(&root).unwrap();
        (plan, layout, root)
    }
    fn transport(&self, plan: &OperationPlanRecord) -> AgentTransport {
        AgentTransport::new(
            plan.context().clone(),
            &self.endpoint,
            self.identity.clone(),
            self.key.clone(),
            Duration::from_secs(30),
        )
        .unwrap()
    }
    async fn mutation(
        &mut self,
        method: Method,
        id: Option<Vec<u8>>,
        lose: bool,
    ) -> (OperationPlanRecord, Vec<u8>) {
        let payload = support::payload(method, &self.target, id);
        let (plan, layout, root) = self.original(&payload.digest());
        let guard = support::retain(&layout, &plan, self.sequence);
        let before = fs::read(guard.path()).unwrap();
        let request =
            IcMutationRequest::new(&plan, self.sequence, guard.record().unwrap(), &payload)
                .unwrap();
        let transport = self.transport(&plan);
        let raw = submit(
            &transport,
            ReservedUpdate::Mutation(&request),
            guard.record().unwrap(),
            &root,
        )
        .await;
        assert_eq!(fs::read(guard.path()).unwrap(), before);
        assert_eq!(guard.record().unwrap().view().pending_mutation, Some(1));
        if lose {
            fs::write(root.join("discarded-oracle-reply.candid"), &raw).unwrap();
        }
        (plan, raw)
    }
    async fn read(&mut self, payload: IcSnapshotTransferReadPayload<'_, '_>) -> Vec<u8> {
        let (plan, layout, root) = self.original(&payload.digest());
        let guard = support::retain(&layout, &plan, self.sequence);
        let request = IcSnapshotTransferReadRequest::new(
            &plan,
            self.sequence,
            guard.record().unwrap(),
            payload,
        )
        .unwrap();
        let transport = self.transport(&plan);
        submit(
            &transport,
            ReservedUpdate::TransferRead(&request),
            guard.record().unwrap(),
            &root,
        )
        .await
    }
    async fn upload(&mut self, payload: &IcSnapshotUploadRequest<'_>) -> Vec<u8> {
        let (plan, layout, root) = self.original(&payload.binding_digest());
        let guard = support::retain(&layout, &plan, self.sequence);
        let request =
            IcSnapshotUploadAttempt::new(&plan, self.sequence, guard.record().unwrap(), payload)
                .unwrap();
        let transport = self.transport(&plan);
        submit(
            &transport,
            ReservedUpdate::Upload(&request),
            guard.record().unwrap(),
            &root,
        )
        .await
    }
}
async fn submit(
    transport: &AgentTransport,
    request: ReservedUpdate<'_>,
    journal: &ic_backup::model::attempt_journal::AttemptJournalRecord,
    root: &Path,
) -> Vec<u8> {
    let prepared = transport.prepare(request, journal).unwrap();
    let id = *prepared.request_id();
    fs::write(root.join("signed-ingress.cbor"), prepared.envelope()).unwrap();
    fs::write(root.join("request-id.txt"), id.to_string()).unwrap();
    match prepared.submit().await.unwrap() {
        UpdateOutcome::Replied { request_id, reply } => {
            assert_eq!(request_id, id);
            fs::write(root.join("reply.candid"), &reply).unwrap();
            reply
        }
        UpdateOutcome::Pending { .. } => {
            panic!("gateway returned pending: no implicit polling allowed")
        }
    }
}

async fn read_complete_regions(
    fixture: &mut Fixture,
    metadata: &IcSnapshotMetadataReply<'_>,
) -> Vec<(SnapshotDataKind, Vec<u8>)> {
    let sizes = [
        metadata.metadata().wasm_module_size,
        metadata.metadata().wasm_memory_size,
        metadata.metadata().stable_memory_size,
    ];
    let mut chunks = Vec::new();
    for (region, size) in sizes.into_iter().enumerate() {
        let mut offset = 0;
        while offset < size {
            let length = (size - offset).min(1024 * 1024);
            let kind = match region {
                0 => SnapshotDataKind::WasmModule {
                    offset,
                    size: length,
                },
                1 => SnapshotDataKind::WasmMemory {
                    offset,
                    size: length,
                },
                _ => SnapshotDataKind::StableMemory {
                    offset,
                    size: length,
                },
            };
            let payload = IcSnapshotDataRequest::new(metadata, kind.clone()).unwrap();
            let raw = fixture
                .read(IcSnapshotTransferReadPayload::Data(&payload))
                .await;
            let reply = IcSnapshotDataReply::decode(&payload, &raw).unwrap();
            chunks.push((kind, reply.chunk().to_vec()));
            offset += length;
        }
    }
    assert_eq!(
        metadata.metadata().wasm_chunk_store,
        Vec::<ic_management_canister_types::ChunkHash>::new()
    );
    chunks
}

#[tokio::test(flavor = "multi_thread")]
async fn real_agent_capture_complete_transfer_upload_and_same_id_restore_preserve_originals() {
    let mut fixture = Fixture::new(Arc::new(ic_agent::identity::BasicIdentity::from_raw_key(
        &[7; 32],
    )))
    .await;
    fixture.mutation(Method::StopCanister, None, false).await;
    assert_eq!(
        serde_json::to_value(
            fixture
                .pic
                .canister_status(fixture.principal(), Some(fixture.caller()))
                .await
                .unwrap()
                .status
        )
        .unwrap(),
        serde_json::to_value(CanisterStatusType::Stopped).unwrap()
    );
    let (source_plan, raw) = fixture
        .mutation(Method::TakeCanisterSnapshot, None, false)
        .await;
    // Transport evidence alone writes no automatic journal receipt.
    let capture = support::payload(Method::TakeCanisterSnapshot, &fixture.target, None);
    let source_id = IcSnapshotReply::decode(&capture, &raw).unwrap().snapshots()[0]
        .id()
        .to_vec();
    let metadata_request = IcSnapshotMetadataRequest::new(&fixture.target, &source_id).unwrap();
    let raw = fixture
        .read(IcSnapshotTransferReadPayload::Metadata(&metadata_request))
        .await;
    let metadata = IcSnapshotMetadataReply::decode(&metadata_request, &raw).unwrap();
    assert_eq!(metadata.metadata().certified_data, 42_u64.to_le_bytes());
    let chunks = read_complete_regions(&mut fixture, &metadata).await;
    let complete = ArtifactChecksumRecord::from_bytes(
        &chunks
            .iter()
            .flat_map(|(_, bytes)| bytes.iter().copied())
            .collect::<Vec<_>>(),
    );
    let upload = IcSnapshotUploadRequest::metadata(&source_plan, &metadata, &complete).unwrap();
    let raw = fixture.upload(&upload).await;
    let allocation = IcSnapshotUploadReply::decode(&upload, &raw).unwrap();
    let IcSnapshotUploadReplyKind::Metadata { snapshot_id } = allocation.kind() else {
        panic!("allocation ID required")
    };
    let destination = snapshot_id.clone();
    assert_ne!(source_id, destination);
    for (kind, bytes) in chunks {
        let payload = IcSnapshotUploadRequest::data(&upload, &destination, kind, &bytes).unwrap();
        let raw = fixture.upload(&payload).await;
        assert!(matches!(
            IcSnapshotUploadReply::decode(&payload, &raw)
                .unwrap()
                .kind(),
            IcSnapshotUploadReplyKind::DataAcknowledgement
        ));
    }
    fixture.mutation(Method::StartCanister, None, false).await;
    fixture
        .pic
        .update_call(
            fixture.principal(),
            ic_agent::export::Principal::anonymous(),
            "write",
            99_u64.to_le_bytes().to_vec(),
        )
        .await
        .unwrap();
    fixture.mutation(Method::StopCanister, None, false).await;
    fixture
        .mutation(Method::LoadCanisterSnapshot, Some(destination), false)
        .await;
    assert_eq!(
        serde_json::to_value(
            fixture
                .pic
                .canister_status(fixture.principal(), Some(fixture.caller()))
                .await
                .unwrap()
                .status
        )
        .unwrap(),
        serde_json::to_value(CanisterStatusType::Stopped).unwrap()
    );
    fixture.mutation(Method::StartCanister, None, false).await;
    let restored = fixture
        .pic
        .query_call(
            fixture.principal(),
            ic_agent::export::Principal::anonymous(),
            "read",
            vec![],
        )
        .await
        .unwrap();
    assert_eq!(restored, [42_u64.to_le_bytes(); 3].concat());
    fs::write(fixture.root.join("qualification.json"),serde_json::to_vec_pretty(&serde_json::json!({"transport":"ic-agent 0.49.2","actual_signed_caller":fixture.caller().to_text(),"calls":fixture.sequence,"source_id":source_id,"original_reservations_pending":true,"automatic_receipts":false,"same_id_restore":true,"complete_heap_stable_global_state":true,"scope":"isolated stopped no-external-effects fixture; no generic application safety or full runner qualification"})).unwrap()).unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn wrong_trusted_root_rejects_actual_reply_without_settling_applied_stop() {
    let mut fixture = Fixture::new(support::identity()).await;
    let payload = support::payload(Method::StopCanister, &fixture.target, None);
    let (plan, layout, root) = fixture.original(&payload.digest());
    let guard = support::retain(&layout, &plan, fixture.sequence);
    let before = fs::read(guard.path()).unwrap();
    let request =
        IcMutationRequest::new(&plan, fixture.sequence, guard.record().unwrap(), &payload).unwrap();
    let mut key = fixture.key.clone();
    *key.last_mut().unwrap() ^= 1;
    let transport = AgentTransport::new(
        plan.context().clone(),
        &fixture.endpoint,
        support::identity(),
        key,
        Duration::from_secs(30),
    )
    .unwrap();
    let prepared = transport
        .prepare(ReservedUpdate::Mutation(&request), guard.record().unwrap())
        .unwrap();
    fs::write(root.join("signed-ingress.cbor"), prepared.envelope()).unwrap();
    assert!(matches!(
        prepared.submit().await,
        Err(TransportError::Indeterminate)
    ));
    assert_eq!(fs::read(guard.path()).unwrap(), before);
    assert_eq!(
        serde_json::to_value(
            fixture
                .pic
                .canister_status(fixture.principal(), Some(fixture.caller()))
                .await
                .unwrap()
                .status
        )
        .unwrap(),
        serde_json::to_value(CanisterStatusType::Stopped).unwrap()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn discarded_actual_stop_reply_halts_with_pending_original_and_no_reissue() {
    let mut fixture = Fixture::new(support::identity()).await;
    let (plan, _oracle) = fixture.mutation(Method::StopCanister, None, true).await;
    let root = fixture.root.join("call-1");
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let mut guard = ic_backup::ops::persistence::AttemptJournalGuard::open(
        &layout,
        &plan.attempt_authority(1).unwrap(),
    )
    .unwrap();
    let original = fs::read(guard.path()).unwrap();
    assert_eq!(guard.record().unwrap().view().pending_mutation, Some(1));
    assert!(guard.reserve_planned_mutation(&plan.digest()).is_err());
    assert_eq!(fs::read(guard.path()).unwrap(), original);
    assert_eq!(fixture.sequence, 1);
    assert_eq!(
        serde_json::to_value(
            fixture
                .pic
                .canister_status(fixture.principal(), Some(fixture.caller()))
                .await
                .unwrap()
                .status
        )
        .unwrap(),
        serde_json::to_value(CanisterStatusType::Stopped).unwrap()
    );
}
