//! Exact application update bytes and original fence acquisition reservation binding.

use crate::model::application_payload::ApplicationPayloadError as Payload;
use crate::model::attempt_journal::MutationReservationError as Reservation;
use crate::model::evidence_digest::EvidenceDigest;
use crate::model::{
    artifacts::ArtifactChecksumRecord,
    attempt_journal::{AttemptAuthorityRecord, AttemptJournalRecord, MAX_OPERATION_ATTEMPTS},
    fence_obligation::{FenceObligationError, FenceObligationRecord},
    operation_plan::{OperationPlanError, OperationPlanRecord},
};
use std::fmt;
use thiserror::Error;

/// Maximum opaque application arguments for one acquisition update, checked before copying.
pub const MAX_FENCE_ACQUISITION_ARGUMENT_BYTES: usize = 1024 * 1024;
/// Maximum exact visible ASCII method bytes; method spelling is never normalized.
pub const MAX_FENCE_ACQUISITION_METHOD_BYTES: usize = 128;

/// Immutable bounded host update envelope; application codecs own argument semantics.
///
/// Receiver and routing target are the same exact application canister. No management,
/// proxy, query, signing or default method is supplied. Retain these original bytes
/// before publishing their plan and obligation; hashes cannot recover missing bytes.
/// The nonrecursive payload digest excludes plan/requirement/obligation hashes,
/// which bind the payload subsequently through the original operation authority.
#[derive(Clone)]
pub struct FenceAcquisitionPayload {
    target: String,
    target_bytes: Vec<u8>,
    method: String,
    arguments: Vec<u8>,
}
impl FenceAcquisitionPayload {
    /// Admit exact application receiver, method and bounded opaque argument bytes.
    /// # Errors
    /// Rejects malformed/management receivers, empty/nonvisible/oversized methods and excessive bytes.
    pub fn new(
        target: &str,
        method: &str,
        arguments: &[u8],
    ) -> Result<Self, FenceAcquisitionError> {
        crate::model::application_payload::validate_bytes(
            arguments,
            MAX_FENCE_ACQUISITION_ARGUMENT_BYTES,
            method,
            MAX_FENCE_ACQUISITION_METHOD_BYTES,
        )?;
        let (target, target_bytes) = crate::model::application_payload::principal(target)?;
        Ok(Self {
            target,
            target_bytes,
            method: method.into(),
            arguments: arguments.into(),
        })
    }
    /// Read the exact canonical application receiver and routing target.
    #[must_use]
    pub fn target(&self) -> &str {
        &self.target
    }
    /// Read exact original method spelling.
    #[must_use]
    pub fn method(&self) -> &str {
        &self.method
    }
    /// Read exact opaque original bytes; the application must qualify whole-unit acquisition semantics.
    #[must_use]
    pub fn arguments(&self) -> &[u8] {
        &self.arguments
    }
    /// Hash NUL-terminated domain, u8 principal length/raw bytes, fixed update byte 1,
    /// big-endian u32 method length/exact bytes and u64 argument length/exact bytes.
    #[must_use]
    pub fn digest(&self) -> ArtifactChecksumRecord {
        let mut bytes = EvidenceDigest::new(b"ic-backup/application-fence-acquisition/v1\0");
        bytes.bytes_u8(&self.target_bytes);
        bytes.byte(1); // Exactly one host replicated update, never a query.
        bytes.bytes_u32(self.method.as_bytes());
        bytes.bytes_u64(&self.arguments);
        bytes.finish()
    }
}
impl fmt::Debug for FenceAcquisitionPayload {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FenceAcquisitionPayload")
            .field("target", &self.target)
            .field("method", &self.method)
            .field("argument_bytes", &self.arguments.len())
            .finish_non_exhaustive()
    }
}

