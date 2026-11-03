//! Public workflow coordination against actual managed `PocketIC` ingress.

use super::evidence::{MetadataEvidence, retain_bytes};
use super::stage::Stage;
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
        ic_request::IcManagementMethodRecord as Method,
        ic_snapshot_download::IcSnapshotDownloadPlan,
        ic_snapshot_metadata::IcSnapshotMetadataRequest,
        ic_snapshot_reply::{IcSnapshotInfo, IcSnapshotReply},
        operation_plan::OperationPlanRecord,
    },
    ops::persistence::{
        AttemptJournalGuard, BackupLayoutGuard, DownloadJournalGuard, ExecutionStageGuard,
        read_complete_execution_workflow,
    },
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
    collections::BTreeMap, convert::Infallible, fs, future::Future, io,
    os::unix::fs::DirBuilderExt, path::Path,
};

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
    let mut driver = CompleteDriver::new(backend, &workflow);
    driver.capture_failure = if lose_reply {
        CaptureFailure::Lost
    } else {
        CaptureFailure::None
    };
    let result = ready(run_new_execution_workflow(&layout, &workflow, &mut driver));
    assert_eq!(driver.binds, [0]);
    assert_eq!(driver.stages, [0]);
    if lose_reply {
        assert!(result.is_err());
        let path = root.join("execution-stage-0/attempt-0.json");
        let original = fs::read(&path).unwrap();
        let journal: AttemptJournalRecord = read_json(&path, MAX_ATTEMPT_JOURNAL_BYTES).unwrap();
        assert_eq!(journal.view().pending_mutation, Some(1));
        assert_eq!(journal.view().mutations_remaining, 0);
        let calls = driver.backend.calls;
        assert!(ready(run_new_execution_workflow(&layout, &workflow, &mut driver)).is_err());
        assert_eq!(driver.binds, [0]);
        assert_eq!(driver.stages, [0]);
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
    binds: Vec<u64>,
    capture_failure: CaptureFailure,
    data_failure: ReadFailure,
    snapshot: Option<IcSnapshotInfo>,
    metadata: Option<MetadataEvidence>,
    stages: Vec<u64>,
    stop_before: Option<u64>,
    source: Option<(OperationPlanRecord, super::Downloaded)>,
}
impl CompleteDriver {
    fn new(backend: Backend, workflow: &ExecutionWorkflowRecord) -> Self {
        Self {
            backend,
            workflow: workflow.clone(),
            binds: Vec::new(),
            capture_failure: CaptureFailure::None,
            data_failure: ReadFailure::None,
            snapshot: None,
            metadata: None,
            stages: Vec::new(),
            stop_before: None,
            source: None,
        }
    }

    fn from_retained(
        backend: Backend,
        workflow: &ExecutionWorkflowRecord,
        snapshot: IcSnapshotInfo,
        metadata: MetadataEvidence,
    ) -> Self {
        Self {
            snapshot: Some(snapshot),
            metadata: Some(metadata),
            ..Self::new(backend, workflow)
        }
    }
    fn capture(&mut self, stage: &ExecutionStageGuard<'_>) -> io::Result<ArtifactChecksumRecord> {
        let payload = management(&self.backend, Method::TakeCanisterSnapshot, None);
        let context = stage.plan().context().clone();
        let oracle = stage.layout().unwrap().root().join("capture.candid");
        let mut provider = CaptureProvider {
            backend: &mut self.backend,
            failure: self.capture_failure,
            oracle: &oracle,
            submitted: false,
        };
        retain_bytes(
            stage.layout().unwrap(),
            "capture-arguments.candid",
            payload.arguments(),
        );
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
        retain_bytes(
            stage.layout().unwrap(),
            "capture.candid",
            &acknowledgement.input().reply,
        );
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
        drop(journal);
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
                    "metadata-arguments.candid",
                    read.payload().arguments(),
                );
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
        self.metadata = Some(MetadataEvidence {
            request,
            raw: response.input().reply.clone(),
        });
        Ok(checkpoint.learned_evidence().clone())
    }

    fn data(&mut self, stage: &ExecutionStageGuard<'_>) -> io::Result<ArtifactChecksumRecord> {
        let snapshot = self.snapshot.as_ref().unwrap();
        let evidence = self.metadata.as_ref().unwrap();
        let raw = &evidence.raw;
        let metadata = evidence.decode().unwrap();
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
        let token = super::TOKEN;
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
        };
        let mut chunks = Vec::new();
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
                    &format!("data-{sequence}-arguments.candid"),
                    read.payload().arguments(),
                );
                chunks.push(
                    crate::pic_journey::IcSnapshotDataReply::decode(
                        &download.requests()[usize::try_from(sequence).unwrap()],
                        &response.input().reply,
                    )
                    .unwrap()
                    .chunk()
                    .to_vec(),
                );
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
        let manifest = artifacts.publish_download_manifest(plan).unwrap();
        self.source = Some((
            plan.clone(),
            super::Downloaded {
                root: layout.root().into(),
                checksum,
                manifest: manifest.clone(),
                chunks,
            },
        ));
        Ok(manifest)
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
        self.binds.push(sequence);
        let kind = Stage::from_sequence(sequence);
        assert_eq!(
            predecessors.len(),
            usize::from(kind.predecessor().is_some())
        );
        if kind == Stage::Metadata {
            let snapshot = self.snapshot.as_ref().unwrap();
            if predecessors[0].stage_sequence() != 0
                || predecessors[0].learned_evidence()
                    != &ArtifactChecksumRecord::from_bytes(snapshot.id())
            {
                return Err(io::Error::other("capture edge commitment mismatch"));
            }
        }
        if self.stop_before == Some(sequence) {
            return Err(io::Error::other("fixture interruption before absent stage"));
        }
        Ok(match kind {
            Stage::Capture => {
                let payload = management(&self.backend, Method::TakeCanisterSnapshot, None);
                self.backend.plan(&payload.digest(), 0)
            }
            Stage::Metadata => {
                let snapshot = self.snapshot.as_ref().unwrap();
                let request =
                    IcSnapshotMetadataRequest::new(&self.backend.target.to_text(), snapshot.id())
                        .unwrap();
                self.backend.plan(&request.digest(), 7)
            }
            Stage::Download => {
                let metadata = self.metadata.as_ref().unwrap().decode().unwrap();
                IcSnapshotDownloadPlan::new(workflow, 9, &metadata, 32 * 1024)
                    .unwrap()
                    .plan()
                    .unwrap()
                    .clone()
            }
            Stage::Allocation | Stage::Upload | Stage::Load | Stage::Start => {
                return Err(io::Error::other("capture fixture has no restore stages"));
            }
        })
    }

    fn execute_stage(
        &mut self,
        stage: &ExecutionStageGuard<'_>,
    ) -> impl Future<Output = Result<ArtifactChecksumRecord, Self::Error>> {
        let sequence = stage.binding().stage_sequence();
        self.stages.push(sequence);
        let result = match Stage::from_sequence(sequence) {
            Stage::Capture => self.capture(stage),
            Stage::Metadata => self.metadata(stage),
            Stage::Download => self.data(stage),
            Stage::Allocation | Stage::Upload | Stage::Load | Stage::Start => {
                Err(io::Error::other("capture fixture has no restore stages"))
            }
        };
        if let Ok(evidence) = &result {
            retain_bytes(
                stage.layout().unwrap(),
                "fixture-learned.sha256",
                evidence.hash().as_bytes(),
            );
        }
        std::future::ready(result)
    }

    fn learned_for_successor(
        &mut self,
        predecessor: &ExecutionStagePredecessorRecord,
        successor_sequence: u64,
    ) -> Result<ArtifactChecksumRecord, Self::Error> {
        if (
            Stage::from_sequence(predecessor.stage_sequence()),
            Stage::from_sequence(successor_sequence),
        ) == (Stage::Capture, Stage::Metadata)
        {
            return Ok(ArtifactChecksumRecord::from_bytes(
                self.snapshot.as_ref().unwrap().id(),
            ));
        }
        Ok(predecessor.learned_evidence().clone())
    }
}

