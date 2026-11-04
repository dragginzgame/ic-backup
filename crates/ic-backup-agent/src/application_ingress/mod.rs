//! Opaque application updates using the existing original-reservation transport.

use crate::{AgentTransport, PreparedUpdate, ReservedUpdate};
use ic_backup::model::{
    application_ingress::{ApplicationMutationRequest, ApplicationObservationRequest},
    artifacts::ArtifactChecksumRecord,
    attempt_journal::AttemptJournalRecord,
};
use ic_platform::agent::RequestId;
use std::fmt;
use thiserror::Error;

/// Redacted transport outcome; no variant grants another dispatch or resets spending.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ApplicationIngressProviderError {
    /// Exact reservation, configured context or signing admission failed before dispatch.
    #[error("application ingress is unsupported")]
    Unsupported,
    /// Integration-owned durable retention refused; original spending remains consumed.
    #[error("application ingress retention is unavailable")]
    Unavailable,
    /// Submission/reply/retention did not establish a retained complete response.
    #[error("application ingress outcome is indeterminate")]
    Indeterminate,
}

/// Exact bounded passive reply, returned only after mandatory durable reply retention.
///
/// Authentication uses the configured trust root. Application attribution, meaning,
/// current authority and any explicit receipt remain integration-owned. No codec,
/// automatic journal settlement or terminal/reference release follows.
pub struct ApplicationIngressResponse {
    request: ArtifactChecksumRecord,
    request_id: RequestId,
    reply: Vec<u8>,
    evidence: ArtifactChecksumRecord,
}
impl ApplicationIngressResponse {
    /// Read the exact original opaque payload commitment.
    #[must_use]
    pub const fn request(&self) -> &ArtifactChecksumRecord {
        &self.request
    }
    /// Read the authenticated original ingress identity.
    #[must_use]
    pub const fn request_id(&self) -> &RequestId {
        &self.request_id
    }
    /// Read exact bounded bytes retained before any cancellable qualification work.
    #[must_use]
    pub fn reply(&self) -> &[u8] {
        &self.reply
    }
    /// Read domain-separated original authority/request/attempt/ingress/reply evidence.
    #[must_use]
    pub const fn evidence(&self) -> &ArtifactChecksumRecord {
        &self.evidence
    }
}
impl fmt::Debug for ApplicationIngressResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ApplicationIngressResponse")
            .field("request", &self.request)
            .field("request_id", &self.request_id)
            .field("reply_bytes", &self.reply.len())
            .field("evidence", &self.evidence)
            .finish_non_exhaustive()
    }
}

/// Exactly one application mutation or separately reserved recovery update.
///
/// The caller must retain original payload bytes, hold its original journal guard
/// throughout await and qualify fresh application authority and never-dispatched
/// custody before calling. Construction or reconstruction of a pending request is
/// never such proof. Mandatory `retain_ingress` must durably retain the signed
/// envelope/request ID under the exact original operation AND reservation kind/
/// attempt, refusing any prior or uncertain dispatch witness before submission.
/// Mandatory `retain_reply` retains exact request ID/bytes before this future returns.
/// Both callbacks are synchronous and must make no remote calls. They have no default.
///
/// The provider reserves/settles nothing, decodes no application protocol and owns
/// no Root-paid effect. Accepted, cancelled, failed or lost responses stay uncertain;
/// there is no retry, polling, envelope import or reissue API. Complete settled
/// replay must use the original coordinator/journals without invoking this provider.
pub struct AgentApplicationIngressProvider<'a, I, R> {
    transport: &'a AgentTransport,
    retain_ingress: I,
    retain_reply: R,
}
// The public entrypoints select exactly one original application reservation lane.
enum ApplicationRequest<'a> {
    Mutation(&'a ApplicationMutationRequest<'a>),
    Observation(&'a ApplicationObservationRequest<'a>),
}

struct ApplicationReplyBinding {
    authority: ArtifactChecksumRecord,
    payload: ArtifactChecksumRecord,
    attempts: [u32; 2],
    domain: &'static [u8],
}

