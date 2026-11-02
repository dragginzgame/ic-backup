//! Connected original-plan qualification; application authority belongs to this fixture.

mod terminal;

use super::super::stage::{self, Stage};
use super::super::{
    Backend, Downloaded, Fault, lifecycle, management,
    planned_restore::Provider as RestoreProvider,
    planned_upload::{Failure, Provider as UploadProvider},
    status, verify_destination, verify_loaded,
};
use super::{
    CompleteDriver, MetadataEvidence, RetainedCapture, read_retained_prefix, retain_bytes,
};
use crate::{ready::ready, support::read_json};
use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::{MutationOutcomeRecord, MutationReceiptRequest},
        execution_workflow::{
            ExecutionStageBindingRecord, ExecutionStagePredecessorRecord, ExecutionWorkflowRecord,
            MAX_EXECUTION_STAGE_BYTES,
        },
        ic_request::IcManagementMethodRecord as Method,
        ic_snapshot_data::IcSnapshotDataReply,
        ic_snapshot_download::IcSnapshotDownloadPlan,
        ic_snapshot_metadata::IcSnapshotMetadataReply,
        ic_snapshot_upload::{
            IcSnapshotUploadReply, IcSnapshotUploadReplyKind, IcSnapshotUploadRequest,
        },
        operation_plan::OperationPlanRecord,
        restore_safety::{
            RestoreSafetyEvidence, RestoreSafetyLaneRecord, RestoreSafetyObservation,
            RestoreSafetyObservationInput, RestoreSafetyRequest, RestoreSafetyRequestInput,
            RestoreSafetyRequirementRecord, RestoreSafetyRequirementRequest, TargetRestoreEvidence,
        },
    },
    ops::persistence::{
        BackupLayoutGuard, DownloadJournalGuard, ExecutionStageGuard,
        create_restore_safety_requirement, read_complete_execution_workflow,
        read_execution_progress, read_execution_workflow, read_operation_plan,
    },
    workflow::{
        execution_workflow::{
            RetainedStageEvidence, run_new_execution_workflow, run_retained_execution_workflow,
        },
        ic_snapshot_restore::{IcRestoreSource, restore_snapshot},
        ic_snapshot_upload::{allocate_snapshot, upload_snapshot_data},
    },
};
use ic_platform::management_canister_types::CanisterStatusType;
use serde_json::json;
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    convert::Infallible,
    fs, io,
    os::unix::fs::DirBuilderExt,
    path::{Path, PathBuf},
};

use super::super::TOKEN;

pub(super) struct RestoreProgress {
    pub(super) source_binding: ExecutionStageBindingRecord,
    pub(super) source_plan: OperationPlanRecord,
    pub(super) source: Downloaded,
    pub(super) allocation: Option<RetainedAllocation>,
    pub(super) acceptance: Option<ArtifactChecksumRecord>,
}

// Each invocation decodes the original evidence and acquires a fresh source layout.
// Journals and upload declarations borrow this context; no admission is cached.
struct RestoreSourceContext<'a> {
    plan: &'a OperationPlanRecord,
    metadata: IcSnapshotMetadataReply<'a>,
    layout: BackupLayoutGuard,
}
impl<'a> RestoreSourceContext<'a> {
    fn open(root: &Path, plan: &'a OperationPlanRecord, evidence: &'a MetadataEvidence) -> Self {
        let metadata = evidence.decode().unwrap();
        let layout = BackupLayoutGuard::acquire(root).unwrap();
        Self {
            plan,
            metadata,
            layout,
        }
    }

    fn journal(&self) -> DownloadJournalGuard<'_> {
        DownloadJournalGuard::open(&self.layout, self.plan.digest().hash()).unwrap()
    }

    fn upload(&self, journal: &DownloadJournalGuard<'_>) -> IcSnapshotUploadRequest<'_> {
        journal
            .prepare_ic_snapshot_upload_metadata(self.plan, TOKEN, &self.metadata)
            .unwrap()
    }
}

// Keep original allocation identity and raw reply evidence visibly associated.
pub(super) struct RetainedAllocation {
    binding: ExecutionStageBindingRecord,
    plan: OperationPlanRecord,
    raw_reply: Vec<u8>,
}

