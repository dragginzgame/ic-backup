//! Exact already reserved IC recovery observations and bounded passive replies.

use crate::model::{
    artifacts::ArtifactChecksumRecord,
    attempt_journal::{AttemptAuthorityRecord, AttemptJournalRecord, MAX_OPERATION_ATTEMPTS},
    ic_lifecycle_reply::MAX_IC_LIFECYCLE_REPLY_BYTES,
    ic_request::{IcManagementRequestRecord, IcRequestError},
    ic_snapshot_reply::MAX_IC_SNAPSHOT_REPLY_BYTES,
    operation_plan::{OperationPlanError, OperationPlanRecord, PlanContextRecord},
};
use std::fmt;
use thiserror::Error;

/// Raw observation reply bound derived from both existing method-specific codecs.
pub const MAX_IC_OBSERVATION_REPLY_BYTES: usize =
    if MAX_IC_SNAPSHOT_REPLY_BYTES < MAX_IC_LIFECYCLE_REPLY_BYTES {
        MAX_IC_SNAPSHOT_REPLY_BYTES
    } else {
        MAX_IC_LIFECYCLE_REPLY_BYTES
    };

/// Structural binding to an original pending mutation and its reserved IC observation.
///
/// Retain original observation bytes before reserving their existing payload digest.
/// This request neither spends nor proves dispatch custody, fresh read permission,
/// authentication or effect attribution. Recovery never permits repeating a lost call.
#[derive(Debug)]
pub struct IcObservationRequest<'a> {
    plan: &'a OperationPlanRecord,
    mutation: &'a IcManagementRequestRecord,
    payload: &'a IcManagementRequestRecord,
    authority: AttemptAuthorityRecord,
    mutation_attempt: u32,
    observation_attempt: u32,
}

impl<'a> IcObservationRequest<'a> {
    /// Bind original mutation bytes and exact already reserved status/list bytes.
    ///
    /// Both payloads must name the same original target. Observation identity reuses
    /// the existing IC payload digest; no second hash or journal owner is introduced.
    /// # Errors
    /// Rejects changed authority/payloads or missing original pending reservations.
    pub fn new(
        plan: &'a OperationPlanRecord,
        operation_sequence: u64,
        journal: &AttemptJournalRecord,
        mutation: &'a IcManagementRequestRecord,
        payload: &'a IcManagementRequestRecord,
    ) -> Result<Self, IcObservationRequestError> {
        let authority = plan.attempt_authority(operation_sequence)?;
        if journal.authority() != &authority {
            return Err(IcObservationRequestError::AuthorityMismatch);
        }
        mutation.validate_mutation_binding(authority.binding())?;
        payload.validate_observation_binding(authority.binding(), &payload.digest())?;
        let current = journal.view();
        let request = Self {
            plan,
            mutation,
            payload,
            authority,
            mutation_attempt: current
                .pending_mutation
                .ok_or(IcObservationRequestError::NoPendingMutation)?,
            observation_attempt: current
                .pending_observation
                .ok_or(IcObservationRequestError::NoPendingObservation)?,
        };
        request.validate_journal(journal)?;
        Ok(request)
    }
    /// Read the full original context, inventory and selection.
    #[must_use]
    pub const fn plan(&self) -> &OperationPlanRecord {
        self.plan
    }
    /// Read the exact original mutation, never an executable retry.
    #[must_use]
    pub const fn mutation(&self) -> &'a IcManagementRequestRecord {
        self.mutation
    }
    /// Read exact observation receiver, routing target, method and Candid bytes.
    #[must_use]
    pub const fn payload(&self) -> &'a IcManagementRequestRecord {
        self.payload
    }
    /// Read the original operation authority and immutable attempt limits.
    #[must_use]
    pub const fn authority(&self) -> &AttemptAuthorityRecord {
        &self.authority
    }
    /// Read the original unresolved mutation attempt.
    #[must_use]
    pub const fn mutation_attempt(&self) -> u32 {
        self.mutation_attempt
    }
    /// Read the original already consumed observation attempt.
    #[must_use]
    pub const fn observation_attempt(&self) -> u32 {
        self.observation_attempt
    }
    /// Recheck current exact reservations and original observation bytes without IO.
    /// # Errors
    /// Rejects changed authority, settled/replaced attempts or different reserved bytes.
    pub fn validate_journal(
        &self,
        journal: &AttemptJournalRecord,
    ) -> Result<(), IcObservationRequestError> {
        ObservationReservation {
            authority: &self.authority,
            mutation_attempt: self.mutation_attempt,
            observation_attempt: self.observation_attempt,
            request: self.payload.digest(),
        }
        .validate(journal)
    }
}

pub(crate) struct ObservationReservation<'a> {
    pub authority: &'a AttemptAuthorityRecord,
    pub mutation_attempt: u32,
    pub observation_attempt: u32,
    pub request: ArtifactChecksumRecord,
}

