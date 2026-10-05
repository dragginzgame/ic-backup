//! Exact reserved IC mutation identity and bounded passive reply association.

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
    authority: AttemptAuthorityRecord,
    mutation_attempt: u32,
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
        let mutation_attempt = journal
            .view()
            .pending_mutation
            .ok_or(IcMutationRequestError::NoPendingMutation)?;
        let request = Self {
            plan,
            payload,
            authority,
            mutation_attempt,
        };
        request.validate_journal(journal)?;
        Ok(request)
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
        &self.authority
    }
    /// Read the original already consumed mutation attempt number.
    #[must_use]
    pub const fn mutation_attempt(&self) -> u32 {
        self.mutation_attempt
    }
    /// Recheck the exact current pending reservation without IO or effect admission.
    /// # Errors
    /// Rejects different authority, replaced/settled mutation or observation recovery.
    pub fn validate_journal(
        &self,
        journal: &AttemptJournalRecord,
    ) -> Result<(), IcMutationRequestError> {
        if journal.authority() != &self.authority {
            return Err(IcMutationRequestError::AuthorityMismatch);
        }
        let current = journal.view();
        if current.pending_mutation != Some(self.mutation_attempt) {
            return Err(IcMutationRequestError::MutationMismatch);
        }
        if current.pending_observation.is_some() {
            return Err(IcMutationRequestError::ObservationPending);
        }
        Ok(())
    }
}

/// Passive provider fields; actual authentication and association are integration-owned.
#[derive(Clone)]
pub struct IcMutationAcknowledgementInput {
    /// Full original authority digest, including plan/context/operation/limits.
    pub authority: ArtifactChecksumRecord,
    /// Original already reserved mutation attempt, never a new call number.
    pub mutation_attempt: u32,
    /// Actual authenticated network/caller/release claimed by the provider.
    pub context: PlanContextRecord,
    /// Actual response routing target claimed by the provider; canonicalized on admission.
    pub target: String,
    /// Exact raw Candid reply, retained under the existing codec byte ceilings.
    pub reply: Vec<u8>,
    /// Opaque provider evidence binding original request/attempt and all actual fields.
    pub evidence: ArtifactChecksumRecord,
}

/// Immutable bounded passive acknowledgement; not a receipt or authenticated outcome.
///
/// No Serde or default admission exists. Raw bytes are hidden from Debug; explicit
/// access retains exact evidence. Decoding happens in the existing method-specific
/// reply owners during pure association, never in a second codec.
#[derive(Clone)]
pub struct IcMutationAcknowledgement {
    input: IcMutationAcknowledgementInput,
}

impl IcMutationAcknowledgement {
    /// Admit finite attempt/raw-byte bounds and normalize the actual target identity.
    /// # Errors
    /// Rejects zero/excessive attempts, oversized raw replies and invalid principals.
    pub fn new(
        mut input: IcMutationAcknowledgementInput,
    ) -> Result<Self, IcMutationAcknowledgementError> {
        if input.mutation_attempt == 0 || input.mutation_attempt > MAX_OPERATION_ATTEMPTS {
            return Err(IcMutationAcknowledgementError::InvalidAttempt);
        }
        if input.reply.len() > MAX_IC_MUTATION_REPLY_BYTES {
            return Err(IcMutationAcknowledgementError::ReplyTooLarge);
        }
        input.target = crate::model::principal::canonical_text(&input.target)
            .ok_or(IcMutationAcknowledgementError::InvalidTarget)?;
        Ok(Self { input })
    }
    /// Read canonical passive association fields and exact raw reply, without mutation access.
    #[must_use]
    pub const fn input(&self) -> &IcMutationAcknowledgementInput {
        &self.input
    }
}

impl fmt::Debug for IcMutationAcknowledgement {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("IcMutationAcknowledgement")
            .field("authority", &self.input.authority)
            .field("mutation_attempt", &self.input.mutation_attempt)
            .field("target", &self.input.target)
            .field("reply_bytes", &self.input.reply.len())
            .finish_non_exhaustive()
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