#[derive(Clone, Copy)]
pub(crate) enum RoundTripScenario {
    Complete,
    Interrupted,
    MetadataLost,
    DataLostSecond,
    LoadLost,
    StartLost,
}
impl RoundTripScenario {
    fn failure(self) -> Failure {
        match self {
            Self::Complete | Self::Interrupted => Failure::None,
            Self::MetadataLost => Failure::MetadataLost,
            Self::DataLostSecond => Failure::DataLostSecond,
            Self::LoadLost => Failure::LoadLost,
            Self::StartLost => Failure::StartLost,
        }
    }
}

// Stage identity and child-operation identity are distinct; Start has no successor.
#[derive(Clone, Copy)]
struct FailureExpectation {
    stage: Stage,
    operation_sequence: u64,
    successor: Option<Stage>,
}

fn workflow(backend: &Backend) -> ExecutionWorkflowRecord {
    let source = super::allocated_workflow(backend).digest();
    let purpose = ArtifactChecksumRecord::from_bytes(
        format!("restore exact original capture {}", source.hash()).as_bytes(),
    );
    stage::catalog(backend, &Stage::RESTORE, &purpose)
}

// Deliberate fixture mutation after the original capture, independently accounted
// through Backend's retained per-call plans. This is not a product workflow step.
pub(in crate::pic_journey) fn change_live_state(backend: &mut Backend) {
    assert!(lifecycle(backend, Method::StartCanister, None, Fault::None, |_| None).is_none());
    backend.write_state(99);
    backend.assert_state(99);
    backend.clear_fixture_chunk_store();
    assert!(lifecycle(backend, Method::StopCanister, None, Fault::None, |_| None).is_none());
}

impl CompleteDriver {
    pub(super) fn bind_restore(
        &self,
        workflow: &ExecutionWorkflowRecord,
        sequence: u64,
        predecessors: &[ExecutionStagePredecessorRecord],
    ) -> OperationPlanRecord {
        let retained = self.restore.as_ref().unwrap();
        if matches!(Stage::from_sequence(sequence), Stage::Load | Stage::Start) {
            return self.backend.plan_with_observations(
                &self.restore_payload(sequence, self.destination()).digest(),
                sequence,
                0,
            );
        }
        let source = RestoreSourceContext::open(
            &retained.source.root,
            &retained.source_plan,
            self.metadata.as_ref().unwrap(),
        );
        let journal = source.journal();
        let upload = source.upload(&journal);
        if Stage::from_sequence(sequence) == Stage::Allocation {
            assert_eq!(predecessors, []);
            assert_eq!(
                retained.source_binding.plan(),
                &retained.source_plan.digest()
            );
            assert_eq!(
                workflow
                    .stage(Stage::Allocation.sequence())
                    .unwrap()
                    .request(),
                ArtifactChecksumRecord::from_bytes(
                    format!(
                        "restore exact original capture {}",
                        retained.source_binding.workflow().hash()
                    )
                    .as_bytes()
                )
                .hash()
            );
            self.backend
                .plan_with_observations(&upload.binding_digest(), sequence, 0)
        } else {
            let allocation = retained.allocation.as_ref().unwrap();
            let reply = IcSnapshotUploadReply::decode(&upload, &allocation.raw_reply).unwrap();
            let data = journal
                .prepare_ic_snapshot_data_upload_plan(
                    workflow,
                    Stage::Upload.sequence(),
                    TOKEN,
                    &reply,
                    32 * 1024,
                )
                .unwrap();
            data.bind(&allocation.binding, &allocation.plan, predecessors.to_vec())
                .unwrap();
            data.plan().unwrap().clone()
        }
    }

