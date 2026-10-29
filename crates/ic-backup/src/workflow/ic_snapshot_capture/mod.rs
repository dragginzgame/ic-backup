//! One freshly reserved capture update under exact original stage evidence.

use crate::{
    model::{
        ic_mutation::{IcMutationAcknowledgement, IcMutationRequest, IcMutationRequestError},
        ic_request::{IcManagementMethodRecord, IcManagementRequestRecord},
        operation_plan::OperationPlanError,
    },
    ops::persistence::{
        AttemptJournalError, ExecutionProgressPersistenceError, ExecutionStageGuard,
        ExecutionWorkflowPersistenceError,
    },
    policy::ic_mutation::{IcMutationAssociationError, validate_acknowledgement},
    ports::ic_mutation::{IcMutationProvider, IcMutationProviderError},
};
use thiserror::Error;

/// Durably reserve an original capture, freshly admit it and submit one exact update.
///
/// Hold no attempt guards on entry. Only `TakeCanisterSnapshot` is accepted, with
/// canonical original target/wire binding checked before spending. The existing
/// journal is opened, never created, and complete original-plan admission owns its
/// durable reservation. Keep the selected journal locked through fresh admission,
/// one provider invocation and bounded passive acknowledgement association.
/// Retained stage/ancestor checks bracket dispatch. Pending or Applied attempts
/// cannot invoke admission or dispatch again; reopening grants no recapture.
///
/// The mandatory fallible `admit` callback must qualify fresh actual context/control,
/// original capture consistency, quiescence/fence obligations and exclusive
/// never-dispatched command custody. Any remote preflight calls need their own prior
/// accounting. There is no default permission, provider or application consistency
/// lane. The existing provider retains its authenticated single-update contract.
/// Admission and submission are awaited while the selected journal stays locked.
/// Cancellation releases that lock while retaining its pending reservation;
/// even cancellation before dispatch grants no repeat callback or submission.
///
/// Success returns a bounded passive acknowledgement and leaves spending pending.
/// Integrations authenticate exact original attribution and explicitly record any
/// qualified receipt using the existing journal owner. An ID or matching wire reply
/// supplies no automatic outcome, complete capture/download, terminal or fence release.
/// # Errors
/// Rejects noncapture/changed payloads, original evidence, spending, fresh admission,
/// provider failures or malformed/mismatched replies. All post-reservation failures
/// retain consumption; returned acknowledgements survive later rejection in typed
/// errors. No failure permits retry, recapture, refund, cleanup or automatic uncertainty.
pub async fn capture_snapshot<E: std::error::Error + 'static>(
    stage: &ExecutionStageGuard<'_>,
    operation_sequence: u64,
    payload: &IcManagementRequestRecord,
    provider: &mut impl IcMutationProvider,
    admit: impl AsyncFnOnce(&IcMutationRequest<'_>) -> Result<(), E>,
) -> Result<IcMutationAcknowledgement, IcSnapshotCaptureExecutionError<E>> {
    if payload.method() != IcManagementMethodRecord::TakeCanisterSnapshot {
        return Err(IcSnapshotCaptureExecutionError::CaptureRequired);
    }
    let plan = stage.plan();
    let authority = plan.attempt_authority(operation_sequence)?;
    let (acknowledgement, admission) = super::ic_mutation::submit_with(
        stage,
        &authority,
        payload,
        provider,
        async |request| {
            admit(request)
                .await
                .map_err(IcSnapshotCaptureExecutionError::Admission)?;
            stage.layout()?;
            Ok::<_, IcSnapshotCaptureExecutionError<E>>(())
        },
        async |journal, request, acknowledgement| {
            validate_acknowledgement(request, journal.record()?, acknowledgement)?;
            stage.layout()?;
            Ok::<_, IcSnapshotCaptureReplyError>(())
        },
    )
    .await?;
    match admission {
        Ok(()) => Ok(acknowledgement),
        Err(source) => Err(IcSnapshotCaptureExecutionError::AfterReply {
            source,
            acknowledgement: Box::new(acknowledgement),
        }),
    }
}

/// Capture-step failures retain original spending and any bounded returned acknowledgement.
#[derive(Debug, Error)]
pub enum IcSnapshotCaptureExecutionError<E: std::error::Error + 'static> {
    /// Only the original capture method may enter this coordinator.
    #[error("snapshot capture requires take_canister_snapshot")]
    CaptureRequired,
    /// Original operation authority cannot be derived.
    #[error(transparent)]
    Plan(#[from] OperationPlanError),
    /// Workflow, stage or original ancestor evidence failed admission.
    #[error(transparent)]
    Stage(#[from] ExecutionWorkflowPersistenceError),
    /// Original selected journal admission failed.
    #[error(transparent)]
    Journal(#[from] AttemptJournalError),
    /// Complete original progress, prerequisites or durable reservation failed.
    #[error(transparent)]
    Progress(#[from] ExecutionProgressPersistenceError),
    /// Canonical original mutation binding or current reservation differs.
    #[error(transparent)]
    Request(#[from] IcMutationRequestError),
    /// Fresh integration-owned admission rejected after reservation.
    #[error("fresh snapshot capture admission failed: {0}")]
    Admission(#[source] E),
    /// The single provider call failed; spending remains pending.
    #[error(transparent)]
    Provider(#[from] IcMutationProviderError),
    /// Admission after a reply failed, retaining its complete bounded evidence.
    #[error("snapshot capture reply rejected: {source}")]
    AfterReply {
        /// Typed original admission failure.
        source: IcSnapshotCaptureReplyError,
        /// Complete returned passive evidence, without settlement authority.
        acknowledgement: Box<IcMutationAcknowledgement>,
    },
}

/// Admission failure after a returned snapshot capture reply.
#[derive(Debug, Error)]
pub enum IcSnapshotCaptureReplyError {
    /// Original selected journal cannot be re-admitted.
    #[error(transparent)]
    Journal(#[from] AttemptJournalError),
    /// Passive exact reply association failed.
    #[error(transparent)]
    Association(#[from] IcMutationAssociationError),
    /// Original stage or ancestor evidence cannot be re-admitted.
    #[error(transparent)]
    Stage(#[from] ExecutionWorkflowPersistenceError),
}

#[cfg(all(test, unix))]
mod tests;
