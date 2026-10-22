//! Actual agent HTTP gateway effects on an isolated no-external-effects fixture.
mod fence_reconciliation;
mod independent;
mod support;
use ic_backup::ports::ic_mutation::{IcMutationProvider, IcMutationProviderError};
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
use ic_backup_agent::{
    AgentSnapshotTransferReadProvider, AgentTransport, AgentUpdateProvider, PreparedUpdate,
    ReservedUpdate,
};
use ic_platform::{
    management_canister_types::{CanisterStatusType, SnapshotDataKind},
    pocket_ic::{PocketIcBuilder, nonblocking::PocketIc},
};
use ic_testkit::pic::{PocketIcManagedServer, PocketIcStartupConfig};
use std::{fs, path::PathBuf, sync::Arc, time::Duration};

#[tokio::test(flavor = "multi_thread")]
#[expect(
    clippy::too_many_lines,
    reason = "keep original preflight, locked async dispatch and no-reissue proof in one trace"
)]
async fn original_capture_stage_awaits_accounted_fresh_admission_then_retains_one_signed_update() {
    use ic_backup::{
        model::execution_workflow::{ExecutionStageBindingRecord, ExecutionWorkflowRecord},
        ops::persistence::{AttemptJournalGuard, ExecutionStageGuard, create_execution_workflow},
        workflow::ic_snapshot_capture::capture_snapshot,
    };
    use std::{cell::Cell, io};
    let mut fixture = Fixture::new(support::identity()).await;
    fixture.mutation(Method::StopCanister, None, false).await;
    // Reserve this independent replicated status call before selected-stage admission.
    // Exclusive fixture custody proves it has never been dispatched. Its pending
    // accounting is retained independently; it is not a recovery observation.
    let status_payload = support::payload(Method::CanisterStatus, &fixture.target, None);
    let (status_plan, status_layout, status_root) = fixture.original(&status_payload.digest());
    let status_guard = support::retain(&status_layout, &status_plan, fixture.sequence);
    let status_path = status_guard.path().clone();
    let original_status = fs::read(&status_path).unwrap();
    drop(status_guard);
    let payload = support::payload(Method::TakeCanisterSnapshot, &fixture.target, None);
    let (plan, layout, _root) = fixture.original(&payload.digest());
    let sequence = fixture.sequence;
    let workflow = ExecutionWorkflowRecord::new(plan.clone());
    create_execution_workflow(&layout, &workflow).unwrap();
    let binding = ExecutionStageBindingRecord::new(&workflow, sequence, &plan, vec![]).unwrap();
    let stage = ExecutionStageGuard::prepare(&layout, binding.clone(), plan.clone()).unwrap();
    let transport = fixture.transport(&plan);
    let target = fixture.principal();
    let caller = fixture.caller();
    let retained = Cell::new(0);
    let observed = Cell::new(0);
    let mut provider = AgentUpdateProvider::new(
        &transport,
        |request: &ReservedUpdate<'_>, prepared: &PreparedUpdate<'_>| {
            let ReservedUpdate::Mutation(request) = request else {
                panic!("expected mutation retention")
            };
            assert_eq!(request.payload().digest(), payload.digest());
            assert!(
                AttemptJournalGuard::open(stage.layout().unwrap(), request.authority()).is_err()
            );
            assert_eq!(observed.get(), 1);
            support::retain_signed(stage.layout().unwrap().root(), prepared)
                .map_err(|_| IcMutationProviderError::Unavailable)?;
            retained.set(retained.get() + 1);
            Ok(())
        },
    );
    let acknowledgement =
        capture_snapshot(&stage, sequence, &payload, &mut provider, async |request| {
            assert_eq!(request.plan().context(), status_plan.context());
            assert_eq!(status_payload.target(), payload.target());
            assert_eq!(observed.get(), 0);
            assert_eq!(fs::read(&status_path)?, original_status);
            let actual = fixture
                .pic
                .canister_status(target, Some(caller))
                .await
                .unwrap();
            assert_eq!(
                serde_json::to_value(actual.status).unwrap(),
                serde_json::to_value(CanisterStatusType::Stopped).unwrap()
            );
            assert_eq!(actual.settings.controllers, vec![caller]);
            observed.set(1);
            fs::write(
                status_root.join("actual-status.json"),
                serde_json::to_vec(&actual).unwrap(),
            )?;
            // This stopped, drained isolated fixture has no external effects. Exclusive
            // controller/command custody admits capture here, never a product default.
            tokio::task::yield_now().await;
            assert!(
                AttemptJournalGuard::open(stage.layout().unwrap(), request.authority()).is_err()
            );
            Ok::<(), io::Error>(())
        })
        .await
        .unwrap();
    let raw = &acknowledgement.input().reply;
    assert_ne!(
        IcSnapshotReply::decode(&payload, raw).unwrap().snapshots()[0].id(),
        [] as [u8; 0]
    );
    fs::write(
        stage.layout().unwrap().root().join("passive-reply.candid"),
        raw,
    )
    .unwrap();
    let guard = AttemptJournalGuard::open(
        stage.layout().unwrap(),
        &plan.attempt_authority(sequence).unwrap(),
    )
    .unwrap();
    assert_eq!(guard.record().unwrap().view().pending_mutation, Some(1));
    let original = fs::read(guard.path()).unwrap();
    drop(guard);
    assert!(
        capture_snapshot(
            &stage,
            sequence,
            &payload,
            &mut provider,
            async |_| -> Result<(), io::Error> {
                panic!("pending capture never re-enters admission")
            }
        )
        .await
        .is_err()
    );
    assert_eq!((observed.get(), retained.get()), (1, 1));
    assert_eq!(
        fs::read(
            stage
                .layout()
                .unwrap()
                .root()
                .join(format!("attempt-{sequence}.json"))
        )
        .unwrap(),
        original
    );
    assert_eq!(fs::read(status_path).unwrap(), original_status);
}

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
    identity: Arc<dyn ic_platform::agent::Identity>,
}
impl Fixture {
    async fn new(identity: Arc<dyn ic_platform::agent::Identity>) -> Self {
        Self::with_wasm(identity, include_str!("state.wat")).await
    }
    async fn with_wasm(identity: Arc<dyn ic_platform::agent::Identity>, wasm_source: &str) -> Self {
        let root = support::root("pocketic");
        fs::write(
            root.join("original.wasm"),
            ic_host_artifacts::wasm_text::compile(
                wasm_source,
                ic_host_artifacts::wasm_text::CompileLimits {
                    source_bytes: 64 * 1024,
                    module_bytes: 64 * 1024,
                },
            )
            .unwrap(),
        )
        .unwrap();
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let bundle = manifest
            .ancestors()
            .nth(2)
            .expect("fixture package belongs to this workspace")
            .join(".tools/testkit-server");
        let binary = ic_testkit::pic::server_assets::check(&bundle)
            .expect("offline Testkit admission; prepare with make install-testkit-server");
        assert!(
            binary.is_absolute(),
            "Testkit must admit an absolute server path"
        );
        let digest = ic_host_fs::read::hash_file_no_follow(&binary, 512 * 1024 * 1024)
            .expect("retain admitted server digest");
        ic_host_fs::durable::write_bytes(
            &root.join("server.sha256"),
            ic_host_fs::durable::WriteOptions {
                mode: ic_host_fs::durable::PublicationMode::CreateNew,
                permissions: 0o600,
            },
            digest.sha256.to_string().as_bytes(),
        )
        .expect("retain private original server identity");
        let builder = PocketIcBuilder::new()
            .with_application_subnet()
            .with_nns_subnet()
            .with_state_dir(root.join("simulator"))
            .with_max_request_time_ms(Some(60_000));
        let owned = PocketIcStartupConfig::Spawn {
            server_binary: binary,
            timeout: Duration::from_secs(120),
            server_hard_ttl: None,
            server_idle_ttl: None,
            server_output_files: Some((root.join("server.stdout"), root.join("server.stderr"))),
        }
        .build_instance_async(builder)
        .await
        .unwrap();
        let mut pic = owned.instance;
        let server = owned.server.expect("spawn mode owns the managed server");
        let target = pic.create_canister().await;
        pic.add_cycles(target, 100_000_000_000_000).await;
        let wasm = fs::read(root.join("original.wasm")).unwrap();
        let release = ArtifactChecksumRecord::from_bytes(&wasm);
        fs::write(root.join("fixture.wasm"), &wasm).unwrap();
        pic.install_canister(target, wasm, vec![], None).await;
        pic.update_call(
            target,
            ic_platform::principal::ExternalPrincipal::anonymous(),
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
    fn caller(&self) -> ic_platform::principal::ExternalPrincipal {
        self.identity.sender().unwrap()
    }
    fn principal(&self) -> ic_platform::principal::ExternalPrincipal {
        ic_platform::Principal::from_text(&self.target)
            .unwrap()
            .into()
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
        let mut provider = AgentUpdateProvider::new(
            &transport,
            |_: &ReservedUpdate<'_>, prepared: &PreparedUpdate<'_>| {
                support::retain_signed(&root, prepared)
                    .map_err(|_| IcMutationProviderError::Unavailable)
            },
        );
        let acknowledgement = provider
            .submit_mutation(&request, guard.record().unwrap())
            .await
            .unwrap();
        let raw = acknowledgement.input().reply.clone();
        fs::write(root.join("reply.candid"), &raw).unwrap();
        assert_eq!(fs::read(guard.path()).unwrap(), before);
        assert_eq!(guard.record().unwrap().view().pending_mutation, Some(1));
        if lose {
            fs::write(root.join("discarded-oracle-reply.candid"), &raw).unwrap();
        }
        (plan, raw)
    }
    async fn read(&mut self, payload: IcSnapshotTransferReadPayload<'_, '_>) -> Vec<u8> {
        use ic_backup::{
            model::execution_workflow::{ExecutionStageBindingRecord, ExecutionWorkflowRecord},
            ops::persistence::{
                AttemptJournalGuard, ExecutionStageGuard, create_execution_workflow,
            },
            ports::ic_observation::IcObservationProviderError,
            workflow::ic_snapshot_transfer_read::read_snapshot,
        };
        let (plan, layout, root) = self.original(&payload.digest());
        let workflow = ExecutionWorkflowRecord::new(plan.clone());
        create_execution_workflow(&layout, &workflow).unwrap();
        let binding =
            ExecutionStageBindingRecord::new(&workflow, self.sequence, &plan, vec![]).unwrap();
        let stage = ExecutionStageGuard::prepare(&layout, binding, plan.clone()).unwrap();
        let transport = self.transport(&plan);
        let mut provider = AgentSnapshotTransferReadProvider::new(
            &transport,
            |_: &IcSnapshotTransferReadRequest<'_, '_>, prepared: &PreparedUpdate<'_>| {
                support::retain_signed(&root, prepared)
                    .map_err(|_| IcObservationProviderError::Unavailable)
            },
        );
        let response = read_snapshot(
            &stage,
            self.sequence,
            payload,
            &mut provider,
            async |request| {
                assert_eq!(request.plan().context(), plan.context());
                // This controlled stopped fixture retains original snapshot identity,
                // controller/read access and exclusive never-dispatched ingress custody.
                tokio::task::yield_now().await;
                assert!(
                    AttemptJournalGuard::open(stage.layout().unwrap(), request.authority())
                        .is_err()
                );
                Ok::<(), std::io::Error>(())
            },
        )
        .await
        .unwrap();
        let guard = AttemptJournalGuard::open(
            stage.layout().unwrap(),
            &plan.attempt_authority(self.sequence).unwrap(),
        )
        .unwrap();
        assert_eq!(guard.record().unwrap().view().pending_mutation, Some(1));
        let raw = response.input().reply.clone();
        fs::write(root.join("reply.candid"), &raw).unwrap();
        raw
    }
    async fn upload(&mut self, payload: &IcSnapshotUploadRequest<'_>) -> Vec<u8> {
        use ic_backup::{
            model::execution_workflow::{ExecutionStageBindingRecord, ExecutionWorkflowRecord},
            ops::persistence::{
                AttemptJournalGuard, ExecutionStageGuard, create_execution_workflow,
            },
            ports::ic_mutation::IcMutationProviderError,
            workflow::ic_snapshot_upload::upload_snapshot,
        };
        let (plan, layout, root) = self.original(&payload.binding_digest());
        let workflow = ExecutionWorkflowRecord::new(plan.clone());
        create_execution_workflow(&layout, &workflow).unwrap();
        let binding =
            ExecutionStageBindingRecord::new(&workflow, self.sequence, &plan, vec![]).unwrap();
        let stage = ExecutionStageGuard::prepare(&layout, binding, plan.clone()).unwrap();
        let transport = self.transport(&plan);
        let mut provider = ic_backup_agent::AgentUpdateProvider::new(
            &transport,
            |_: &ReservedUpdate<'_>, prepared: &PreparedUpdate<'_>| {
                support::retain_signed(&root, prepared)
                    .map_err(|_| IcMutationProviderError::Unavailable)
            },
        );
        let acknowledgement = upload_snapshot(
            &stage,
            self.sequence,
            payload,
            &mut provider,
            async |request| {
                assert_eq!(request.plan().context(), plan.context());
                // Only this isolated fixture owns the authenticated complete source,
                // allocation attribution, controller and never-dispatched ingress custody.
                tokio::task::yield_now().await;
                assert!(
                    AttemptJournalGuard::open(stage.layout().unwrap(), request.authority())
                        .is_err()
                );
                Ok::<(), std::io::Error>(())
            },
        )
        .await
        .unwrap();
        let guard = AttemptJournalGuard::open(
            stage.layout().unwrap(),
            &plan.attempt_authority(self.sequence).unwrap(),
        )
        .unwrap();
        assert_eq!(guard.record().unwrap().view().pending_mutation, Some(1));
        let raw = acknowledgement.input().reply.clone();
        fs::write(root.join("reply.candid"), &raw).unwrap();
        raw
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
        Vec::<ic_platform::management_canister_types::ChunkHash>::new()
    );
    chunks
}

#[tokio::test(flavor = "multi_thread")]
async fn real_agent_capture_complete_transfer_upload_and_same_id_restore_preserve_originals() {
    let mut fixture = Fixture::new(Arc::new(ic_platform::agent::BasicIdentity::from_raw_key(
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
            ic_platform::principal::ExternalPrincipal::anonymous(),
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
            ic_platform::principal::ExternalPrincipal::anonymous(),
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
    let mut provider = AgentUpdateProvider::new(
        &transport,
        |_: &ReservedUpdate<'_>, prepared: &PreparedUpdate<'_>| {
            support::retain_signed(&root, prepared)
                .map_err(|_| IcMutationProviderError::Indeterminate)
        },
    );
    assert!(matches!(
        provider
            .submit_mutation(&request, guard.record().unwrap())
            .await,
        Err(IcMutationProviderError::Indeterminate)
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

#[tokio::test(flavor = "multi_thread")]
async fn reserved_agent_status_after_discarded_stop_reply_retains_both_pending_attempts() {
    use ic_backup::{
        model::ic_observation::{IcObservationRequest, IcObservationResponse},
        policy::ic_observation::{IcObservationReplyView, validate_response},
        ports::ic_observation::{IcObservationProvider, IcObservationProviderError},
    };
    use ic_backup_agent::AgentObservationProvider;
    let mut fixture = Fixture::new(support::identity()).await;
    let (plan, _) = fixture.mutation(Method::StopCanister, None, true).await;
    let root = fixture.root.join("call-1");
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let authority = plan.attempt_authority(1).unwrap();
    let mut guard =
        ic_backup::ops::persistence::AttemptJournalGuard::open(&layout, &authority).unwrap();
    let mutation = support::payload(Method::StopCanister, &fixture.target, None);
    let observation = support::payload(Method::CanisterStatus, &fixture.target, None);
    guard
        .reserve_observation(1, observation.digest().hash())
        .unwrap();
    let before = fs::read(guard.path()).unwrap();
    let request =
        IcObservationRequest::new(&plan, 1, guard.record().unwrap(), &mutation, &observation)
            .unwrap();
    let retained = root.join("reserved-status");
    fs::create_dir(&retained).unwrap();
    let transport = fixture.transport(&plan);
    let mut provider = AgentObservationProvider::new(
        &transport,
        |actual: &ReservedUpdate<'_>, prepared: &PreparedUpdate<'_>| {
            let ReservedUpdate::Observation(actual) = actual else {
                panic!("one original status only")
            };
            assert_eq!(actual.observation_attempt(), 2);
            assert!(
                ic_backup::ops::persistence::AttemptJournalGuard::open(&layout, actual.authority())
                    .is_err()
            );
            support::retain_signed(&retained, prepared)
                .map_err(|_| IcObservationProviderError::Unavailable)
        },
    );
    let response: IcObservationResponse = provider
        .observe(&request, guard.record().unwrap())
        .await
        .unwrap();
    let associated = validate_response(&request, guard.record().unwrap(), &response).unwrap();
    // Actual certified status is passive recovery evidence, never a stop receipt.
    fs::write(retained.join("reply.candid"), &response.input().reply).unwrap();
    let IcObservationReplyView::Status(reply) = associated.reply() else {
        panic!("status projection required")
    };
    let ic_backup::model::ic_lifecycle_reply::IcLifecycleReplyKind::Status(status) = reply.kind()
    else {
        panic!("decoded status required")
    };
    assert_eq!(status.status(), CanisterStatusType::Stopped);
    assert_eq!(fs::read(guard.path()).unwrap(), before);
    assert_eq!(guard.record().unwrap().view().pending_mutation, Some(1));
    assert_eq!(guard.record().unwrap().view().pending_observation, Some(2));
    assert_eq!(
        provider
            .observe(&request, guard.record().unwrap())
            .await
            .unwrap_err(),
        IcObservationProviderError::Unavailable
    );
    assert_eq!(fs::read(guard.path()).unwrap(), before);
}

#[tokio::test(flavor = "multi_thread")]
#[expect(
    clippy::too_many_lines,
    reason = "one actual source/allocation/full-MiB-write/readback custody trace keeps original reservations and separate reply bounds visible"
)]
async fn actual_agent_allocation_list_and_full_mib_readback_keep_both_originals_pending() {
    use ic_backup::{
        model::{
            ic_snapshot_upload_data_observation::IcSnapshotUploadDataObservationRequest,
            ic_snapshot_upload_observation::IcSnapshotUploadObservationRequest,
        },
        policy::ic_observation::IcObservationReplyView,
        ports::{
            ic_observation::IcObservationProviderError,
            ic_snapshot_upload::IcSnapshotUploadProvider,
            ic_snapshot_upload_data_observation::IcSnapshotUploadDataObservationProvider,
            ic_snapshot_upload_observation::IcSnapshotUploadObservationProvider,
        },
    };
    use ic_backup_agent::AgentObservationProvider;
    let mut fixture = Fixture::new(support::identity()).await;
    fixture.mutation(Method::StopCanister, None, false).await;
    let (source_plan, capture_raw) = fixture
        .mutation(Method::TakeCanisterSnapshot, None, false)
        .await;
    let capture = support::payload(Method::TakeCanisterSnapshot, &fixture.target, None);
    let source_id = IcSnapshotReply::decode(&capture, &capture_raw)
        .unwrap()
        .snapshots()[0]
        .id()
        .to_vec();
    let metadata_request = IcSnapshotMetadataRequest::new(&fixture.target, &source_id).unwrap();
    let raw = fixture
        .read(IcSnapshotTransferReadPayload::Metadata(&metadata_request))
        .await;
    let metadata = IcSnapshotMetadataReply::decode(&metadata_request, &raw).unwrap();
    let chunks = read_complete_regions(&mut fixture, &metadata).await;
    let checksum = ArtifactChecksumRecord::from_bytes(
        &chunks
            .iter()
            .flat_map(|(_, b)| b.iter().copied())
            .collect::<Vec<_>>(),
    );
    let upload = IcSnapshotUploadRequest::metadata(&source_plan, &metadata, &checksum).unwrap();
    let destination;
    {
        let (plan, layout, root) = fixture.original(&upload.binding_digest());
        let sequence = fixture.sequence;
        let mut journal = support::retain(&layout, &plan, sequence);
        let transport = fixture.transport(&plan);
        let mut mutations = AgentUpdateProvider::new(
            &transport,
            |_: &ReservedUpdate<'_>, prepared: &PreparedUpdate<'_>| {
                support::retain_signed(&root, prepared)
                    .map_err(|_| IcMutationProviderError::Unavailable)
            },
        );
        let request =
            IcSnapshotUploadAttempt::new(&plan, sequence, journal.record().unwrap(), &upload)
                .unwrap();
        let ack = mutations
            .submit_upload(&request, journal.record().unwrap())
            .await
            .unwrap();
        let allocation = IcSnapshotUploadReply::decode(&upload, &ack.input().reply).unwrap();
        let IcSnapshotUploadReplyKind::Metadata { snapshot_id } = allocation.kind() else {
            panic!("actual new destination required")
        };
        destination = snapshot_id.clone();
        fs::write(root.join("allocation-reply.candid"), &ack.input().reply).unwrap();
        let list = support::payload(Method::ListCanisterSnapshots, &fixture.target, None);
        journal
            .reserve_observation(1, list.digest().hash())
            .unwrap();
        let before = fs::read(journal.path()).unwrap();
        let request = IcSnapshotUploadObservationRequest::new(
            &plan,
            sequence,
            journal.record().unwrap(),
            &upload,
            &list,
        )
        .unwrap();
        let retained = root.join("reserved-list");
        fs::create_dir(&retained).unwrap();
        let mut observations = AgentObservationProvider::new(
            &transport,
            |actual: &ReservedUpdate<'_>, prepared: &PreparedUpdate<'_>| {
                assert!(matches!(actual, ReservedUpdate::UploadObservation(_)));
                support::retain_signed(&retained, prepared)
                    .map_err(|_| IcObservationProviderError::Unavailable)
            },
        );
        let response = observations
            .observe_upload(&request, journal.record().unwrap())
            .await
            .unwrap();
        let view = ic_backup::policy::ic_snapshot_upload_observation::validate_response(
            &request,
            journal.record().unwrap(),
            &response,
        )
        .unwrap();
        let IcObservationReplyView::Inventory(inventory) = view.reply() else {
            panic!("actual inventory required")
        };
        assert!(inventory.snapshots().iter().any(|s| s.id() == destination));
        fs::write(retained.join("reply.candid"), &response.input().reply).unwrap();
        assert_eq!(fs::read(journal.path()).unwrap(), before);
        assert_eq!(journal.record().unwrap().view().pending_mutation, Some(1));
        assert_eq!(
            journal.record().unwrap().view().pending_observation,
            Some(2)
        );
        assert_eq!(
            observations
                .observe_upload(&request, journal.record().unwrap())
                .await
                .unwrap_err(),
            IcObservationProviderError::Unavailable
        );
    }
    // Fresh destination metadata is an independently accounted ordinary read.
    let destination_request =
        IcSnapshotMetadataRequest::new(&fixture.target, &destination).unwrap();
    let raw = fixture
        .read(IcSnapshotTransferReadPayload::Metadata(
            &destination_request,
        ))
        .await;
    let destination_metadata = IcSnapshotMetadataReply::decode(&destination_request, &raw).unwrap();
    let (kind, bytes) = chunks
        .iter()
        .find(|(k, _)| matches!(k, SnapshotDataKind::WasmMemory { .. }))
        .unwrap();
    assert_eq!(bytes.len(), 1024 * 1024);
    let data = IcSnapshotUploadRequest::data(&upload, &destination, kind.clone(), bytes).unwrap();
    let read = IcSnapshotDataRequest::new(&destination_metadata, kind.clone()).unwrap();
    let (plan, layout, root) = fixture.original(&data.binding_digest());
    let sequence = fixture.sequence;
    let mut journal = support::retain(&layout, &plan, sequence);
    let transport = fixture.transport(&plan);
    let mut mutations = AgentUpdateProvider::new(
        &transport,
        |_: &ReservedUpdate<'_>, prepared: &PreparedUpdate<'_>| {
            support::retain_signed(&root, prepared)
                .map_err(|_| IcMutationProviderError::Unavailable)
        },
    );
    let request =
        IcSnapshotUploadAttempt::new(&plan, sequence, journal.record().unwrap(), &data).unwrap();
    let ack = mutations
        .submit_upload(&request, journal.record().unwrap())
        .await
        .unwrap();
    assert!(matches!(
        IcSnapshotUploadReply::decode(&data, &ack.input().reply)
            .unwrap()
            .kind(),
        IcSnapshotUploadReplyKind::DataAcknowledgement
    ));
    fs::write(root.join("write-reply.candid"), &ack.input().reply).unwrap();
    journal
        .reserve_observation(1, read.digest().hash())
        .unwrap();
    let before = fs::read(journal.path()).unwrap();
    let request = IcSnapshotUploadDataObservationRequest::new(
        &plan,
        sequence,
        journal.record().unwrap(),
        &data,
        &read,
    )
    .unwrap();
    let retained = root.join("reserved-readback");
    fs::create_dir(&retained).unwrap();
    let mut observations = AgentObservationProvider::new(
        &transport,
        |actual: &ReservedUpdate<'_>, prepared: &PreparedUpdate<'_>| {
            assert!(matches!(actual, ReservedUpdate::UploadDataObservation(_)));
            support::retain_signed(&retained, prepared)
                .map_err(|_| IcObservationProviderError::Unavailable)
        },
    );
    let response = observations
        .observe_upload_data(&request, journal.record().unwrap())
        .await
        .unwrap();
    let view = ic_backup::policy::ic_snapshot_upload_data_observation::validate_response(
        &request,
        journal.record().unwrap(),
        &response,
    )
    .unwrap();
    assert_eq!(view.reply().chunk(), bytes);
    assert!(view.matches_original_chunk());
    assert!(response.input().reply.len() > 1024 * 1024);
    fs::write(retained.join("reply.candid"), &response.input().reply).unwrap();
    assert_eq!(fs::read(journal.path()).unwrap(), before);
    assert_eq!(journal.record().unwrap().view().pending_mutation, Some(1));
    assert_eq!(
        journal.record().unwrap().view().pending_observation,
        Some(2)
    );
    assert_eq!(
        observations
            .observe_upload_data(&request, journal.record().unwrap())
            .await
            .unwrap_err(),
        IcObservationProviderError::Unavailable
    );
}

#[tokio::test(flavor = "multi_thread")]
#[expect(
    clippy::too_many_lines,
    reason = "keep retained originals, locked acquisition and actual reply-custody failure chronological"
)]
async fn configured_application_update_retains_empty_reply_without_claiming_active_fence() {
    use ic_backup::{
        model::{
            consistency::{
                ApplicationFenceBinding, ConsistencyGuaranteeRecord, ConsistencyRequirementRecord,
            },
            execution_workflow::{ExecutionStageBindingRecord, ExecutionWorkflowRecord},
            fence_acquisition::{FenceAcquisitionPayload, FenceAcquisitionRequest},
            fence_obligation::FenceObligationRecord,
        },
        ops::persistence::{
            AttemptJournalGuard, ExecutionStageGuard, FenceObligationRequirement,
            create_consistency_requirement, create_execution_workflow, create_fence_obligation,
        },
        policy::fence_acquisition::validate_acknowledgement,
        ports::fence_acquisition::FenceAcquisitionProviderError,
        workflow::fence_acquisition::{FenceAcquisitionExecutionError, acquire_fence},
    };
    use ic_backup_agent::AgentFenceAcquisitionProvider;
    let mut fixture = Fixture::new(support::identity()).await;
    for refuse_reply in [false, true] {
        let value = if refuse_reply { 1000_u64 } else { 999_u64 };
        // This fixture update changes a counter, not an application fence. It qualifies
        // opaque transport and custody only; no whole-unit acquisition is inferred.
        let payload =
            FenceAcquisitionPayload::new(&fixture.target, "write", &value.to_le_bytes()).unwrap();
        let (plan, layout, _root) = fixture.original(&payload.digest());
        let sequence = fixture.sequence;
        let workflow = ExecutionWorkflowRecord::new(plan.clone());
        create_execution_workflow(&layout, &workflow).unwrap();
        let binding = ExecutionStageBindingRecord::new(&workflow, sequence, &plan, vec![]).unwrap();
        let stage = ExecutionStageGuard::prepare(&layout, binding, plan.clone()).unwrap();
        let root = stage.layout().unwrap().root().to_path_buf();
        let requirement = ConsistencyRequirementRecord::new(
            &plan,
            ConsistencyGuaranteeRecord::ApplicationCoordinated,
        );
        let fence = ApplicationFenceBinding {
            identity: ArtifactChecksumRecord::from_bytes(b"fixture original fence"),
            membership_revision: ArtifactChecksumRecord::from_bytes(b"fixture original membership"),
        };
        let retained_requirement = FenceObligationRequirement::Capture {
            requirement: &requirement,
            fence: &fence,
        };
        create_consistency_requirement(stage.layout().unwrap(), &plan, &requirement).unwrap();
        let obligation =
            FenceObligationRecord::for_capture(&plan, &requirement, sequence, &fence).unwrap();
        create_fence_obligation(
            stage.layout().unwrap(),
            &plan,
            retained_requirement,
            &obligation,
        )
        .unwrap();
        let original_obligation = fs::read(root.join("fence-obligation.json")).unwrap();
        let authority = plan.attempt_authority(sequence).unwrap();
        let transport = fixture.transport(&plan);
        let mut provider = AgentFenceAcquisitionProvider::new(
            &transport,
            |actual: &FenceAcquisitionRequest<'_>, prepared: &PreparedUpdate<'_>| {
                assert_eq!(actual.authority(), &authority);
                assert!(
                    AttemptJournalGuard::open(stage.layout().unwrap(), actual.authority()).is_err()
                );
                support::retain_signed(&root, prepared)
                    .map_err(|_| FenceAcquisitionProviderError::Unavailable)
            },
            |actual: &FenceAcquisitionRequest<'_>,
             id: &ic_platform::agent::RequestId,
             reply: &[u8]| {
                assert!(
                    AttemptJournalGuard::open(stage.layout().unwrap(), actual.authority()).is_err()
                );
                assert_eq!(
                    fs::read_to_string(root.join("request-id.txt")).unwrap(),
                    id.to_string()
                );
                assert_eq!(reply, &[] as &[u8]);
                if refuse_reply {
                    return Err(FenceAcquisitionProviderError::Unavailable);
                }
                retain_reply(&root, reply).map_err(|_| FenceAcquisitionProviderError::Unavailable)
            },
        );
        let result = acquire_fence(
            &stage,
            retained_requirement,
            &obligation,
            &payload,
            &mut provider,
            async |actual| {
                assert_eq!(actual.obligation(), &obligation);
                tokio::task::yield_now().await;
                assert!(
                    AttemptJournalGuard::open(stage.layout().unwrap(), actual.authority()).is_err()
                );
                // Isolated fixture admission for an exact opaque write, not Active-fence proof.
                Ok::<(), std::convert::Infallible>(())
            },
        )
        .await;
        let guard = AttemptJournalGuard::open(stage.layout().unwrap(), &authority).unwrap();
        let original = fs::read(guard.path()).unwrap();
        let request =
            FenceAcquisitionRequest::new(&plan, &obligation, guard.record().unwrap(), &payload)
                .unwrap();
        if refuse_reply {
            assert!(matches!(
                result,
                Err(FenceAcquisitionExecutionError::Provider(
                    FenceAcquisitionProviderError::Indeterminate
                ))
            ));
            assert!(!root.join("reply.candid").exists());
        } else {
            validate_acknowledgement(&request, guard.record().unwrap(), &result.unwrap()).unwrap();
            assert_eq!(
                fs::read(root.join("reply.candid")).unwrap(),
                Vec::<u8>::new()
            );
        }
        assert_eq!(guard.record().unwrap().view().pending_mutation, Some(1));
        drop(guard);
        assert!(
            acquire_fence(
                &stage,
                retained_requirement,
                &obligation,
                &payload,
                &mut provider,
                async |_| -> Result<(), std::convert::Infallible> {
                    panic!("pending acquisition reached fresh admission again")
                }
            )
            .await
            .is_err()
        );
        assert_eq!(
            fs::read(root.join(format!("attempt-{sequence}.json"))).unwrap(),
            original
        );
        assert_eq!(
            fs::read(root.join("fence-obligation.json")).unwrap(),
            original_obligation
        );
        // Independently controlled Testkit oracle confirms dispatch despite failed reply retention.
        let observed_bytes = fixture
            .pic
            .query_call(fixture.principal(), fixture.caller(), "read", vec![])
            .await
            .unwrap();
        assert_eq!(&observed_bytes[..8], &value.to_le_bytes());
        fs::write(root.join("oracle-state.bin"), observed_bytes).unwrap();
    }
}

/// Create-only durable exact passive reply evidence, including valid empty application replies.
fn retain_reply(root: &std::path::Path, reply: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(root.join("reply.candid"))?;
    file.write_all(reply)?;
    file.sync_all()?;
    fs::File::open(root)?.sync_all()
}