pub(crate) fn run_complete(data_failure: ReadFailure) {
    complete_capture(data_failure);
}

pub(super) fn capture_source() -> super::CapturedSource {
    let mut driver = complete_capture(ReadFailure::None);
    let (plan, source) = driver.source.take().unwrap();
    super::CapturedSource {
        backend: driver.backend,
        plan,
        snapshot: driver.snapshot.unwrap(),
        raw_metadata: driver.metadata.unwrap().raw,
        source,
    }
}

fn complete_capture(data_failure: ReadFailure) -> CompleteDriver {
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
    let mut driver = CompleteDriver::new(backend, &workflow);
    driver.data_failure = data_failure;
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
        let path = root.join(format!(
            "execution-stage-9/attempt-{}.json",
            data_failure.position().unwrap()
        ));
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
    driver
}

struct RetainedCapture {
    stages: BTreeMap<u64, RetainedStageEvidence>,
    snapshot: IcSnapshotInfo,
    metadata: MetadataEvidence,
}

fn read_retained_prefix(root: &Path, backend: &Backend) -> RetainedCapture {
    let capture_payload = management(backend, Method::TakeCanisterSnapshot, None);
    assert_eq!(
        fs::read(root.join("execution-stage-0/capture-arguments.candid")).unwrap(),
        capture_payload.arguments()
    );
    let raw_capture = fs::read(root.join("execution-stage-0/capture.candid")).unwrap();
    let capture_reply = IcSnapshotReply::decode(&capture_payload, &raw_capture).unwrap();
    assert_eq!(capture_reply.snapshots().len(), 1);
    let snapshot = capture_reply.snapshots()[0].clone();
    let metadata_request =
        IcSnapshotMetadataRequest::new(&backend.target.to_text(), snapshot.id()).unwrap();
    assert_eq!(
        fs::read(root.join("execution-stage-7/metadata-arguments.candid")).unwrap(),
        metadata_request.arguments()
    );
    let raw_metadata = fs::read(root.join("execution-stage-7/metadata.candid")).unwrap();
    let metadata = MetadataEvidence {
        request: metadata_request,
        raw: raw_metadata,
    };
    let metadata_reply = metadata.decode().unwrap();
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
    let metadata_wire_checksum = ArtifactChecksumRecord::from_bytes(&metadata.raw);
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
    RetainedCapture {
        stages: retained,
        snapshot,
        metadata,
    }
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
    let mut driver = CompleteDriver::new(backend, &workflow);
    driver.stop_before = Some(9);
    assert!(ready(run_new_execution_workflow(&layout, &workflow, &mut driver)).is_err());
    assert_eq!(driver.stages, [0, 7]);
    assert!(!root.join("execution-stage-9").exists());
    let capture = fs::read(root.join("execution-stage-0/attempt-0.json")).unwrap();
    let metadata = fs::read(root.join("execution-stage-7/attempt-7.json")).unwrap();
    let calls = driver.backend.calls;
    let RetainedCapture {
        stages: retained,
        snapshot,
        metadata: evidence,
    } = read_retained_prefix(&root, &driver.backend);
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
    let mut resumed = CompleteDriver::from_retained(backend, &workflow, snapshot, evidence);
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
