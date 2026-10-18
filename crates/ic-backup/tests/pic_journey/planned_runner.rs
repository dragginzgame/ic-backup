//! One-shot public workflow coordinator against actual managed `PocketIC` ingress.

use super::{
    Fault,
    backend::Backend,
    lifecycle, management,
    planned_download::{
        CaptureFailure, CaptureProvider, ReadFailure, ReadProvider, workflow as allocated_workflow,
    },
    status,
};
use crate::ready::ready;
use crate::support::read_json;
use ic_backup::model::ic_update_response::IcUpdateResponseInput;
use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::{
            AppliedReceiptEvidence, AttemptJournalRecord, MAX_ATTEMPT_JOURNAL_BYTES,
            MutationOutcomeRecord, MutationReceiptRequest,
        },
        download_journal::DownloadArtifactRequest,
        execution_workflow::{
            ExecutionStageBindingRecord, ExecutionStagePredecessorRecord, ExecutionWorkflowRecord,
            MAX_EXECUTION_STAGE_BYTES,
        },
        ic_mutation::{IcMutationAcknowledgement, IcMutationRequest},
        ic_request::IcManagementMethodRecord as Method,
        ic_snapshot_download::IcSnapshotDownloadPlan,
        ic_snapshot_metadata::{IcSnapshotMetadataReply, IcSnapshotMetadataRequest},
        ic_snapshot_reply::{IcSnapshotInfo, IcSnapshotReply},
        operation_plan::OperationPlanRecord,
    },
    ops::persistence::{
        AttemptJournalGuard, BackupLayoutGuard, DownloadJournalGuard, ExecutionStageGuard,
        read_complete_execution_workflow,
    },
    ports::ic_mutation::{IcMutationProvider, IcMutationProviderError},
    workflow::{
        execution_workflow::{
            ExecutionWorkflowRunError, ExecutionWorkflowStageDriver, RetainedStageEvidence,
            run_new_execution_workflow, run_retained_execution_workflow,
        },
        ic_snapshot_capture::capture_snapshot,
        ic_snapshot_download::download_snapshot,
        ic_snapshot_metadata::read_snapshot_metadata,
    },
};
use ic_platform::management_canister_types::CanisterStatusType;
use std::{
    collections::BTreeMap,
    convert::Infallible,
    fs,
    future::Future,
    io::{self, Write},
    os::unix::fs::DirBuilderExt,
    path::Path,
};

struct Provider<'a> {
    backend: &'a mut Backend,
    lose_reply: bool,
    called: bool,
}
impl IcMutationProvider for Provider<'_> {
    fn submit_mutation(
        &mut self,
        request: &IcMutationRequest<'_>,
        journal: &ic_backup::model::attempt_journal::AttemptJournalRecord,
    ) -> impl Future<Output = Result<IcMutationAcknowledgement, IcMutationProviderError>> {
        std::future::ready((|| {
            request.validate_journal(journal).unwrap();
            assert!(!self.called);
            self.called = true;
            let payload = request.payload();
            let raw = self
                .backend
                .management(payload.method().name(), payload.arguments());
            fs::write(
                self.backend
                    .root
                    .join("runner-original-capture-reply.candid"),
                &raw,
            )
            .unwrap();
            if self.lose_reply {
                return Err(IcMutationProviderError::Indeterminate);
            }
            IcMutationAcknowledgement::new(IcUpdateResponseInput {
                authority: request.authority().digest(),
                mutation_attempt: request.mutation_attempt(),
                context: request.plan().context().clone(),
                target: self.backend.target.to_text(),
                evidence: ArtifactChecksumRecord::from_bytes(&raw),
                reply: raw,
            })
            .map_err(|_| IcMutationProviderError::Indeterminate)
        })())
    }
}

struct Driver {
    backend: Backend,
    lose_reply: bool,
    binds: usize,
    executions: usize,
}
impl ExecutionWorkflowStageDriver for Driver {
    type Error = io::Error;

    fn bind_stage(
        &mut self,
        _workflow: &ExecutionWorkflowRecord,
        sequence: u64,
        predecessors: &[ExecutionStagePredecessorRecord],
    ) -> Result<OperationPlanRecord, Self::Error> {
        assert_eq!(sequence, 0);
        assert_eq!(predecessors, []);
        self.binds += 1;
        let payload = management(&self.backend, Method::TakeCanisterSnapshot, None);
        Ok(self.backend.plan(&payload.digest(), 0))
    }

