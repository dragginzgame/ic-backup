//! Complete original planned data transfer with explicit integration-qualified receipts.

use crate::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::{MutationOutcomeRecord, MutationReceiptRequest},
        ic_snapshot_download::{IcSnapshotDownloadPlan, IcSnapshotDownloadPlanningError},
        ic_snapshot_transfer_read::{
            IcSnapshotTransferReadError, IcSnapshotTransferReadPayload,
            IcSnapshotTransferReadRequest, IcSnapshotTransferReadResponse,
        },
    },
    ops::persistence::{
        AttemptJournalError, AttemptJournalGuard, ExecutionProgressPersistenceError,
        ExecutionStageGuard, ExecutionWorkflowPersistenceError, IcSnapshotArtifactError,
        IcSnapshotArtifactWriter, read_execution_progress,
    },
    policy::ic_snapshot_transfer_read::{
        IcSnapshotTransferReadAssociationError, IcSnapshotTransferReadReply, validate_response,
    },
    ports::ic_snapshot_transfer_read::IcSnapshotTransferReadProvider,
    workflow::ic_snapshot_transfer_read::{IcSnapshotTransferReadExecutionError, read_snapshot},
};
use thiserror::Error;

/// Stream every original data request into an existing private writer and publish it.
///
/// The metadata read and its independently qualified receipt/checkpoint must already
/// exist. Prepare the metadata-derived data stage and its complete original journals,
/// retain exact token/raw-ID association, then create the writer with exact raw metadata.
/// Hold no attempt guards on entry. Read-free plans and any consumed original attempt
/// reject; this is a fresh transfer, never partial-read resume or artifact recovery.
///
/// For every request, `admit` qualifies fresh access, metadata/command custody and
/// application requirements under the existing single-read coordinator. `qualify`
/// must independently authenticate exact original response attribution and return an
/// explicit original Applied receipt. Wire shape alone cannot qualify that receipt.
/// It runs under the selected original journal lock. Append exact decoded bytes before
/// the sole journal owner records that receipt; only then may the next dependent read
/// run. Failed qualification, append or persistence stops without another provider call.
/// Admission, submission and qualification are awaited under their selected journal
/// guards. Retain each reply durably before cancellable qualification work. Dropping
/// this future consumes the writer, retaining partial bytes, earlier receipts and
/// the current pending reservation; it grants no resume or repeat-call authority.
///
/// All original limits/records remain unchanged. Errors consume the writer and preserve
/// partial bytes, pending spending and every recorded receipt. Returned bounded replies
/// survive post-read rejection. Success uses the existing fresh checksum/durable
/// publisher, but supplies no immutable manifest/checkpoint, complete product backup,
/// terminal proof, default provider, refund or fence/reference release. Stable bytes,
/// complete authenticated backend transfer and application safety stay integration-owned.
/// # Errors
/// Rejects changed stage/plan/metadata/writer, prior consumption, spending, admission,
/// provider or receipt failures, malformed replies and local IO/publication failure.
pub async fn download_snapshot<E: std::error::Error + 'static>(
    stage: &ExecutionStageGuard<'_>,
    download: &IcSnapshotDownloadPlan<'_, '_>,
    mut writer: IcSnapshotArtifactWriter<'_, '_, '_>,
    provider: &mut impl IcSnapshotTransferReadProvider,
    mut admit: impl AsyncFnMut(&IcSnapshotTransferReadRequest<'_, '_>) -> Result<(), E>,
    mut qualify: impl AsyncFnMut(
        &IcSnapshotTransferReadRequest<'_, '_>,
        &IcSnapshotTransferReadResponse,
    ) -> Result<MutationReceiptRequest, E>,
) -> Result<ArtifactChecksumRecord, IcSnapshotDownloadExecutionError<E>> {
    download.validate_binding(stage.binding())?;
    let plan = download
        .plan()
        .ok_or(IcSnapshotDownloadPlanningError::NoDataReads)?;
    if stage.plan() != plan
        || download
            .requests()
            .iter()
            .any(|request| request.metadata().digest() != writer.coverage().metadata().digest())
        || writer.coverage().covered_region_bytes() != [0; 3]
        || writer.coverage().covered_chunks() != 0
    {
        return Err(IcSnapshotDownloadExecutionError::OriginalMismatch);
    }
    writer.validate_transfer_origin(stage.layout()?.root(), plan.digest().hash())?;
    let progress = read_execution_progress(stage.layout()?, &plan.digest())?;
    if progress.attempts.mutations_used != 0 || progress.attempts.observations_used != 0 {
        return Err(IcSnapshotDownloadExecutionError::AlreadyAttempted);
    }
    for (index, payload) in download.requests().iter().enumerate() {
        // Planning already bounded the ordinal by the original stage allowance.
        let sequence =
            u64::try_from(index).map_err(|_| IcSnapshotDownloadPlanningError::CountOverflow)?;
        let response = read_snapshot(
            stage,
            sequence,
            IcSnapshotTransferReadPayload::Data(payload),
            provider,
            &mut admit,
        )
        .await?;
        writer = append_qualified(stage, sequence, payload, writer, &response, &mut qualify)
            .await
            .map_err(|source| IcSnapshotDownloadExecutionError::AfterReply {
                operation_sequence: sequence,
                source,
                response: Box::new(response),
            })?;
    }
    let progress = read_execution_progress(stage.layout()?, &plan.digest())?;
    if progress.applied_operations != download.requests().len() {
        return Err(IcSnapshotDownloadExecutionError::AlreadyAttempted);
    }
    writer.validate_transfer_origin(stage.layout()?.root(), plan.digest().hash())?;
    let checksum = writer.finish()?;
    if let Err(source) = stage.layout() {
        return Err(IcSnapshotDownloadExecutionError::AfterPublication { source, checksum });
    }
    Ok(checksum)
}

