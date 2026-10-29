//! One original same-ID load/start with fresh safety and independently qualified settlement.

mod source;
pub use source::{IcRestoreSource, IcRestoreSourceError};

use crate::{
    model::{
        attempt_journal::MutationReceiptRequest,
        execution_workflow::ExecutionStagePredecessorRecord,
        ic_mutation::{IcMutationAcknowledgement, IcMutationRequest, IcMutationRequestError},
        restore_safety::{RestoreSafetyObservation, RestoreSafetyRequest},
    },
    ops::persistence::{
        AttemptJournalError, ExecutionProgressPersistenceError, ExecutionStageCheckpointError,
        ExecutionStageGuard, ExecutionWorkflowPersistenceError, RestoreSafetyPersistenceError,
        read_restore_safety_requirement,
    },
    policy::{
        ic_mutation::{IcMutationAssociationError, IcMutationReplyView, validate_acknowledgement},
        restore_safety::{RestoreSafetyError, validate},
    },
    ports::ic_mutation::{IcMutationProvider, IcMutationProviderError},
};
use thiserror::Error;

/// Execute and checkpoint one exact singleton original load or start stage.
///
/// Retain the complete original stage/journals, source plan, safety requirement and
/// source references first; hold no attempt guards. `source` explicitly binds an
/// existing snapshot or the complete original upload destination independently of
/// the load plan. Re-admit that provenance across all boundaries. `safety` binds the exact original
/// load/start bytes and a fresh integration-owned challenge. Read the requirement
/// under both layouts before spending and recheck it across admission and settlement.
/// No source is reconstructed, uploaded, replaced or released by this function.
///
/// After durable reservation, mandatory `admit` must qualify current authenticated
/// control, snapshot-origin permissions, complete uploaded source, application safety,
/// stable byte/command custody and proof of no prior dispatch. Return actual fresh
/// safety observations; the existing policy checks exact context/inventory/selection,
/// original lane/fence and load/start preconditions before one provider invocation.
/// All remote preflight/qualification calls need separate prior accounting. There is
/// no default no-external-effects lane or automatic stopped/load-success inference.
///
/// Mandatory `qualify` independently authenticates original successful attribution
/// and durably retains exact request/reply evidence before its explicit Applied
/// receipt. Record through the existing journal owner under exclusion, then release
/// the lock and checkpoint canonical request/raw-reply evidence. Load settlement
/// does not establish application acceptance for a later start: qualify that afresh.
/// Pending/Applied stages never invoke callbacks/providers again. Checkpoint replay
/// stays local; failure preserves spending, replies, obligations and references.
/// This is one bounded step, not complete restore, terminal or fence/reference release.
/// Admission, submission and independent qualification are awaited under the
/// selected journal lock. Cancellation retains its current pending/Applied state
/// and grants no repeat invocation. Qualification must durably retain returned
/// bytes before awaiting work that could be cancelled; in-memory replies do not
/// survive cancellation. The core imposes no executor or `Send` requirement.
/// # Errors
/// Rejects changed/non-singleton originals, retained source/requirement failures,
/// spending, fresh admission/safety, provider/association, qualification/receipt or
/// checkpoint failure. Returned bounded replies survive post-reply rejection.
pub async fn restore_snapshot<E: std::error::Error + 'static>(
    stage: &ExecutionStageGuard<'_>,
    source: IcRestoreSource<'_>,
    safety: &RestoreSafetyRequest<'_>,
    provider: &mut impl IcMutationProvider,
    admit: impl AsyncFnOnce(
        &IcMutationRequest<'_>,
        &RestoreSafetyRequest<'_>,
    ) -> Result<RestoreSafetyObservation, E>,
    qualify: impl AsyncFnOnce(
        &IcMutationRequest<'_>,
        &IcMutationAcknowledgement,
    ) -> Result<MutationReceiptRequest, E>,
) -> Result<(IcMutationAcknowledgement, ExecutionStagePredecessorRecord), IcRestoreExecutionError<E>>
{
    let plan = stage.plan();
    let sequence = safety.binding().operation_sequence();
    if plan.operations().len() != 1 || plan.operations()[0].operation_sequence() != sequence {
        return Err(IcRestoreExecutionError::OriginalMismatch);
    }
    let retain = || {
        source.validate(stage, safety.wire())?;
        read_restore_safety_requirement(
            stage.layout()?,
            source.layout,
            plan,
            source.plan,
            &safety.requirement().digest(),
        )?;
        Ok::<_, IcRestoreReplyError<E>>(())
    };
    retain()?;
    let authority = plan
        .attempt_authority(sequence)
        .map_err(IcMutationRequestError::from)?;
    let (acknowledgement, settlement) = super::ic_mutation::submit_with(
        stage,
        &authority,
        safety.wire(),
        provider,
        async |request| {
            let observation = admit(request, safety)
                .await
                .map_err(IcRestoreExecutionError::Admission)?;
            validate(safety, &observation)?;
            retain()?;
            Ok::<_, IcRestoreExecutionError<E>>(())
        },
        async |journal, request, acknowledgement| {
            let admitted = validate_acknowledgement(request, journal.record()?, acknowledgement)?;
            retain()?;
            let receipt = qualify(request, acknowledgement)
                .await
                .map_err(IcRestoreReplyError::Qualification)?;
            let receipt = super::qualified_receipt::admit_applied(
                receipt,
                request.mutation_attempt(),
                safety.wire().digest().hash(),
            )
            .ok_or(IcRestoreReplyError::ReceiptRequired)?;
            retain()?;
            journal.record_mutation(receipt)?;
            retain()?;
            let IcMutationReplyView::Lifecycle(reply) = admitted.reply() else {
                return Err(IcRestoreReplyError::ReceiptRequired);
            };
            Ok::<_, IcRestoreReplyError<E>>(reply.digest())
        },
    )
    .await?;
    super::qualified_receipt::checkpoint_reply(
        stage,
        acknowledgement,
        settlement,
        |source, acknowledgement| IcRestoreExecutionError::AfterReply {
            source,
            acknowledgement,
        },
    )
}

