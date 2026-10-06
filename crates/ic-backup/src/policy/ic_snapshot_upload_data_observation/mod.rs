//! Passive exact data-upload readback association; byte equality is no receipt.

use crate::{
    model::{
        attempt_journal::AttemptJournalRecord,
        ic_snapshot_data::{IcSnapshotDataError, IcSnapshotDataReply},
        ic_snapshot_upload_data_observation::{
            IcSnapshotUploadDataObservationRequest, IcSnapshotUploadDataObservationResponse,
        },
    },
    policy::ic_observation::{IcObservationAssociationError, ObservationClaims, validate_claims},
};
use thiserror::Error;

mod settlement;
pub use settlement::{
    IcSnapshotUploadDataSettlementError, IcSnapshotUploadDataSettlementView, validate_settlement,
};

/// Read-only exact data evidence and original-chunk equality; no write outcome.
#[derive(Debug)]
pub struct IcSnapshotUploadDataObservationView<'request, 'metadata> {
    response: &'request IcSnapshotUploadDataObservationResponse,
    reply: IcSnapshotDataReply<'request, 'metadata>,
    matches_original_chunk: bool,
}
impl<'request, 'metadata> IcSnapshotUploadDataObservationView<'request, 'metadata> {
    /// Read original passive claims and raw reply evidence.
    #[must_use]
    pub const fn response(&self) -> &'request IcSnapshotUploadDataObservationResponse {
        self.response
    }
    /// Read the canonical bounded data decoder's exact byte/metadata projection.
    #[must_use]
    pub const fn reply(&self) -> &IcSnapshotDataReply<'request, 'metadata> {
        &self.reply
    }
    /// Compare actual chunk SHA-256 with originally encoded upload bytes.
    ///
    /// Equal bytes may predate this upload or come from another writer. Different
    /// bytes or absent/failed observations cannot alone establish `NotApplied`.
    #[must_use]
    pub const fn matches_original_chunk(&self) -> bool {
        self.matches_original_chunk
    }
}

/// Recheck exact pending reservations and passive claims, then decode exact read bytes.
///
/// Authentication, observation chronology, exclusive write attribution, original
/// allocation and stable destination custody remain integration-owned. This performs
/// no IO, observation, receipt, retry, refund or terminal/reference release.
/// # Errors
/// Rejects current reservation/claim drift and existing bounded data codec failures.
pub fn validate_response<'request, 'metadata>(
    request: &IcSnapshotUploadDataObservationRequest<'request, '_, 'metadata>,
    journal: &AttemptJournalRecord,
    response: &'request IcSnapshotUploadDataObservationResponse,
) -> Result<
    IcSnapshotUploadDataObservationView<'request, 'metadata>,
    IcSnapshotUploadDataObservationAssociationError,
> {
    request
        .validate_journal(journal)
        .map_err(IcObservationAssociationError::from)?;
    validate_claims(
        &ObservationClaims {
            authority: request.authority(),
            mutation_attempt: request.mutation_attempt(),
            observation_attempt: request.observation_attempt(),
            request: request.payload().digest(),
            context: request.plan().context(),
            target: request.payload().target(),
        },
        response.input(),
    )?;
    let reply = IcSnapshotDataReply::decode(request.payload(), &response.input().reply)?;
    let matches_original_chunk = reply.chunk_checksum() == request.original_chunk_checksum();
    Ok(IcSnapshotUploadDataObservationView {
        response,
        reply,
        matches_original_chunk,
    })
}

/// Canonical passive-claim or data-codec denial, with no spending/outcome change.
#[derive(Debug, Error)]
pub enum IcSnapshotUploadDataObservationAssociationError {
    /// Existing current reservation/actual claim association failed.
    #[error(transparent)]
    Association(#[from] IcObservationAssociationError),
    /// Existing exact data reply decoding failed.
    #[error(transparent)]
    Data(#[from] IcSnapshotDataError),
}

#[cfg(test)]
mod tests;