    pub(super) fn upload(
        &mut self,
        stage: &ExecutionStageGuard<'_>,
    ) -> io::Result<ArtifactChecksumRecord> {
        let retained = self.restore.as_mut().unwrap();
        let source = RestoreSourceContext::open(
            &retained.source.root,
            &retained.source_plan,
            self.metadata.as_ref().unwrap(),
        );
        let journal = source.journal();
        let layout = &source.layout;
        assert_eq!(
            journal
                .read_download_manifest(&retained.source_plan, &retained.source.manifest)
                .unwrap(),
            *journal.record().unwrap()
        );
        let original = fs::read(journal.path()).unwrap();
        let references = fs::read(layout.root().join("restore-references.json")).unwrap();
        let upload = source.upload(&journal);
        let mut provider = UploadProvider {
            backend: &mut self.backend,
            failure: self.restore_failure,
            submitted: BTreeSet::new(),
        };
        // Exclusive fixture ownership supplies attribution and never-dispatched custody;
        // the real isolated ingress and original byte retention precede each receipt.
        let qualify = async |request: &ic_backup::model::ic_snapshot_upload::IcSnapshotUploadAttempt<'_, '_>, acknowledgement: &ic_backup::model::ic_mutation::IcMutationAcknowledgement| {
            let sequence = request.authority().binding().operation_sequence();
            retain_bytes(stage.layout().unwrap(), &format!("upload-{sequence}-arguments.candid"), request.payload().arguments());
            retain_bytes(stage.layout().unwrap(), &format!("upload-{sequence}.candid"), &acknowledgement.input().reply);
            let reply = IcSnapshotUploadReply::decode(request.payload(), &acknowledgement.input().reply).unwrap();
            Ok::<_, Infallible>(MutationReceiptRequest { attempt: request.mutation_attempt(), request: request.payload().binding_digest().hash().into(), outcome: MutationOutcomeRecord::Applied, evidence: reply.digest().hash().into() })
        };
        let result = if Stage::from_sequence(stage.binding().stage_sequence()) == Stage::Allocation
        {
            ready(allocate_snapshot(
                stage,
                Stage::Allocation.sequence(),
                &upload,
                &mut provider,
                async |request| {
                    assert_eq!(
                        request.payload().source_checksum(),
                        &retained.source.checksum
                    );
                    Ok::<_, Infallible>(())
                },
                qualify,
            ))
            .map(|(ack, checkpoint)| {
                retained.allocation = Some(RetainedAllocation {
                    binding: stage.binding().clone(),
                    plan: stage.plan().clone(),
                    raw_reply: ack.input().reply.clone(),
                });
                checkpoint.learned_evidence().clone()
            })
            .map_err(|error| io::Error::other(error.to_string()))
        } else {
            let allocation = retained.allocation.as_ref().unwrap();
            let reply = IcSnapshotUploadReply::decode(&upload, &allocation.raw_reply).unwrap();
            let data = journal
                .prepare_ic_snapshot_data_upload_plan(
                    &self.workflow,
                    Stage::Upload.sequence(),
                    TOKEN,
                    &reply,
                    32 * 1024,
                )
                .unwrap();
            assert_eq!(
                &data
                    .bind(
                        &allocation.binding,
                        &allocation.plan,
                        stage.binding().predecessors().to_vec()
                    )
                    .unwrap(),
                stage.binding()
            );
            ready(upload_snapshot_data(
                stage,
                &data,
                &journal,
                TOKEN,
                &mut provider,
                async |request| {
                    assert_eq!(
                        request.payload().source_checksum(),
                        &retained.source.checksum
                    );
                    Ok::<_, Infallible>(())
                },
                qualify,
            ))
            .map(|checkpoint| checkpoint.learned_evidence().clone())
            .map_err(|error| io::Error::other(error.to_string()))
        };
        assert_eq!(fs::read(journal.path()).unwrap(), original);
        assert_eq!(
            fs::read(layout.root().join("restore-references.json")).unwrap(),
            references
        );
        result
    }

    fn restore_payload(
        &self,
        sequence: u64,
        destination: Vec<u8>,
    ) -> ic_backup::model::ic_request::IcManagementRequestRecord {
        management(
            &self.backend,
            if Stage::from_sequence(sequence) == Stage::Load {
                Method::LoadCanisterSnapshot
            } else {
                Method::StartCanister
            },
            (Stage::from_sequence(sequence) == Stage::Load).then_some(destination),
        )
    }

    fn destination(&self) -> Vec<u8> {
        let retained = self.restore.as_ref().unwrap();
        let source = RestoreSourceContext::open(
            &retained.source.root,
            &retained.source_plan,
            self.metadata.as_ref().unwrap(),
        );
        let journal = source.journal();
        let upload = source.upload(&journal);
        let reply = IcSnapshotUploadReply::decode(
            &upload,
            &retained.allocation.as_ref().unwrap().raw_reply,
        )
        .unwrap();
        let IcSnapshotUploadReplyKind::Metadata { snapshot_id } = reply.kind() else {
            panic!("allocation reply")
        };
        snapshot_id.clone()
    }

