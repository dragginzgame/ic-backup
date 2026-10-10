//! Complete finite original data uploads with explicit integration-qualified receipts.

use super::{IcSnapshotUploadExecutionError, upload_snapshot};
use crate::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::{MutationOutcomeRecord, MutationReceiptRequest},
        execution_workflow::ExecutionStagePredecessorRecord,
        ic_mutation::IcMutationAcknowledgement,
        ic_snapshot_upload::{
            IcSnapshotDataUploadPlan, IcSnapshotDataUploadPlanningError, IcSnapshotUploadAttempt,
            IcSnapshotUploadAttemptError, IcSnapshotUploadRequest,
        },
    },
    ops::persistence::{
        AttemptJournalError, AttemptJournalGuard, DownloadJournalGuard,
        ExecutionProgressPersistenceError, ExecutionStageCheckpointError, ExecutionStageGuard,
        ExecutionWorkflowPersistenceError, IcSnapshotUploadArtifactError, read_execution_progress,
    },
    policy::ic_snapshot_upload::{IcSnapshotUploadAssociationError, validate_acknowledgement},
    ports::ic_snapshot_upload::IcSnapshotUploadProvider,
};
use thiserror::Error;

/// Upload every exact original data operation, then checkpoint qualified reply evidence.
///
/// Prepare the source-derived data plan, allocation predecessor, retained stage and
/// complete original journals first; hold no attempt guards. Empty plans and any prior
/// consumption reject before dispatch: this is a fresh stage, never partial upload
/// resume. Original source and destination/command custody remain integration-owned.
/// Fresh guarded preparation buffers one payload and rechecks its immutable binding
/// before existing single-upload reservation, mandatory fresh `admit` and one call.
///
/// `qualify` must independently authenticate exact original data-write attribution and
/// durably retain original request/reply bytes before returning its explicit Applied
/// receipt under the selected journal lock. An empty reply never supplies a receipt.
/// Account remote qualification separately before its calls. Record through the sole
/// attempt owner before the next dependency can dispatch; failures stop immediately.
///
/// After all exact receipts, release journal locks and checkpoint the full original plan
/// and ordered canonical request/raw-reply evidence through the existing stage owner.
/// This is declared stage settlement, not authenticated whole-backend completeness,
/// same-release load/start admission, terminal proof or fence/source-reference release.
/// No hidden observation, reissue, refund, default provider or journal is added.
/// # Errors
/// Rejects changed originals, consumed attempts, source/preparation failures, spending,
/// admission/provider/reply/qualification/receipt or checkpoint failure. Bounded returned
/// replies survive post-reply rejection; spending, recorded receipts and source remain.
pub fn upload_snapshot_data<E: std::error::Error + 'static>(
    stage: &ExecutionStageGuard<'_>,
    upload: &IcSnapshotDataUploadPlan<'_, '_, '_>,
    source: &DownloadJournalGuard<'_>,
    snapshot: &str,
    provider: &mut impl IcSnapshotUploadProvider,
    mut admit: impl FnMut(&IcSnapshotUploadAttempt<'_, '_>) -> Result<(), E>,
    mut qualify: impl FnMut(
        &IcSnapshotUploadAttempt<'_, '_>,
        &IcMutationAcknowledgement,
    ) -> Result<MutationReceiptRequest, E>,
) -> Result<ExecutionStagePredecessorRecord, IcSnapshotDataUploadExecutionError<E>> {
    upload.validate_binding(stage.binding())?;
    let plan = upload
        .plan()
        .ok_or(IcSnapshotDataUploadPlanningError::NoDataWrites)?;
    if stage.plan() != plan {
        return Err(IcSnapshotDataUploadPlanningError::OriginalMismatch.into());
    }
    let progress = read_execution_progress(stage.layout()?, &plan.digest())?;
    if progress.attempts.mutations_used != 0 || progress.attempts.observations_used != 0 {
        return Err(IcSnapshotDataUploadExecutionError::AlreadyAttempted);
    }
    let mut evidence = b"ic-backup/ic-snapshot-data-upload/v1\0".to_vec();
    evidence.extend_from_slice(plan.digest().hash().as_bytes());
    for (index, kind) in upload.kinds().iter().enumerate() {
        let sequence =
            u64::try_from(index).map_err(|_| IcSnapshotDataUploadPlanningError::CountOverflow)?;
        let metadata = upload.metadata();
        let payload = source.prepare_ic_snapshot_upload_data(
            metadata.source_plan(),
            snapshot,
            metadata,
            upload.destination(),
            kind.clone(),
        )?;
        // The single-call owner rechecks the exact full source/context/request before spending.
        let acknowledgement = upload_snapshot(stage, sequence, &payload, provider, &mut admit)?;
        let digest = record_qualified(stage, sequence, &payload, &acknowledgement, &mut qualify)
            .map_err(|source| IcSnapshotDataUploadExecutionError::AfterReply {
                operation_sequence: sequence,
                source,
                acknowledgement: Box::new(acknowledgement),
            })?;
        evidence.extend_from_slice(digest.hash().as_bytes());
    }
    let progress = read_execution_progress(stage.layout()?, &plan.digest())?;
    if progress.applied_operations != upload.kinds().len() {
        return Err(IcSnapshotDataUploadExecutionError::AlreadyAttempted);
    }
    Ok(stage.checkpoint(ArtifactChecksumRecord::from_bytes(&evidence))?)
}

