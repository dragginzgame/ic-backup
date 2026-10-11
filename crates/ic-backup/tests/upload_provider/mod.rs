//! Refusal at the upload interface only; no IC response or effect simulation.

use ic_backup::{
    model::{ic_mutation::IcMutationAcknowledgement, ic_snapshot_upload::IcSnapshotUploadAttempt},
    ports::{ic_mutation::IcMutationProviderError, ic_snapshot_upload::IcSnapshotUploadProvider},
};

pub struct RefusingProvider {
    pub calls: usize,
    error: IcMutationProviderError,
}

impl RefusingProvider {
    pub const fn new(error: IcMutationProviderError) -> Self {
        Self { calls: 0, error }
    }
}

impl IcSnapshotUploadProvider for RefusingProvider {
    fn submit_upload(
        &mut self,
        request: &IcSnapshotUploadAttempt<'_, '_>,
        journal: &ic_backup::model::attempt_journal::AttemptJournalRecord,
    ) -> impl std::future::Future<Output = Result<IcMutationAcknowledgement, IcMutationProviderError>>
    {
        std::future::ready({
            request.validate_journal(journal).unwrap();
            self.calls += 1;
            assert_eq!(self.calls, 1, "no hidden retry or local replay dispatch");
            Err(self.error)
        })
    }
}
