//! Passive native replies and explicit drift injection; no simulated IC effects.

use crate::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::{AttemptAuthorityRecord, AttemptJournalRecord},
        ic_mutation::{IcMutationAcknowledgement, IcMutationRequest},
        ic_request::IcManagementMethodRecord,
        ic_snapshot_transfer_read::{
            IcSnapshotTransferReadRequest, IcSnapshotTransferReadResponse,
        },
        ic_snapshot_upload::IcSnapshotUploadAttempt,
        ic_update_response::IcUpdateResponseInput,
        operation_plan::PlanContextRecord,
    },
    ports::{
        ic_mutation::{IcMutationProvider, IcMutationProviderError},
        ic_observation::IcObservationProviderError,
        ic_snapshot_transfer_read::IcSnapshotTransferReadProvider,
        ic_snapshot_upload::IcSnapshotUploadProvider,
    },
};
use std::{
    fs,
    future::{Future, ready},
    path::PathBuf,
};

pub(crate) struct Provider<E> {
    pub calls: usize,
    pub reply: Result<Vec<u8>, E>,
    pub change_stage: Option<PathBuf>,
    pub replace_layout: Option<PathBuf>,
    pub target: Option<&'static str>,
    pub evidence: Option<ArtifactChecksumRecord>,
    pub method: Option<IcManagementMethodRecord>,
}

impl<E: Clone> Provider<E> {
    pub(crate) fn new(reply: Result<Vec<u8>, E>) -> Self {
        Self {
            calls: 0,
            reply,
            change_stage: None,
            replace_layout: None,
            target: None,
            evidence: None,
            method: None,
        }
    }

    fn response(
        &mut self,
        authority: &AttemptAuthorityRecord,
        attempt: u32,
        context: &PlanContextRecord,
        target: &str,
    ) -> Result<IcUpdateResponseInput, E> {
        self.calls += 1;
        if let Some(path) = &self.change_stage {
            fs::write(path, b"{}").unwrap();
        }
        if let Some(path) = &self.replace_layout {
            fs::rename(path, path.with_extension("retained-original")).unwrap();
            fs::create_dir(path).unwrap();
        }
        let mut input = super::update::input(authority, attempt, context, self.reply.clone()?);
        input.target = self.target.unwrap_or(target).into();
        if let Some(evidence) = &self.evidence {
            input.evidence = evidence.clone();
        }
        Ok(input)
    }
}

impl IcMutationProvider for Provider<IcMutationProviderError> {
    fn submit_mutation(
        &mut self,
        request: &IcMutationRequest<'_>,
        journal: &AttemptJournalRecord,
    ) -> impl Future<Output = Result<IcMutationAcknowledgement, IcMutationProviderError>> {
        request.validate_journal(journal).unwrap();
        if let Some(method) = self.method {
            assert_eq!(request.payload().method(), method);
        }
        ready(
            self.response(
                request.authority(),
                request.mutation_attempt(),
                request.plan().context(),
                request.payload().target(),
            )
            .and_then(|input| {
                IcMutationAcknowledgement::new(input)
                    .map_err(|_| IcMutationProviderError::Indeterminate)
            }),
        )
    }
}

impl IcSnapshotUploadProvider for Provider<IcMutationProviderError> {
    fn submit_upload(
        &mut self,
        request: &IcSnapshotUploadAttempt<'_, '_>,
        journal: &AttemptJournalRecord,
    ) -> impl Future<Output = Result<IcMutationAcknowledgement, IcMutationProviderError>> {
        request.validate_journal(journal).unwrap();
        ready(
            self.response(
                request.authority(),
                request.mutation_attempt(),
                request.plan().context(),
                request.payload().target(),
            )
            .and_then(|input| {
                IcMutationAcknowledgement::new(input)
                    .map_err(|_| IcMutationProviderError::Indeterminate)
            }),
        )
    }
}

impl IcSnapshotTransferReadProvider for Provider<IcObservationProviderError> {
    fn read_snapshot(
        &mut self,
        request: &IcSnapshotTransferReadRequest<'_, '_>,
        journal: &AttemptJournalRecord,
    ) -> impl Future<Output = Result<IcSnapshotTransferReadResponse, IcObservationProviderError>>
    {
        request.validate_journal(journal).unwrap();
        ready(
            self.response(
                request.authority(),
                request.mutation_attempt(),
                request.plan().context(),
                request.payload().target(),
            )
            .and_then(|input| {
                IcSnapshotTransferReadResponse::new(input)
                    .map_err(|_| IcObservationProviderError::Indeterminate)
            }),
        )
    }
}

/// Suspends after validating the exact reserved request; cancellation tests own polling.
#[derive(Default)]
pub(crate) struct SuspendedProvider {
    pub calls: usize,
}
impl IcMutationProvider for SuspendedProvider {
    async fn submit_mutation(
        &mut self,
        request: &IcMutationRequest<'_>,
        journal: &AttemptJournalRecord,
    ) -> Result<IcMutationAcknowledgement, IcMutationProviderError> {
        request.validate_journal(journal).unwrap();
        self.calls += 1;
        std::future::pending().await
    }
}

impl IcSnapshotTransferReadProvider for SuspendedProvider {
    async fn read_snapshot(
        &mut self,
        request: &IcSnapshotTransferReadRequest<'_, '_>,
        journal: &crate::model::attempt_journal::AttemptJournalRecord,
    ) -> Result<IcSnapshotTransferReadResponse, IcObservationProviderError> {
        request.validate_journal(journal).unwrap();
        self.calls += 1;
        std::future::pending().await
    }
}

impl IcSnapshotUploadProvider for SuspendedProvider {
    async fn submit_upload(
        &mut self,
        request: &IcSnapshotUploadAttempt<'_, '_>,
        journal: &crate::model::attempt_journal::AttemptJournalRecord,
    ) -> Result<IcMutationAcknowledgement, IcMutationProviderError> {
        request.validate_journal(journal).unwrap();
        self.calls += 1;
        std::future::pending().await
    }
}
