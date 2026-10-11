//! One freshly reserved application acquisition under exact original stage/obligation custody.

use crate::{
    model::{
        fence_acquisition::{
            FenceAcquisitionAcknowledgement, FenceAcquisitionError, FenceAcquisitionPayload,
            FenceAcquisitionRequest, original_authority,
        },
        fence_obligation::FenceObligationRecord,
        operation_plan::OperationPlanError,
    },
    ops::persistence::{
        AttemptJournalError, AttemptJournalGuard, ExecutionProgressPersistenceError,
        ExecutionStageGuard, ExecutionWorkflowPersistenceError, FenceObligationPersistenceError,
        FenceObligationRequirement, read_fence_obligation,
    },
    policy::fence_acquisition::{FenceAcquisitionAcknowledgementError, validate_acknowledgement},
    ports::fence_acquisition::{FenceAcquisitionProvider, FenceAcquisitionProviderError},
};
use thiserror::Error;

/// Reserve the original acquisition, freshly admit it and invoke one exact application update.
///
/// Hold no attempt guards on entry. Retain the full original stage, every journal,
/// requirement and whole-unit obligation before calling. Restore requires its source
/// layout/plan and outside-snapshot custody. Exact original payload admission precedes
/// spending; complete original-plan admission owns the durable mutation reservation.
/// The selected journal remains locked across admission and the provider await.
/// Cancellation retains pending spending and denies redispatch through this step.
///
/// Mandatory fresh `admit` must qualify actual context, whole-unit membership,
/// application method/argument purpose, original fence/revisions, permissions,
/// dependencies, command custody and proof this reservation has never dispatched.
/// Any remote preflight has its own prior accounting. The configured provider retains
/// exact signed ingress before its one update and exact reply bytes before returning.
/// There is no default application, permission or retention callback.
///
/// Recheck retained stage/ancestors and requirement/obligation around dispatch.
/// Success returns passive association only and leaves the mutation pending.
/// Applications independently qualify attribution and Active custody before explicit
/// settlement; lost replies use original reconciliation accounting. No receipt, refund,
/// automatic uncertainty, acquired fence or terminal/fence/reference release follows.
/// # Errors
/// Rejects changed/missing originals, binding/spending, fresh admission, provider or
/// association failures. Every post-reservation error retains consumption; post-reply
/// rejections retain the returned acknowledgement. No failure permits redispatch.
pub async fn acquire_fence<E: std::error::Error + 'static>(
    stage: &ExecutionStageGuard<'_>,
    requirement: FenceObligationRequirement<'_>,
    obligation: &FenceObligationRecord,
    payload: &FenceAcquisitionPayload,
    provider: &mut impl FenceAcquisitionProvider,
    admit: impl AsyncFnOnce(&FenceAcquisitionRequest<'_>) -> Result<(), E>,
) -> Result<FenceAcquisitionAcknowledgement, FenceAcquisitionExecutionError<E>> {
    let plan = stage.plan();
    let layout = stage.layout()?;
    read_fence_obligation(layout, plan, requirement, &obligation.digest())?;
    let authority = plan.attempt_authority(obligation.acquisition_operation())?;
    let mut journal = AttemptJournalGuard::open(layout, &authority)?;
    original_authority(plan, obligation, journal.record()?, payload)?;
    journal.reserve_planned_mutation(&plan.digest())?;
    let request = FenceAcquisitionRequest::new(plan, obligation, journal.record()?, payload)?;
    admit(&request)
        .await
        .map_err(FenceAcquisitionExecutionError::Admission)?;
    read_fence_obligation(stage.layout()?, plan, requirement, &obligation.digest())?;
    let acknowledgement = provider.acquire_fence(&request, journal.record()?).await?;
    let association = match journal.record() {
        Ok(record) => record,
        Err(source) => {
            return Err(FenceAcquisitionExecutionError::AfterReplyJournal {
                source,
                acknowledgement: Box::new(acknowledgement),
            });
        }
    };
    if let Err(source) = validate_acknowledgement(&request, association, &acknowledgement) {
        return Err(FenceAcquisitionExecutionError::Association {
            source,
            acknowledgement: Box::new(acknowledgement),
        });
    }
    let layout = match stage.layout() {
        Ok(layout) => layout,
        Err(source) => {
            return Err(FenceAcquisitionExecutionError::AfterReplyStage {
                source,
                acknowledgement: Box::new(acknowledgement),
            });
        }
    };
    if let Err(source) = read_fence_obligation(layout, plan, requirement, &obligation.digest()) {
        return Err(FenceAcquisitionExecutionError::AfterReplyObligation {
            source,
            acknowledgement: Box::new(acknowledgement),
        });
    }
    Ok(acknowledgement)
}

/// Acquisition-step failure preserving original spending, obligations and passive replies.
#[derive(Debug, Error)]
pub enum FenceAcquisitionExecutionError<E: std::error::Error + 'static> {
    /// Exact original acquisition authority cannot be derived.
    #[error(transparent)]
    Plan(#[from] OperationPlanError),
    /// Original stage, workflow or ancestor evidence differs.
    #[error(transparent)]
    Stage(#[from] ExecutionWorkflowPersistenceError),
    /// Original requirement or obligation cannot be admitted.
    #[error(transparent)]
    Obligation(#[from] FenceObligationPersistenceError),
    /// Original selected journal admission failed.
    #[error(transparent)]
    Journal(#[from] AttemptJournalError),
    /// Complete original journals, prerequisites or durable reservation failed.
    #[error(transparent)]
    Progress(#[from] ExecutionProgressPersistenceError),
    /// Exact original payload or current reservation differs.
    #[error(transparent)]
    Request(#[from] FenceAcquisitionError),
    /// Fresh application admission rejected after reservation.
    #[error("fresh fence acquisition admission failed: {0}")]
    Admission(#[source] E),
    /// Single provider invocation failed; no nonapplication or refund is inferred.
    #[error(transparent)]
    Provider(#[from] FenceAcquisitionProviderError),
    /// Passive original authority/attempt association rejected the reply.
    #[error("fence acquisition acknowledgement association failed: {source}")]
    Association {
        /// Canonical passive association rejection.
        source: FenceAcquisitionAcknowledgementError,
        /// Exact returned passive acknowledgement, without acquisition proof.
        acknowledgement: Box<FenceAcquisitionAcknowledgement>,
    },
    /// Selected journal re-admission failed after reply.
    #[error("fence acquisition journal changed after reply: {source}")]
    AfterReplyJournal {
        /// Original journal rejection.
        source: AttemptJournalError,
        /// Exact returned passive acknowledgement.
        acknowledgement: Box<FenceAcquisitionAcknowledgement>,
    },
    /// Retained stage/ancestor admission failed after reply.
    #[error("fence acquisition stage changed after reply: {source}")]
    AfterReplyStage {
        /// Original stage/ancestor rejection.
        source: ExecutionWorkflowPersistenceError,
        /// Exact returned passive acknowledgement.
        acknowledgement: Box<FenceAcquisitionAcknowledgement>,
    },
    /// Retained requirement/obligation admission failed after reply.
    #[error("fence acquisition obligation changed after reply: {source}")]
    AfterReplyObligation {
        /// Original requirement/obligation rejection.
        source: FenceObligationPersistenceError,
        /// Exact returned passive acknowledgement.
        acknowledgement: Box<FenceAcquisitionAcknowledgement>,
    },
}

#[cfg(all(test, unix))]
mod tests;