    fn execute_stage(
        &mut self,
        stage: &ExecutionStageGuard<'_>,
    ) -> impl Future<Output = Result<ArtifactChecksumRecord, Self::Error>> {
        let result = (|| {
            self.executions += 1;
            let payload = management(&self.backend, Method::TakeCanisterSnapshot, None);
            let expected = stage.plan().context().clone();
            let mut provider = Provider {
                backend: &mut self.backend,
                lose_reply: self.lose_reply,
                called: false,
            };
            let acknowledgement = ready(capture_snapshot(
                stage,
                0,
                &payload,
                &mut provider,
                async |request| {
                    // This stopped isolated app has no external effects or timers.
                    // The fixture owns exact direct control and original ingress custody.
                    assert_eq!(request.plan().context(), &expected);
                    assert_eq!(request.payload().digest(), payload.digest());
                    Ok::<(), io::Error>(())
                },
            ))
            .map_err(|error| io::Error::other(error.to_string()))?;
            assert!(provider.called);
            let reply = IcSnapshotReply::decode(&payload, &acknowledgement.input().reply)
                .map_err(|error| io::Error::other(error.to_string()))?;
            assert_eq!(reply.snapshots().len(), 1);
            let mut journal = AttemptJournalGuard::open(
                stage.layout().unwrap(),
                &stage.plan().attempt_authority(0).unwrap(),
            )
            .unwrap();
            let attempt = journal.record().unwrap().view().pending_mutation.unwrap();
            journal
                .record_mutation(MutationReceiptRequest {
                    attempt,
                    request: payload.digest().hash().into(),
                    outcome: MutationOutcomeRecord::Applied,
                    evidence: reply.digest().hash().into(),
                })
                .unwrap();
            Ok(reply.digest())
        })();
        std::future::ready(result)
    }
}

pub(crate) fn run(lose_reply: bool) {
    let mut backend = Backend::new();
    assert!(
        lifecycle(
            &mut backend,
            Method::StopCanister,
            None,
            Fault::None,
            |_| None
        )
        .is_none()
    );
    status(&mut backend, CanisterStatusType::Stopped);
    let root = backend.root.join("one-shot-workflow-runner");
    fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let payload = management(&backend, Method::TakeCanisterSnapshot, None);
    let workflow = ExecutionWorkflowRecord::new(backend.plan(&payload.digest(), 0));
    let mut driver = Driver {
        backend,
        lose_reply,
        binds: 0,
        executions: 0,
    };
    let result = ready(run_new_execution_workflow(&layout, &workflow, &mut driver));
    assert_eq!(driver.binds, 1);
    assert_eq!(driver.executions, 1);
    if lose_reply {
        assert!(result.is_err());
        let path = root.join("execution-stage-0/attempt-0.json");
        let original = fs::read(&path).unwrap();
        let calls = driver.backend.calls;
        assert!(ready(run_new_execution_workflow(&layout, &workflow, &mut driver)).is_err());
        assert_eq!(driver.binds, 1);
        assert_eq!(driver.executions, 1);
        assert_eq!(driver.backend.calls, calls);
        assert_eq!(fs::read(&path).unwrap(), original);
        assert!(read_complete_execution_workflow(&layout, &workflow.digest()).is_err());
    } else {
        let view = result.unwrap();
        assert_eq!(view.stages().len(), 1);
        let calls = driver.backend.calls;
        assert_eq!(
            read_complete_execution_workflow(&layout, &workflow.digest()).unwrap(),
            view
        );
        assert_eq!(driver.backend.calls, calls);
    }
    eprintln!(
        "PocketIC one-shot workflow evidence: {}",
        driver.backend.root.display()
    );
}

