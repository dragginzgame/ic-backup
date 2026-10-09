//! One freshly reserved capture update under exact original stage evidence.

use crate::{
    model::{
        ic_mutation::{IcMutationAcknowledgement, IcMutationRequest, IcMutationRequestError},
        ic_request::{IcManagementMethodRecord, IcManagementRequestRecord},
        operation_plan::OperationPlanError,
    },
    ops::persistence::{
        AttemptJournalError, AttemptJournalGuard, ExecutionProgressPersistenceError,
        ExecutionStageGuard, ExecutionWorkflowPersistenceError,
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
pub fn capture_snapshot<E: std::error::Error + 'static>(
    stage: &ExecutionStageGuard<'_>,
    operation_sequence: u64,
    payload: &IcManagementRequestRecord,
    provider: &mut impl IcMutationProvider,
    admit: impl FnOnce(&IcMutationRequest<'_>) -> Result<(), E>,
) -> Result<IcMutationAcknowledgement, IcSnapshotCaptureExecutionError<E>> {
    if payload.method() != IcManagementMethodRecord::TakeCanisterSnapshot {
        return Err(IcSnapshotCaptureExecutionError::CaptureRequired);
    }
    let plan = stage.plan();
    let authority = plan.attempt_authority(operation_sequence)?;
    payload
        .validate_mutation_binding(authority.binding())
        .map_err(IcMutationRequestError::from)?;
    let mut journal = AttemptJournalGuard::open(stage.layout()?, &authority)?;
    journal.reserve_planned_mutation(&plan.digest())?;
    let request = IcMutationRequest::new(plan, operation_sequence, journal.record()?, payload)?;
    admit(&request).map_err(IcSnapshotCaptureExecutionError::Admission)?;
    stage.layout()?;
    let acknowledgement = provider.submit_mutation(&request)?;
    let association = match journal.record() {
        Ok(record) => record,
        Err(source) => {
            return Err(IcSnapshotCaptureExecutionError::AfterReplyJournal {
                source,
                acknowledgement: Box::new(acknowledgement),
            });
        }
    };
    if let Err(source) = validate_acknowledgement(&request, association, &acknowledgement) {
        return Err(IcSnapshotCaptureExecutionError::Association {
            source,
            acknowledgement: Box::new(acknowledgement),
        });
    }
    if let Err(source) = stage.layout() {
        return Err(IcSnapshotCaptureExecutionError::AfterReplyStage {
            source,
            acknowledgement: Box::new(acknowledgement),
        });
    }
    Ok(acknowledgement)
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
    /// Passive bounded association rejected a returned acknowledgement.
    #[error("snapshot capture acknowledgement association failed: {source}")]
    Association {
        /// Existing canonical mutation association rejection.
        source: IcMutationAssociationError,
        /// Exact returned acknowledgement, without authenticated outcome.
        acknowledgement: Box<IcMutationAcknowledgement>,
    },
    /// Selected journal re-admission failed after a reply.
    #[error("snapshot capture journal changed after reply: {source}")]
    AfterReplyJournal {
        /// Original journal rejection.
        source: AttemptJournalError,
        /// Exact bounded returned acknowledgement.
        acknowledgement: Box<IcMutationAcknowledgement>,
    },
    /// Stage or ancestor re-admission failed after a reply.
    #[error("snapshot capture stage changed after reply: {source}")]
    AfterReplyStage {
        /// Original stage or ancestor rejection.
        source: ExecutionWorkflowPersistenceError,
        /// Exact bounded returned acknowledgement.
        acknowledgement: Box<IcMutationAcknowledgement>,
    },
}

#[cfg(all(test, unix))]
mod tests;
