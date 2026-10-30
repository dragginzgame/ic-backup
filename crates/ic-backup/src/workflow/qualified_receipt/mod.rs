//! Exact original Applied receipt admission; callers keep domain work before recording.

use crate::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::{MutationOutcomeRecord, MutationReceiptRequest},
        execution_workflow::ExecutionStagePredecessorRecord,
    },
    ops::persistence::{
        AttemptJournalError, AttemptJournalGuard, ExecutionStageCheckpointError,
        ExecutionStageGuard,
    },
};

// Domain preparation runs after exact receipt matching and before recording.
// Download uses it to append bytes; restore uses it to re-admit source custody.
pub(super) fn record_applied<T, E: From<AttemptJournalError>>(
    journal: &mut AttemptJournalGuard<'_>,
    receipt: MutationReceiptRequest,
    expected_attempt: u32,
    expected_request: &str,
    mismatch: E,
    prepare: impl FnOnce() -> Result<T, E>,
) -> Result<T, E> {
    if receipt.outcome != MutationOutcomeRecord::Applied
        || receipt.attempt != expected_attempt
        || receipt.request != expected_request
    {
        return Err(mismatch);
    }
    let result = prepare()?;
    journal.record_mutation(receipt)?;
    Ok(result)
}

// Keep the bounded returned reply available on every post-reply rejection.
pub(super) fn retain_reply<R, T, E, X>(
    reply: R,
    result: Result<T, E>,
    reject: impl FnOnce(E, Box<R>) -> X,
) -> Result<(R, T), X> {
    match result {
        Ok(value) => Ok((reply, value)),
        Err(source) => Err(reject(source, Box::new(reply))),
    }
}

// Called only after the single-call coordinator has released its attempt guard.
// Any reply survives qualification or checkpoint rejection in the caller's error.
pub(super) fn checkpoint_reply<R, E, X>(
    stage: &ExecutionStageGuard<'_>,
    reply: R,
    settlement: Result<ArtifactChecksumRecord, E>,
    reject: impl FnOnce(E, Box<R>) -> X,
) -> Result<(R, ExecutionStagePredecessorRecord), X>
where
    E: From<ExecutionStageCheckpointError>,
{
    retain_reply(
        reply,
        settlement.and_then(|evidence| stage.reconcile_checkpoint(evidence).map_err(E::from)),
        reject,
    )
}