struct CompleteDriver {
    backend: Backend,
    workflow: ExecutionWorkflowRecord,
    data_failure: ReadFailure,
    snapshot: Option<IcSnapshotInfo>,
    raw_metadata: Option<Vec<u8>>,
    stages: Vec<u64>,
    stop_before_data: bool,
}
impl CompleteDriver {
    fn capture(&mut self, stage: &ExecutionStageGuard<'_>) -> io::Result<ArtifactChecksumRecord> {
        let payload = management(&self.backend, Method::TakeCanisterSnapshot, None);
        let context = stage.plan().context().clone();
        let oracle = stage.layout().unwrap().root().join("capture.candid");
        let mut provider = CaptureProvider {
            backend: &mut self.backend,
            failure: CaptureFailure::None,
            oracle: &oracle,
            submitted: false,
        };
        let acknowledgement = ready(capture_snapshot(
            stage,
            0,
            &payload,
            &mut provider,
            async |request| {
                assert_eq!(request.plan().context(), &context);
                assert_eq!(request.payload().digest(), payload.digest());
                Ok::<(), Infallible>(())
            },
        ))
        .map_err(|error| io::Error::other(error.to_string()))?;
        assert!(provider.submitted);
        let reply = IcSnapshotReply::decode(&payload, &acknowledgement.input().reply)
            .map_err(|error| io::Error::other(error.to_string()))?;
        assert_eq!(reply.snapshots().len(), 1);
        let mut journal = AttemptJournalGuard::open(
            stage.layout().unwrap(),
            &stage.plan().attempt_authority(0).unwrap(),
        )
        .unwrap();
        let attempt = journal.record().unwrap().view().pending_mutation.unwrap();
        journal
            .record_mutation(MutationReceiptRequest {
                attempt,
                request: payload.digest().hash().into(),
                outcome: MutationOutcomeRecord::Applied,
                evidence: reply.digest().hash().into(),
            })
            .unwrap();
        self.snapshot = Some(reply.snapshots()[0].clone());
        Ok(reply.digest())
    }

    fn metadata(&mut self, stage: &ExecutionStageGuard<'_>) -> io::Result<ArtifactChecksumRecord> {
        let snapshot = self.snapshot.as_ref().unwrap();
        let request =
            IcSnapshotMetadataRequest::new(&self.backend.target.to_text(), snapshot.id()).unwrap();
        let context = stage.plan().context().clone();
        let (response, checkpoint) = ready(read_snapshot_metadata(
            stage,
            7,
            &request,
            &mut self.backend,
            async |read| {
                assert_eq!(read.plan().context(), &context);
                assert_eq!(read.payload().digest(), request.digest());
                Ok::<(), Infallible>(())
            },
            async |read, response| {
                retain_bytes(
                    stage.layout().unwrap(),
                    "metadata.candid",
                    &response.input().reply,
                );
                Ok::<_, Infallible>(MutationReceiptRequest {
                    attempt: read.mutation_attempt(),
                    request: read.payload().digest().hash().into(),
                    outcome: MutationOutcomeRecord::Applied,
                    evidence: response.input().evidence.hash().into(),
                })
            },
        ))
        .map_err(|error| io::Error::other(error.to_string()))?;
        self.raw_metadata = Some(response.input().reply.clone());
        Ok(checkpoint.learned_evidence().clone())
    }

    fn data(&mut self, stage: &ExecutionStageGuard<'_>) -> io::Result<ArtifactChecksumRecord> {
        let snapshot = self.snapshot.as_ref().unwrap();
        let request =
            IcSnapshotMetadataRequest::new(&self.backend.target.to_text(), snapshot.id()).unwrap();
        let raw = self.raw_metadata.as_ref().unwrap();
        let metadata = IcSnapshotMetadataReply::decode(&request, raw).unwrap();
        let download =
            IcSnapshotDownloadPlan::new(&self.workflow, 9, &metadata, 32 * 1024).unwrap();
        let plan = download.plan().unwrap();
        assert_eq!(stage.plan(), plan);
        let layout = stage.layout().unwrap();
        fs::DirBuilder::new()
            .mode(0o700)
            .create(layout.root().join("artifacts"))?;
        layout
            .retain_restore(
                &self.backend.root.join("unfinished-runner-restore.json"),
                plan.digest().hash(),
            )
            .unwrap();
        let token = "runner-original-snapshot";
        let mut artifacts = DownloadJournalGuard::create(
            layout,
            plan.digest().hash(),
            vec![DownloadArtifactRequest {
                canister_id: self.backend.target.to_text(),
                snapshot_id: token.into(),
                snapshot_taken_at_timestamp: snapshot.taken_at_timestamp(),
                snapshot_total_size_bytes: snapshot.total_size(),
            }],
        )
        .unwrap();
        let writer = artifacts
            .stage_ic_snapshot_artifact(token, &metadata, raw)
            .unwrap();
        let oracle = layout.root().join("discarded-data.candid");
        let mut provider = ReadProvider {
            backend: &mut self.backend,
            failure: self.data_failure,
            oracle: &oracle,
            calls: 0,
            fail_at: 1,
        };
        let checksum = ready(download_snapshot(
            stage,
            &download,
            writer,
            &mut provider,
            async |read| {
                assert_eq!(read.plan().context(), plan.context());
                Ok::<(), Infallible>(())
            },
            async |read, response| {
                let sequence = read.authority().binding().operation_sequence();
                retain_bytes(
                    layout,
                    &format!("data-{sequence}.candid"),
                    &response.input().reply,
                );
                Ok::<_, Infallible>(MutationReceiptRequest {
                    attempt: read.mutation_attempt(),
                    request: read.payload().digest().hash().into(),
                    outcome: MutationOutcomeRecord::Applied,
                    evidence: response.input().evidence.hash().into(),
                })
            },
        ))
        .map_err(|error| io::Error::other(error.to_string()))?;
        assert_eq!(provider.calls, download.requests().len());
        assert_eq!(
            artifacts
                .verify_ic_snapshot_artifact(plan, token, &metadata)
                .unwrap(),
            checksum
        );
        Ok(artifacts.publish_download_manifest(plan).unwrap())
    }
}
impl ExecutionWorkflowStageDriver for CompleteDriver {
    type Error = io::Error;

