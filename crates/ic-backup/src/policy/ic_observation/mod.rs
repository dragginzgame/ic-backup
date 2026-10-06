//! Pure exact reserved IC observation association; no mutation settlement.

use crate::model::{
    artifacts::ArtifactChecksumRecord,
    attempt_journal::{AttemptAuthorityRecord, AttemptJournalRecord},
    ic_lifecycle_reply::{IcLifecycleReply, IcLifecycleReplyError},
    ic_observation::{
        IcObservationRequest, IcObservationRequestError, IcObservationResponse,
        IcObservationResponseInput,
    },
    ic_request::{IcManagementMethodRecord, IcManagementRequestRecord},
    ic_snapshot_reply::{IcSnapshotReply, IcSnapshotReplyError},
    operation_plan::PlanContextRecord,
};
use thiserror::Error;

/// Existing method-specific wire projections; no fresh permission or effect attribution.
#[derive(Debug)]
pub enum IcObservationReplyView<'a> {
    /// Canonical bounded snapshot inventory with unchanged raw identities/metadata.
    Inventory(IcSnapshotReply<'a>),
    /// Required status/controller projection; Stopped alone proves no drain or load outcome.
    Status(IcLifecycleReply<'a>),
}

/// Read-only association to exact original reserved observation evidence.
#[derive(Debug)]
pub struct IcObservationResponseView<'a> {
    response: &'a IcObservationResponse,
    reply: IcObservationReplyView<'a>,
}
impl<'a> IcObservationResponseView<'a> {
    /// Read immutable original claims and exact raw response evidence.
    #[must_use]
    pub const fn response(&self) -> &'a IcObservationResponse {
        self.response
    }
    /// Read the existing decoder's bounded method-specific projection.
    #[must_use]
    pub const fn reply(&self) -> &IcObservationReplyView<'a> {
        &self.reply
    }
}

/// Match current original reservations and actual claims, then reuse existing decoders.
///
/// This performs no IO, dispatch or receipt transition. Successful wire association
/// proves no actual authentication, chronology, freshness, exclusive capture attribution,
/// lifecycle equivalence, load success, application safety or permission. Zero/one/many
/// snapshots and Stopped/controller projections never automatically settle a mutation.
/// Lost replies retain the pending observation; they cannot mean settled Uncertain.
/// # Errors
/// Rejects changed reservations/authority/attempts/bytes/context/target and invalid wire.
pub fn validate_response<'a>(
    request: &IcObservationRequest<'a>,
    journal: &AttemptJournalRecord,
    response: &'a IcObservationResponse,
) -> Result<IcObservationResponseView<'a>, IcObservationAssociationError> {
    request.validate_journal(journal)?;
    validate_association(
        &ObservationAssociation {
            authority: request.authority(),
            mutation_attempt: request.mutation_attempt(),
            observation_attempt: request.observation_attempt(),
            payload: request.payload(),
            context: request.plan().context(),
        },
        response,
    )
}

pub(crate) struct ObservationAssociation<'binding, 'payload> {
    pub authority: &'binding AttemptAuthorityRecord,
    pub mutation_attempt: u32,
    pub observation_attempt: u32,
    pub payload: &'payload IcManagementRequestRecord,
    pub context: &'binding PlanContextRecord,
}

pub(crate) fn validate_association<'a>(
    binding: &ObservationAssociation<'_, 'a>,
    response: &'a IcObservationResponse,
) -> Result<IcObservationResponseView<'a>, IcObservationAssociationError> {
    validate_claims(
        &ObservationClaims {
            authority: binding.authority,
            mutation_attempt: binding.mutation_attempt,
            observation_attempt: binding.observation_attempt,
            request: binding.payload.digest(),
            context: binding.context,
            target: binding.payload.target(),
        },
        response.input(),
    )?;
    let reply = match binding.payload.method() {
        IcManagementMethodRecord::ListCanisterSnapshots => IcObservationReplyView::Inventory(
            IcSnapshotReply::decode(binding.payload, &response.input().reply)?,
        ),
        // The sealed request excludes mutations. Status stays with its existing owner.
        _ => IcObservationReplyView::Status(IcLifecycleReply::decode(
            binding.payload,
            &response.input().reply,
        )?),
    };
    Ok(IcObservationResponseView { response, reply })
}

pub(crate) struct ObservationClaims<'a> {
    pub authority: &'a AttemptAuthorityRecord,
    pub mutation_attempt: u32,
    pub observation_attempt: u32,
    pub request: ArtifactChecksumRecord,
    pub context: &'a PlanContextRecord,
    pub target: &'a str,
}

pub(crate) fn validate_claims(
    binding: &ObservationClaims<'_>,
    input: &IcObservationResponseInput,
) -> Result<(), IcObservationAssociationError> {
    if input.authority != binding.authority.digest() {
        return Err(IcObservationAssociationError::AuthorityMismatch);
    }
    if input.mutation_attempt != binding.mutation_attempt
        || input.observation_attempt != binding.observation_attempt
    {
        return Err(IcObservationAssociationError::AttemptMismatch);
    }
    if input.request != binding.request {
        return Err(IcObservationAssociationError::RequestMismatch);
    }
    if &input.context != binding.context {
        return Err(IcObservationAssociationError::ContextMismatch);
    }
    if input.target != binding.target {
        return Err(IcObservationAssociationError::TargetMismatch);
    }
    Ok(())
}

/// Typed passive association denial; all original spending/obligations remain retained.
#[derive(Debug, Error)]
pub enum IcObservationAssociationError {
    /// Current original journal no longer matches the request.
    #[error(transparent)]
    Reservation(#[from] IcObservationRequestError),
    /// Another original operation authority was claimed.
    #[error("IC observation response authority mismatch")]
    AuthorityMismatch,
    /// Another original mutation or observation attempt was claimed.
    #[error("IC observation response attempt mismatch")]
    AttemptMismatch,
    /// Another exact observation payload digest was claimed.
    #[error("IC observation response request mismatch")]
    RequestMismatch,
    /// Actual claimed network/caller/release differs.
    #[error("IC observation response context mismatch")]
    ContextMismatch,
    /// Actual claimed response target differs.
    #[error("IC observation response target mismatch")]
    TargetMismatch,
    /// Existing inventory decoder rejected the reply.
    #[error(transparent)]
    Inventory(#[from] IcSnapshotReplyError),
    /// Existing status decoder rejected the reply.
    #[error(transparent)]
    Status(#[from] IcLifecycleReplyError),
}

#[cfg(test)]
mod tests;
