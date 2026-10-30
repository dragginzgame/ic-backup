//! One original acquisition observation under retained stage and obligation custody.

use crate::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::{ObservationOutcomeRecord, ObservationReceiptRequest},
        fence_acquisition::{FenceAcquisitionError, FenceAcquisitionPayload},
        fence_obligation::FenceObligationRecord,
        fence_reconciliation::{
            FenceReconciliationIntent, FenceReconciliationObservation, FenceReconciliationRequest,
            FenceReconciliationRequestError,
        },
        operation_plan::OperationPlanError,
    },
    ops::persistence::{
        AttemptJournalError, ExecutionProgressPersistenceError, ExecutionStageGuard,
        ExecutionWorkflowPersistenceError, FenceObligationPersistenceError,
        FenceObligationRequirement, read_fence_obligation,
    },
    policy::fence_reconciliation::{FenceReconciliationError, validate},
    ports::fence_reconciliation::{FenceReconciliationProvider, FenceReconciliationProviderError},
};
use thiserror::Error;

/// Reserve and observe one exact pending acquisition with mandatory fresh application admission.
///
/// Hold no attempt guards on entry. Retain the original stage, complete journals,
/// acquisition bytes, requirement and obligation; restore retains both source and
/// restore plans/layouts and outside-snapshot custody. Original payload admission
/// precedes spending. Complete original-plan admission durably reserves one new
/// observation; selected journal exclusion spans admission and the provider await.
///
/// `admit` qualifies actual current read authority, context, whole-unit membership,
/// exact application observation semantics and never-dispatched custody for this
/// reservation. Any remote preflight needs separate prior accounting. The provider
/// retains/authenticates original request and result evidence and independently
/// qualifies exclusive attribution, nonapplication or settled uncertainty. An Active
/// fence or its current absence alone cannot qualify either acquisition outcome.
///
/// Success only returns matched passive evidence and leaves both attempts pending.
/// Integration-owned durable retention and explicit settlement remain necessary.
/// Cancellation, admission/provider failure or lost replies retain spending and
/// deny reentry while the observation is pending. This step never reconstructs and
/// redispatches a pending observation, retries acquisition, writes a receipt, refunds
/// allowance or releases a fence/reference. No default application provider exists.
/// # Errors
/// Rejects missing/changed originals, exact payload or pending mutation, reservation,
/// fresh admission, provider and passive association failures. Errors after a reply
/// retain that complete observation; failures never authorize another dispatch.
pub async fn observe_fence_acquisition<E: std::error::Error + 'static>(
    stage: &ExecutionStageGuard<'_>,
    requirement: FenceObligationRequirement<'_>,
    obligation: &FenceObligationRecord,
    payload: &FenceAcquisitionPayload,
    challenge: ArtifactChecksumRecord,
    provider: &mut impl FenceReconciliationProvider,
    admit: impl AsyncFnOnce(&FenceReconciliationRequest<'_, '_>) -> Result<(), E>,
) -> Result<FenceReconciliationObservation, FenceReconciliationExecutionError<E>> {
    let plan = stage.plan();
    let mut journal = super::fence_acquisition::open_original::<
        FenceReconciliationExecutionError<E>,
    >(stage, requirement, obligation, payload)?;
    let intent = FenceReconciliationIntent::new(plan, obligation, journal.record()?, challenge)?;
    journal.reserve_planned_observation(
        &plan.digest(),
        intent.mutation_attempt(),
        intent.digest().hash(),
    )?;
    let request = FenceReconciliationRequest::new(&intent, journal.record()?)?;
    admit(&request)
        .await
        .map_err(FenceReconciliationExecutionError::Admission)?;
    read_fence_obligation(stage.layout()?, plan, requirement, &obligation.digest())?;
    let observation = provider
        .observe_fence_acquisition(&request, journal.record()?)
        .await?;
    let admit_reply = || -> Result<(), FenceReconciliationReplyError> {
        validate(&request, journal.record()?, &observation)?;
        read_fence_obligation(stage.layout()?, plan, requirement, &obligation.digest())?;
        Ok(())
    };
    let admitted = admit_reply();
    super::qualified_receipt::retain_reply(observation, admitted, |source, observation| {
        FenceReconciliationExecutionError::AfterReply {
            source,
            observation,
        }
    })
    .map(|(reply, ())| reply)
}

