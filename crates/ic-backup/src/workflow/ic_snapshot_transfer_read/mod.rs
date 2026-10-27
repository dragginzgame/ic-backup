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
    admit(&request)
        .await
        .map_err(IcSnapshotTransferReadExecutionError::Admission)?;
    stage.layout()?;
    let response = provider.read_snapshot(&request, journal.record()?).await?;
    let admit_reply = || {
        let admitted = validate_response(&request, journal.record()?, &response)?;
        stage.layout()?;
        Ok::<_, IcSnapshotTransferReadReplyError>(admitted)
    };
    let admitted = match admit_reply() {
        Ok(admitted) => admitted,
        Err(source) => {
            return Err(IcSnapshotTransferReadExecutionError::AfterReply {
                source,
                response: Box::new(response),
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
    /// Admission after a reply failed, retaining its complete bounded evidence.
    #[error("snapshot transfer read reply rejected: {source}")]
    AfterReply {
        /// Typed original admission failure.
        source: IcSnapshotTransferReadReplyError,
        /// Complete returned passive evidence, without settlement authority.
        response: Box<IcSnapshotTransferReadResponse>,
    },
}

/// Admission failure after a returned snapshot transfer read reply.
#[derive(Debug, Error)]
pub enum IcSnapshotTransferReadReplyError {
    /// Original selected journal cannot be re-admitted.
    #[error(transparent)]
    Journal(#[from] AttemptJournalError),
    /// Passive exact reply association failed.
    #[error(transparent)]
    Association(#[from] IcSnapshotTransferReadAssociationError),
    /// Original stage or ancestor evidence cannot be re-admitted.
    #[error(transparent)]
    Stage(#[from] ExecutionWorkflowPersistenceError),
}

#[cfg(all(test, unix))]
mod tests;
