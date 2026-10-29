//! Exact original Applied receipt admission; callers keep domain work before recording.

use crate::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::{MutationOutcomeRecord, MutationReceiptRequest},
        execution_workflow::ExecutionStagePredecessorRecord,
    },
    ops::persistence::{ExecutionStageCheckpointError, ExecutionStageGuard},
};

pub(super) fn admit_applied(
    receipt: MutationReceiptRequest,
    expected_attempt: u32,
    expected_request: &str,
) -> Option<MutationReceiptRequest> {
    (receipt.outcome == MutationOutcomeRecord::Applied
        && receipt.attempt == expected_attempt
        && receipt.request == expected_request)
        .then_some(receipt)
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
    match settlement.and_then(|evidence| stage.reconcile_checkpoint(evidence).map_err(E::from)) {
        Ok(predecessor) => Ok((reply, predecessor)),
        Err(source) => Err(reject(source, Box::new(reply))),
    }
}
