//! One retained submission and passive acknowledgement for mutation and upload lanes.

use crate::{AgentTransport, PreparedUpdate, ReservedUpdate, UpdateOutcome};
use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::AttemptJournalRecord,
        ic_mutation::{IcMutationAcknowledgement, IcMutationAcknowledgementInput},
    },
    ports::ic_mutation::IcMutationProviderError,
};

pub(crate) struct AcknowledgementIdentity<'a> {
    pub domain: &'static [u8],
    pub authority: &'a ArtifactChecksumRecord,
    pub mutation_attempt: u32,
    pub target: &'a str,
}

pub(crate) async fn submit_acknowledged<'a>(
    transport: &'a AgentTransport,
    request: ReservedUpdate<'a>,
    journal: &'a AttemptJournalRecord,
    mut retain: impl FnMut(&PreparedUpdate<'_>) -> Result<(), IcMutationProviderError>,
    identity: AcknowledgementIdentity<'_>,
) -> Result<IcMutationAcknowledgement, IcMutationProviderError> {
    let prepared = transport
        .prepare(request, journal)
        .map_err(|_| IcMutationProviderError::Unsupported)?;
    retain(&prepared)?;
    let outcome = prepared
        .submit()
        .await
        .map_err(|_| IcMutationProviderError::Indeterminate)?;
    let UpdateOutcome::Replied { request_id, reply } = outcome else {
        return Err(IcMutationProviderError::Indeterminate);
    };
    let evidence = crate::reply_evidence::reply_evidence(
        identity.domain,
        identity.authority,
        &[identity.mutation_attempt],
        &request_id,
        &reply,
    );
    IcMutationAcknowledgement::new(IcMutationAcknowledgementInput {
        authority: identity.authority.clone(),
        mutation_attempt: identity.mutation_attempt,
        context: transport.context.clone(),
        target: identity.target.into(),
        reply,
        evidence,
    })
    .map_err(|_| IcMutationProviderError::Indeterminate)
}
