//! Explicit local simulator custody/accounting. No mainnet provider or retry loop.

use candid::Principal;
use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::{
            AttemptJournalRecordError, MutationOutcomeRecord, MutationReceiptRequest,
        },
        operation_plan::OperationPlanRecord,
    },
    ops::persistence::{
        AttemptJournalError, AttemptJournalGuard, BackupLayoutGuard, create_operation_plan,
        read_operation_plan,
    },
};
use pocket_ic::{PocketIc, PocketIcBuilder, common::rest::RawEffectivePrincipal};
use serde_json::json;
use std::{collections::BTreeMap, fs, path::PathBuf, time::Instant};

pub(super) struct Backend {
    pub pic: PocketIc,
    pub target: Principal,
    pub root: PathBuf,
    pub release: ArtifactChecksumRecord,
    pub calls: u64,
    plans: BTreeMap<u64, ArtifactChecksumRecord>,
    started: Instant,
    trace: Vec<serde_json::Value>,
}

impl Backend {
    pub fn new() -> Self {
        let root = crate::support::temp_root("ic-backup-pocketic");
        let binary = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../.tools/ic/bin/pocket-ic")
            .canonicalize()
            .expect("prepare the pinned PocketIC server with make install-ic-tools");
        let version = std::process::Command::new(&binary)
            .arg("--version")
            .output()
            .unwrap();
        assert!(version.status.success());
        assert_eq!(
            String::from_utf8(version.stdout).unwrap().trim(),
            "pocket-ic-server 16.0.0"
        );
        let server_digest = ArtifactChecksumRecord::from_bytes(&fs::read(&binary).unwrap());
        let installed = fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.tools/ic/files.sha256"),
        )
        .unwrap();
        let expected = installed
            .lines()
            .find_map(|line| line.strip_suffix("  bin/pocket-ic"))
            .expect("installed PocketIC checksum");
        assert_eq!(server_digest.hash(), expected);
        fs::write(root.join("server.sha256"), server_digest.hash()).unwrap();
        let pic = PocketIcBuilder::new()
            .with_server_binary(binary)
            .with_max_request_time_ms(Some(60_000))
            .with_state_dir(root.join("simulator"))
            .with_application_subnet()
            .build();
        let wasm = wat::parse_str(include_str!("../state.wat")).unwrap();
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
            target,
            root,
            release,
            calls: 0,
            plans: BTreeMap::new(),
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
            "version":1,"context":{"network":network.hash(),"caller":Principal::anonymous().to_text(),"release":self.release.hash()},
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
                Principal::management_canister(),
                RawEffectivePrincipal::CanisterId(self.target.as_slice().into()),
                Principal::anonymous(),
                method,
                arguments.into(),
            )
            .unwrap_or_else(|error| panic!("PocketIC {method}: {error:?}"));
        self.trace.push(json!({"sequence":self.calls,"instance":self.pic.instance_id(),"receiver":"aaaaa-aa","target":self.target.to_text(),"caller":Principal::anonymous().to_text(),"method":method,"arguments_sha256":ArtifactChecksumRecord::from_bytes(arguments).hash(),"reply_sha256":ArtifactChecksumRecord::from_bytes(&raw).hash()}));
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
        let index = self.calls + 1;
        let path = self.root.join("calls").join(index.to_string());
        fs::create_dir(&path).unwrap();
        let layout = BackupLayoutGuard::acquire(&path).unwrap();
        let plan = self.plan(digest, index);
        create_operation_plan(&layout, &plan).unwrap();
        assert!(self.plans.insert(index, plan.digest()).is_none());
        fs::write(path.join("arguments.candid"), arguments).unwrap();
        fs::write(path.join("method.txt"), method).unwrap();
        let mut journal =
            AttemptJournalGuard::create(&layout, plan.attempt_authority(index).unwrap()).unwrap();
        let attempt = journal.reserve_mutation().unwrap();
        let raw = self.management(method, arguments);
        if lose_reply {
            // Test oracle retains the actually executed reply separately. Recovery
            // code sees no reply and cannot use this file to settle the mutation.
            fs::write(path.join("discarded-oracle-reply.candid"), &raw).unwrap();
            assert_eq!(
                journal.record().unwrap().view().pending_mutation,
                Some(attempt)
            );
            return (index, None);
        }
        let evidence = decode(&raw);
        fs::write(path.join("reply.candid"), &raw).unwrap();
        journal
            .record_mutation(MutationReceiptRequest {
                attempt,
                request: digest.hash().into(),
                outcome: MutationOutcomeRecord::Applied,
                evidence: evidence.hash().into(),
            })
            .unwrap();
        assert_eq!(journal.record().unwrap().view().mutations_remaining, 0);
        (index, Some(raw))
    }

    pub fn write_state(&self, value: u64) {
        self.pic
            .update_call(
                self.target,
                Principal::anonymous(),
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
            .query_call(self.target, Principal::anonymous(), "read", vec![])
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
        let bytes = serde_json::to_vec(&json!({"purpose":purpose,"original_plan":plan.digest().hash(),"original_sequence":sequence,"context":plan.context(),"actual_target":self.target.to_text(),"actual_caller":Principal::anonymous().to_text(),"exclusive_completed_ingress_history":self.trace})).unwrap();
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
