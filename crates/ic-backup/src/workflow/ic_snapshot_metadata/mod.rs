//! One original metadata stage, with explicit qualified receipt and learned checkpoint.

use crate::{
    model::{
        attempt_journal::{MutationOutcomeRecord, MutationReceiptRequest},
        execution_workflow::ExecutionStagePredecessorRecord,
        ic_snapshot_metadata::IcSnapshotMetadataRequest,
        ic_snapshot_transfer_read::{
            IcSnapshotTransferReadError, IcSnapshotTransferReadPayload,
            IcSnapshotTransferReadRequest, IcSnapshotTransferReadResponse,
        },
    },
    ops::persistence::{
        AttemptJournalError, AttemptJournalGuard, ExecutionStageCheckpointError,
        ExecutionStageGuard, ExecutionWorkflowPersistenceError,
    },
    policy::ic_snapshot_transfer_read::{
        IcSnapshotTransferReadAssociationError, IcSnapshotTransferReadReply, validate_response,
    },
    ports::ic_snapshot_transfer_read::IcSnapshotTransferReadProvider,
    workflow::ic_snapshot_transfer_read::{IcSnapshotTransferReadExecutionError, read_snapshot},
};
use thiserror::Error;

/// Read one new exact singleton metadata stage, record its qualified receipt and checkpoint.
///
/// Prepare the original stage and complete journals first; hold no attempt guards on
/// entry. Reuse the canonical single-read owner for durable spending, mandatory fresh
/// admission and exactly one provider call. A pending or previously Applied read is
/// never dispatched again, including after interruption before checkpoint publication.
///
/// `admit` owns fresh read permission, raw-ID/application and never-dispatched custody.
/// `qualify` runs under the original journal lock and must independently authenticate
/// exact response attribution AND durably retain original request/reply bytes before
/// returning the explicit Applied receipt. There is no default qualification or byte
/// store. Opaque provider evidence and valid wire shape cannot substitute for it.
/// Account any remote qualification separately before its calls.
///
/// Record the receipt through the sole spending owner, release its lock, then publish
/// the canonical all-Applied checkpoint with the exact request/raw-metadata digest as
/// learned evidence. The returned response and predecessor can feed the existing
/// metadata decoder and `IcSnapshotDownloadPlan::bind`; preparing the data stage and
/// writer remains explicit. Failure retains spending, receipts, checkpoints and bytes;
/// post-read errors retain the bounded response. Reopen/checkpoint recovery never
/// repeats a read. No data calls, artifact, default Agent bridge, complete backup,
/// terminal proof or fence/reference release follows.
/// # Errors
/// Rejects non-singleton or changed originals, spending, fresh admission, provider,
/// association, qualification, receipt and checkpoint failures without cleanup/refund.
pub fn read_snapshot_metadata<E: std::error::Error + 'static>(
    stage: &ExecutionStageGuard<'_>,
    operation_sequence: u64,
    payload: &IcSnapshotMetadataRequest,
    provider: &mut impl IcSnapshotTransferReadProvider,
    admit: impl FnOnce(&IcSnapshotTransferReadRequest<'_, '_>) -> Result<(), E>,
    qualify: impl FnOnce(
        &IcSnapshotTransferReadRequest<'_, '_>,
        &IcSnapshotTransferReadResponse,
    ) -> Result<MutationReceiptRequest, E>,
) -> Result<
    (
        IcSnapshotTransferReadResponse,
        ExecutionStagePredecessorRecord,
    ),
    IcSnapshotMetadataExecutionError<E>,
> {
    let operations = stage.plan().operations();
    if operations.len() != 1
        || operations[0].operation_sequence() != operation_sequence
        || operations[0].request() != payload.digest().hash()
    {
        return Err(IcSnapshotMetadataExecutionError::OriginalMismatch);
    }
    let response = read_snapshot(
        stage,
        operation_sequence,
        IcSnapshotTransferReadPayload::Metadata(payload),
        provider,
        admit,
    )?;
    match settle_metadata(stage, operation_sequence, payload, &response, qualify) {
        Ok(predecessor) => Ok((response, predecessor)),
        Err(source) => Err(IcSnapshotMetadataExecutionError::AfterReply {
            source,
            response: Box::new(response),
        }),
    }
}