    #[expect(
        clippy::too_many_lines,
        reason = "keep fresh source/safety admission, one real ingress, qualified receipt and full post-load acceptance together"
    )]
    pub(super) fn restore(
        &mut self,
        stage: &ExecutionStageGuard<'_>,
    ) -> io::Result<ArtifactChecksumRecord> {
        let sequence = stage.binding().stage_sequence();
        let destination = self.destination();
        let payload = self.restore_payload(sequence, destination.clone());
        let retained = self.restore.as_mut().unwrap();
        let source = RestoreSourceContext::open(
            &retained.source.root,
            &retained.source_plan,
            self.metadata.as_ref().unwrap(),
        );
        let metadata = &source.metadata;
        let source_layout = &source.layout;
        let journal = source.journal();
        assert_eq!(
            journal
                .verify_ic_snapshot_artifact(&retained.source_plan, TOKEN, metadata)
                .unwrap(),
            retained.source.checksum
        );
        // These are separately planned/reserved qualification calls, never hidden
        // calls inside a product provider or inferred upload/load receipts.
        if Stage::from_sequence(sequence) == Stage::Load {
            verify_destination(
                &mut self.backend,
                metadata,
                &destination,
                &retained.source.chunks,
            );
        }
        status(&mut self.backend, CanisterStatusType::Stopped);
        let requirement = RestoreSafetyRequirementRecord::new(
            stage.plan(),
            &retained.source_plan,
            RestoreSafetyRequirementRequest {
                source_artifacts: retained.source.manifest.clone(),
                safety: RestoreSafetyLaneRecord::NoIrreversibleEffects,
                expected_fence: None,
            },
        )
        .unwrap();
        create_restore_safety_requirement(
            stage.layout().unwrap(),
            source_layout,
            stage.plan(),
            &retained.source_plan,
            &requirement,
        )
        .unwrap();
        let safety = RestoreSafetyRequest::new(
            stage.plan(),
            &retained.source_plan,
            &requirement,
            &payload,
            RestoreSafetyRequestInput {
                operation_sequence: sequence,
                challenge: ArtifactChecksumRecord::from_bytes(
                    format!(
                        "exact fixture boundary {}:{sequence}",
                        self.workflow.digest().hash()
                    )
                    .as_bytes(),
                ),
                max_remote_observations: 0,
            },
        )
        .unwrap();
        let restore_source = if Stage::from_sequence(sequence) == Stage::Load {
            let upload = source.upload(&journal);
            let original = retained.allocation.as_ref().unwrap();
            let allocation = IcSnapshotUploadReply::decode(&upload, &original.raw_reply).unwrap();
            let data = journal
                .prepare_ic_snapshot_data_upload_plan(
                    &self.workflow,
                    Stage::Upload.sequence(),
                    TOKEN,
                    &allocation,
                    32 * 1024,
                )
                .unwrap();
            IcRestoreSource::uploaded(
                source_layout,
                &data,
                stage.workflow_layout(),
                &stage.binding().predecessors()[0],
            )
            .unwrap()
        } else {
            IcRestoreSource::existing(source_layout, &retained.source_plan, metadata.request())
        };
        let backend = RefCell::new(&mut self.backend);
        let mut provider = RestoreProvider {
            backend: &backend,
            failure: self.restore_failure,
        };
        let (acknowledgement, checkpoint) = ready(restore_snapshot(
            stage,
            restore_source,
            &safety,
            &mut provider,
            async |request, safety| {
                // Actual stopped/direct-controller evidence and no external work/timers
                // belong to this one-owner installed program, not a generic default.
                Ok::<_, io::Error>(
                    RestoreSafetyObservation::new(RestoreSafetyObservationInput {
                        request: safety.digest(),
                        context: request.plan().context().clone(),
                        inventory: request.plan().inventory().clone(),
                        source_plan_intent: retained.source_plan.digest(),
                        source_artifacts: retained.source.manifest.clone(),
                        targets: vec![TargetRestoreEvidence {
                            target: backend.borrow().target.to_text(),
                            state: CanisterStatusType::Stopped,
                            lifecycle_evidence: ArtifactChecksumRecord::from_bytes(
                                b"exclusive fixture stopped/control history",
                            ),
                            restored_acceptance: retained.acceptance.clone(),
                        }],
                        safety: RestoreSafetyEvidence::NoIrreversibleEffects(
                            ArtifactChecksumRecord::from_bytes(
                                b"closed program without irreversible effects",
                            ),
                        ),
                        evidence: ArtifactChecksumRecord::from_bytes(
                            b"exclusive original fixture safety",
                        ),
                        remote_observations: 0,
                    })
                    .unwrap(),
                )
            },
            async |request, acknowledgement| {
                retain_bytes(
                    stage.layout().unwrap(),
                    "restore-arguments.candid",
                    request.payload().arguments(),
                );
                retain_bytes(
                    stage.layout().unwrap(),
                    "restore-reply.candid",
                    &acknowledgement.input().reply,
                );
                Ok(MutationReceiptRequest {
                    attempt: request.mutation_attempt(),
                    request: request.payload().digest().hash().into(),
                    outcome: MutationOutcomeRecord::Applied,
                    evidence: acknowledgement.input().evidence.hash().into(),
                })
            },
        ))
        .map_err(|error| io::Error::other(error.to_string()))?;
        assert_eq!(acknowledgement.input().reply, b"DIDL\0\0");
        if Stage::from_sequence(sequence) == Stage::Load {
            let acceptance = verify_loaded(
                &mut backend.borrow_mut(),
                metadata,
                &destination,
                &retained.source.chunks,
            );
            retain_bytes(
                stage.layout().unwrap(),
                "fixture-accepted.sha256",
                acceptance.hash().as_bytes(),
            );
            retained.acceptance = Some(acceptance);
        } else {
            backend.borrow().assert_state(42);
        }
        Ok(checkpoint.learned_evidence().clone())
    }
}

