//! Three separately accounted async recovery ports over one retained submission owner.
use crate::{AgentTransport, PreparedUpdate, ReservedUpdate, UpdateOutcome};
use ic_agent::RequestId;
use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::AttemptJournalRecord,
        ic_observation::{IcObservationRequest, IcObservationResponse, IcObservationResponseInput},
        ic_snapshot_upload_data_observation::{
            IcSnapshotUploadDataObservationRequest, IcSnapshotUploadDataObservationResponse,
        },
        ic_snapshot_upload_observation::IcSnapshotUploadObservationRequest,
    },
    ports::{
        ic_observation::{IcObservationProvider, IcObservationProviderError},
        ic_snapshot_upload_data_observation::IcSnapshotUploadDataObservationProvider,
        ic_snapshot_upload_observation::IcSnapshotUploadObservationProvider,
    },
};

/// One original status/list/data recovery observation, with mandatory durable ingress retention.
///
/// The caller durably reserves this separate observation before invocation and holds
/// its current journal guard through the future. `retain` must durably retain the exact
/// signed envelope/request ID with the complete original plan, mutation/observation
/// identities and exclusive never-dispatched custody. It makes no remote calls and
/// must refuse previously dispatched originals. There is no default retainer.
///
/// The callback receives only `Observation`, `UploadObservation` or `UploadDataObservation`
/// requests through these ports. Current read permissions, original recovery chronology,
/// authentic baseline/metadata/destination custody and exclusive attribution remain
/// integration-owned. No fresh allowance, hidden retry, poll or automatic receipt exists.
/// Accepted/lost/cancelled observations and their mutations stay pending. Passive
/// certificate-verified replies are not settled outcomes or release authority.
pub struct AgentObservationProvider<'a, R> {
    transport: &'a AgentTransport,
    retain: R,
}

struct ObservationReplyBinding<'a> {
    domain: &'static [u8],
    authority: ArtifactChecksumRecord,
    mutation_attempt: u32,
    observation_attempt: u32,
    request: ArtifactChecksumRecord,
    target: &'a str,
}

impl<'a, R> AgentObservationProvider<'a, R> {
    /// Configure exact transport and integration-owned durable retention; no effects occur.
    pub const fn new(transport: &'a AgentTransport, retain: R) -> Self {
        Self { transport, retain }
    }

    fn response_input(
        &self,
        binding: ObservationReplyBinding<'_>,
        id: &RequestId,
        reply: Vec<u8>,
    ) -> IcObservationResponseInput {
        let evidence = crate::reply_evidence::reply_evidence(
            binding.domain,
            &binding.authority,
            &[binding.mutation_attempt, binding.observation_attempt],
            id,
            &reply,
        );
        IcObservationResponseInput {
            authority: binding.authority,
            mutation_attempt: binding.mutation_attempt,
            observation_attempt: binding.observation_attempt,
            request: binding.request,
            context: self.transport.context.clone(),
            target: binding.target.to_owned(),
            reply,
            evidence,
        }
    }
}

impl<R> AgentObservationProvider<'_, R>
where
    R: FnMut(&ReservedUpdate<'_>, &PreparedUpdate<'_>) -> Result<(), IcObservationProviderError>,
{
    async fn submit_once(
        &mut self,
        request: ReservedUpdate<'_>,
        journal: &AttemptJournalRecord,
    ) -> Result<(RequestId, Vec<u8>), IcObservationProviderError> {
        let prepared = self
            .transport
            .prepare(request, journal)
            .map_err(|_| IcObservationProviderError::Unsupported)?;
        (self.retain)(&prepared.request, &prepared)?;
        let outcome = prepared
            .submit()
            .await
            .map_err(|_| IcObservationProviderError::Indeterminate)?;
        match outcome {
            UpdateOutcome::Replied { request_id, reply } => Ok((request_id, reply)),
            UpdateOutcome::Pending { .. } => Err(IcObservationProviderError::Indeterminate),
        }
    }
}

impl<R> IcObservationProvider for AgentObservationProvider<'_, R>
where
    R: FnMut(&ReservedUpdate<'_>, &PreparedUpdate<'_>) -> Result<(), IcObservationProviderError>,
{
    async fn observe(
        &mut self,
        request: &IcObservationRequest<'_>,
        journal: &AttemptJournalRecord,
    ) -> Result<IcObservationResponse, IcObservationProviderError> {
        let (id, reply) = self
            .submit_once(ReservedUpdate::Observation(request), journal)
            .await?;
        let input = self.response_input(
            ObservationReplyBinding {
                domain: b"ic-backup-agent.observation-reply.v1\0",
                authority: request.authority().digest(),
                mutation_attempt: request.mutation_attempt(),
                observation_attempt: request.observation_attempt(),
                request: request.payload().digest(),
                target: request.payload().target(),
            },
            &id,
            reply,
        );
        IcObservationResponse::new(input).map_err(|_| IcObservationProviderError::Indeterminate)
    }
}

impl<R> IcSnapshotUploadObservationProvider for AgentObservationProvider<'_, R>
where
    R: FnMut(&ReservedUpdate<'_>, &PreparedUpdate<'_>) -> Result<(), IcObservationProviderError>,
{
    async fn observe_upload(
        &mut self,
        request: &IcSnapshotUploadObservationRequest<'_, '_>,
        journal: &AttemptJournalRecord,
    ) -> Result<IcObservationResponse, IcObservationProviderError> {
        let (id, reply) = self
            .submit_once(ReservedUpdate::UploadObservation(request), journal)
            .await?;
        let input = self.response_input(
            ObservationReplyBinding {
                domain: b"ic-backup-agent.upload-observation-reply.v1\0",
                authority: request.authority().digest(),
                mutation_attempt: request.mutation_attempt(),
                observation_attempt: request.observation_attempt(),
                request: request.payload().digest(),
                target: request.payload().target(),
            },
            &id,
            reply,
        );
        IcObservationResponse::new(input).map_err(|_| IcObservationProviderError::Indeterminate)
    }
}

impl<R> IcSnapshotUploadDataObservationProvider for AgentObservationProvider<'_, R>
where
    R: FnMut(&ReservedUpdate<'_>, &PreparedUpdate<'_>) -> Result<(), IcObservationProviderError>,
{
    async fn observe_upload_data(
        &mut self,
        request: &IcSnapshotUploadDataObservationRequest<'_, '_, '_>,
        journal: &AttemptJournalRecord,
    ) -> Result<IcSnapshotUploadDataObservationResponse, IcObservationProviderError> {
        let (id, reply) = self
            .submit_once(ReservedUpdate::UploadDataObservation(request), journal)
            .await?;
        let input = self.response_input(
            ObservationReplyBinding {
                domain: b"ic-backup-agent.upload-data-observation-reply.v1\0",
                authority: request.authority().digest(),
                mutation_attempt: request.mutation_attempt(),
                observation_attempt: request.observation_attempt(),
                request: request.payload().digest(),
                target: request.payload().target(),
            },
            &id,
            reply,
        );
        IcSnapshotUploadDataObservationResponse::new(input)
            .map_err(|_| IcObservationProviderError::Indeterminate)
    }
}