    fn bind_stage(
        &mut self,
        workflow: &ExecutionWorkflowRecord,
        sequence: u64,
        predecessors: &[ExecutionStagePredecessorRecord],
    ) -> Result<OperationPlanRecord, Self::Error> {
        assert_eq!(workflow, &self.workflow);
        assert_eq!(predecessors.len(), usize::from(sequence != 0));
        if sequence == 7 {
            let snapshot = self.snapshot.as_ref().unwrap();
            if predecessors[0].stage_sequence() != 0
                || predecessors[0].learned_evidence()
                    != &ArtifactChecksumRecord::from_bytes(snapshot.id())
            {
                return Err(io::Error::other("capture edge commitment mismatch"));
            }
        }
        if self.stop_before_data && sequence == 9 {
            return Err(io::Error::other("stop before data stage"));
        }
        Ok(match sequence {
            0 => {
                let payload = management(&self.backend, Method::TakeCanisterSnapshot, None);
                self.backend.plan(&payload.digest(), 0)
            }
            7 => {
                let snapshot = self.snapshot.as_ref().unwrap();
                let request =
                    IcSnapshotMetadataRequest::new(&self.backend.target.to_text(), snapshot.id())
                        .unwrap();
                self.backend.plan(&request.digest(), 7)
            }
            9 => {
                let snapshot = self.snapshot.as_ref().unwrap();
                let request =
                    IcSnapshotMetadataRequest::new(&self.backend.target.to_text(), snapshot.id())
                        .unwrap();
                let metadata =
                    IcSnapshotMetadataReply::decode(&request, self.raw_metadata.as_ref().unwrap())
                        .unwrap();
                IcSnapshotDownloadPlan::new(workflow, 9, &metadata, 32 * 1024)
                    .unwrap()
                    .plan()
                    .unwrap()
                    .clone()
            }
            _ => panic!("fixed original stage"),
        })
    }

    fn execute_stage(
        &mut self,
        stage: &ExecutionStageGuard<'_>,
    ) -> impl Future<Output = Result<ArtifactChecksumRecord, Self::Error>> {
        let sequence = stage.binding().stage_sequence();
        self.stages.push(sequence);
        std::future::ready(match sequence {
            0 => self.capture(stage),
            7 => self.metadata(stage),
            9 => self.data(stage),
            _ => panic!("fixed original stage"),
        })
    }

    fn learned_for_successor(
        &mut self,
        predecessor: &ExecutionStagePredecessorRecord,
        successor_sequence: u64,
    ) -> Result<ArtifactChecksumRecord, Self::Error> {
        if (predecessor.stage_sequence(), successor_sequence) == (0, 7) {
            return Ok(ArtifactChecksumRecord::from_bytes(
                self.snapshot.as_ref().unwrap().id(),
            ));
        }
        Ok(predecessor.learned_evidence().clone())
    }
}

fn retain_bytes(layout: &BackupLayoutGuard, name: &str, bytes: &[u8]) {
    let file = fs::File::create(layout.root().join(name)).unwrap();
    (&file).write_all(bytes).unwrap();
    file.sync_all().unwrap();
    fs::File::open(layout.root()).unwrap().sync_all().unwrap();
}

