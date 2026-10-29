//! Passive original application reservations; reconstructing one is never a resend permit.

use super::{ApplicationIngressError, ApplicationIngressPayload};
use crate::model::attempt_journal::ObservationReservation;
use crate::model::{
    attempt_journal::{AttemptAuthorityRecord, AttemptJournalRecord},
    operation_plan::OperationPlanRecord,
};

/// Exact original application mutation after durable reservation.
///
/// The caller holds its original guard and independently admits fresh authority,
/// dependencies, application safety and exclusive never-dispatched custody.
#[derive(Debug)]
pub struct ApplicationMutationRequest<'a> {
    plan: &'a OperationPlanRecord,
    payload: &'a ApplicationIngressPayload,
    authority: AttemptAuthorityRecord,
    mutation_attempt: u32,
}
impl<'a> ApplicationMutationRequest<'a> {
    /// Bind original exact opaque bytes to the current pending mutation.
    /// # Errors
    /// Rejects changed authority/payload, absent mutation or pending recovery.
    pub fn new(
        plan: &'a OperationPlanRecord,
        sequence: u64,
        journal: &AttemptJournalRecord,
        payload: &'a ApplicationIngressPayload,
    ) -> Result<Self, ApplicationIngressError> {
        let authority = authority(plan, sequence, journal)?;
        payload.validate_mutation_binding(authority.binding())?;
        let mutation_attempt = journal
            .pending_mutation()
            .ok_or(ApplicationIngressError::NoPendingMutation)?;
        let request = Self {
            plan,
            payload,
            authority,
            mutation_attempt,
        };
        request.validate_journal(journal)?;
        Ok(request)
    }
    /// Read the complete original plan/context and inventory.
    #[must_use]
    pub const fn plan(&self) -> &OperationPlanRecord {
        self.plan
    }
    /// Read exact opaque update fields.
    #[must_use]
    pub const fn payload(&self) -> &ApplicationIngressPayload {
        self.payload
    }
    /// Read full original authority and finite allowances.
    #[must_use]
    pub const fn authority(&self) -> &AttemptAuthorityRecord {
        &self.authority
    }
    /// Read the already consumed mutation attempt.
    #[must_use]
    pub const fn mutation_attempt(&self) -> u32 {
        self.mutation_attempt
    }
    /// Recheck current exact original reservation without authorizing dispatch.
    /// # Errors
    /// Rejects changed authority, mutation or pending observation.
    pub fn validate_journal(
        &self,
        journal: &AttemptJournalRecord,
    ) -> Result<(), ApplicationIngressError> {
        journal.validate_pending_mutation(
            &self.authority,
            self.mutation_attempt,
            ApplicationIngressError::AuthorityMismatch,
            ApplicationIngressError::MutationMismatch,
            ApplicationIngressError::ObservationPending,
        )
    }
}

/// Exact separately reserved application recovery update for an unresolved mutation.
///
/// This consumes the observation lane, even when its opaque method reads status.
/// It never reuses mutation allowance or supplies application settlement semantics.
#[derive(Debug)]
pub struct ApplicationObservationRequest<'a> {
    plan: &'a OperationPlanRecord,
    payload: &'a ApplicationIngressPayload,
    reservation: ObservationReservation,
}
impl<'a> ApplicationObservationRequest<'a> {
    /// Bind exact recovery bytes to the independently reserved observation digest.
    /// # Errors
    /// Rejects changed authority/target, absent attempts or different reserved bytes.
    pub fn new(
        plan: &'a OperationPlanRecord,
        sequence: u64,
        journal: &AttemptJournalRecord,
        payload: &'a ApplicationIngressPayload,
    ) -> Result<Self, ApplicationIngressError> {
        let authority = authority(plan, sequence, journal)?;
        payload.validate_observation_binding(authority.binding())?;
        let reservation = ObservationReservation::new(authority, journal, payload.digest())
            .map_err(crate::model::ic_observation::IcObservationRequestError::from)?;
        Ok(Self {
            plan,
            payload,
            reservation,
        })
    }
    /// Read the complete original mutation plan/context and inventory.
    #[must_use]
    pub const fn plan(&self) -> &OperationPlanRecord {
        self.plan
    }
    /// Read exact original recovery update fields.
    #[must_use]
    pub const fn payload(&self) -> &ApplicationIngressPayload {
        self.payload
    }
    /// Read original authority; its request still names the unresolved mutation.
    #[must_use]
    pub const fn authority(&self) -> &AttemptAuthorityRecord {
        &self.reservation.authority
    }
    /// Read the original unresolved mutation attempt.
    #[must_use]
    pub const fn mutation_attempt(&self) -> u32 {
        self.reservation.mutation_attempt
    }
    /// Read the independently consumed observation attempt.
    #[must_use]
    pub const fn observation_attempt(&self) -> u32 {
        self.reservation.observation_attempt
    }
    /// Recheck exact current observation identity without a dispatch permit.
    /// # Errors
    /// Rejects settled/replaced attempts, changed authority or different reserved bytes.
    pub fn validate_journal(
        &self,
        journal: &AttemptJournalRecord,
    ) -> Result<(), ApplicationIngressError> {
        self.reservation
            .validate(journal)
            .map_err(crate::model::ic_observation::IcObservationRequestError::from)?;
        Ok(())
    }
}

fn authority(
    plan: &OperationPlanRecord,
    sequence: u64,
    journal: &AttemptJournalRecord,
) -> Result<AttemptAuthorityRecord, ApplicationIngressError> {
    let authority = plan.attempt_authority(sequence)?;
    if journal.authority() != &authority {
        return Err(ApplicationIngressError::AuthorityMismatch);
    }
    Ok(authority)
}
