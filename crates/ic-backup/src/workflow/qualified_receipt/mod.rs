//! Exact original Applied receipt recording after independent reply qualification.

use crate::{
    model::attempt_journal::{MutationOutcomeRecord, MutationReceiptRequest},
    ops::persistence::{AttemptJournalError, AttemptJournalGuard},
};

pub(super) struct AppliedReceiptRecorder<'journal, 'layout, 'request> {
    pub(super) journal: &'journal mut AttemptJournalGuard<'layout>,
    pub(super) expected_attempt: u32,
    pub(super) expected_request: &'request str,
}

impl AppliedReceiptRecorder<'_, '_, '_> {
    // The caller's hook owns domain admission and any append that must precede
    // the receipt. Keep journal recording here as the final shared transition.
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
        Q: From<AttemptJournalError>,
    {
        let receipt = qualify(request, reply).await.map_err(qualification_error)?;
        if receipt.outcome != MutationOutcomeRecord::Applied
            || receipt.attempt != self.expected_attempt
            || receipt.request != self.expected_request
        {
            return Err(receipt_error());
        }
        let result = before_record()?;
        self.journal.record_mutation(receipt)?;
        Ok(result)
    }
}