impl ObservationReservation<'_> {
    pub(crate) fn validate(
        &self,
        journal: &AttemptJournalRecord,
    ) -> Result<(), IcObservationRequestError> {
        if journal.authority() != self.authority {
            return Err(IcObservationRequestError::AuthorityMismatch);
        }
        let current = journal.view();
        if current.pending_mutation != Some(self.mutation_attempt) {
            return Err(IcObservationRequestError::MutationMismatch);
        }
        if current.pending_observation != Some(self.observation_attempt) {
            return Err(IcObservationRequestError::ObservationMismatch);
        }
        if journal.pending_observation_request() != Some(self.request.hash()) {
            return Err(IcObservationRequestError::RequestMismatch);
        }
        Ok(())
    }
}

/// Passive provider fields; actual authentication and observation timing remain external.
#[derive(Clone)]
pub struct IcObservationResponseInput {
    /// Exact full original authority digest including immutable allowances.
    pub authority: ArtifactChecksumRecord,
    /// Exact original unresolved mutation attempt.
    pub mutation_attempt: u32,
    /// Exact already reserved observation attempt, not another call number.
    pub observation_attempt: u32,
    /// Exact existing observation payload digest retained in its reservation.
    pub request: ArtifactChecksumRecord,
    /// Actual network/caller/release association claimed by the provider.
    pub context: PlanContextRecord,
    /// Actual response target claimed by the provider; canonicalized on admission.
    pub target: String,
    /// Exact bounded raw Candid reply.
    pub reply: Vec<u8>,
    /// Opaque retained association/authentication/timing evidence; not self-authenticating.
    pub evidence: ArtifactChecksumRecord,
}

/// Immutable bounded passive observation, with no outcome, Default or Serde admission.
#[derive(Clone)]
pub struct IcObservationResponse {
    input: IcObservationResponseInput,
}
impl IcObservationResponse {
    /// Admit canonical target, chronological finite attempt IDs and bounded raw bytes.
    /// # Errors
    /// Rejects zero/excessive/reversed attempts, oversized replies and invalid principals.
    pub fn new(mut input: IcObservationResponseInput) -> Result<Self, IcObservationResponseError> {
        validate_response_input(&mut input, MAX_IC_OBSERVATION_REPLY_BYTES)?;
        Ok(Self { input })
    }
    /// Read immutable canonical claims and exact raw bytes; no mutation access.
    #[must_use]
    pub const fn input(&self) -> &IcObservationResponseInput {
        &self.input
    }
}
impl fmt::Debug for IcObservationResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_response_input(&self.input, formatter, "IcObservationResponse")
    }
}

pub(crate) fn validate_response_input(
    input: &mut IcObservationResponseInput,
    max_reply_bytes: usize,
) -> Result<(), IcObservationResponseError> {
    if input.mutation_attempt == 0
        || input.observation_attempt <= input.mutation_attempt
        || input.observation_attempt > MAX_OPERATION_ATTEMPTS
    {
        return Err(IcObservationResponseError::InvalidAttempts);
    }
    if input.reply.len() > max_reply_bytes {
        return Err(IcObservationResponseError::ReplyTooLarge);
    }
    input.target = crate::model::principal::canonical_text(&input.target)
        .ok_or(IcObservationResponseError::InvalidTarget)?;
    Ok(())
}

pub(crate) fn fmt_response_input(
    input: &IcObservationResponseInput,
    formatter: &mut fmt::Formatter<'_>,
    name: &str,
) -> fmt::Result {
    formatter
        .debug_struct(name)
        .field("authority", &input.authority)
        .field("mutation_attempt", &input.mutation_attempt)
        .field("observation_attempt", &input.observation_attempt)
        .field("request", &input.request)
        .field("target", &input.target)
        .field("reply_bytes", &input.reply.len())
        .finish_non_exhaustive()
}

/// Structural original observation denial; no spending or effect outcome changes.
#[derive(Debug, Error)]
pub enum IcObservationRequestError {
    /// Journal differs from original plan/operation/allowances.
    #[error("IC observation original authority mismatch")]
    AuthorityMismatch,
    /// Original mutation has no pending reservation.
    #[error("IC observation requires a pending original mutation")]
    NoPendingMutation,
    /// Original recovery observation has no pending reservation.
    #[error("IC observation requires a pending original observation")]
    NoPendingObservation,
    /// Current original mutation changed or settled.
    #[error("IC observation original mutation mismatch")]
    MutationMismatch,
    /// Current original observation changed or settled.
    #[error("IC observation original attempt mismatch")]
    ObservationMismatch,
    /// Reserved observation digest differs from exact canonical payload bytes.
    #[error("IC observation reserved request mismatch")]
    RequestMismatch,
    /// Original mutation or observation target/digest/class differs.
    #[error(transparent)]
    Payload(#[from] IcRequestError),
    /// Original plan cannot derive the requested operation authority.
    #[error(transparent)]
    Plan(#[from] OperationPlanError),
}

/// Bounded passive response rejection with no raw bytes in diagnostics.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum IcObservationResponseError {
    /// IDs must be chronological within the existing 1,024 total-attempt bound.
    #[error("invalid IC observation response attempts")]
    InvalidAttempts,
    /// Raw bytes exceed the existing finite codec ceilings.
    #[error("IC observation response reply too large")]
    ReplyTooLarge,
    /// Actual target is not a principal.
    #[error("invalid IC observation response target")]
    InvalidTarget,
}

#[cfg(test)]
mod tests;
