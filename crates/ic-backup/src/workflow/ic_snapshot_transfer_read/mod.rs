//! One freshly reserved metadata/data update; no automatic receipt or retry.

use crate::{
    model::{
        ic_snapshot_transfer_read::{
            IcSnapshotTransferReadError, IcSnapshotTransferReadPayload,
            IcSnapshotTransferReadRequest, IcSnapshotTransferReadResponse,
        },
        operation_plan::OperationPlanError,
    },
    ops::persistence::{
        AttemptJournalError, AttemptJournalGuard, ExecutionProgressPersistenceError,
        ExecutionStageGuard, ExecutionWorkflowPersistenceError,
    },
    policy::ic_snapshot_transfer_read::{
        IcSnapshotTransferReadAssociationError, IcSnapshotTransferReadView, validate_response,
    },
    ports::{
        ic_observation::IcObservationProviderError,
        ic_snapshot_transfer_read::IcSnapshotTransferReadProvider,
    },
};
use thiserror::Error;

/// Durably reserve one original planned read, freshly admit it and invoke its provider once.
///
/// Hold no attempt guards on entry. The selected original journal remains locked
/// through admission, dispatch and passive response association. Missing originals,
/// pending attempts and unfulfilled prerequisites reject before provider invocation.
/// This only accepts a newly reserved attempt; reconstructed pending requests are
/// never dispatched. Every failure after reservation retains its consumed allowance.
/// Admission and provider submission are awaited under that same selected guard.
/// Dropping the future releases the guard without refunding or granting reentry.
///
/// `admit` must qualify actual fresh method-specific access, original metadata/raw-ID
/// custody, current application requirements and exclusive never-dispatched command
/// custody for this exact request. There is no default admission. Any remote preflight
/// calls require their own prior accounting. The provider retains its existing
/// authenticated single-update contract, without retries or hidden observations.
///
/// The returned bounded reply has only structural association. Integrations must
/// independently authenticate it and explicitly use the existing journal receipt
/// owner to record a qualified outcome. Even success leaves the attempt pending.
/// No artifact, complete-transfer, fence or terminal/reference-release proof follows.
/// # Errors
/// Rejects changed originals, payloads, spending, fresh admission, provider failure
/// or malformed/mismatched replies. Replies returned before a later rejection are
/// retained in the error. No failure grants a repeat call, refund or cleanup.
pub async fn read_snapshot<E: std::error::Error + 'static>(
    stage: &ExecutionStageGuard<'_>,
    operation_sequence: u64,
    payload: IcSnapshotTransferReadPayload<'_, '_>,
    provider: &mut impl IcSnapshotTransferReadProvider,
    admit: impl AsyncFnOnce(&IcSnapshotTransferReadRequest<'_, '_>) -> Result<(), E>,
) -> Result<IcSnapshotTransferReadResponse, IcSnapshotTransferReadExecutionError<E>> {
    let (response, ()) = read_snapshot_with(
        stage,
        operation_sequence,
        payload,
        provider,
        admit,
        async |_, _, _| (),
    )
    .await?;
    Ok(response)
}

