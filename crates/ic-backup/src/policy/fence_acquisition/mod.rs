//! Pure update acknowledgement association; never effect settlement or dispatch admission.

use crate::model::{
    attempt_journal::AttemptJournalRecord,
    fence_acquisition::{
        FenceAcquisitionAcknowledgement, FenceAcquisitionError, FenceAcquisitionRequest,
    },
};
use thiserror::Error;

/// Match a passive reply to the exact original still pending reservation.
///
/// This authenticates neither network/caller/reply nor whole-unit acquisition or
/// current fence custody. Success does not produce any mutation outcome, write a
/// receipt, invoke a provider, authorize retry or release obligations/references.
/// Actual integrations must separately qualify any direct effect settlement;
/// uncertain effects use the existing reserved reconciliation contract.
/// # Errors
/// Rejects changed/settled/recovering reservations, other authority or attempt.
pub fn validate_acknowledgement<'a>(
    request: &FenceAcquisitionRequest<'_>,
    journal: &AttemptJournalRecord,
    acknowledgement: &'a FenceAcquisitionAcknowledgement,
) -> Result<&'a FenceAcquisitionAcknowledgement, FenceAcquisitionAcknowledgementError> {
    request.validate_journal(journal)?;
    if acknowledgement.authority != request.authority().digest() {
        return Err(FenceAcquisitionAcknowledgementError::AuthorityMismatch);
    }
    if acknowledgement.mutation_attempt != request.mutation_attempt() {
        return Err(FenceAcquisitionAcknowledgementError::AttemptMismatch);
    }
    Ok(acknowledgement)
}

/// Typed passive association denial, without settlement or spending changes.
#[derive(Debug, Error)]
pub enum FenceAcquisitionAcknowledgementError {
    /// Current reservation does not match the original request.
    #[error(transparent)]
    Reservation(#[from] FenceAcquisitionError),
    /// Reply association names another full original authority.
    #[error("fence acquisition acknowledgement authority mismatch")]
    AuthorityMismatch,
    /// Reply association names another mutation attempt.
    #[error("fence acquisition acknowledgement attempt mismatch")]
    AttemptMismatch,
}

#[cfg(test)]
mod tests;