pub(crate) fn run_complete(data_failure: ReadFailure) {
    let mut backend = Backend::new();
    assert!(
        lifecycle(
            &mut backend,
            Method::StopCanister,
            None,
            Fault::None,
            |_| None
        )
        .is_none()
    );
    status(&mut backend, CanisterStatusType::Stopped);
    let workflow = allocated_workflow(&backend);
    let root = backend.root.join("complete-workflow-runner");
    fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let mut driver = CompleteDriver {
        backend,
        workflow: workflow.clone(),
        data_failure,
        snapshot: None,
        raw_metadata: None,
        stages: Vec::new(),
        stop_before_data: false,
    };
    let result = ready(run_new_execution_workflow(&layout, &workflow, &mut driver));
    assert_eq!(driver.stages, [0, 7, 9]);
    let calls = driver.backend.calls;
    if data_failure == ReadFailure::None {
        let view = result.unwrap();
        assert_eq!(view.stages().len(), 3);
        assert_eq!(
            read_complete_execution_workflow(&layout, &workflow.digest()).unwrap(),
            view
        );
        assert!(
            root.join("execution-stage-9/download-journal.json")
                .exists()
        );
    } else {
        assert!(result.is_err());
        let path = root.join("execution-stage-9/attempt-1.json");
        let original = fs::read(&path).unwrap();
        assert!(ready(run_new_execution_workflow(&layout, &workflow, &mut driver)).is_err());
        assert_eq!(driver.stages, [0, 7, 9]);
        assert_eq!(fs::read(&path).unwrap(), original);
        assert!(
            !root
                .join("execution-stage-9/execution-settlement.json")
                .exists()
        );
        assert!(read_complete_execution_workflow(&layout, &workflow.digest()).is_err());
    }
    assert_eq!(driver.backend.calls, calls);
    eprintln!(
        "PocketIC complete workflow runner evidence: {}",
        driver.backend.root.display()
    );
}

fn read_retained_prefix(
    root: &Path,
    backend: &Backend,
) -> (
    BTreeMap<u64, RetainedStageEvidence>,
    IcSnapshotInfo,
    Vec<u8>,
) {
    let capture_payload = management(backend, Method::TakeCanisterSnapshot, None);
    let raw_capture = fs::read(root.join("execution-stage-0/capture.candid")).unwrap();
    let capture_reply = IcSnapshotReply::decode(&capture_payload, &raw_capture).unwrap();
    assert_eq!(capture_reply.snapshots().len(), 1);
    let snapshot = capture_reply.snapshots()[0].clone();
    let metadata_request =
        IcSnapshotMetadataRequest::new(&backend.target.to_text(), snapshot.id()).unwrap();
    let raw_metadata = fs::read(root.join("execution-stage-7/metadata.candid")).unwrap();
    let metadata_reply = IcSnapshotMetadataReply::decode(&metadata_request, &raw_metadata).unwrap();
    let capture_journal: AttemptJournalRecord = read_json(
        &root.join("execution-stage-0/attempt-0.json"),
        MAX_ATTEMPT_JOURNAL_BYTES,
    )
    .unwrap();
    assert_eq!(
        capture_journal.applied_receipt(),
        Some(AppliedReceiptEvidence::Mutation {
            attempt: 1,
            evidence: capture_reply.digest().hash(),
        })
    );
    let metadata_journal: AttemptJournalRecord = read_json(
        &root.join("execution-stage-7/attempt-7.json"),
        MAX_ATTEMPT_JOURNAL_BYTES,
    )
    .unwrap();
    let metadata_wire_checksum = ArtifactChecksumRecord::from_bytes(&raw_metadata);
    assert_eq!(
        metadata_journal.applied_receipt(),
        Some(AppliedReceiptEvidence::Mutation {
            attempt: 1,
            evidence: metadata_wire_checksum.hash(),
        })
    );
    assert_ne!(metadata_wire_checksum, metadata_reply.digest());
    assert_ne!(
        capture_reply.digest(),
        ArtifactChecksumRecord::from_bytes(snapshot.id())
    );
    let binding = |sequence| {
        let record: ExecutionStageBindingRecord = read_json(
            &root.join(format!("execution-stage-{sequence}/stage-binding.json")),
            MAX_EXECUTION_STAGE_BYTES,
        )
        .unwrap();
        record.digest()
    };
    let retained = BTreeMap::from([
        (
            0,
            RetainedStageEvidence::new(binding(0), capture_reply.digest()).with_successor_evidence(
                BTreeMap::from([(7, ArtifactChecksumRecord::from_bytes(snapshot.id()))]),
            ),
        ),
        (
            7,
            RetainedStageEvidence::new(binding(7), metadata_reply.digest()),
        ),
    ]);
    (retained, snapshot, raw_metadata)
}

