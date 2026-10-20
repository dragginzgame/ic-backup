//! Explicit local simulator custody/accounting. No mainnet provider or retry loop.

use crate::ready::ready;
use ic_backup::model::ic_update_response::IcUpdateResponseInput;
use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::{
            AttemptJournalRecordError, MutationOutcomeRecord, MutationReceiptRequest,
        },
        ic_snapshot_transfer_read::{
            IcSnapshotTransferReadPayload, IcSnapshotTransferReadRequest,
            IcSnapshotTransferReadResponse,
        },
        operation_plan::OperationPlanRecord,
    },
    ops::persistence::{
        AttemptJournalError, AttemptJournalGuard, BackupLayoutGuard, create_operation_plan,
        read_operation_plan,
    },
    policy::ic_snapshot_transfer_read::{IcSnapshotTransferReadReply, validate_response},
    ports::{
        ic_observation::IcObservationProviderError,
        ic_snapshot_transfer_read::IcSnapshotTransferReadProvider,
    },
};
use ic_platform::principal::ExternalPrincipal;
use ic_testkit::{
    pic::{PocketIcManagedServer, PocketIcStartupConfig},
    pocket_ic::{PocketIc, PocketIcBuilder, common::rest::RawEffectivePrincipal},
};
use serde_json::json;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

pub(super) struct Backend {
    pub pic: PocketIc,
    // Instance teardown precedes the managed server, retaining its original output files.
    _server: PocketIcManagedServer,
    pub target: ExternalPrincipal,
    pub root: PathBuf,
    pub release: ArtifactChecksumRecord,
    pub calls: u64,
    plans: BTreeMap<u64, ArtifactChecksumRecord>,
    submitted_reads: BTreeSet<(String, u32)>,
    started: Instant,
    trace: Vec<serde_json::Value>,
}