/// Explicitly retain an independently qualified original fence-acquisition observation.
///
/// Supply the original challenge and retained returned observation, including a late
/// reply. Hold no attempt guards on entry. This opens the exact existing journal and
/// admits the complete original plan/journal set before qualification and recording.
/// Both original attempts must still be pending; no provider is called or reservation
/// added. Selected journal exclusion spans the asynchronous qualification callback.
///
/// `qualify` independently authenticates the completed observation and durably retained
/// request/result evidence, exclusive attribution or nonapplication/settled uncertainty,
/// and actual whole-unit/outside-snapshot custody. Return the exact matched attempt,
/// request, outcome and observation evidence. Passive matching alone supplies no receipt.
/// Any remote qualification requires separate prior accounting outside this step.
///
/// Success delegates one receipt to the existing journal owner and retains all original
/// allowances, obligations and source references. It supplies no current Active custody,
/// capture/load/start permission, checkpoint, fence/reference release or prune admission.
/// Settled uncertainty closes the observation while keeping the mutation pending.
/// Cancellation before recording keeps both attempts pending. A write or post-write
/// admission failure may leave a retained receipt; reopen original accounting to resolve
/// it rather than repeating an observation or assuming that no settlement occurred.
/// The borrowed observation remains available on every failure.
/// # Errors
/// Rejects changed originals, payload/reservation/result mismatch, missing/held journals,
/// qualification or exact receipt mismatch, and durable-recording failures.
pub async fn settle_fence_acquisition_observation<E: std::error::Error + 'static>(
    stage: &ExecutionStageGuard<'_>,
    requirement: FenceObligationRequirement<'_>,
    obligation: &FenceObligationRecord,
    payload: &FenceAcquisitionPayload,
    challenge: ArtifactChecksumRecord,
    observation: &FenceReconciliationObservation,
    qualify: impl AsyncFnOnce(
        &FenceReconciliationRequest<'_, '_>,
        &FenceReconciliationObservation,
    ) -> Result<ObservationReceiptRequest, E>,
) -> Result<ObservationOutcomeRecord, FenceReconciliationSettlementError<E>> {
    let plan = stage.plan();
    let mut journal = super::fence_acquisition::open_original::<
        FenceReconciliationSettlementError<E>,
    >(stage, requirement, obligation, payload)?;
    journal.admit_planned_progress(&plan.digest())?;
    let intent = FenceReconciliationIntent::new(plan, obligation, journal.record()?, challenge)?;
    let request = FenceReconciliationRequest::new(&intent, journal.record()?)?;
    let admitted = validate(&request, journal.record()?, observation)?;
    let receipt = qualify(&request, observation)
        .await
        .map_err(FenceReconciliationSettlementError::Qualification)?;
    if receipt.attempt != admitted.observation_attempt()
        || receipt.request != admitted.request().hash()
        || receipt.outcome != admitted.outcome()
        || receipt.evidence != admitted.evidence().hash()
    {
        return Err(FenceReconciliationSettlementError::ReceiptMismatch);
    }
    validate(&request, journal.record()?, observation)?;
    read_fence_obligation(stage.layout()?, plan, requirement, &obligation.digest())?;
    let outcome = receipt.outcome;
    journal.record_planned_observation(&plan.digest(), receipt)?;
    read_fence_obligation(stage.layout()?, plan, requirement, &obligation.digest())?;
    Ok(outcome)
}