/// Original restore-step failure; no variant authorizes reissue or cleanup.
#[derive(Debug, Error)]
pub enum IcRestoreExecutionError<E: std::error::Error + 'static> {
    /// Only an exact singleton original load/start stage is admitted.
    #[error("restore requires an exact singleton original load/start stage")]
    OriginalMismatch,
    /// Original workflow/stage/ancestor admission failed.
    #[error(transparent)]
    Stage(#[from] ExecutionWorkflowPersistenceError),
    /// Existing journal admission failed.
    #[error(transparent)]
    Journal(#[from] AttemptJournalError),
    /// Complete original progress/prerequisites or reservation failed.
    #[error(transparent)]
    Progress(#[from] ExecutionProgressPersistenceError),
    /// Exact original mutation binding failed.
    #[error(transparent)]
    Request(#[from] IcMutationRequestError),
    /// Original source/requirement retention failed before reply.
    #[error(transparent)]
    Retention(#[from] IcRestoreReplyError<E>),
    /// Integration-owned fresh admission failed after reservation.
    #[error("fresh restore admission failed: {0}")]
    Admission(#[source] E),
    /// Existing current restore-safety policy rejected before dispatch.
    #[error(transparent)]
    Safety(#[from] RestoreSafetyError),
    /// Exactly one provider call failed; original spending stays pending.
    #[error(transparent)]
    Provider(#[from] IcMutationProviderError),
    /// Failure after reply retains its original bounded acknowledgement.
    #[error("restore reply settlement failed: {source}")]
    AfterReply {
        /// Existing evidence/qualification/persistence rejection.
        source: IcRestoreReplyError<E>,
        /// Exact original bounded reply without an inferred outcome.
        acknowledgement: Box<IcMutationAcknowledgement>,
    },
}

/// Post-reply restore rejection, preserving original receipts and pending spending.
#[derive(Debug, Error)]
pub enum IcRestoreReplyError<E: std::error::Error + 'static> {
    /// Existing snapshot or complete uploaded destination no longer matches.
    #[error(transparent)]
    Source(#[from] IcRestoreSourceError),
    /// Original stage/ancestor/layout admission failed.
    #[error(transparent)]
    Stage(#[from] ExecutionWorkflowPersistenceError),
    /// Existing original source/requirement admission failed.
    #[error(transparent)]
    Retention(#[from] RestoreSafetyPersistenceError),
    /// Existing selected journal/receipt admission failed.
    #[error(transparent)]
    Journal(#[from] AttemptJournalError),
    /// Existing bounded original acknowledgement association failed.
    #[error(transparent)]
    Association(#[from] IcMutationAssociationError),
    /// Authentication/attribution or durable original-byte retention failed.
    #[error("restore reply qualification failed: {0}")]
    Qualification(#[source] E),
    /// Require an explicit qualified exact original Applied receipt.
    #[error("restore requires an explicit original Applied receipt")]
    ReceiptRequired,
    /// Existing checkpoint failed; Applied and occupied evidence remain.
    #[error(transparent)]
    Checkpoint(#[from] ExecutionStageCheckpointError),
}

#[cfg(test)]
mod tests;
