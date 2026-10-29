//! One retained submission and passive acknowledgement for mutation and upload lanes.

use crate::{AgentTransport, PreparedUpdate, ReservedUpdate};
use ic_backup::model::ic_update_response::IcUpdateResponseInput;
use ic_backup::{
    model::{
        artifacts::ArtifactChecksumRecord, attempt_journal::AttemptJournalRecord,
        ic_mutation::IcMutationAcknowledgement,
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
    let (request_id, reply) = crate::submission::submit_once(
        transport,
        request,
        journal,
        |prepared| retain(prepared),
        |_, _, _| Ok(()),
        || IcMutationProviderError::Unsupported,
        || IcMutationProviderError::Indeterminate,
    )
    .await?;
    let evidence = crate::reply_evidence::reply_evidence(
        identity.domain,
        identity.authority,
        &[identity.mutation_attempt],
        &request_id,
        &reply,
    );
    IcMutationAcknowledgement::new(IcUpdateResponseInput {
        authority: identity.authority.clone(),
        mutation_attempt: identity.mutation_attempt,
        context: transport.context.clone(),
        target: identity.target.into(),
        reply,
        evidence,
    })
    .map_err(|_| IcMutationProviderError::Indeterminate)
}
