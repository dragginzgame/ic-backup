//! Passive original pending mutation identity, without spending or dispatch authority.
use super::{AttemptAuthorityRecord, AttemptJournalRecord};

#[derive(Debug)]
pub(crate) struct MutationReservation {
    pub authority: AttemptAuthorityRecord,
    pub mutation_attempt: u32,
}
impl MutationReservation {
    pub(crate) fn new<E>(
        authority: AttemptAuthorityRecord,
        journal: &AttemptJournalRecord,
        missing: E,
        authority_mismatch: E,
        mutation_mismatch: E,
        observation_pending: E,
    ) -> Result<Self, E> {
        let mutation_attempt = journal.pending_mutation().ok_or(missing)?;
        let reservation = Self {
            authority,
            mutation_attempt,
        };
        reservation.validate(
            journal,
            authority_mismatch,
            mutation_mismatch,
            observation_pending,
        )?;
        Ok(reservation)
    }
    pub(crate) fn validate<E>(
        &self,
        journal: &AttemptJournalRecord,
        authority_mismatch: E,
        mutation_mismatch: E,
        observation_pending: E,
    ) -> Result<(), E> {
        journal.validate_pending_mutation(
            &self.authority,
            self.mutation_attempt,
            authority_mismatch,
            mutation_mismatch,
            observation_pending,
        )
    }
}