// Complete a compound read while its original guard and decoded reply remain alive.
// The continuation owns qualification and append/receipt ordering; checkpointing
// happens only after this function has released the selected journal guard.
pub(super) async fn read_snapshot_with<E: std::error::Error + 'static, T>(
    stage: &ExecutionStageGuard<'_>,
    operation_sequence: u64,
    payload: IcSnapshotTransferReadPayload<'_, '_>,
    provider: &mut impl IcSnapshotTransferReadProvider,
    admit: impl AsyncFnOnce(&IcSnapshotTransferReadRequest<'_, '_>) -> Result<(), E>,
    complete: impl AsyncFnOnce(
        &mut AttemptJournalGuard<'_>,
        &IcSnapshotTransferReadRequest<'_, '_>,
        &IcSnapshotTransferReadView<'_, '_>,
    ) -> T,
) -> Result<(IcSnapshotTransferReadResponse, T), IcSnapshotTransferReadExecutionError<E>> {
    let plan = stage.plan();
    let authority = plan.attempt_authority(operation_sequence)?;
    payload.validate_binding(&authority)?;
    let mut journal = AttemptJournalGuard::open(stage.layout()?, &authority)?;
    journal.reserve_planned_mutation(&plan.digest())?;
    let request =
        IcSnapshotTransferReadRequest::new(plan, operation_sequence, journal.record()?, payload)?;
    let response = super::reserved_effect::submit_once(
        &journal,
        &request,
        async |request| {
            admit(request)
                .await
                .map_err(IcSnapshotTransferReadExecutionError::Admission)
        },
        || {
            stage
                .layout()
                .map(|_| ())
                .map_err(IcSnapshotTransferReadExecutionError::Stage)
        },
        async |request, record| {
            provider
                .read_snapshot(request, record)
                .await
                .map_err(IcSnapshotTransferReadExecutionError::Provider)
        },
        IcSnapshotTransferReadExecutionError::Journal,
    )
    .await?;
    let admitted = match super::reserved_effect::check_returned(
        stage,
        &journal,
        &request,
        &response,
        validate_response,
    ) {
        Ok((_, admitted)) => admitted,
        Err(error) => {
            return Err(match error {
                super::reserved_effect::ReturnedError::Journal(source) => {
                    IcSnapshotTransferReadExecutionError::AfterReplyJournal {
                        source,
                        response: Box::new(response),
                    }
                }
                super::reserved_effect::ReturnedError::Association(source) => {
                    IcSnapshotTransferReadExecutionError::Association {
                        source,
                        response: Box::new(response),
                    }
                }
                super::reserved_effect::ReturnedError::Stage(source) => {
                    IcSnapshotTransferReadExecutionError::AfterReplyStage {
                        source,
                        response: Box::new(response),
                    }
                }
            });
        }
    };
    let result = complete(&mut journal, &request, &admitted).await;
    Ok((response, result))
}

/// Typed step failures retain original spending; returned raw replies remain available.
#[derive(Debug, Error)]
pub enum IcSnapshotTransferReadExecutionError<E: std::error::Error + 'static> {
    /// The original operation cannot be derived.
    #[error(transparent)]
    Plan(#[from] OperationPlanError),
    /// Retained workflow, stage or ancestors changed before dispatch.
    #[error(transparent)]
    Stage(#[from] ExecutionWorkflowPersistenceError),
    /// Original journal admission or durable reservation failed.
    #[error(transparent)]
    Journal(#[from] AttemptJournalError),
    /// Complete original progress, prerequisites or reservation failed.
    #[error(transparent)]
    Progress(#[from] ExecutionProgressPersistenceError),
    /// Exact original payload/reservation admission failed.
    #[error(transparent)]
    Request(#[from] IcSnapshotTransferReadError),
    /// Integration-owned fresh admission failed after durable reservation.
    #[error("fresh snapshot read admission failed: {0}")]
    Admission(#[source] E),
    /// The single provider call failed; its reservation stays pending.
    #[error(transparent)]
    Provider(#[from] IcObservationProviderError),
    /// Passive association rejected a bounded returned reply.
    #[error("snapshot read response association failed: {source}")]
    Association {
        /// Canonical structural rejection.
        source: IcSnapshotTransferReadAssociationError,
        /// Exact bounded returned response, without authentication or outcome.
        response: Box<IcSnapshotTransferReadResponse>,
    },
    /// Journal re-admission failed after a reply; retain the reply.
    #[error("snapshot read journal changed after reply: {source}")]
    AfterReplyJournal {
        /// Original journal rejection.
        source: AttemptJournalError,
        /// Exact bounded returned response.
        response: Box<IcSnapshotTransferReadResponse>,
    },
    /// Stage/ancestor re-admission failed after a reply; retain the reply.
    #[error("snapshot read stage changed after reply: {source}")]
    AfterReplyStage {
        /// Original stage or ancestor rejection.
        source: ExecutionWorkflowPersistenceError,
        /// Exact bounded returned response.
        response: Box<IcSnapshotTransferReadResponse>,
    },
}

#[cfg(all(test, unix))]
mod tests;
