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

#[derive(Debug)]
pub(crate) enum MutationReservationError {
    NoPendingMutation,
    AuthorityMismatch,
    MutationMismatch,
    ObservationPending,
}