fn files(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut result = BTreeMap::new();
    for entry in fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            result.extend(files(&entry.path()));
        } else {
            assert!(entry.file_type().unwrap().is_file());
            result.insert(entry.path(), fs::read(entry.path()).unwrap());
        }
    }
    result
}

fn retained(
    layout: &BackupLayoutGuard,
    workflow: &ExecutionWorkflowRecord,
    backend: &Backend,
) -> BTreeMap<u64, RetainedStageEvidence> {
    let mut evidence = if workflow.stage(Stage::Capture.sequence()).is_ok() {
        read_retained_prefix(layout.root(), backend).stages
    } else {
        BTreeMap::new()
    };
    for sequence in workflow
        .stages()
        .iter()
        .map(ic_backup::model::operation_plan::PlannedOperationRecord::operation_sequence)
        .filter(|sequence| {
            layout
                .root()
                .join(format!("execution-stage-{sequence}"))
                .exists()
        })
    {
        if matches!(
            Stage::from_sequence(sequence),
            Stage::Capture | Stage::Metadata
        ) {
            continue;
        }
        let binding: ExecutionStageBindingRecord = read_json(
            &layout
                .root()
                .join(format!("execution-stage-{sequence}/stage-binding.json")),
            MAX_EXECUTION_STAGE_BYTES,
        )
        .unwrap();
        let learned = fs::read_to_string(
            layout
                .root()
                .join(format!("execution-stage-{sequence}/fixture-learned.sha256")),
        )
        .map_or_else(
            |_| {
                ArtifactChecksumRecord::from_bytes(
                    b"pending original must refuse, not infer a receipt",
                )
            },
            |hash| ArtifactChecksumRecord::from_hash(&hash).unwrap(),
        );
        evidence.insert(
            sequence,
            RetainedStageEvidence::new(binding.digest(), learned),
        );
    }
    evidence
}

// Callers keep the original source layout and download journal excluded across
// both replay phases. Every request and raw reply is checked against its owner.
fn replay_chunks(download: &IcSnapshotDownloadPlan<'_, '_>, source_root: &Path) -> Vec<Vec<u8>> {
    download
        .requests()
        .iter()
        .enumerate()
        .map(|(i, request)| {
            assert_eq!(
                fs::read(source_root.join(format!("data-{i}-arguments.candid"))).unwrap(),
                request.arguments()
            );
            let raw = fs::read(source_root.join(format!("data-{i}.candid"))).unwrap();
            IcSnapshotDataReply::decode(request, &raw)
                .unwrap()
                .chunk()
                .to_vec()
        })
        .collect()
}

