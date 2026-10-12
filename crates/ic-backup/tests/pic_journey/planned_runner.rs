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
use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::{MutationOutcomeRecord, MutationReceiptRequest},
        download_journal::DownloadArtifactRequest,
        execution_workflow::{ExecutionStagePredecessorRecord, ExecutionWorkflowRecord},
        ic_mutation::{
            IcMutationAcknowledgement, IcMutationAcknowledgementInput, IcMutationRequest,
        },
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
            ExecutionWorkflowStageDriver, RetainedStageEvidence, run_new_execution_workflow,
            run_retained_execution_workflow,
        },
        ic_snapshot_capture::capture_snapshot,
        ic_snapshot_download::download_snapshot,
        ic_snapshot_metadata::read_snapshot_metadata,
    },
};
use ic_management_canister_types::CanisterStatusType;
use std::{
    collections::BTreeMap,
    convert::Infallible,
    fs,
    future::Future,
    io::{self, Write},
    os::unix::fs::DirBuilderExt,
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
            IcMutationAcknowledgement::new(IcMutationAcknowledgementInput {
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
    retained: BTreeMap<u64, RetainedStageEvidence>,
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
        let result = match sequence {
            0 => self.capture(stage),
            7 => self.metadata(stage),
            9 => self.data(stage),
            _ => panic!("fixed original stage"),
        };
        if let Ok(learned) = &result {
            self.retained.insert(
                sequence,
                RetainedStageEvidence::new(stage.binding().digest(), learned.clone()),
            );
        }
        std::future::ready(result)
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
        retained: BTreeMap::new(),
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
        retained: BTreeMap::new(),
    };
    assert!(ready(run_new_execution_workflow(&layout, &workflow, &mut driver)).is_err());
    assert_eq!(driver.stages, [0, 7]);
    assert!(!root.join("execution-stage-9").exists());
    let capture = fs::read(root.join("execution-stage-0/attempt-0.json")).unwrap();
    let metadata = fs::read(root.join("execution-stage-7/attempt-7.json")).unwrap();
    let calls = driver.backend.calls;
    let retained = driver.retained.clone();
    assert_eq!(retained.len(), 2);
    driver.stop_before_data = false;
    let view = ready(run_retained_execution_workflow(
        &layout,
        &workflow.digest(),
        &retained,
        &mut driver,
    ))
    .unwrap();
    assert_eq!(driver.stages, [0, 7, 9]);
    assert!(driver.backend.calls > calls);
    assert_eq!(
        fs::read(root.join("execution-stage-0/attempt-0.json")).unwrap(),
        capture
    );
    assert_eq!(
        fs::read(root.join("execution-stage-7/attempt-7.json")).unwrap(),
        metadata
    );
    assert_eq!(view.stages().len(), 3);
    assert_eq!(
        read_complete_execution_workflow(&layout, &workflow.digest()).unwrap(),
        view
    );
    eprintln!(
        "PocketIC retained workflow evidence: {}",
        driver.backend.root.display()
    );
}
