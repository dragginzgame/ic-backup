//! Pure admission of independently qualified original lifecycle settlement.

use super::{
    IcObservationAssociationError, IcObservationReplyView, IcObservationResponseView,
    validate_response,
};
use crate::model::{
    artifacts::ArtifactChecksumRecord,
    attempt_journal::{AttemptJournalRecord, ObservationOutcomeRecord},
    ic_observation::{
        IcLifecycleAttribution, IcLifecycleSettlement, IcObservationRequest, IcObservationResponse,
    },
    ic_request::IcManagementMethodRecord as Method,
};
use thiserror::Error;

/// Matched passive original attribution, never an authenticated receipt.
#[derive(Debug)]
pub struct IcLifecycleSettlementView<'a> {
    observation: IcObservationResponseView<'a>,
    settlement: &'a IcLifecycleSettlement,
    outcome: ObservationOutcomeRecord,
}
impl<'a> IcLifecycleSettlementView<'a> {
    /// Read the independently claimed outcome; integration qualification precedes recording.
    #[must_use]
    pub const fn outcome(&self) -> ObservationOutcomeRecord {
        self.outcome
    }
    /// Read exact bounded status association under original reservations.
    #[must_use]
    pub const fn observation(&self) -> &IcObservationResponseView<'a> {
        &self.observation
    }
    /// Read original attribution and retained qualification evidence.
    #[must_use]
    pub const fn settlement(&self) -> &'a IcLifecycleSettlement {
        self.settlement
    }
}

/// Bind independently qualified stop/start/load settlement to the original status evidence.
///
/// Reuses current reservation/actual claim checks and the bounded status decoder.
/// Current status, controllers and module hashes never infer any outcome. Applied
/// load claims independently qualify exact restored state; desired status does not
/// identify the original request. Opposite state cannot exclude transient application.
/// Lost replies cannot enter this successful-observation boundary as uncertainty.
///
/// This matches passive claims only. Authentication, chronology, attribution,
/// freshness, read permission and command custody remain integration-owned. Only
/// that integration explicitly records the existing journal receipt. No IO, calls,
/// automatic transitions, retries, refunds, restart or terminal/release admission occur.
/// # Errors
/// Rejects capture/inventory lanes, changed reservations/actual claims, invalid wire,
/// different authority/attempts, challenge or exact retained status/evidence identity.
pub fn validate_lifecycle_settlement<'a>(
    request: &IcObservationRequest<'a>,
    journal: &AttemptJournalRecord,
    response: &'a IcObservationResponse,
    challenge: &ArtifactChecksumRecord,
    settlement: &'a IcLifecycleSettlement,
) -> Result<IcLifecycleSettlementView<'a>, IcLifecycleSettlementError> {
    if !matches!(
        request.mutation().method(),
        Method::StopCanister | Method::StartCanister | Method::LoadCanisterSnapshot
    ) || request.payload().method() != Method::CanisterStatus
    {
        return Err(IcLifecycleSettlementError::UnsupportedMethod);
    }
    let observation = validate_response(request, journal, response)?;
    if settlement.authority != request.authority().digest() {
        return Err(IcLifecycleSettlementError::AuthorityMismatch);
    }
    if settlement.mutation_attempt != request.mutation_attempt()
        || settlement.observation_attempt != request.observation_attempt()
    {
        return Err(IcLifecycleSettlementError::AttemptMismatch);
    }
    if &settlement.challenge != challenge {
        return Err(IcLifecycleSettlementError::ChallengeMismatch);
    }
    let IcObservationReplyView::Status(status) = observation.reply() else {
        return Err(IcLifecycleSettlementError::UnsupportedMethod);
    };
    if settlement.status != status.digest() {
        return Err(IcLifecycleSettlementError::StatusMismatch);
    }
    if settlement.observation_evidence != response.input().evidence {
        return Err(IcLifecycleSettlementError::ObservationEvidenceMismatch);
    }
    let outcome = match &settlement.attribution {
        IcLifecycleAttribution::Applied { .. } => ObservationOutcomeRecord::Applied,
        IcLifecycleAttribution::NotApplied { .. } => ObservationOutcomeRecord::NotApplied,
        IcLifecycleAttribution::Unresolved { .. } => ObservationOutcomeRecord::Uncertain,
    };
    Ok(IcLifecycleSettlementView {
        observation,
        settlement,
        outcome,
    })
}

/// Typed local claim denial; original allowances, obligations and references remain retained.
#[derive(Debug, Error)]
pub enum IcLifecycleSettlementError {
    /// Only original stop/start/load with a separately reserved status call is supported.
    #[error("lifecycle settlement requires original stop/start/load and status observation")]
    UnsupportedMethod,
    /// Existing association, current reservation or bounded decoder rejected.
    #[error(transparent)]
    Observation(#[from] IcObservationAssociationError),
    /// Complete original operation authority differs.
    #[error("lifecycle settlement original authority differs")]
    AuthorityMismatch,
    /// Original mutation or observation attempt differs.
    #[error("lifecycle settlement attempt identities differ")]
    AttemptMismatch,
    /// Caller-owned current qualification challenge differs.
    #[error("lifecycle settlement challenge differs")]
    ChallengeMismatch,
    /// Exact original status request/raw reply digest differs.
    #[error("lifecycle settlement status evidence differs")]
    StatusMismatch,
    /// Retained opaque observation evidence differs even if reply bytes match.
    #[error("lifecycle settlement observation evidence differs")]
    ObservationEvidenceMismatch,
}

#[cfg(test)]
mod tests;