fn record_qualified<E: std::error::Error + 'static>(
    stage: &ExecutionStageGuard<'_>,
    sequence: u64,
    payload: &IcSnapshotUploadRequest<'_>,
    acknowledgement: &IcMutationAcknowledgement,
    qualify: &mut impl FnMut(
        &IcSnapshotUploadAttempt<'_, '_>,
        &IcMutationAcknowledgement,
    ) -> Result<MutationReceiptRequest, E>,
) -> Result<ArtifactChecksumRecord, IcSnapshotDataUploadReplyError<E>> {
    let plan = stage.plan();
    let authority = plan
        .attempt_authority(sequence)
        .map_err(IcSnapshotUploadAttemptError::from)?;
    let mut journal = AttemptJournalGuard::open(stage.layout()?, &authority)?;
    let request = IcSnapshotUploadAttempt::new(plan, sequence, journal.record()?, payload)?;
    let admitted = validate_acknowledgement(&request, journal.record()?, acknowledgement)?;
    let receipt = qualify(&request, acknowledgement)
        .map_err(IcSnapshotDataUploadReplyError::Qualification)?;
    if receipt.outcome != MutationOutcomeRecord::Applied
        || receipt.attempt != request.mutation_attempt()
        || receipt.request != payload.binding_digest().hash()
    {
        return Err(IcSnapshotDataUploadReplyError::ReceiptRequired);
    }
    stage.layout()?;
    journal.record_mutation(receipt)?;
    stage.layout()?;
    Ok(admitted.reply().digest())
}

/// Data-stage failure preserves source, original spending and returned acknowledgement bytes.
#[derive(Debug, Error)]
pub enum IcSnapshotDataUploadExecutionError<E: std::error::Error + 'static> {
    /// Fresh-only entrypoint rejects any original consumed mutation or observation.
    #[error("snapshot data upload original stage was already attempted")]
    AlreadyAttempted,
    /// Existing exact original data-plan/binding admission.
    #[error(transparent)]
    Planning(#[from] IcSnapshotDataUploadPlanningError),
    /// Existing retained workflow/stage/ancestor admission.
    #[error(transparent)]
    Stage(#[from] ExecutionWorkflowPersistenceError),
    /// Complete original progress admission.
    #[error(transparent)]
    Progress(#[from] ExecutionProgressPersistenceError),
    /// Fresh exact source preparation failed before the selected operation reserves.
    #[error(transparent)]
    Source(#[from] IcSnapshotUploadArtifactError),
    /// One exact original upload failed; its owner retains bounded returned evidence.
    #[error(transparent)]
    Upload(#[from] IcSnapshotUploadExecutionError<E>),
    /// Receipt settlement failed after a bounded returned acknowledgement.
    #[error("snapshot data upload reply rejected for operation {operation_sequence}: {source}")]
    AfterReply {
        /// Exact original failed operation sequence.
        operation_sequence: u64,
        /// Original qualification or persistence rejection.
        source: IcSnapshotDataUploadReplyError<E>,
        /// Exact bounded reply, without an inferred outcome.
        acknowledgement: Box<IcMutationAcknowledgement>,
    },
    /// Existing all-Applied stage checkpoint failed; receipts and occupied evidence remain.
    #[error(transparent)]
    Checkpoint(#[from] ExecutionStageCheckpointError),
}

/// Post-reply data-write rejection, without a second outcome/accounting owner.
#[derive(Debug, Error)]
pub enum IcSnapshotDataUploadReplyError<E: std::error::Error + 'static> {
    /// Independent attribution/authentication or durable original-byte retention failed.
    #[error("snapshot data upload qualification failed: {0}")]
    Qualification(#[source] E),
    /// Require the explicit exact original Applied receipt.
    #[error("snapshot data upload requires an explicit original Applied receipt")]
    ReceiptRequired,
    /// Original workflow/stage/ancestor admission failed.
    #[error(transparent)]
    Stage(#[from] ExecutionWorkflowPersistenceError),
    /// Existing selected journal/receipt persistence failed.
    #[error(transparent)]
    Journal(#[from] AttemptJournalError),
    /// Existing original payload/current reservation binding failed.
    #[error(transparent)]
    Request(#[from] IcSnapshotUploadAttemptError),
    /// Existing bounded acknowledgement association failed.
    #[error(transparent)]
    Association(#[from] IcSnapshotUploadAssociationError),
}