impl Backend {
    pub fn new() -> Self {
        let root = crate::support::temp_root("ic-backup-pocketic");
        fs::write(
            root.join("original.wasm"),
            ic_host_artifacts::wasm_text::compile(
                include_str!("../state.wat"),
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
            .with_max_request_time_ms(Some(60_000))
            .with_state_dir(root.join("simulator"))
            .with_application_subnet();
        let owned = PocketIcStartupConfig::Spawn {
            server_binary: binary,
            timeout: Duration::from_secs(120),
            server_hard_ttl: None,
            server_idle_ttl: None,
            server_output_files: Some((root.join("server.stdout"), root.join("server.stderr"))),
        }
        .build_instance(builder)
        .unwrap();
        let pic = owned.instance;
        let server = owned.server.expect("spawn mode owns the managed server");
        let wasm = fs::read(root.join("original.wasm")).unwrap();
        let release = ArtifactChecksumRecord::from_bytes(&wasm);
        fs::write(root.join("fixture.wasm"), &wasm).unwrap();
        // Explicit simulator fixture setup, outside backup/restore authority.
        let target = pic.create_canister();
        pic.add_cycles(target, 100_000_000_000_000);
        pic.install_canister(target, wasm, vec![], None);
        pic.upload_chunk(target, None, b"retained chunk-store state".to_vec())
            .unwrap();
        fs::create_dir(root.join("calls")).unwrap();
        let backend = Self {
            pic,
            _server: server,
            target,
            root,
            release,
            calls: 0,
            plans: BTreeMap::new(),
            submitted_reads: BTreeSet::new(),
            started: Instant::now(),
            trace: Vec::new(),
        };
        backend.write_state(42);
        backend.assert_state(42);
        backend
    }

    pub fn plan(&self, digest: &ArtifactChecksumRecord, sequence: u64) -> OperationPlanRecord {
        let network = ArtifactChecksumRecord::from_bytes(
            format!(
                "explicit PocketIC instance {} at {}",
                self.pic.instance_id(),
                self.pic.get_server_url()
            )
            .as_bytes(),
        );
        serde_json::from_value(json!({
            "version":1,"context":{"network":network.hash(),"caller":ExternalPrincipal::anonymous().to_text(),"release":self.release.hash()},
            "inventory":{"version":1,"targets":[{"canister_id":self.target.to_text(),"parent_canister_id":null,"role":null,"module_hash":self.release.hash()}]},
            "selected_targets":[self.target.to_text()],"graph":{"version":1,"nodes":[{"operation_sequence":sequence,"depends_on":[]}]},
            "operations":[{"operation_sequence":sequence,"target":self.target.to_text(),"request":digest.hash(),"budget":{"mutations":1,"observations":1}}],"budget":{"mutations":1,"observations":1}
        })).unwrap()
    }

    pub fn layout(&self, index: u64) -> BackupLayoutGuard {
        BackupLayoutGuard::acquire(&self.root.join("calls").join(index.to_string())).unwrap()
    }

    pub fn retained_plan(&self, index: u64) -> OperationPlanRecord {
        read_operation_plan(&self.layout(index), &self.plans[&index]).unwrap()
    }

    pub fn management(&mut self, method: &str, arguments: &[u8]) -> Vec<u8> {
        assert!(self.calls < 64, "finite simulator management-call ceiling");
        self.calls += 1;
        let raw = self
            .pic
            .update_call_with_effective_principal(
                ExternalPrincipal::management_canister(),
                RawEffectivePrincipal::CanisterId(self.target.as_slice().into()),
                ExternalPrincipal::anonymous(),
                method,
                arguments.into(),
            )
            .unwrap_or_else(|error| panic!("PocketIC {method}: {error:?}"));
        self.trace.push(json!({"sequence":self.calls,"instance":self.pic.instance_id(),"receiver":"aaaaa-aa","target":self.target.to_text(),"caller":ExternalPrincipal::anonymous().to_text(),"method":method,"arguments_sha256":ArtifactChecksumRecord::from_bytes(arguments).hash(),"reply_sha256":ArtifactChecksumRecord::from_bytes(&raw).hash()}));
        fs::write(
            self.root.join("management-trace.json"),
            serde_json::to_vec_pretty(&self.trace).unwrap(),
        )
        .unwrap();
        raw
    }

    pub fn call(
        &mut self,
        method: &str,
        arguments: &[u8],
        digest: &ArtifactChecksumRecord,
        decode: impl FnOnce(&[u8]) -> ArtifactChecksumRecord,
        lose_reply: bool,
    ) -> (u64, Option<Vec<u8>>) {
        let (index, layout, plan) = self.begin_call(method, arguments, digest);
        let mut journal =
            AttemptJournalGuard::create(&layout, plan.attempt_authority(index).unwrap()).unwrap();
        let attempt = journal.reserve_mutation().unwrap();
        let raw = self.management(method, arguments);
        if lose_reply {
            // Test oracle retains the actually executed reply separately. Recovery
            // code sees no reply and cannot use this file to settle the mutation.
            Self::retain_lost_reply(layout.root(), &journal, attempt, &raw);
            return (index, None);
        }
        let evidence = decode(&raw);
        Self::retain_settled_reply(
            layout.root(),
            &mut journal,
            attempt,
            digest,
            &evidence,
            &raw,
        );
        (index, Some(raw))
    }

    pub fn read(&mut self, payload: IcSnapshotTransferReadPayload<'_, '_>) -> (u64, Vec<u8>) {
        let digest = payload.digest();
        let (index, layout, plan) = self.begin_call(payload.method(), payload.arguments(), &digest);
        let mut journal =
            AttemptJournalGuard::create(&layout, plan.attempt_authority(index).unwrap()).unwrap();
        let attempt = journal.reserve_mutation().unwrap();
        let request =
            IcSnapshotTransferReadRequest::new(&plan, index, journal.record().unwrap(), payload)
                .unwrap();
        let response = ready(self.read_snapshot(&request, journal.record().unwrap())).unwrap();
        let view = validate_response(&request, journal.record().unwrap(), &response).unwrap();
        let evidence = match view.reply() {
            IcSnapshotTransferReadReply::Metadata(reply) => reply.digest(),
            IcSnapshotTransferReadReply::Data(reply) => reply.digest(),
        };
        let raw = response.input().reply.clone();
        // Explicit isolated-instance qualification, not an automatic policy receipt.
        Self::retain_settled_reply(
            layout.root(),
            &mut journal,
            attempt,
            &digest,
            &evidence,
            &raw,
        );
        (index, raw)
    }

    fn begin_call(
        &mut self,
        method: &str,
        arguments: &[u8],
        digest: &ArtifactChecksumRecord,
    ) -> (u64, BackupLayoutGuard, OperationPlanRecord) {
        let index = self.calls + 1;
        let path = self.root.join("calls").join(index.to_string());
        fs::create_dir(&path).unwrap();
        let layout = BackupLayoutGuard::acquire(&path).unwrap();
        let plan = self.plan(digest, index);
        create_operation_plan(&layout, &plan).unwrap();
        assert!(self.plans.insert(index, plan.digest()).is_none());
        fs::write(path.join("arguments.candid"), arguments).unwrap();
        fs::write(path.join("method.txt"), method).unwrap();
        (index, layout, plan)
    }

    fn retain_lost_reply(root: &Path, journal: &AttemptJournalGuard<'_>, attempt: u32, raw: &[u8]) {
        fs::write(root.join("discarded-oracle-reply.candid"), raw).unwrap();
        assert_eq!(
            journal.record().unwrap().view().pending_mutation,
            Some(attempt)
        );
    }

    fn retain_settled_reply(
        root: &Path,
        journal: &mut AttemptJournalGuard<'_>,
        attempt: u32,
        request: &ArtifactChecksumRecord,
        evidence: &ArtifactChecksumRecord,
        raw: &[u8],
    ) {
        fs::write(root.join("reply.candid"), raw).unwrap();
        journal
            .record_mutation(MutationReceiptRequest {
                attempt,
                request: request.hash().into(),
                outcome: MutationOutcomeRecord::Applied,
                evidence: evidence.hash().into(),
            })
            .unwrap();
        assert_eq!(journal.record().unwrap().view().mutations_remaining, 0);
    }

    pub fn write_state(&self, value: u64) {
        self.pic
            .update_call(
                self.target,
                ExternalPrincipal::anonymous(),
                "write",
                value.to_le_bytes().into(),
            )
            .unwrap();
    }

    pub fn clear_fixture_chunk_store(&self) {
        // Explicit pre-restore fixture mutation, outside backup/restore spending.
        // Retained source artifacts and both remote source/destination snapshots survive.
        self.pic.clear_chunk_store(self.target, None).unwrap();
        assert_eq!(
            self.pic.stored_chunks(self.target, None).unwrap(),
            Vec::<Vec<u8>>::new()
        );
        fs::write(
            self.root.join("fixture-chunk-store-cleared.txt"),
            b"pre-load application fixture change; separate from backup ingress accounting\n",
        )
        .unwrap();
    }

    pub fn assert_state(&self, expected: u64) {
        let raw = self
            .pic
            .query_call(self.target, ExternalPrincipal::anonymous(), "read", vec![])
            .unwrap();
        assert_eq!(
            raw,
            expected.to_le_bytes().repeat(3),
            "heap/global/stable state"
        );
    }

    pub fn assert_local_replay(&self, pending: Option<u64>) {
        let before = self.calls;
        for (index, expected) in &self.plans {
            let layout = self.layout(*index);
            let plan = read_operation_plan(&layout, expected).unwrap();
            let journal =
                AttemptJournalGuard::open(&layout, &plan.attempt_authority(*index).unwrap())
                    .unwrap();
            let view = journal.record().unwrap().view();
            assert_eq!(view.mutations_used, 1);
            assert_eq!(view.mutations_remaining, 0);
            if pending == Some(*index) {
                assert_eq!(
                    (view.pending_mutation, view.pending_observation),
                    (Some(1), Some(2))
                );
                assert_eq!(view.observations_remaining, 0);
                assert!(!view.applied);
            } else {
                assert!(view.applied);
                assert_eq!(view.pending_mutation, None);
                assert_eq!(view.pending_observation, None);
            }
            let bytes = fs::read(journal.path()).unwrap();
            drop(journal);
            let reopened =
                AttemptJournalGuard::open(&layout, &plan.attempt_authority(*index).unwrap())
                    .unwrap();
            assert_eq!(fs::read(reopened.path()).unwrap(), bytes);
        }
        assert_eq!(self.calls, before);
    }
    pub fn assert_pending_lifecycle_replay(&self, sequence: u64) {
        let before = self.calls;
        let plan = self.retained_plan(sequence);
        let layout = self.layout(sequence);
        let mut journal =
            AttemptJournalGuard::open(&layout, &plan.attempt_authority(sequence).unwrap()).unwrap();
        let original = fs::read(journal.path()).unwrap();
        let view = journal.record().unwrap().view();
        assert_eq!(
            (view.pending_mutation, view.pending_observation),
            (Some(1), Some(2))
        );
        assert_eq!(
            (view.mutations_remaining, view.observations_remaining),
            (0, 0)
        );
        assert!(!view.applied);
        let observation_request = journal
            .record()
            .unwrap()
            .pending_observation_request()
            .unwrap()
            .to_owned();
        for denial in [
            journal.reserve_mutation(),
            journal.reserve_observation(1, &observation_request),
        ] {
            assert!(matches!(
                denial,
                Err(AttemptJournalError::Record(
                    AttemptJournalRecordError::ObservationPending { attempt: 2 }
                ))
            ));
        }
        assert_eq!(fs::read(journal.path()).unwrap(), original);
        drop(journal);
        let reopened =
            AttemptJournalGuard::open(&layout, &plan.attempt_authority(sequence).unwrap()).unwrap();
        assert_eq!(fs::read(reopened.path()).unwrap(), original);
        assert_eq!(reopened.record().unwrap().view(), view);
        assert_eq!(self.calls, before);
        assert!(!self.root.join("post-load-verification.json").exists());
        assert_eq!(
            self.trace
                .iter()
                .filter(|row| row["method"] == "start_canister")
                .count(),
            1,
            "only pre-restore fixture work started"
        );
        fs::write(self.root.join("pending-lifecycle.json"), serde_json::to_vec_pretty(&serde_json::json!({"sequence":sequence,"original_plan":plan.digest(),"retained_view":view,"management_calls":self.calls})).unwrap()).unwrap();
    }

    pub fn recovery_evidence(
        &self,
        plan: &OperationPlanRecord,
        sequence: u64,
        purpose: &str,
    ) -> ArtifactChecksumRecord {
        assert_eq!(self.plans[&sequence], plan.digest());
        let bytes = serde_json::to_vec(&json!({"purpose":purpose,"original_plan":plan.digest().hash(),"original_sequence":sequence,"context":plan.context(),"actual_target":self.target.to_text(),"actual_caller":ExternalPrincipal::anonymous().to_text(),"exclusive_completed_ingress_history":self.trace})).unwrap();
        fs::write(
            self.root
                .join("calls")
                .join(sequence.to_string())
                .join("qualification-evidence.json"),
            &bytes,
        )
        .unwrap();
        ArtifactChecksumRecord::from_bytes(&bytes)
    }

    pub fn complete(
        &self,
        fault: &str,
        original: &[u8],
        destination: &[u8],
        checksum: &ArtifactChecksumRecord,
        manifest: &ArtifactChecksumRecord,
    ) {
        assert_eq!(self.trace.len(), usize::try_from(self.calls).unwrap());
        assert_eq!(
            self.trace
                .iter()
                .filter(|row| row["method"] == "take_canister_snapshot")
                .count(),
            if self.root.join("post-load-verification.json").exists() {
                2
            } else {
                1
            }
        );
        assert_eq!(
            self.trace
                .iter()
                .filter(|row| row["method"] == "upload_canister_snapshot_metadata")
                .count(),
            1
        );
        for (method, expected) in [
            ("load_canister_snapshot", 1),
            ("stop_canister", 2),
            (
                "start_canister",
                if self.root.join("pending-lifecycle.json").exists() {
                    1
                } else {
                    2
                },
            ),
        ] {
            assert_eq!(
                self.trace
                    .iter()
                    .filter(|row| row["method"] == method)
                    .count(),
                expected,
                "exact intended lifecycle ingresses for {method}"
            );
        }
        let result = json!({"case":fault,"result":"passed","server":"16.0.0","client":"16.0.0","instance":self.pic.instance_id(),"target":self.target.to_text(),"release_sha256":self.release.hash(),"original_snapshot_id":original,"destination_snapshot_id":destination,"artifact_sha256":checksum.hash(),"manifest_sha256":manifest.hash(),"management_calls":self.calls,"elapsed_ms":self.started.elapsed().as_millis(),"journey_restored":self.root.join("post-load-verification.json").exists(),"pending_lifecycle":self.root.join("pending-lifecycle.json").exists(),"originals":self.plans.iter().map(|(sequence, digest)| json!({"sequence":sequence,"plan_sha256":digest.hash()})).collect::<Vec<_>>()});
        fs::write(
            self.root.join("qualification-result.json"),
            serde_json::to_vec_pretty(&result).unwrap(),
        )
        .unwrap();
    }
}

impl IcSnapshotTransferReadProvider for Backend {
    fn read_snapshot(
        &mut self,
        request: &IcSnapshotTransferReadRequest<'_, '_>,
        journal: &ic_backup::model::attempt_journal::AttemptJournalRecord,
    ) -> impl std::future::Future<
        Output = Result<IcSnapshotTransferReadResponse, IcObservationProviderError>,
    > {
        std::future::ready({
            request.validate_journal(journal).unwrap();
            // This private fixture owns the isolated instance and original ingress history.
            // It never reconstructs this memory as a production never-dispatched proof.
            let payload = request.payload();
            let actual = self.plan(
                &payload.digest(),
                request.authority().binding().operation_sequence(),
            );
            assert_eq!(request.plan().context(), actual.context());
            assert_eq!(payload.receiver(), "aaaaa-aa");
            assert_eq!(payload.target(), self.target.to_text());
            assert!(self.submitted_reads.insert((
                request.authority().digest().hash().into(),
                request.mutation_attempt()
            )));
            let raw = self.management(payload.method(), payload.arguments());
            IcSnapshotTransferReadResponse::new(IcUpdateResponseInput {
                authority: request.authority().digest(),
                mutation_attempt: request.mutation_attempt(),
                context: actual.context().clone(),
                target: self.target.to_text(),
                evidence: ArtifactChecksumRecord::from_bytes(&raw),
                reply: raw,
            })
            .map_err(|_| IcObservationProviderError::Indeterminate)
        })
    }
}
