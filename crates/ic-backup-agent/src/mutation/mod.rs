//! Async mutation port over the existing exact single-update transport.

use crate::{AgentTransport, PreparedUpdate, ReservedUpdate, UpdateOutcome};
use ic_backup::{
    model::{
        attempt_journal::AttemptJournalRecord,
        ic_mutation::{
            IcMutationAcknowledgement, IcMutationAcknowledgementInput, IcMutationRequest,
        },
    },
    ports::ic_mutation::{IcMutationProvider, IcMutationProviderError},
};

/// Single-update Agent provider with mandatory durable signed-ingress retention.
///
/// `retain` must durably retain the exact envelope and request ID under the original
/// plan/attempt and exclusive never-dispatched command custody before returning.
/// Failure stops before network submission. It must perform no remote calls or
/// retries; no default retention callback exists. The original selected journal
/// remains borrowed under its guard across signing, retention and async submission.
///
/// Fresh authority, application safety and prior accounting belong to the calling
/// coordinator's mandatory admission. Certificate verification uses the configured
/// trusted root; it does not authenticate declared network/release or prove whole
/// application safety. Returned acknowledgements remain passive and pending until
/// independently qualified. Accepted, cancelled or failed calls grant no reissue.
pub struct AgentMutationProvider<'a, R> {
    transport: &'a AgentTransport,
    retain: R,
}

impl<'a, R> AgentMutationProvider<'a, R> {
    /// Bind the fixed transport and integration-owned durable retention callback.
    /// Construction performs no signing, IO, spending or fresh admission.
    pub const fn new(transport: &'a AgentTransport, retain: R) -> Self {
        Self { transport, retain }
    }
}

impl<R> IcMutationProvider for AgentMutationProvider<'_, R>
where
    R: FnMut(&IcMutationRequest<'_>, &PreparedUpdate<'_>) -> Result<(), IcMutationProviderError>,
{
    async fn submit_mutation(
        &mut self,
        request: &IcMutationRequest<'_>,
        journal: &AttemptJournalRecord,
    ) -> Result<IcMutationAcknowledgement, IcMutationProviderError> {
        let prepared = self
            .transport
            .prepare(ReservedUpdate::Mutation(request), journal)
            .map_err(|_| IcMutationProviderError::Unsupported)?;
        (self.retain)(request, &prepared)?;
        let outcome = prepared
            .submit()
            .await
            .map_err(|_| IcMutationProviderError::Indeterminate)?;
        let UpdateOutcome::Replied { request_id, reply } = outcome else {
            return Err(IcMutationProviderError::Indeterminate);
        };
        // Fixed-size passive evidence binds original authority, ingress and raw reply.
        // It is not an Applied receipt or independent application attribution.
        let evidence = crate::reply_evidence::reply_evidence(
            b"ic-backup-agent.mutation-reply.v1\0",
            &request.authority().digest(),
            &[request.mutation_attempt()],
            &request_id,
            &reply,
        );
        IcMutationAcknowledgement::new(IcMutationAcknowledgementInput {
            authority: request.authority().digest(),
            mutation_attempt: request.mutation_attempt(),
            context: self.transport.context.clone(),
            target: request.payload().target().into(),
            reply,
            evidence,
        })
        .map_err(|_| IcMutationProviderError::Indeterminate)
    }
}