async fn append_qualified<'journal, 'layout, 'metadata, E: std::error::Error + 'static>(
    stage: &ExecutionStageGuard<'_>,
    sequence: u64,
    payload: &crate::model::ic_snapshot_data::IcSnapshotDataRequest<'_>,
    writer: IcSnapshotArtifactWriter<'journal, 'layout, 'metadata>,
    response: &IcSnapshotTransferReadResponse,
    qualify: &mut impl AsyncFnMut(
        &IcSnapshotTransferReadRequest<'_, '_>,
        &IcSnapshotTransferReadResponse,
    ) -> Result<MutationReceiptRequest, E>,
) -> Result<IcSnapshotArtifactWriter<'journal, 'layout, 'metadata>, IcSnapshotDownloadReplyError<E>>
{
    let plan = stage.plan();
    let authority = plan
        .attempt_authority(sequence)
        .map_err(IcSnapshotTransferReadError::from)?;
    let mut journal = AttemptJournalGuard::open(stage.layout()?, &authority)?;
    let request = IcSnapshotTransferReadRequest::new(
        plan,
        sequence,
        journal.record()?,
        IcSnapshotTransferReadPayload::Data(payload),
    )?;
    let admitted = validate_response(&request, journal.record()?, response)?;
    let receipt = qualify(&request, response)
        .await
        .map_err(IcSnapshotDownloadReplyError::Qualification)?;
    if receipt.outcome != MutationOutcomeRecord::Applied
        || receipt.attempt != request.mutation_attempt()
        || receipt.request != request.payload().digest().hash()
    {
        return Err(IcSnapshotDownloadReplyError::ReceiptRequired);
    }
    writer.validate_transfer_origin(stage.layout()?.root(), plan.digest().hash())?;
    let IcSnapshotTransferReadReply::Data(reply) = admitted.reply() else {
        return Err(IcSnapshotDownloadReplyError::ReceiptRequired);
    };
    let writer = writer.append(reply)?;
    journal.record_mutation(receipt)?;
    stage.layout()?;
    Ok(writer)
}

/// Failures preserve original spending and partial/published artifact evidence.
#[derive(Debug, Error)]
pub enum IcSnapshotDownloadExecutionError<E: std::error::Error + 'static> {
    /// Exact retained writer, stage or metadata differs.
    #[error("snapshot download originals differ")]
    OriginalMismatch,
    /// A prior attempt cannot be replayed by this fresh transfer entrypoint.
    #[error("snapshot download original data stage was already attempted")]
    AlreadyAttempted,
    /// Existing original download plan/binding admission.
    #[error(transparent)]
    Planning(#[from] IcSnapshotDownloadPlanningError),
    /// Existing retained stage/ancestor admission.
    #[error(transparent)]
    Stage(#[from] ExecutionWorkflowPersistenceError),
    /// Complete original progress admission.
    #[error(transparent)]
    Progress(#[from] ExecutionProgressPersistenceError),
    /// One original read failed; its owner retains any returned bounded reply.
    #[error(transparent)]
    Read(#[from] IcSnapshotTransferReadExecutionError<E>),
    /// Canonical writer/checksum/durable publication failure.
    #[error(transparent)]
    Artifact(#[from] IcSnapshotArtifactError),
    /// Rejection after a returned read retains the exact bounded response.
    #[error("snapshot download reply rejected for operation {operation_sequence}: {source}")]
    AfterReply {
        /// Exact original data operation.
        operation_sequence: u64,
        /// Original rejection without undoing receipt or spending.
        source: IcSnapshotDownloadReplyError<E>,
        /// Exact bounded returned bytes and passive claims.
        response: Box<IcSnapshotTransferReadResponse>,
    },
    /// Publication succeeded but stage re-admission failed; retain the published checksum.
    #[error("snapshot download stage changed after publication: {source}")]
    AfterPublication {
        /// Retained stage/ancestor admission failure.
        source: ExecutionWorkflowPersistenceError,
        /// Existing durable publisher's returned checksum.
        checksum: ArtifactChecksumRecord,
    },
}

/// Rejection of a returned data reply while retaining its original attempt.
#[derive(Debug, Error)]
pub enum IcSnapshotDownloadReplyError<E: std::error::Error + 'static> {
    /// Fresh authenticated exact attribution was not qualified by the integration.
    #[error("snapshot download reply qualification failed: {0}")]
    Qualification(#[source] E),
    /// Require the explicit exact original Applied receipt, without outcome inference.
    #[error("snapshot download requires an explicit original Applied receipt")]
    ReceiptRequired,
    /// Existing original stage admission.
    #[error(transparent)]
    Stage(#[from] ExecutionWorkflowPersistenceError),
    /// Original selected journal/spending/receipt admission.
    #[error(transparent)]
    Journal(#[from] AttemptJournalError),
    /// Original request/current reservation admission.
    #[error(transparent)]
    Request(#[from] IcSnapshotTransferReadError),
    /// Existing bounded passive decoder/association.
    #[error(transparent)]
    Association(#[from] IcSnapshotTransferReadAssociationError),
    /// Exact local bytes/custody admission failed.
    #[error(transparent)]
    Artifact(#[from] IcSnapshotArtifactError),
}

#[cfg(test)]
mod tests;