impl<'a> ApplicationRequest<'a> {
    fn into_submission(self) -> (ReservedUpdate<'a>, ApplicationReplyBinding) {
        match self {
            Self::Mutation(request) => (
                ReservedUpdate::ApplicationMutation(request),
                ApplicationReplyBinding {
                    authority: request.authority().digest(),
                    payload: request.payload().digest(),
                    attempts: [request.mutation_attempt(), 0],
                    domain: b"ic-backup-agent.application-mutation-reply.v1\0",
                },
            ),
            Self::Observation(request) => (
                ReservedUpdate::ApplicationObservation(request),
                ApplicationReplyBinding {
                    authority: request.authority().digest(),
                    payload: request.payload().digest(),
                    attempts: [request.mutation_attempt(), request.observation_attempt()],
                    domain: b"ic-backup-agent.application-observation-reply.v1\0",
                },
            ),
        }
    }
}

impl<'a, I, R> AgentApplicationIngressProvider<'a, I, R> {
    /// Bind exact configured transport and both mandatory durable retainers without effects.
    pub const fn new(transport: &'a AgentTransport, retain_ingress: I, retain_reply: R) -> Self {
        Self {
            transport,
            retain_ingress,
            retain_reply,
        }
    }
}
impl<I, R> AgentApplicationIngressProvider<'_, I, R>
where
    I: FnMut(
        &ReservedUpdate<'_>,
        &PreparedUpdate<'_>,
    ) -> Result<(), ApplicationIngressProviderError>,
    R: FnMut(&ReservedUpdate<'_>, &RequestId, &[u8]) -> Result<(), ApplicationIngressProviderError>,
{
    /// Submit one original already reserved mutation after fresh caller-owned admission.
    /// # Errors
    /// Rejects changed originals or retention; uncertain outcomes retain pending spending.
    pub async fn submit_mutation(
        &mut self,
        request: &ApplicationMutationRequest<'_>,
        journal: &AttemptJournalRecord,
    ) -> Result<ApplicationIngressResponse, ApplicationIngressProviderError> {
        self.submit(ApplicationRequest::Mutation(request), journal)
            .await
    }
    /// Submit one independently reserved recovery observation through update ingress.
    /// # Errors
    /// Rejects absent/different observations; no mutation allowance or fence is inferred.
    pub async fn submit_observation(
        &mut self,
        request: &ApplicationObservationRequest<'_>,
        journal: &AttemptJournalRecord,
    ) -> Result<ApplicationIngressResponse, ApplicationIngressProviderError> {
        self.submit(ApplicationRequest::Observation(request), journal)
            .await
    }
    async fn submit(
        &mut self,
        request: ApplicationRequest<'_>,
        journal: &AttemptJournalRecord,
    ) -> Result<ApplicationIngressResponse, ApplicationIngressProviderError> {
        let (request, binding) = request.into_submission();
        let commitment = ArtifactChecksumRecord::from_bytes(
            &[
                binding.authority.hash().as_bytes(),
                binding.payload.hash().as_bytes(),
            ]
            .concat(),
        );
        let (request_id, reply) = crate::submission::submit_once(
            self.transport,
            request,
            journal,
            |prepared| (self.retain_ingress)(&prepared.request, prepared),
            |request, id, reply| (self.retain_reply)(request, id, reply),
        )
        .await?;
        let evidence = crate::reply_evidence::reply_evidence(
            binding.domain,
            &commitment,
            &binding.attempts,
            &request_id,
            &reply,
        );
        Ok(ApplicationIngressResponse {
            request: binding.payload,
            request_id,
            reply,
            evidence,
        })
    }
}

impl From<crate::submission::SubmissionFailure> for ApplicationIngressProviderError {
    fn from(error: crate::submission::SubmissionFailure) -> Self {
        match error {
            crate::submission::SubmissionFailure::Unsupported => Self::Unsupported,
            crate::submission::SubmissionFailure::Indeterminate => Self::Indeterminate,
        }
    }
}

#[cfg(test)]
mod tests;
