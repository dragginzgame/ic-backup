//! Passive original pending mutation identity, without spending or dispatch authority.
use super::{AttemptAuthorityRecord, AttemptJournalRecord};

#[derive(Debug)]
pub(crate) struct MutationReservation {
    pub authority: AttemptAuthorityRecord,
    pub mutation_attempt: u32,
}
impl MutationReservation {
    pub(crate) fn new(
        authority: AttemptAuthorityRecord,
        journal: &AttemptJournalRecord,
    ) -> Result<Self, MutationReservationError> {
        let mutation_attempt = journal
            .pending_mutation()
            .ok_or(MutationReservationError::NoPendingMutation)?;
        let reservation = Self {
            authority,
            mutation_attempt,
        };
        reservation.validate(journal)?;
        Ok(reservation)
    }
    pub(crate) fn validate(
        &self,
        journal: &AttemptJournalRecord,
    ) -> Result<(), MutationReservationError> {
        journal.validate_pending_mutation(&self.authority, self.mutation_attempt)
    }
}

/// Exact original pending-mutation refusal; no variant spends or authorizes a call.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum MutationReservationError {
    /// No original unresolved mutation is retained.
    #[error("original mutation requires a pending reservation")]
    NoPendingMutation,
    /// Full original authority, including immutable allowances, differs.
    #[error("original mutation reservation authority mismatch")]
    AuthorityMismatch,
    /// The original pending mutation changed or settled after binding.
    #[error("original mutation reservation changed or settled")]
    MutationMismatch,
    /// Separately accounted observation recovery is pending.
    #[error("original mutation reservation has a pending observation")]
    ObservationPending,
}
