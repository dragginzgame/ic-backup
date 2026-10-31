//! Three separately accounted async recovery ports over one retained submission owner.
use crate::{AgentTransport, PreparedUpdate, ReservedUpdate};
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

impl<'a> ObservationReplyBinding<'a> {
    fn from_request(request: &ReservedUpdate<'a>) -> Option<Self> {
        Some(match request {
            ReservedUpdate::Observation(request) => Self {
                domain: b"ic-backup-agent.observation-reply.v1\0",
                authority: request.authority().digest(),
                mutation_attempt: request.mutation_attempt(),
                observation_attempt: request.observation_attempt(),
                request: request.payload().digest(),
                target: request.payload().target(),
            },
            ReservedUpdate::UploadObservation(request) => Self {
                domain: b"ic-backup-agent.upload-observation-reply.v1\0",
                authority: request.authority().digest(),
                mutation_attempt: request.mutation_attempt(),
                observation_attempt: request.observation_attempt(),
                request: request.payload().digest(),
                target: request.payload().target(),
            },
            ReservedUpdate::UploadDataObservation(request) => Self {
                domain: b"ic-backup-agent.upload-data-observation-reply.v1\0",
                authority: request.authority().digest(),
                mutation_attempt: request.mutation_attempt(),
                observation_attempt: request.observation_attempt(),
                request: request.payload().digest(),
                target: request.payload().target(),
            },
            ReservedUpdate::ApplicationMutation(_)
            | ReservedUpdate::ApplicationObservation(_)
            | ReservedUpdate::Mutation(_)
            | ReservedUpdate::TransferRead(_)
            | ReservedUpdate::Upload(_)
            | ReservedUpdate::FenceAcquisition(_) => return None,
        })
    }
}

impl<'a, R> AgentObservationProvider<'a, R> {
    /// Configure exact transport and integration-owned durable retention; no effects occur.
    pub const fn new(transport: &'a AgentTransport, retain: R) -> Self {
        Self { transport, retain }
    }
}

impl<R> AgentObservationProvider<'_, R>
where
    R: FnMut(&ReservedUpdate<'_>, &PreparedUpdate<'_>) -> Result<(), IcObservationProviderError>,
{
    async fn submit_response(
        &mut self,
        request: ReservedUpdate<'_>,
        journal: &AttemptJournalRecord,
    ) -> Result<IcObservationResponseInput, IcObservationProviderError> {
        let binding = ObservationReplyBinding::from_request(&request)
            .ok_or(IcObservationProviderError::Unsupported)?;
        let (id, reply) = crate::submission::submit_once(
            self.transport,
            request,
            journal,
            |prepared| (self.retain)(&prepared.request, prepared),
            |_, _, _| Ok(()),
        )
        .await?;
        let evidence = crate::reply_evidence::reply_evidence(
            binding.domain,
            &binding.authority,
            &[binding.mutation_attempt, binding.observation_attempt],
            &id,
            &reply,
        );
        Ok(IcObservationResponseInput {
            authority: binding.authority,
            mutation_attempt: binding.mutation_attempt,
            observation_attempt: binding.observation_attempt,
            request: binding.request,
            context: self.transport.context.clone(),
            target: binding.target.to_owned(),
            reply,
            evidence,
        })
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
        let input = self
            .submit_response(ReservedUpdate::Observation(request), journal)
            .await?;
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
        let input = self
            .submit_response(ReservedUpdate::UploadObservation(request), journal)
            .await?;
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
        let input = self
            .submit_response(ReservedUpdate::UploadDataObservation(request), journal)
            .await?;
        IcSnapshotUploadDataObservationResponse::new(input)
            .map_err(|_| IcObservationProviderError::Indeterminate)
    }
}