fn replay_allocation(
    layout: &BackupLayoutGuard,
    journal: &DownloadJournalGuard<'_>,
    source_plan: &OperationPlanRecord,
    metadata: &IcSnapshotMetadataReply<'_>,
) -> Option<RetainedAllocation> {
    let sequence = Stage::Allocation.sequence();
    let root = layout.root().join(format!("execution-stage-{sequence}"));
    if !root.exists() {
        return None;
    }
    let binding: ExecutionStageBindingRecord =
        read_json(&root.join("stage-binding.json"), MAX_EXECUTION_STAGE_BYTES).unwrap();
    let allocation_layout = BackupLayoutGuard::acquire(&root).unwrap();
    let plan = read_operation_plan(&allocation_layout, binding.plan()).unwrap();
    let upload = journal
        .prepare_ic_snapshot_upload_metadata(source_plan, TOKEN, metadata)
        .unwrap();
    assert_eq!(
        fs::read(
            allocation_layout
                .root()
                .join(format!("upload-{sequence}-arguments.candid"))
        )
        .unwrap(),
        upload.arguments()
    );
    assert_eq!(
        plan.operations()[0].request(),
        upload.binding_digest().hash()
    );
    Some(RetainedAllocation {
        binding,
        plan,
        raw_reply: fs::read(
            allocation_layout
                .root()
                .join(format!("upload-{sequence}.candid")),
        )
        .unwrap(),
    })
}

fn rebuild(
    backend: Backend,
    layout: &BackupLayoutGuard,
    capture_layout: &BackupLayoutGuard,
    workflow: &ExecutionWorkflowRecord,
    stop_before: Option<u64>,
    failure: Failure,
) -> CompleteDriver {
    let RetainedCapture {
        snapshot,
        metadata: evidence,
        ..
    } = read_retained_prefix(capture_layout.root(), &backend);
    let metadata = evidence.decode().unwrap();
    let original_capture = super::allocated_workflow(&backend);
    let original_capture =
        read_execution_workflow(capture_layout, &original_capture.digest()).unwrap();
    let download = IcSnapshotDownloadPlan::new(
        &original_capture,
        Stage::Download.sequence(),
        &metadata,
        32 * 1024,
    )
    .unwrap();
    let source_root = capture_layout
        .root()
        .join(format!("execution-stage-{}", Stage::Download.sequence()));
    let source_layout = BackupLayoutGuard::acquire(&source_root).unwrap();
    let source_plan =
        read_operation_plan(&source_layout, &download.plan().unwrap().digest()).unwrap();
    assert_eq!(
        read_execution_progress(&source_layout, &source_plan.digest())
            .unwrap()
            .applied_operations,
        download.requests().len()
    );
    let journal = DownloadJournalGuard::open(&source_layout, source_plan.digest().hash()).unwrap();
    let checksum = journal
        .verify_ic_snapshot_artifact(&source_plan, TOKEN, &metadata)
        .unwrap();
    let manifest = journal.record().unwrap().digest();
    assert_eq!(
        journal
            .read_download_manifest(&source_plan, &manifest)
            .unwrap(),
        *journal.record().unwrap()
    );
    let chunks = replay_chunks(&download, &source_root);
    let allocation = replay_allocation(layout, &journal, &source_plan, &metadata);
    let acceptance = fs::read_to_string(layout.root().join(format!(
        "execution-stage-{}/fixture-accepted.sha256",
        Stage::Load.sequence()
    )))
    .ok()
    .map(|hash| ArtifactChecksumRecord::from_hash(&hash).unwrap());
    let mut driver = CompleteDriver::from_retained(
        backend,
        workflow,
        snapshot,
        evidence,
        Some(RestoreProgress {
            source_binding: read_json(
                &source_root.join("stage-binding.json"),
                MAX_EXECUTION_STAGE_BYTES,
            )
            .unwrap(),
            source_plan,
            source: Downloaded {
                root: source_root,
                checksum,
                manifest,
                chunks,
            },
            allocation,
            acceptance,
        }),
    );
    driver.stop_before = stop_before;
    driver.restore_failure = failure;
    driver
}

