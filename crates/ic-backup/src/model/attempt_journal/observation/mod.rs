//! Passive pending-observation identity owned by original attempt accounting.
use super::{AttemptAuthorityRecord, AttemptJournalRecord};
use crate::model::artifacts::ArtifactChecksumRecord;

#[derive(Debug)]
pub(crate) struct ObservationReservation {
    pub authority: AttemptAuthorityRecord,
    pub mutation_attempt: u32,
    pub observation_attempt: u32,
    pub request: ArtifactChecksumRecord,
}

/// Read only the already consumed original attempt IDs, in mutation-first order.
fn pending_attempts(
    journal: &AttemptJournalRecord,
) -> Result<(u32, u32), ObservationReservationError> {
    Ok((
        journal
            .pending_mutation()
            .ok_or(ObservationReservationError::NoPendingMutation)?,
        journal
            .pending_observation()
            .ok_or(ObservationReservationError::NoPendingObservation)?,
    ))
}

impl ObservationReservation {
    pub(crate) fn new(
        authority: AttemptAuthorityRecord,
        journal: &AttemptJournalRecord,
        request: ArtifactChecksumRecord,
    ) -> Result<Self, ObservationReservationError> {
        let (mutation_attempt, observation_attempt) = pending_attempts(journal)?;
        let reservation = Self {
            authority,
            mutation_attempt,
            observation_attempt,
            request,
        };
        reservation.validate(journal)?;
        Ok(reservation)
    }

    pub(crate) fn validate(
        &self,
        journal: &AttemptJournalRecord,
    ) -> Result<(), ObservationReservationError> {
        Self::validate_identity(
            journal,
            &self.authority,
            self.mutation_attempt,
            self.observation_attempt,
            self.request.hash(),
        )
    }

    pub(crate) fn validate_identity(
        journal: &AttemptJournalRecord,
        authority: &AttemptAuthorityRecord,
        mutation_attempt: u32,
        observation_attempt: u32,
        request: &str,
    ) -> Result<(), ObservationReservationError> {
        if journal.authority() != authority {
            return Err(ObservationReservationError::AuthorityMismatch);
        }
        if journal.pending_mutation() != Some(mutation_attempt) {
            return Err(ObservationReservationError::MutationMismatch);
        }
        if journal.pending_observation() != Some(observation_attempt) {
            return Err(ObservationReservationError::ObservationMismatch);
        }
        if journal.pending_observation_request() != Some(request) {
            return Err(ObservationReservationError::RequestMismatch);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) enum ObservationReservationError {
    AuthorityMismatch,
    NoPendingMutation,
    NoPendingObservation,
    MutationMismatch,
    ObservationMismatch,
    RequestMismatch,
}
