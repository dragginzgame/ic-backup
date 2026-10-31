//! Exact application update over the existing single-submit transport.
use crate::{AgentTransport, PreparedUpdate, ReservedUpdate};
use ic_backup::{
    model::{
        attempt_journal::AttemptJournalRecord,
        fence_acquisition::{FenceAcquisitionAcknowledgement, FenceAcquisitionRequest},
    },
    ports::fence_acquisition::{FenceAcquisitionProvider, FenceAcquisitionProviderError},
};
use ic_platform::agent::RequestId;

/// One original application acquisition update with mandatory durable ingress/reply retention.
///
/// `retain_ingress` durably retains exact signed bytes/request ID under the original
/// plan, obligation and attempt before submission, and refuses prior dispatch.
/// `retain_reply` durably retains the exact bounded authenticated reply/request ID
/// before returning passive association. Both callbacks make no remote calls.
/// Neither callback has a default, and neither acknowledgement nor certificate
/// verification qualifies whole-unit acquisition or Active fence custody.
///
/// The caller holds the original journal guard through await and owns fresh permissions,
/// application codecs/purpose/revisions, complete membership and dependencies, exclusive
/// never-dispatched custody and restore's outside-snapshot obligations. Accepted, lost,
/// cancelled or rejected replies preserve pending spending and grant no repeated update.
pub struct AgentFenceAcquisitionProvider<'a, I, R> {
    transport: &'a AgentTransport,
    retain_ingress: I,
    retain_reply: R,
}

impl<'a, I, R> AgentFenceAcquisitionProvider<'a, I, R> {
    /// Configure exact transport and both integration-owned durable retainers without effects.
    pub const fn new(transport: &'a AgentTransport, retain_ingress: I, retain_reply: R) -> Self {
        Self {
            transport,
            retain_ingress,
            retain_reply,
        }
    }
}

impl<I, R> FenceAcquisitionProvider for AgentFenceAcquisitionProvider<'_, I, R>
where
    I: FnMut(
        &FenceAcquisitionRequest<'_>,
        &PreparedUpdate<'_>,
    ) -> Result<(), FenceAcquisitionProviderError>,
    R: FnMut(
        &FenceAcquisitionRequest<'_>,
        &RequestId,
        &[u8],
    ) -> Result<(), FenceAcquisitionProviderError>,
{
    async fn acquire_fence(
        &mut self,
        request: &FenceAcquisitionRequest<'_>,
        journal: &AttemptJournalRecord,
    ) -> Result<FenceAcquisitionAcknowledgement, FenceAcquisitionProviderError> {
        let (request_id, reply) = crate::submission::submit_once(
            self.transport,
            ReservedUpdate::FenceAcquisition(request),
            journal,
            |prepared| (self.retain_ingress)(request, prepared),
            |_, request_id, reply| (self.retain_reply)(request, request_id, reply),
        )
        .await?;
        let evidence = crate::reply_evidence::reply_evidence(
            b"ic-backup-agent.fence-acquisition-reply.v1\0",
            &request.authority().digest(),
            &[request.mutation_attempt()],
            &request_id,
            &reply,
        );
        FenceAcquisitionAcknowledgement::new(
            request.authority().digest(),
            request.mutation_attempt(),
            evidence,
        )
        .map_err(|_| FenceAcquisitionProviderError::Indeterminate)
    }
}

impl From<crate::submission::SubmissionFailure> for FenceAcquisitionProviderError {
    fn from(error: crate::submission::SubmissionFailure) -> Self {
        match error {
            crate::submission::SubmissionFailure::Unsupported => Self::Unsupported,
            crate::submission::SubmissionFailure::Indeterminate => Self::Indeterminate,
        }
    }
}
