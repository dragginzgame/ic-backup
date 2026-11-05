//! Actual original allocation and complete data stages using the public coordinators.

use super::stage::Stage;
use super::{Backend, CapturedSource, TOKEN, extents, planned_runner, verify_destination};
use ic_backup::model::ic_update_response::IcUpdateResponseInput;
use ic_backup::workflow::ic_snapshot_upload::IcSnapshotUploadReplyError;
use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::{AttemptBudgetRecord, MutationOutcomeRecord, MutationReceiptRequest},
        effect_graph::{EffectGraphRecord, EffectNodeRecord, EffectNodeRequest},
        execution_workflow::{ExecutionStageBindingRecord, ExecutionWorkflowRecord},
        ic_mutation::IcMutationAcknowledgement,
        ic_snapshot_metadata::{IcSnapshotMetadataReply, IcSnapshotMetadataRequest},
        ic_snapshot_upload::{
            IcSnapshotUploadAttempt, IcSnapshotUploadKind, IcSnapshotUploadReply,
            IcSnapshotUploadReplyKind, IcSnapshotUploadRequest,
        },
        operation_plan::{
            OperationPlanRecord, PlanBudgetRecord, PlannedOperationRecord, PlannedOperationRequest,
        },
    },
    ops::persistence::{
        BackupLayoutGuard, DownloadJournalGuard, ExecutionStageGuard, create_execution_workflow,
        read_execution_progress,
    },
    ports::{ic_mutation::IcMutationProviderError, ic_snapshot_upload::IcSnapshotUploadProvider},
    workflow::ic_snapshot_upload::{
        IcSnapshotAllocationExecutionError, IcSnapshotDataUploadExecutionError,
        IcSnapshotUploadExecutionError, allocate_snapshot, upload_snapshot_data,
    },
};
use std::{collections::BTreeSet, convert::Infallible, fs, os::unix::fs::DirBuilderExt};

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum Failure {
    None,
    MetadataLost,
    MetadataMalformed,
    DataLost,
    DataMalformed,
    DataLostSecond,
    DataMalformedSecond,
    DataAllowance,
    LoadLost,
    LoadMalformed,
    StartLost,
    StartMalformed,
}
#[derive(Clone, Copy)]
pub(super) enum ReplyFault {
    Lost,
    Malformed,
}

#[derive(Clone, Copy)]
pub(super) struct ReplyFailure {
    pub(super) stage: Stage,
    pub(super) operation: Option<u64>,
    outcome: ReplyFault,
}

impl Failure {
    pub(super) fn reply(self) -> Option<ReplyFailure> {
        let (stage, operation, outcome) = match self {
            Self::None | Self::DataAllowance => return None,
            Self::MetadataLost => (Stage::Allocation, None, ReplyFault::Lost),
            Self::MetadataMalformed => (Stage::Allocation, None, ReplyFault::Malformed),
            Self::DataLost => (Stage::Upload, None, ReplyFault::Lost),
            Self::DataMalformed => (Stage::Upload, None, ReplyFault::Malformed),
            Self::DataLostSecond => (Stage::Upload, Some(1), ReplyFault::Lost),
            Self::DataMalformedSecond => (Stage::Upload, Some(1), ReplyFault::Malformed),
            Self::LoadLost => (Stage::Load, None, ReplyFault::Lost),
            Self::LoadMalformed => (Stage::Load, None, ReplyFault::Malformed),
            Self::StartLost => (Stage::Start, None, ReplyFault::Lost),
            Self::StartMalformed => (Stage::Start, None, ReplyFault::Malformed),
        };
        Some(ReplyFailure {
            stage,
            operation,
            outcome,
        })
    }

    pub(super) fn reply_at(self, stage: Stage, sequence: u64) -> Option<ReplyFault> {
        self.reply()
            .filter(|failure| {
                failure.stage == stage
                    && failure
                        .operation
                        .is_none_or(|original| original == sequence)
            })
            .map(|failure| failure.outcome)
    }
}