/// Explicit settlement rejection; no error permits observation reissue or evidence cleanup.
#[derive(Debug, Error)]
pub enum FenceReconciliationSettlementError<E: std::error::Error + 'static> {
    /// Original acquisition authority cannot be derived.
    #[error(transparent)]
    Plan(#[from] OperationPlanError),
    /// Retained stage or ancestor evidence differs.
    #[error(transparent)]
    Stage(#[from] ExecutionWorkflowPersistenceError),
    /// Original requirement or obligation cannot be admitted.
    #[error(transparent)]
    Obligation(#[from] FenceObligationPersistenceError),
    /// Existing selected journal cannot be admitted.
    #[error(transparent)]
    Journal(#[from] AttemptJournalError),
    /// Original acquisition payload or authority differs.
    #[error(transparent)]
    Payload(#[from] FenceAcquisitionError),
    /// Complete original accounting or durable receipt publication failed.
    #[error(transparent)]
    Progress(#[from] ExecutionProgressPersistenceError),
    /// Original pending mutation/observation identity differs.
    #[error(transparent)]
    Request(#[from] FenceReconciliationRequestError),
    /// Passive exact result matching failed.
    #[error(transparent)]
    Association(#[from] FenceReconciliationError),
    /// Independent application qualification rejected; original attempts stay pending.
    #[error("fence observation settlement qualification failed: {0}")]
    Qualification(#[source] E),
    /// Qualified receipt does not retain the exact matched observation.
    #[error("fence observation settlement receipt mismatch")]
    ReceiptMismatch,
}

/// Observation-step failure preserving pending attempts, original obligations and replies.
#[derive(Debug, Error)]
pub enum FenceReconciliationExecutionError<E: std::error::Error + 'static> {
    /// Original acquisition authority cannot be derived.
    #[error(transparent)]
    Plan(#[from] OperationPlanError),
    /// Retained stage or ancestor evidence differs.
    #[error(transparent)]
    Stage(#[from] ExecutionWorkflowPersistenceError),
    /// Original purpose-specific requirement or obligation cannot be admitted.
    #[error(transparent)]
    Obligation(#[from] FenceObligationPersistenceError),
    /// Original selected journal admission failed.
    #[error(transparent)]
    Journal(#[from] AttemptJournalError),
    /// Original acquisition bytes or authority differ.
    #[error(transparent)]
    Payload(#[from] FenceAcquisitionError),
    /// Complete journals or durable observation reservation failed.
    #[error(transparent)]
    Progress(#[from] ExecutionProgressPersistenceError),
    /// Original pending mutation or current observation binding differs.
    #[error(transparent)]
    Request(#[from] FenceReconciliationRequestError),
    /// Fresh application admission rejected after reservation.
    #[error("fresh fence reconciliation admission failed: {0}")]
    Admission(#[source] E),
    /// The one provider invocation failed; the observation remains pending.
    #[error(transparent)]
    Provider(#[from] FenceReconciliationProviderError),
    /// Admission after the reply failed, retaining its complete original evidence.
    #[error("fence reconciliation reply rejected: {source}")]
    AfterReply {
        /// Typed original admission failure.
        source: FenceReconciliationReplyError,
        /// Complete returned passive evidence, without settlement authority.
        observation: Box<FenceReconciliationObservation>,
    },
}

/// Admission failure after a returned fence reconciliation reply.
#[derive(Debug, Error)]
pub enum FenceReconciliationReplyError {
    /// Original selected journal cannot be re-admitted.
    #[error(transparent)]
    Journal(#[from] AttemptJournalError),
    /// Passive exact reply association failed.
    #[error(transparent)]
    Association(#[from] FenceReconciliationError),
    /// Original stage or ancestor evidence cannot be re-admitted.
    #[error(transparent)]
    Stage(#[from] ExecutionWorkflowPersistenceError),
    /// Original requirement or obligation cannot be re-admitted.
    #[error(transparent)]
    Obligation(#[from] FenceObligationPersistenceError),
}