fn settle_metadata<E: std::error::Error + 'static>(
    stage: &ExecutionStageGuard<'_>,
    sequence: u64,
    payload: &IcSnapshotMetadataRequest,
    response: &IcSnapshotTransferReadResponse,
    qualify: impl FnOnce(
        &IcSnapshotTransferReadRequest<'_, '_>,
        &IcSnapshotTransferReadResponse,
    ) -> Result<MutationReceiptRequest, E>,
) -> Result<ExecutionStagePredecessorRecord, IcSnapshotMetadataSettlementError<E>> {
    let plan = stage.plan();
    let authority = plan
        .attempt_authority(sequence)
        .map_err(IcSnapshotTransferReadError::from)?;
    let mut journal = AttemptJournalGuard::open(stage.layout()?, &authority)?;
    let request = IcSnapshotTransferReadRequest::new(
        plan,
        sequence,
        journal.record()?,
        IcSnapshotTransferReadPayload::Metadata(payload),
    )?;
    let admitted = validate_response(&request, journal.record()?, response)?;
    let receipt =
        qualify(&request, response).map_err(IcSnapshotMetadataSettlementError::Qualification)?;
    if receipt.outcome != MutationOutcomeRecord::Applied
        || receipt.attempt != request.mutation_attempt()
        || receipt.request != payload.digest().hash()
    {
        return Err(IcSnapshotMetadataSettlementError::ReceiptRequired);
    }
    let IcSnapshotTransferReadReply::Metadata(metadata) = admitted.reply() else {
        return Err(IcSnapshotMetadataSettlementError::ReceiptRequired);
    };
    stage.layout()?;
    journal.record_mutation(receipt)?;
    drop(journal);
    Ok(stage.checkpoint(metadata.digest())?)
}

/// Metadata coordination failures retain exact original spending and returned evidence.
#[derive(Debug, Error)]
pub enum IcSnapshotMetadataExecutionError<E: std::error::Error + 'static> {
    /// The original plan is not exactly this single metadata operation.
    #[error("snapshot metadata requires an exact singleton original stage")]
    OriginalMismatch,
    /// Canonical single-read failure, including any bounded returned response.
    #[error(transparent)]
    Read(#[from] IcSnapshotTransferReadExecutionError<E>),
    /// Receipt or checkpoint failed after a response; all originals remain retained.
    #[error("snapshot metadata reply settlement failed: {source}")]
    AfterReply {
        /// Exact original rejection; no retry or refund follows.
        source: IcSnapshotMetadataSettlementError<E>,
        /// Original bounded reply and passive claims.
        response: Box<IcSnapshotTransferReadResponse>,
    },
}

/// Rejection after one returned metadata reply; no second accounting owner.
#[derive(Debug, Error)]
pub enum IcSnapshotMetadataSettlementError<E: std::error::Error + 'static> {
    /// Actual authentication, attribution or durable original-byte retention failed.
    #[error("snapshot metadata qualification failed: {0}")]
    Qualification(#[source] E),
    /// Require an independently qualified exact original Applied receipt.
    #[error("snapshot metadata requires an explicit original Applied receipt")]
    ReceiptRequired,
    /// Original stage or retained ancestor admission failed.
    #[error(transparent)]
    Stage(#[from] ExecutionWorkflowPersistenceError),
    /// Original selected journal or receipt admission failed.
    #[error(transparent)]
    Journal(#[from] AttemptJournalError),
    /// Exact original request/current reservation admission failed.
    #[error(transparent)]
    Request(#[from] IcSnapshotTransferReadError),
    /// Existing bounded passive response admission failed.
    #[error(transparent)]
    Association(#[from] IcSnapshotTransferReadAssociationError),
    /// Canonical all-Applied checkpoint publication/re-admission failed.
    #[error(transparent)]
    Checkpoint(#[from] ExecutionStageCheckpointError),
}

#[cfg(all(test, unix))]
mod tests;