pub(super) struct Provider<'a> {
    pub(super) backend: &'a mut Backend,
    pub(super) failure: Failure,
    pub(super) submitted: BTreeSet<(String, u32)>,
}
impl IcSnapshotUploadProvider for Provider<'_> {
    async fn submit_upload(
        &mut self,
        request: &IcSnapshotUploadAttempt<'_, '_>,
        journal: &ic_backup::model::attempt_journal::AttemptJournalRecord,
    ) -> Result<IcMutationAcknowledgement, IcMutationProviderError> {
        request.validate_journal(journal).unwrap();
        assert!(self.submitted.insert((
            request.authority().digest().hash().into(),
            request.mutation_attempt()
        )));
        let payload = request.payload();
        let actual = self.backend.plan(
            &payload.binding_digest(),
            request.authority().binding().operation_sequence(),
        );
        assert_eq!(request.plan().context(), actual.context());
        assert_eq!(payload.receiver(), "aaaaa-aa");
        assert_eq!(payload.target(), self.backend.target.to_text());
        let raw = self
            .backend
            .management(payload.method(), payload.arguments())
            .await;
        let stage = match payload.kind() {
            IcSnapshotUploadKind::Metadata => Stage::Allocation,
            IcSnapshotUploadKind::Data { .. } => Stage::Upload,
        };
        let fault = self
            .failure
            .reply_at(stage, request.authority().binding().operation_sequence());
        if fault.is_some() {
            // The independent test oracle never enters outcome/destination admission.
            fs::write(
                self.backend.root.join("discarded-upload-oracle.candid"),
                &raw,
            )
            .unwrap();
        }
        let reply = match fault {
            Some(ReplyFault::Lost) => return Err(IcMutationProviderError::Indeterminate),
            Some(ReplyFault::Malformed) => vec![],
            None => raw,
        };
        IcMutationAcknowledgement::new(IcUpdateResponseInput {
            authority: request.authority().digest(),
            mutation_attempt: request.mutation_attempt(),
            context: actual.context().clone(),
            target: self.backend.target.to_text(),
            evidence: ArtifactChecksumRecord::from_bytes(&reply),
            reply,
        })
        .map_err(|_| IcMutationProviderError::Indeterminate)
    }
}
fn retain_reply(
    layout: &BackupLayoutGuard,
    sequence: u64,
    acknowledgement: &IcMutationAcknowledgement,
) {
    super::evidence::retain_bytes(
        layout,
        &format!("reply-{sequence}.candid"),
        &acknowledgement.input().reply,
    );
}
fn allocation(
    backend: &Backend,
    metadata: &IcSnapshotUploadRequest<'_>,
    data_count: usize,
) -> ExecutionWorkflowRecord {
    let count = u32::try_from(data_count).unwrap();
    let mut plan = backend.plan_request(&metadata.binding_digest(), 0, 0);
    plan.graph = EffectGraphRecord::new(vec![
        EffectNodeRecord::new(EffectNodeRequest {
            operation_sequence: 0,
            depends_on: vec![],
        })
        .unwrap(),
        EffectNodeRecord::new(EffectNodeRequest {
            operation_sequence: 7,
            depends_on: vec![0],
        })
        .unwrap(),
    ])
    .unwrap();
    plan.operations.push(
        PlannedOperationRecord::new(PlannedOperationRequest {
            operation_sequence: 7,
            target: backend.target.to_text(),
            request: ArtifactChecksumRecord::from_bytes(b"original full data allocation")
                .hash()
                .into(),
            budget: AttemptBudgetRecord::new(count, 0).unwrap(),
        })
        .unwrap(),
    );
    plan.budget = PlanBudgetRecord::new(count + 1, 0).unwrap();
    ExecutionWorkflowRecord::new(OperationPlanRecord::new(plan).unwrap())
}
fn assert_failed(
    stage: ExecutionStageGuard<'_>,
    workflow_layout: &BackupLayoutGuard,
    payload: &IcSnapshotUploadRequest<'_>,
    sequence: u64,
    provider: &mut Provider<'_>,
    result: Result<IcMutationAcknowledgement, IcSnapshotUploadExecutionError<Infallible>>,
) {
    match result {
        Err(IcSnapshotUploadExecutionError::Provider(IcMutationProviderError::Indeterminate)) => {}
        Err(IcSnapshotUploadExecutionError::AfterReply {
            source: IcSnapshotUploadReplyError::Association(_),
            acknowledgement,
        }) => assert_eq!(acknowledgement.input().reply, [] as [u8; 0]),
        _ => panic!("lost/malformed upload safe stop"),
    }
    let binding = stage.binding().clone();
    let root = stage.layout().unwrap().root().to_path_buf();
    let original = fs::read(root.join(format!("attempt-{sequence}.json"))).unwrap();
    let references = stage.layout().unwrap().restore_references().unwrap();
    assert!(
        stage
            .reconcile_checkpoint(ArtifactChecksumRecord::from_bytes(b"not an outcome"))
            .is_err()
    );
    drop(stage);
    let (stage, view) = ExecutionStageGuard::resume(
        workflow_layout,
        binding.workflow(),
        binding.stage_sequence(),
        &binding.digest(),
    )
    .unwrap();
    assert_eq!(view.attempts.mutations_used, 1);
    assert_eq!(view.applied_operations, 0);
    let calls = provider.backend.calls;
    let authority = stage.plan().attempt_authority(sequence).unwrap();
    let mut journal =
        ic_backup::ops::persistence::AttemptJournalGuard::open(stage.layout().unwrap(), &authority)
            .unwrap();
    IcSnapshotUploadAttempt::new(stage.plan(), sequence, journal.record().unwrap(), payload)
        .unwrap();
    assert!(
        journal
            .reserve_planned_mutation(&stage.plan().digest())
            .is_err()
    );
    assert_eq!(provider.backend.calls, calls);
    assert_eq!(
        fs::read(root.join(format!("attempt-{sequence}.json"))).unwrap(),
        original
    );
    assert_eq!(
        stage.layout().unwrap().restore_references().unwrap(),
        references
    );
}

