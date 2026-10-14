//! Exact original Applied receipt recording after independent reply qualification.

use crate::{
    model::attempt_journal::{MutationOutcomeRecord, MutationReceiptRequest},
    ops::persistence::{
        AttemptJournalError, AttemptJournalGuard, ExecutionStageGuard,
        ExecutionWorkflowPersistenceError,
    },
};

pub(super) struct AppliedReceiptRecorder<'stage, 'journal, 'layout, 'request> {
    pub(super) stage: &'stage ExecutionStageGuard<'layout>,
    pub(super) journal: &'journal mut AttemptJournalGuard<'layout>,
    pub(super) expected_attempt: u32,
    pub(super) expected_request: &'request str,
}

impl AppliedReceiptRecorder<'_, '_, '_, '_> {
    pub(super) async fn record_applied<R, A, E, Q, T>(
        &mut self,
        request: &R,
        reply: &A,
        qualify: impl AsyncFnOnce(&R, &A) -> Result<MutationReceiptRequest, E>,
        qualification_error: fn(E) -> Q,
        receipt_error: fn() -> Q,
        before_record: impl FnOnce() -> Result<T, Q>,
    ) -> Result<T, Q>
    where
        Q: From<ExecutionWorkflowPersistenceError> + From<AttemptJournalError>,
    {
        let receipt = qualify(request, reply).await.map_err(qualification_error)?;
        if receipt.outcome != MutationOutcomeRecord::Applied
            || receipt.attempt != self.expected_attempt
            || receipt.request != self.expected_request
        {
            return Err(receipt_error());
        }
        let result = before_record()?;
        self.stage.layout()?;
        self.journal.record_mutation(receipt)?;
        Ok(result)
    }
}
