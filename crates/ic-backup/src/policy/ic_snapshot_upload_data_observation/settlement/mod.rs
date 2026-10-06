//! Local admission of already qualified original-write settlement claims.

use super::{
    IcSnapshotUploadDataObservationAssociationError, IcSnapshotUploadDataObservationView,
    validate_response,
};
use crate::model::{
    artifacts::ArtifactChecksumRecord,
    attempt_journal::{AttemptJournalRecord, ObservationOutcomeRecord},
    ic_snapshot_upload_data_observation::{
        IcSnapshotUploadDataAttribution, IcSnapshotUploadDataObservationRequest,
        IcSnapshotUploadDataObservationResponse, IcSnapshotUploadDataSettlement,
    },
};
use thiserror::Error;

/// Matched passive attribution and exact readback; never an authenticated receipt.
#[derive(Debug)]
pub struct IcSnapshotUploadDataSettlementView<'request, 'metadata> {
    observation: IcSnapshotUploadDataObservationView<'request, 'metadata>,
    settlement: &'request IcSnapshotUploadDataSettlement,
    outcome: ObservationOutcomeRecord,
}
impl<'request, 'metadata> IcSnapshotUploadDataSettlementView<'request, 'metadata> {
    /// Read the passive claimed outcome; only a qualified integration may record it.
    #[must_use]
    pub const fn outcome(&self) -> ObservationOutcomeRecord {
        self.outcome
    }
    /// Read the existing exact bounded readback projection, with no additional decoding owner.
    #[must_use]
    pub const fn observation(&self) -> &IcSnapshotUploadDataObservationView<'request, 'metadata> {
        &self.observation
    }
    /// Read retained attribution and complete settlement evidence, without copying counters.
    #[must_use]
    pub const fn settlement(&self) -> &'request IcSnapshotUploadDataSettlement {
        self.settlement
    }
}

/// Match original reservations, fresh challenge and exact independently qualified read evidence.
///
/// Applied requires matching original bytes plus exclusive original-write attribution;
/// byte equality alone supplies no claim. `NotApplied` requires qualified exclusion,
/// even when initialized/preexisting bytes match. Unresolved requires an actually
/// settled authenticated read; lost replies cannot enter this successful-read boundary.
///
/// Authentication, original allocation/write attribution, challenge freshness and
/// stable custody remain integration-owned. This performs no IO, remote call, receipt
/// transition, retry, refund or load/start/terminal/reference/fence-release admission.
/// # Errors
/// Rejects current reservation/claim/codec drift, changed settlement identity or evidence,
/// stale challenge, and an Applied claim whose current bytes differ from original intent.
pub fn validate_settlement<'request, 'metadata>(
    request: &IcSnapshotUploadDataObservationRequest<'request, '_, 'metadata>,
    journal: &AttemptJournalRecord,
    response: &'request IcSnapshotUploadDataObservationResponse,
    challenge: &ArtifactChecksumRecord,
    settlement: &'request IcSnapshotUploadDataSettlement,
) -> Result<
    IcSnapshotUploadDataSettlementView<'request, 'metadata>,
    IcSnapshotUploadDataSettlementError,
> {
    let observation = validate_response(request, journal, response)?;
    if settlement.authority != request.authority().digest() {
        return Err(IcSnapshotUploadDataSettlementError::AuthorityMismatch);
    }
    if settlement.mutation_attempt != request.mutation_attempt()
        || settlement.observation_attempt != request.observation_attempt()
    {
        return Err(IcSnapshotUploadDataSettlementError::AttemptMismatch);
    }
    if &settlement.challenge != challenge {
        return Err(IcSnapshotUploadDataSettlementError::ChallengeMismatch);
    }
    if settlement.readback != observation.reply().digest() {
        return Err(IcSnapshotUploadDataSettlementError::ReadbackMismatch);
    }
    if settlement.observation_evidence != response.input().evidence {
        return Err(IcSnapshotUploadDataSettlementError::ObservationEvidenceMismatch);
    }
    let outcome = match &settlement.attribution {
        IcSnapshotUploadDataAttribution::Applied { .. } => {
            if !observation.matches_original_chunk() {
                return Err(IcSnapshotUploadDataSettlementError::AppliedBytesMismatch);
            }
            ObservationOutcomeRecord::Applied
        }
        IcSnapshotUploadDataAttribution::NotApplied { .. } => ObservationOutcomeRecord::NotApplied,
        IcSnapshotUploadDataAttribution::Unresolved { .. } => ObservationOutcomeRecord::Uncertain,
    };
    Ok(IcSnapshotUploadDataSettlementView {
        observation,
        settlement,
        outcome,
    })
}

/// Exact local settlement admission denial; all spending and obligations remain retained.
#[derive(Debug, Error)]
pub enum IcSnapshotUploadDataSettlementError {
    /// Existing reservation, passive actual claims or data decoder rejected.
    #[error(transparent)]
    Observation(#[from] IcSnapshotUploadDataObservationAssociationError),
    /// The full original plan/operation/allowances differ.
    #[error("data upload settlement original authority differs")]
    AuthorityMismatch,
    /// Original write or read attempt differs.
    #[error("data upload settlement attempt identities differ")]
    AttemptMismatch,
    /// Qualification challenge differs from the caller's current challenge.
    #[error("data upload settlement challenge differs")]
    ChallengeMismatch,
    /// Exact metadata/request/raw reply digest differs.
    #[error("data upload settlement readback differs")]
    ReadbackMismatch,
    /// Retained opaque observation evidence differs despite potentially equal bytes.
    #[error("data upload settlement observation evidence differs")]
    ObservationEvidenceMismatch,
    /// Applied cannot admit a current extent different from the original encoded write.
    #[error("data upload settlement applied bytes differ from original intent")]
    AppliedBytesMismatch,
}

#[cfg(test)]
mod tests;