#[expect(
    clippy::too_many_lines,
    reason = "retain the isolated source/allocation/data custody chronology in one qualification trace"
)]
pub(crate) async fn run(failure: Failure) {
    let CapturedSource {
        mut backend,
        plan: source_plan,
        snapshot,
        raw_metadata: raw,
        source,
    } = planned_runner::capture_source().await;
    let request = IcSnapshotMetadataRequest::new(&backend.target.to_text(), snapshot.id()).unwrap();
    let metadata = IcSnapshotMetadataReply::decode(&request, &raw).unwrap();
    let source_layout = BackupLayoutGuard::acquire(&source.root).unwrap();
    let source_journal =
        DownloadJournalGuard::open(&source_layout, source_plan.digest().hash()).unwrap();
    assert_eq!(metadata.metadata().stable_memory_size, 65_536);
    assert!(metadata.metadata().wasm_memory_size >= 131_072);
    assert_eq!(metadata.metadata().certified_data, 42_u64.to_le_bytes());
    assert_eq!(metadata.metadata().wasm_chunk_store.len(), 1);
    assert_eq!(
        source_journal
            .read_download_manifest(&source_plan, &source.manifest)
            .unwrap(),
        *source_journal.record().unwrap()
    );
    let original_source = fs::read(source_journal.path()).unwrap();
    let original_references = source_layout.restore_references().unwrap();
    let upload = source_journal
        .prepare_ic_snapshot_upload_metadata(&source_plan, TOKEN, &metadata)
        .unwrap();
    let kinds = extents(&metadata);
    let workflow = allocation(
        &backend,
        &upload,
        if failure == Failure::DataAllowance {
            1
        } else {
            kinds.len()
        },
    );
    let root = backend.root.join("original-upload-workflow");
    fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
    let layout = BackupLayoutGuard::acquire(&root).unwrap();
    create_execution_workflow(&layout, &workflow).unwrap();
    let metadata_plan = backend.plan_with_observations(&upload.binding_digest(), 0, 0);
    let binding = ExecutionStageBindingRecord::new(&workflow, 0, &metadata_plan, vec![]).unwrap();
    let stage = ExecutionStageGuard::prepare(&layout, binding, metadata_plan).unwrap();
    stage
        .layout()
        .unwrap()
        .retain_restore(
            &source.root.join("unfinished-upload.json"),
            source_plan.digest().hash(),
        )
        .unwrap();
    let mut provider = Provider {
        backend: &mut backend,
        failure,
        submitted: BTreeSet::new(),
    };
    let result = allocate_snapshot(
        &stage,
        0,
        &upload,
        &mut provider,
        async |request| {
            // This stopped fixture owns current controller, immutable complete source and
            // original never-dispatched custody; the generic library has no default.
            assert_eq!(request.payload().source_checksum(), &source.checksum);
            Ok::<_, Infallible>(())
        },
        async |request, acknowledgement| {
            // The fixture independently owns this exact successful allocation ingress.
            // Retain actual original reply bytes before qualifying its explicit receipt.
            retain_reply(stage.layout().unwrap(), 0, acknowledgement);
            let reply =
                IcSnapshotUploadReply::decode(request.payload(), &acknowledgement.input().reply)
                    .unwrap();
            Ok(MutationReceiptRequest {
                attempt: request.mutation_attempt(),
                request: request.payload().binding_digest().hash().into(),
                outcome: MutationOutcomeRecord::Applied,
                evidence: reply.digest().hash().into(),
            })
        },
    )
    .await;
    if failure.reply_at(Stage::Allocation, 0).is_some() {
        let result = match result {
            Err(IcSnapshotAllocationExecutionError::Upload(source)) => Err(source),
            Ok((acknowledgement, _)) => Ok(acknowledgement),
            Err(source) => panic!("unexpected allocation settlement: {source}"),
        };
        assert_failed(stage, &layout, &upload, 0, &mut provider, result);
        assert!(!layout.root().join("execution-stage-7").exists());
        assert_eq!(provider.submitted.len(), 1);
        assert_eq!(fs::read(source_journal.path()).unwrap(), original_source);
        assert_eq!(
            source_layout.restore_references().unwrap(),
            original_references
        );
        backend.close().await;
        return;
    }
    let (acknowledgement, predecessor) = result.unwrap();
    let reply = IcSnapshotUploadReply::decode(&upload, &acknowledgement.input().reply).unwrap();
    let IcSnapshotUploadReplyKind::Metadata {
        snapshot_id: destination,
    } = reply.kind()
    else {
        panic!("exact allocation reply")
    };
    // The fixture owns exact successful allocation ingress, not singleton-list attribution.
    let calls = provider.backend.calls;
    let planned = source_journal.prepare_ic_snapshot_data_upload_plan(
        &workflow,
        7,
        TOKEN,
        &reply,
        1024 * 1024,
    );
    if failure == Failure::DataAllowance {
        assert!(matches!(planned, Err(ic_backup::ops::persistence::IcSnapshotDataUploadPreparationError::Planning(ic_backup::model::ic_snapshot_upload::IcSnapshotDataUploadPlanningError::InsufficientAllowance { .. }))));
        assert_eq!(provider.backend.calls, calls);
        assert!(!layout.root().join("execution-stage-7").exists());
        assert_eq!(fs::read(source_journal.path()).unwrap(), original_source);
        backend.close().await;
        return;
    }
    let data = planned.unwrap();
    let binding = data
        .bind(stage.binding(), stage.plan(), vec![predecessor])
        .unwrap();
    let plan = data.plan().unwrap().clone();
    drop(stage);
    let stage = ExecutionStageGuard::prepare(&layout, binding, plan).unwrap();
    stage
        .layout()
        .unwrap()
        .retain_restore(
            &source.root.join("unfinished-upload.json"),
            source_plan.digest().hash(),
        )
        .unwrap();
    let result = upload_snapshot_data(
        &stage,
        &data,
        &source_journal,
        TOKEN,
        &mut provider,
        async |request| {
            assert_eq!(request.payload().source_checksum(), &source.checksum);
            assert!(
                matches!(request.payload().kind(), IcSnapshotUploadKind::Data { snapshot_id, .. } if snapshot_id == destination)
            );
            Ok::<_, Infallible>(())
        },
        async |request, acknowledgement| {
            retain_reply(
                stage.layout().unwrap(),
                request.authority().binding().operation_sequence(),
                acknowledgement,
            );
            let admitted =
                IcSnapshotUploadReply::decode(request.payload(), &acknowledgement.input().reply)
                    .unwrap();
            Ok(MutationReceiptRequest {
                attempt: request.mutation_attempt(),
                request: request.payload().binding_digest().hash().into(),
                outcome: MutationOutcomeRecord::Applied,
                evidence: admitted.digest().hash().into(),
            })
        },
    ).await;
    if let Some(fault) = failure.reply().filter(|fault| fault.stage == Stage::Upload) {
        let pending = fault.operation.unwrap_or(0);
        assert_data_failed(
            stage,
            &layout,
            &data,
            &source_journal,
            &mut provider,
            pending,
            result,
        )
        .await;
        assert_eq!(
            provider.submitted.len(),
            usize::try_from(pending).unwrap() + 2
        );
        assert_eq!(fs::read(source_journal.path()).unwrap(), original_source);
        assert_eq!(
            source_layout.restore_references().unwrap(),
            original_references
        );
        backend.close().await;
        return;
    }
    let checkpoint = result.unwrap();
    assert_eq!(checkpoint.stage_sequence(), 7);
    assert_eq!(
        read_execution_progress(stage.layout().unwrap(), &stage.plan().digest())
            .unwrap()
            .applied_operations,
        data.kinds().len()
    );
    let calls = provider.backend.calls;
    assert!(matches!(
        upload_snapshot_data(
            &stage,
            &data,
            &source_journal,
            TOKEN,
            &mut provider,
            async |_| -> Result<(), Infallible> { panic!("Applied cannot replay") },
            async |_, _| -> Result<MutationReceiptRequest, Infallible> {
                panic!("Applied cannot settle again")
            }
        )
        .await,
        Err(IcSnapshotDataUploadExecutionError::AlreadyAttempted)
    ));
    assert_eq!(provider.backend.calls, calls);
    verify_destination(provider.backend, &metadata, destination, &source.chunks).await;
    assert_eq!(fs::read(source_journal.path()).unwrap(), original_source);
    assert_eq!(
        source_layout.restore_references().unwrap(),
        original_references
    );
    assert_eq!(provider.submitted.len(), data.kinds().len() + 1);
    drop(stage);
    let restore_source = ic_backup::workflow::ic_snapshot_restore::IcRestoreSource::uploaded(
        &source_layout,
        &data,
        &layout,
        &checkpoint,
    )
    .unwrap();
    super::planned_restore::run(
        provider.backend,
        &source_plan,
        &source_layout,
        &source_journal,
        &metadata,
        destination,
        &source,
        failure,
        &restore_source,
    )
    .await;
    assert_eq!(
        source_journal
            .verify_ic_snapshot_artifact(&source_plan, TOKEN, &metadata)
            .unwrap(),
        source.checksum
    );
    assert_eq!(
        source_journal
            .read_download_manifest(&source_plan, &source.manifest)
            .unwrap(),
        *source_journal.record().unwrap()
    );
    assert_eq!(fs::read(source_journal.path()).unwrap(), original_source);
    // Restore adds its own reference before admission; every older reference survives.
    // The restore fixture checks exact post-admission reference equality on every exit.
    let retained_references = source_layout.restore_references().unwrap();
    assert!(
        original_references
            .entries()
            .iter()
            .all(|entry| retained_references.entries().contains(entry))
    );

    backend.close().await;
}