/// Admit exact original acquisition authority and payload before reserving spending.
///
/// This checks declarations only: no pending reservation, fresh permission,
/// never-dispatched custody or executable application contract is established.
/// # Errors
/// Rejects changed obligation/plan, journal authority, receiver or exact wire bytes.
pub fn original_authority(
    plan: &OperationPlanRecord,
    obligation: &FenceObligationRecord,
    journal: &AttemptJournalRecord,
    payload: &FenceAcquisitionPayload,
) -> Result<AttemptAuthorityRecord, FenceAcquisitionError> {
    obligation.validate_plan(plan)?;
    let authority = plan.attempt_authority(obligation.acquisition_operation())?;
    if journal.authority() != &authority {
        return Err(FenceAcquisitionError::AuthorityMismatch);
    }
    if payload.target() != authority.binding().target() {
        return Err(FenceAcquisitionError::TargetMismatch);
    }
    if payload.digest().hash() != authority.binding().request() {
        return Err(FenceAcquisitionError::PayloadMismatch);
    }
    Ok(authority)
}

/// Structural binding to an already reserved acquisition; never a fresh dispatch permit.
///
/// Integration admission additionally requires durable originals, exact request semantics,
/// actual permissions/context/unit, dependency/custody qualification and proof that this
/// reservation has never been dispatched. Reconstructing a pending request proves none
/// of those conditions. After interruption without that proof, reconcile instead.
#[derive(Debug)]
pub struct FenceAcquisitionRequest<'a> {
    plan: &'a OperationPlanRecord,
    obligation: &'a FenceObligationRecord,
    payload: &'a FenceAcquisitionPayload,
    authority: AttemptAuthorityRecord,
    mutation_attempt: u32,
}
impl<'a> FenceAcquisitionRequest<'a> {
    /// Bind original exact application bytes only after the canonical journal reserves mutation.
    /// # Errors
    /// Rejects changed originals, target/bytes, missing mutation or an acquisition already in observation recovery.
    pub fn new(
        plan: &'a OperationPlanRecord,
        obligation: &'a FenceObligationRecord,
        journal: &AttemptJournalRecord,
        payload: &'a FenceAcquisitionPayload,
    ) -> Result<Self, FenceAcquisitionError> {
        let authority = original_authority(plan, obligation, journal, payload)?;
        let mutation_attempt = journal
            .pending_mutation()
            .ok_or(FenceAcquisitionError::NoPendingMutation)?;
        let request = Self {
            plan,
            obligation,
            payload,
            authority,
            mutation_attempt,
        };
        request.validate_journal(journal)?;
        Ok(request)
    }
    /// Read original full context/inventory/selected unit; target routing does not narrow coverage.
    #[must_use]
    pub const fn plan(&self) -> &OperationPlanRecord {
        self.plan
    }
    /// Read original purpose/fence/revisions under the retained requirement.
    #[must_use]
    pub const fn obligation(&self) -> &FenceObligationRecord {
        self.obligation
    }
    /// Read exact immutable application update bytes.
    #[must_use]
    pub const fn payload(&self) -> &FenceAcquisitionPayload {
        self.payload
    }
    /// Read full original operation identity and spending limits.
    #[must_use]
    pub const fn authority(&self) -> &AttemptAuthorityRecord {
        &self.authority
    }
    /// Read the already consumed original mutation number.
    #[must_use]
    pub const fn mutation_attempt(&self) -> u32 {
        self.mutation_attempt
    }
    /// Recheck current reservation before associating an acknowledgement; performs no IO or effects.
    /// # Errors
    /// Rejects changed authority, settled/replaced mutation or pending observation recovery.
    pub fn validate_journal(
        &self,
        journal: &AttemptJournalRecord,
    ) -> Result<(), FenceAcquisitionError> {
        journal
            .validate_pending_mutation(&self.authority, self.mutation_attempt)
            .map_err(Into::into)
    }
}