#[expect(
    clippy::too_many_lines,
    reason = "keep original intents, interruption custody, refusal and completed replay in one qualification trace"
)]
pub(crate) fn run_round_trip(scenario: RoundTripScenario) {
    let failure = scenario.failure();
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
    let capture_workflow = super::allocated_workflow(&backend);
    let workflow = workflow(&backend);
    let custody_root = backend.root.join("connected-original-workflow");
    let capture_root = custody_root.join("capture");
    let root = custody_root.join("restore");
    fs::DirBuilder::new()
        .mode(0o700)
        .create(&custody_root)
        .unwrap();
    fs::DirBuilder::new()
        .mode(0o700)
        .create(&capture_root)
        .unwrap();
    fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
    let capture_layout = BackupLayoutGuard::acquire(&capture_root).unwrap();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    let original_workflow = workflow.digest();
    // Both immutable allocations and their total ceilings precede any capture.
    // Separate originals let restore retain source custody during ancestor checks.
    retain_bytes(
        &capture_layout,
        "fixture-restore-allocation.json",
        &serde_json::to_vec(&workflow).unwrap(),
    );
    let mut driver = CompleteDriver::new(backend, &capture_workflow);
    driver.restore_failure = failure;
    let captured = ready(run_new_execution_workflow(
        &capture_layout,
        &capture_workflow,
        &mut driver,
    ))
    .unwrap();
    let mut executed = driver.stages.clone();
    assert_eq!(executed, [0, 7, 9]);
    change_live_state(&mut driver.backend);
    {
        let source = driver.restore.as_ref().unwrap();
        let source_layout = BackupLayoutGuard::acquire(&source.source.root).unwrap();
        source_layout
            .retain_restore(
                &root.join("execution-workflow.json"),
                original_workflow.hash(),
            )
            .unwrap();
    }
    driver.workflow = workflow.clone();
    driver.stages.clear();
    driver.stop_before =
        matches!(scenario, RoundTripScenario::Interrupted).then_some(Stage::Allocation.sequence());
    let mut result = ready(run_new_execution_workflow(&layout, &workflow, &mut driver));
    executed.extend_from_slice(&driver.stages);
    if let RoundTripScenario::Interrupted = scenario {
        for stop in [
            Some(Stage::Load.sequence()),
            Some(Stage::Start.sequence()),
            None,
        ] {
            assert!(result.is_err());
            assert!(
                !root
                    .join(format!("execution-stage-{}", driver.stop_before.unwrap()))
                    .exists()
            );
            let original = files(&custody_root);
            let evidence = retained(&layout, &workflow, &driver.backend);
            let backend = driver.backend;
            driver = rebuild(backend, &layout, &capture_layout, &workflow, stop, failure);
            let calls = driver.backend.calls;
            result = ready(run_retained_execution_workflow(
                &layout,
                &original_workflow,
                &evidence,
                &mut driver,
            ));
            executed.extend_from_slice(&driver.stages);
            assert!(driver.backend.calls > calls);
            for (path, bytes) in original {
                assert_eq!(fs::read(path).unwrap(), bytes);
            }
        }
    }
    let expected_failure = match failure {
        Failure::MetadataLost => Some(FailureExpectation {
            stage: Stage::Allocation,
            operation_sequence: Stage::Allocation.sequence(),
            successor: Some(Stage::Upload),
        }),
        Failure::DataLostSecond => Some(FailureExpectation {
            stage: Stage::Upload,
            operation_sequence: 1,
            successor: Some(Stage::Load),
        }),
        Failure::LoadLost => Some(FailureExpectation {
            stage: Stage::Load,
            operation_sequence: Stage::Load.sequence(),
            successor: Some(Stage::Start),
        }),
        Failure::StartLost => Some(FailureExpectation {
            stage: Stage::Start,
            operation_sequence: Stage::Start.sequence(),
            successor: None,
        }),
        Failure::None => None,
        _ => panic!("selected round-trip fault"),
    };
    if let Some(expected) = expected_failure {
        let sequence = expected.stage.sequence();
        assert!(result.is_err());
        let before = files(&custody_root);
        let calls = driver.backend.calls;
        let stages = driver.stages.clone();
        let evidence = retained(&layout, &workflow, &driver.backend);
        assert!(ready(run_new_execution_workflow(&layout, &workflow, &mut driver)).is_err());
        assert!(
            ready(run_retained_execution_workflow(
                &layout,
                &original_workflow,
                &evidence,
                &mut driver
            ))
            .is_err()
        );
        assert_eq!(driver.backend.calls, calls);
        assert_eq!(driver.stages, stages);
        let mut after = files(&custody_root);
        let mutex = root.join(format!(
            "execution-stage-{sequence}/execution-settlement.json.lock"
        ));
        assert_eq!(after.remove(&mutex), Some(Vec::new()));
        assert_eq!(
            after.keys().collect::<Vec<_>>(),
            before.keys().collect::<Vec<_>>()
        );
        for (path, bytes) in before {
            assert!(
                after[&path] == bytes,
                "original evidence changed at {}",
                path.display()
            );
        }
        if let Some(successor) = expected.successor {
            assert!(
                !root
                    .join(format!("execution-stage-{}", successor.sequence()))
                    .exists()
            );
        }
        assert!(read_complete_execution_workflow(&layout, &original_workflow).is_err());
        let binding: ExecutionStageBindingRecord = read_json(
            &root.join(format!("execution-stage-{sequence}/stage-binding.json")),
            MAX_EXECUTION_STAGE_BYTES,
        )
        .unwrap();
        let (stage, progress) =
            ExecutionStageGuard::resume(&layout, &original_workflow, sequence, &binding.digest())
                .unwrap();
        let operation = progress
            .operations
            .iter()
            .find(|row| row.operation_sequence == expected.operation_sequence)
            .unwrap();
        assert_eq!(operation.attempts.pending_mutation, Some(1));
        assert_eq!(operation.attempts.mutations_remaining, 0);
        if expected.stage == Stage::Upload {
            assert_eq!(progress.applied_operations, 1);
        }
        let source = driver.restore.as_ref().unwrap();
        assert!(source.source.root.join("restore-references.json").is_file());
        assert!(source.source.root.join("download-manifest.json").is_file());
        drop(stage);
        terminal::check(&driver, &layout, &capture_layout, false);
    } else {
        let complete = result.unwrap();
        assert_eq!(
            executed,
            Stage::CAPTURE
                .into_iter()
                .chain(Stage::RESTORE)
                .map(Stage::sequence)
                .collect::<Vec<_>>()
        );
        driver.backend.assert_state(42);
        let before = files(&custody_root);
        let evidence = retained(&layout, &workflow, &driver.backend);
        let calls = driver.backend.calls;
        // Forget all learned in-memory state: completed replay must invoke no driver.
        let backend = driver.backend;
        driver = CompleteDriver::new(backend, &workflow);
        driver.stop_before = Some(Stage::Capture.sequence());
        assert_eq!(
            ready(run_retained_execution_workflow(
                &layout,
                &original_workflow,
                &evidence,
                &mut driver
            ))
            .unwrap(),
            complete
        );
        assert_eq!(driver.backend.calls, calls);
        assert_eq!(driver.stages, [] as [u64; 0]);
        assert_eq!(files(&custody_root), before);
        assert_eq!(
            read_complete_execution_workflow(&layout, &original_workflow).unwrap(),
            complete
        );
        // Terminal admission must also work after reconstructing original evidence,
        // without dispatch, cached completion flags or a second application query.
        let backend = driver.backend;
        driver = rebuild(backend, &layout, &capture_layout, &workflow, None, failure);
        terminal::check(&driver, &layout, &capture_layout, true);
    }
    if failure == Failure::None {
        let before = files(&custody_root);
        let evidence = retained(&capture_layout, &capture_workflow, &driver.backend);
        let calls = driver.backend.calls;
        assert_eq!(
            ready(run_retained_execution_workflow(
                &capture_layout,
                &capture_workflow.digest(),
                &evidence,
                &mut driver
            ))
            .unwrap(),
            captured
        );
        assert_eq!(driver.backend.calls, calls);
        assert_eq!(driver.stages, [] as [u64; 0]);
        assert_eq!(files(&custody_root), before);
    }
    assert_eq!(
        fs::read(capture_root.join("fixture-restore-allocation.json")).unwrap(),
        serde_json::to_vec(&workflow).unwrap()
    );
    retain_bytes(&capture_layout, "fixture-connected-result.json", &serde_json::to_vec_pretty(&json!({
        "scope":"isolated application with separately accounted fixture setup/verification",
        "capture":capture_workflow.digest(), "restore":original_workflow,
        "target":driver.backend.target.to_text(), "executed_stages":executed,
        "completed":failure == Failure::None, "pending_stage":expected_failure.map(|expected| expected.stage.sequence()),
        "fixture_source_reference_released":failure == Failure::None,
        "management_calls":driver.backend.calls,
    })).unwrap());
    assert_eq!(workflow.digest(), original_workflow);
    eprintln!(
        "Connected original-workflow evidence retained at {}",
        driver.backend.root.display()
    );
}