async fn assert_data_failed(
    stage: ExecutionStageGuard<'_>,
    workflow_layout: &BackupLayoutGuard,
    data: &ic_backup::model::ic_snapshot_upload::IcSnapshotDataUploadPlan<'_, '_, '_>,
    source: &DownloadJournalGuard<'_>,
    provider: &mut Provider<'_>,
    pending: u64,
    result: Result<
        ic_backup::model::execution_workflow::ExecutionStagePredecessorRecord,
        IcSnapshotDataUploadExecutionError<Infallible>,
    >,
) {
    match result {
        Err(IcSnapshotDataUploadExecutionError::Upload(
            IcSnapshotUploadExecutionError::Provider(IcMutationProviderError::Indeterminate),
        )) => {}
        Err(IcSnapshotDataUploadExecutionError::Upload(
            IcSnapshotUploadExecutionError::AfterReply {
                source: IcSnapshotUploadReplyError::Association(_),
                acknowledgement,
            },
        )) => assert_eq!(acknowledgement.input().reply, [] as [u8; 0]),
        _ => panic!("original data stage safe stop"),
    }
    let binding = stage.binding().clone();
    let root = stage.layout().unwrap().root().to_path_buf();
    let originals: Vec<_> = (0..data.kinds().len())
        .map(|i| fs::read(root.join(format!("attempt-{i}.json"))).unwrap())
        .collect();
    let references = stage.layout().unwrap().restore_references().unwrap();
    assert!(
        stage
            .reconcile_checkpoint(ArtifactChecksumRecord::from_bytes(b"not complete"))
            .is_err()
    );
    drop(stage);
    let (stage, view) =
        ExecutionStageGuard::resume(workflow_layout, binding.workflow(), 7, &binding.digest())
            .unwrap();
    assert_eq!(u64::from(view.attempts.mutations_used), pending + 1);
    assert_eq!(u64::try_from(view.applied_operations).unwrap(), pending);
    let calls = provider.backend.calls;
    assert!(matches!(
        upload_snapshot_data(
            &stage,
            data,
            source,
            TOKEN,
            provider,
            async |_| -> Result<(), Infallible> { panic!("pending cannot reissue") },
            async |_, _| -> Result<MutationReceiptRequest, Infallible> {
                panic!("pending cannot settle without evidence")
            }
        )
        .await,
        Err(IcSnapshotDataUploadExecutionError::AlreadyAttempted)
    ));
    assert_eq!(provider.backend.calls, calls);
    for (i, original) in originals.iter().enumerate() {
        assert_eq!(
            &fs::read(root.join(format!("attempt-{i}.json"))).unwrap(),
            original
        );
    }
    assert_eq!(
        stage.layout().unwrap().restore_references().unwrap(),
        references
    );
}