/// Passive exact update reply association, without acquisition outcome or fence custody proof.
#[derive(Clone, Debug)]
pub struct FenceAcquisitionAcknowledgement {
    /// Full original authority digest, including original plan/context/operation/limits.
    pub authority: ArtifactChecksumRecord,
    /// Original already reserved mutation attempt, not an independently numbered call.
    pub mutation_attempt: u32,
    /// Qualified provider's retained exact reply association evidence; not an authenticated signature.
    pub evidence: ArtifactChecksumRecord,
}
impl FenceAcquisitionAcknowledgement {
    /// Admit a passive acknowledgement with a finite original mutation identity.
    /// # Errors
    /// Rejects zero or attempt numbers above the existing combined attempt ceiling.
    pub fn new(
        authority: ArtifactChecksumRecord,
        mutation_attempt: u32,
        evidence: ArtifactChecksumRecord,
    ) -> Result<Self, FenceAcquisitionError> {
        if mutation_attempt == 0 || mutation_attempt > MAX_OPERATION_ATTEMPTS {
            return Err(FenceAcquisitionError::InvalidAttempt);
        }
        Ok(Self {
            authority,
            mutation_attempt,
            evidence,
        })
    }
}

/// Typed payload/original reservation rejection; no error refunds or releases anything.
#[derive(Debug, Error)]
pub enum FenceAcquisitionError {
    /// Receiver is not a bounded canonical principal.
    #[error("invalid fence acquisition receiver")]
    InvalidTarget,
    /// This port cannot call the management canister.
    #[error("fence acquisition requires an application receiver")]
    ManagementReceiver,
    /// Method exceeds the exact visible ASCII envelope bound.
    #[error("invalid fence acquisition method")]
    InvalidMethod,
    /// Raw arguments exceed the finite input bound.
    #[error("fence acquisition arguments exceed the input bound")]
    ArgumentsTooLarge,
    /// Journal is not the exact original acquisition authority.
    #[error("fence acquisition original journal authority mismatch")]
    AuthorityMismatch,
    /// Receiver differs from the original operation target.
    #[error("fence acquisition original target mismatch")]
    TargetMismatch,
    /// Method/arguments differ from the original declared payload digest.
    #[error("fence acquisition original payload mismatch")]
    PayloadMismatch,
    /// Original mutation was not already reserved.
    #[error("fence acquisition requires a pending mutation reservation")]
    NoPendingMutation,
    /// Current journal no longer retains this exact unresolved mutation.
    #[error("fence acquisition pending mutation mismatch")]
    MutationMismatch,
    /// An observation is pending; original mutation must not be dispatched again.
    #[error("fence acquisition observation recovery is pending")]
    ObservationPending,
    /// Acknowledgement attempt is outside the finite original journal range.
    #[error("invalid fence acquisition acknowledgement attempt")]
    InvalidAttempt,
    /// Original obligation admission failed.
    #[error(transparent)]
    Obligation(#[from] FenceObligationError),
    /// Original plan authority cannot be derived.
    #[error(transparent)]
    Plan(#[from] OperationPlanError),
}

#[cfg(test)]
mod tests;

impl From<crate::model::attempt_journal::MutationReservationError> for FenceAcquisitionError {
    fn from(error: crate::model::attempt_journal::MutationReservationError) -> Self {
        match error {
            Reservation::NoPendingMutation => Self::NoPendingMutation,
            Reservation::AuthorityMismatch => Self::AuthorityMismatch,
            Reservation::MutationMismatch => Self::MutationMismatch,
            Reservation::ObservationPending => Self::ObservationPending,
        }
    }
}

impl From<crate::model::application_payload::ApplicationPayloadError> for FenceAcquisitionError {
    fn from(error: crate::model::application_payload::ApplicationPayloadError) -> Self {
        match error {
            Payload::ArgumentsTooLarge => Self::ArgumentsTooLarge,
            Payload::InvalidMethod => Self::InvalidMethod,
            Payload::InvalidPrincipal => Self::InvalidTarget,
            Payload::ManagementPrincipal => Self::ManagementReceiver,
        }
    }
}
