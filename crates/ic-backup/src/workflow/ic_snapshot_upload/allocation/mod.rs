//! Original metadata allocation with independently qualified receipt and learned checkpoint.

use super::{IcSnapshotUploadExecutionError, upload_snapshot_with};
use crate::{
    model::{
        attempt_journal::MutationReceiptRequest,
        execution_workflow::ExecutionStagePredecessorRecord,
        ic_mutation::IcMutationAcknowledgement,
        ic_snapshot_upload::{
            IcSnapshotUploadAttempt, IcSnapshotUploadKind, IcSnapshotUploadRequest,
        },
    },
    ops::persistence::{
        AttemptJournalError, ExecutionStageCheckpointError, ExecutionStageGuard,
        ExecutionWorkflowPersistenceError,
    },
    ports::ic_snapshot_upload::IcSnapshotUploadProvider,
};
use thiserror::Error;

/// Allocate one new snapshot through an exact singleton original metadata stage.
///
/// Prepare the retained workflow/stage and complete original journals first; hold no
/// attempt guards. Reuse [`super::upload_snapshot`] for exact source/payload binding, durable
/// spending, mandatory fresh `admit` and one provider invocation. A pending or Applied
/// allocation never dispatches again, including after failed checkpoint publication.
///
/// The mandatory `qualify` callback runs under the original journal lock. It must
/// independently authenticate exclusive allocation attribution and durably retain the
/// original request/reply bytes before returning its exact Applied receipt. Neither a
/// decoded destination ID nor provider evidence supplies that qualification. Account
/// any remote qualification independently before its calls; no default is installed.
/// Admission, submission and qualification are async under their selected journal
/// guards. Retain exact replies before cancellable qualification work. Cancellation
/// keeps pending/Applied spending and denies allocation reentry; no runtime or `Send`
/// bound is imposed by the core.
///
/// Record only that exact qualified receipt through the sole attempt owner, release
/// the journal lock, then checkpoint the canonical allocation request/raw-reply digest.
/// Return the bounded acknowledgement and original predecessor for explicit data-stage
/// binding within the original workflow ceiling. Original source/metadata custody,
/// destination attribution and data planning remain integration-owned. No data call,
/// replacement, complete transfer, load/start, terminal or fence/reference release
/// follows. Recovery reopens retained evidence and never repeats this allocation.
/// # Errors
/// Rejects non-metadata/non-singleton originals, spending/admission/provider/reply,
/// qualification, receipt and checkpoint failures. Post-reply failures retain bounded
/// acknowledgement bytes; spending, recorded outcomes and occupied evidence remain.
pub async fn allocate_snapshot<E: std::error::Error + 'static>(
    stage: &ExecutionStageGuard<'_>,
    operation_sequence: u64,
    payload: &IcSnapshotUploadRequest<'_>,
    provider: &mut impl IcSnapshotUploadProvider,
    admit: impl AsyncFnOnce(&IcSnapshotUploadAttempt<'_, '_>) -> Result<(), E>,
    qualify: impl AsyncFnOnce(
        &IcSnapshotUploadAttempt<'_, '_>,
        &IcMutationAcknowledgement,
    ) -> Result<MutationReceiptRequest, E>,
) -> Result<
    (IcMutationAcknowledgement, ExecutionStagePredecessorRecord),
    IcSnapshotAllocationExecutionError<E>,
> {
    let operations = stage.plan().operations();
    if !matches!(payload.kind(), IcSnapshotUploadKind::Metadata)
        || operations.len() != 1
        || operations[0].operation_sequence() != operation_sequence
        || operations[0].request() != payload.binding_digest().hash()
    {
        return Err(IcSnapshotAllocationExecutionError::OriginalMismatch);
    }
    let (acknowledgement, settlement) = upload_snapshot_with(
        stage,
        operation_sequence,
        payload,
        provider,
        admit,
        async |journal, request, admitted| {
            super::super::qualified_receipt::AppliedReceiptRecorder {
                journal,
                expected_attempt: request.mutation_attempt(),
                expected_request: payload.binding_digest().hash(),
            }
            .record_applied(
                request,
                admitted.acknowledgement(),
                qualify,
                IcSnapshotAllocationSettlementError::Qualification,
                || IcSnapshotAllocationSettlementError::ReceiptRequired,
                || {
                    stage
                        .layout()
                        .map(|_| ())
                        .map_err(IcSnapshotAllocationSettlementError::Stage)
                },
            )
            .await?;
            Ok::<_, IcSnapshotAllocationSettlementError<E>>(admitted.reply().digest())
        },
    )
    .await?;
    // The upload owner releases its journal before checkpoint admission locks it again.
    match settlement.and_then(|evidence| Ok(stage.reconcile_checkpoint(evidence)?)) {
        Ok(predecessor) => Ok((acknowledgement, predecessor)),
        Err(source) => Err(IcSnapshotAllocationExecutionError::AfterReply {
            source,
            acknowledgement: Box::new(acknowledgement),
        }),
    }
}

/// Allocation coordination failures retain original spending and bounded returned bytes.
#[derive(Debug, Error)]
pub enum IcSnapshotAllocationExecutionError<E: std::error::Error + 'static> {
    /// Original plan is not exactly this singleton metadata allocation.
    #[error("snapshot allocation requires an exact singleton original metadata stage")]
    OriginalMismatch,
    /// Canonical single-upload failure, including bounded returned acknowledgements.
    #[error(transparent)]
    Upload(#[from] IcSnapshotUploadExecutionError<E>),
    /// Receipt or checkpoint failed after reply; all originals remain retained.
    #[error("snapshot allocation reply settlement failed: {source}")]
    AfterReply {
        /// Exact settlement rejection; no retry or refund follows.
        source: IcSnapshotAllocationSettlementError<E>,
        /// Original bounded reply and passive claims.
        acknowledgement: Box<IcMutationAcknowledgement>,
    },
}

/// Post-reply allocation rejection; no second accounting or outcome owner.
#[derive(Debug, Error)]
pub enum IcSnapshotAllocationSettlementError<E: std::error::Error + 'static> {
    /// Independent authentication, attribution or durable original-byte retention failed.
    #[error("snapshot allocation qualification failed: {0}")]
    Qualification(#[source] E),
    /// Require an independently qualified exact original Applied receipt.
    #[error("snapshot allocation requires an explicit original Applied receipt")]
    ReceiptRequired,
    /// Original stage or ancestor admission failed.
    #[error(transparent)]
    Stage(#[from] ExecutionWorkflowPersistenceError),
    /// Selected journal or receipt admission failed.
    #[error(transparent)]
    Journal(#[from] AttemptJournalError),
    /// Canonical all-Applied checkpoint publication/re-admission failed.
    #[error(transparent)]
    Checkpoint(#[from] ExecutionStageCheckpointError),
}

#[cfg(all(test, unix))]
mod tests;