fn refuse_changed_retained_capture_edges(
    root: &Path,
    layout: &BackupLayoutGuard,
    workflow: &ExecutionWorkflowRecord,
    retained: &BTreeMap<u64, RetainedStageEvidence>,
    driver: &mut CompleteDriver,
    capture: &[u8],
    metadata: &[u8],
) {
    let binding: ExecutionStageBindingRecord = read_json(
        &root.join("execution-stage-0/stage-binding.json"),
        MAX_EXECUTION_STAGE_BYTES,
    )
    .unwrap();
    let original = retained.get(&0).unwrap();
    let changed = [
        RetainedStageEvidence::new(
            binding.digest(),
            ArtifactChecksumRecord::from_bytes(b"wrong"),
        ),
        original.clone().with_successor_evidence(BTreeMap::new()),
    ];
    let calls = driver.backend.calls;
    for evidence in changed {
        let mut supplied = retained.clone();
        supplied.insert(0, evidence);
        assert!(matches!(
            ready(run_retained_execution_workflow(
                layout,
                &workflow.digest(),
                &supplied,
                driver,
            )),
            Err(ExecutionWorkflowRunError::RetainedLearnedMismatch(7))
        ));
        assert_eq!(driver.stages, [0, 7]);
        assert_eq!(driver.backend.calls, calls);
        assert!(!root.join("execution-stage-9").exists());
        assert_eq!(
            fs::read(root.join("execution-stage-0/attempt-0.json")).unwrap(),
            capture
        );
        assert_eq!(
            fs::read(root.join("execution-stage-7/attempt-7.json")).unwrap(),
            metadata
        );
    }
}

pub(crate) fn run_retained() {
    let mut backend = Backend::new();
    assert!(
        lifecycle(
            &mut backend,
            Method::StopCanister,
            None,
            Fault::None,
            |_| None
        )
        .is_none()
    );
    status(&mut backend, CanisterStatusType::Stopped);
    let workflow = allocated_workflow(&backend);
    let root = backend.root.join("retained-workflow-runner");
    fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let mut driver = CompleteDriver {
        backend,
        workflow: workflow.clone(),
        data_failure: ReadFailure::None,
        snapshot: None,
        raw_metadata: None,
        stages: Vec::new(),
        stop_before_data: true,
    };
    assert!(ready(run_new_execution_workflow(&layout, &workflow, &mut driver)).is_err());
    assert_eq!(driver.stages, [0, 7]);
    assert!(!root.join("execution-stage-9").exists());
    let capture = fs::read(root.join("execution-stage-0/attempt-0.json")).unwrap();
    let metadata = fs::read(root.join("execution-stage-7/attempt-7.json")).unwrap();
    let calls = driver.backend.calls;
    let (retained, snapshot, raw_metadata) = read_retained_prefix(&root, &driver.backend);
    let capture_edge = ArtifactChecksumRecord::from_bytes(snapshot.id());
    refuse_changed_retained_capture_edges(
        &root,
        &layout,
        &workflow,
        &retained,
        &mut driver,
        &capture,
        &metadata,
    );
    // Drop the original driver state. Rebuild only from retained raw replies,
    // exact original bindings and the still-running isolated backend.
    let backend = driver.backend;
    let mut resumed = CompleteDriver {
        backend,
        workflow: workflow.clone(),
        data_failure: ReadFailure::None,
        snapshot: Some(snapshot),
        raw_metadata: Some(raw_metadata),
        stages: Vec::new(),
        stop_before_data: false,
    };
    let view = ready(run_retained_execution_workflow(
        &layout,
        &workflow.digest(),
        &retained,
        &mut resumed,
    ))
    .unwrap();
    assert_eq!(resumed.stages, [9]);
    assert!(resumed.backend.calls > calls);
    assert_eq!(
        fs::read(root.join("execution-stage-0/attempt-0.json")).unwrap(),
        capture
    );
    assert_eq!(
        fs::read(root.join("execution-stage-7/attempt-7.json")).unwrap(),
        metadata
    );
    assert_eq!(view.stages().len(), 3);
    let metadata_binding: ExecutionStageBindingRecord = read_json(
        &root.join("execution-stage-7/stage-binding.json"),
        MAX_EXECUTION_STAGE_BYTES,
    )
    .unwrap();
    assert_eq!(
        metadata_binding.predecessors()[0].learned_evidence(),
        &capture_edge
    );
    assert_eq!(
        read_complete_execution_workflow(&layout, &workflow.digest()).unwrap(),
        view
    );
    eprintln!(
        "PocketIC retained workflow evidence: {}",
        resumed.backend.root.display()
    );
}
