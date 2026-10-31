//! Exact reserved IC mutation identity and bounded passive reply association.

use crate::model::attempt_journal::MutationReservation;
use crate::model::attempt_journal::MutationReservationError as Reservation;
use crate::model::ic_update_response::IcUpdateResponseInput;
use crate::model::{
    attempt_journal::{AttemptAuthorityRecord, AttemptJournalRecord},
    ic_lifecycle_reply::MAX_IC_LIFECYCLE_REPLY_BYTES,
    ic_request::{IcManagementRequestRecord, IcRequestError},
    ic_snapshot_reply::MAX_IC_SNAPSHOT_REPLY_BYTES,
    operation_plan::{OperationPlanError, OperationPlanRecord},
};
use std::fmt;
use thiserror::Error;

/// Maximum owned raw reply bytes, derived from both existing finite codec owners.
pub const MAX_IC_MUTATION_REPLY_BYTES: usize =
    if MAX_IC_SNAPSHOT_REPLY_BYTES < MAX_IC_LIFECYCLE_REPLY_BYTES {
        MAX_IC_SNAPSHOT_REPLY_BYTES
    } else {
        MAX_IC_LIFECYCLE_REPLY_BYTES
    };

/// Structural request for one already reserved original host-ingress mutation.
///
/// Retained identity does not prove fresh permissions, prerequisites, application
/// safety or exclusive custody. A pending reservation does not prove it was never
/// dispatched. This request is not a dispatch permit or an interruption retry.
#[derive(Debug)]
pub struct IcMutationRequest<'a> {
    plan: &'a OperationPlanRecord,
    payload: &'a IcManagementRequestRecord,
    reservation: MutationReservation,
}

impl<'a> IcMutationRequest<'a> {
    /// Bind exact original plan, allowances and IC bytes after mutation reservation.
    ///
    /// Accepts only capture, load, start and stop; observations have separate
    /// reservations. Derivation never creates a journal, spends or resets allowance.
    /// # Errors
    /// Rejects unknown operations, changed authority/bytes, absent pending mutation
    /// or an observation already reserved for recovery.
    pub fn new(
        plan: &'a OperationPlanRecord,
        operation_sequence: u64,
        journal: &AttemptJournalRecord,
        payload: &'a IcManagementRequestRecord,
    ) -> Result<Self, IcMutationRequestError> {
        let authority = plan.attempt_authority(operation_sequence)?;
        if journal.authority() != &authority {
            return Err(IcMutationRequestError::AuthorityMismatch);
        }
        payload.validate_mutation_binding(authority.binding())?;
        let reservation = MutationReservation::new(authority, journal)?;
        Ok(Self {
            plan,
            payload,
            reservation,
        })
    }

    /// Read original full context, inventory, selection and dependency declarations.
    #[must_use]
    pub const fn plan(&self) -> &OperationPlanRecord {
        self.plan
    }
    /// Read exact management receiver, routing target, method and Candid argument bytes.
    #[must_use]
    pub const fn payload(&self) -> &'a IcManagementRequestRecord {
        self.payload
    }
    /// Read canonical original identity and immutable mutation/observation limits.
    #[must_use]
    pub const fn authority(&self) -> &AttemptAuthorityRecord {
        &self.reservation.authority
    }
    /// Read the original already consumed mutation attempt number.
    #[must_use]
    pub const fn mutation_attempt(&self) -> u32 {
        self.reservation.mutation_attempt
    }
    /// Recheck the exact current pending reservation without IO or effect admission.
    /// # Errors
    /// Rejects different authority, replaced/settled mutation or observation recovery.
    pub fn validate_journal(
        &self,
        journal: &AttemptJournalRecord,
    ) -> Result<(), IcMutationRequestError> {
        self.reservation.validate(journal).map_err(Into::into)
    }
}

/// Immutable bounded passive acknowledgement; not a receipt or authenticated outcome.
///
/// No Serde or default admission exists. Raw bytes are hidden from Debug; explicit
/// access retains exact evidence. Decoding happens in the existing method-specific
/// reply owners during pure association, never in a second codec.
#[derive(Clone)]
pub struct IcMutationAcknowledgement {
    input: IcUpdateResponseInput,
}

impl IcMutationAcknowledgement {
    /// Admit finite attempt/raw-byte bounds and normalize the actual target identity.
    /// # Errors
    /// Rejects zero/excessive attempts, oversized raw replies and invalid principals.
    pub fn new(mut input: IcUpdateResponseInput) -> Result<Self, IcMutationAcknowledgementError> {
        input.admit(MAX_IC_MUTATION_REPLY_BYTES)?;
        Ok(Self { input })
    }
    /// Read canonical passive association fields and exact raw reply, without mutation access.
    #[must_use]
    pub const fn input(&self) -> &IcUpdateResponseInput {
        &self.input
    }
}

impl fmt::Debug for IcMutationAcknowledgement {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.input
            .fmt_redacted(formatter, "IcMutationAcknowledgement")
    }
}

/// Original reservation denial; no variant consumes, refunds or grants call authority.
#[derive(Debug, Error)]
pub enum IcMutationRequestError {
    /// Journal differs from full original plan/context/operation/request/allowance identity.
    #[error("IC mutation original authority mismatch")]
    AuthorityMismatch,
    /// There is no unresolved original mutation reservation.
    #[error("IC mutation requires a pending original mutation")]
    NoPendingMutation,
    /// Current pending mutation changed or was settled after binding.
    #[error("IC mutation original attempt mismatch")]
    MutationMismatch,
    /// Recovery observation is pending; association cannot bypass its original owner.
    #[error("IC mutation has a pending recovery observation")]
    ObservationPending,
    /// Exact closed method, target or original Candid payload digest differs.
    #[error(transparent)]
    Payload(#[from] IcRequestError),
    /// Original plan cannot derive the requested operation authority.
    #[error(transparent)]
    Plan(#[from] OperationPlanError),
}

/// Passive reply boundary denial with no raw payload in diagnostics.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum IcMutationAcknowledgementError {
    /// Original attempt must be within the existing 1..=1,024 total-attempt bound.
    #[error("invalid IC mutation acknowledgement attempt")]
    InvalidAttempt,
    /// Raw reply exceeded the existing finite codec byte bounds.
    #[error("IC mutation acknowledgement reply too large")]
    ReplyTooLarge,
    /// Actual provider target is not a principal.
    #[error("invalid IC mutation acknowledgement target")]
    InvalidTarget,
}

#[cfg(test)]
mod tests;

impl From<crate::model::attempt_journal::MutationReservationError> for IcMutationRequestError {
    fn from(error: crate::model::attempt_journal::MutationReservationError) -> Self {
        match error {
            Reservation::NoPendingMutation => Self::NoPendingMutation,
            Reservation::AuthorityMismatch => Self::AuthorityMismatch,
            Reservation::MutationMismatch => Self::MutationMismatch,
            Reservation::ObservationPending => Self::ObservationPending,
        }
    }
}

impl From<crate::model::ic_update_response::UpdateResponseAdmissionError>
    for IcMutationAcknowledgementError
{
    fn from(error: crate::model::ic_update_response::UpdateResponseAdmissionError) -> Self {
        match error {
            crate::model::ic_update_response::UpdateResponseAdmissionError::InvalidAttempt => {
                Self::InvalidAttempt
            }
            crate::model::ic_update_response::UpdateResponseAdmissionError::ReplyTooLarge => {
                Self::ReplyTooLarge
            }
            crate::model::ic_update_response::UpdateResponseAdmissionError::InvalidTarget => {
                Self::InvalidTarget
            }
        }
    }
}
