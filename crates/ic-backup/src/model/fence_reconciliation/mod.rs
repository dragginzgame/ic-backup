//! Original fence-acquisition observation identity and exact retained reservation binding.

use crate::model::{
    artifacts::ArtifactChecksumRecord,
    attempt_journal::{AttemptAuthorityRecord, AttemptJournalRecord, MAX_OPERATION_ATTEMPTS},
    consistency::ApplicationFenceEvidence,
    fence_obligation::{FenceObligationError, FenceObligationRecord},
    inventory::{InventoryRecord, InventoryRecordError, MAX_INVENTORY_TARGETS},
    operation_plan::{OperationPlanError, OperationPlanRecord, PlanContextRecord},
    restore_safety::RestoreFenceEvidence,
};
use thiserror::Error;

/// Descriptive maximum underlying remote observations for this single reserved observation.
///
/// This is not spending authority. Every paid call requires prior durable accounting;
/// this port cannot batch multiple remote calls under one observation reservation.
pub const MAX_FENCE_RECONCILIATION_REMOTE_OBSERVATIONS: u32 = 1;

/// Ephemeral original observation intent, derived before its durable reservation.
///
/// The original obligation must already be retained under its exact requirement
/// and guarded plans. This pure binding authenticates no provider, request bytes
/// or persisted declaration and grants no fresh dispatch authority.
#[derive(Clone, Debug)]
pub struct FenceReconciliationIntent<'a> {
    plan: &'a OperationPlanRecord,
    obligation: &'a FenceObligationRecord,
    authority: AttemptAuthorityRecord,
    mutation_attempt: u32,
    challenge: ArtifactChecksumRecord,
}
impl<'a> FenceReconciliationIntent<'a> {
    /// Bind the original pending acquisition, full obligation and caller-owned fresh challenge.
    ///
    /// Derivation never creates/resets a journal. Recovering the same intent may
    /// associate a late reply; it never authorizes repeating a lost observation.
    /// # Errors
    /// Rejects another plan/journal/allowance or an acquisition without a pending mutation.
    pub fn new(
        plan: &'a OperationPlanRecord,
        obligation: &'a FenceObligationRecord,
        journal: &AttemptJournalRecord,
        challenge: ArtifactChecksumRecord,
    ) -> Result<Self, FenceReconciliationRequestError> {
        obligation.validate_plan(plan)?;
        let authority = plan.attempt_authority(obligation.acquisition_operation())?;
        if journal.authority() != &authority {
            return Err(FenceReconciliationRequestError::AuthorityMismatch);
        }
        let mutation_attempt = journal
            .view()
            .pending_mutation
            .ok_or(FenceReconciliationRequestError::NoPendingMutation)?;
        Ok(Self {
            plan,
            obligation,
            authority,
            mutation_attempt,
            challenge,
        })
    }
    /// Read the exact original whole-unit obligation; never current Active custody.
    #[must_use]
    pub const fn obligation(&self) -> &FenceObligationRecord {
        self.obligation
    }
    /// Read the original context, full inventory and selected unit.
    #[must_use]
    pub const fn plan(&self) -> &OperationPlanRecord {
        self.plan
    }
    /// Read immutable acquisition identity and original attempt allowances.
    #[must_use]
    pub const fn authority(&self) -> &AttemptAuthorityRecord {
        &self.authority
    }
    /// Read the exact unresolved original mutation attempt.
    #[must_use]
    pub const fn mutation_attempt(&self) -> u32 {
        self.mutation_attempt
    }
    /// Read the integration-owned current challenge; equality does not prove freshness.
    #[must_use]
    pub const fn challenge(&self) -> &ArtifactChecksumRecord {
        &self.challenge
    }
    /// Hash NUL-terminated v1 domain, 64 ASCII obligation/authority hashes,
    /// big-endian u32 mutation attempt and 64 ASCII challenge bytes.
    ///
    /// The digest is reserved before obtaining an observation attempt number.
    /// Actual replies also bind that allocated number, preventing cross-attempt replay.
    #[must_use]
    pub fn digest(&self) -> ArtifactChecksumRecord {
        observation_digest(
            &self.obligation.digest(),
            &self.authority.digest(),
            self.mutation_attempt,
            &self.challenge,
        )
    }
    fn validate_mutation(
        &self,
        journal: &AttemptJournalRecord,
    ) -> Result<(), FenceReconciliationRequestError> {
        if journal.authority() != &self.authority {
            return Err(FenceReconciliationRequestError::AuthorityMismatch);
        }
        if journal.view().pending_mutation != Some(self.mutation_attempt) {
            return Err(FenceReconciliationRequestError::MutationMismatch);
        }
        Ok(())
    }
}
fn observation_digest(
    obligation: &ArtifactChecksumRecord,
    authority: &ArtifactChecksumRecord,
    mutation: u32,
    challenge: &ArtifactChecksumRecord,
) -> ArtifactChecksumRecord {
    let mut bytes = b"ic-backup/fence-reconciliation/v1\0".to_vec();
    bytes.extend_from_slice(obligation.hash().as_bytes());
    bytes.extend_from_slice(authority.hash().as_bytes());
    bytes.extend_from_slice(&mutation.to_be_bytes());
    bytes.extend_from_slice(challenge.hash().as_bytes());
    ArtifactChecksumRecord::from_bytes(&bytes)
}

/// Ephemeral observation request bound to the exact already-reserved original journal event.
///
/// Binding is structural, not a dispatch permit. Before invocation the integration
/// admits durable custody, original dependencies, actual authority and no earlier
/// dispatch of this reservation. Lost replies remain pending and cannot be resent.
#[derive(Clone, Debug)]
pub struct FenceReconciliationRequest<'a, 'plan> {
    intent: &'a FenceReconciliationIntent<'plan>,
    observation_attempt: u32,
}
impl<'a, 'plan> FenceReconciliationRequest<'a, 'plan> {
    /// Bind only after the existing journal owner durably reserves this exact intent digest.
    /// # Errors
    /// Rejects absent/different pending mutation, observation, request or original authority.
    pub fn new(
        intent: &'a FenceReconciliationIntent<'plan>,
        journal: &AttemptJournalRecord,
    ) -> Result<Self, FenceReconciliationRequestError> {
        intent.validate_mutation(journal)?;
        let observation_attempt = journal
            .view()
            .pending_observation
            .ok_or(FenceReconciliationRequestError::NoPendingObservation)?;
        let request = Self {
            intent,
            observation_attempt,
        };
        request.validate_journal(journal)?;
        Ok(request)
    }
    /// Read the original challenge-bound intent retained by its caller.
    #[must_use]
    pub const fn intent(&self) -> &FenceReconciliationIntent<'plan> {
        self.intent
    }
    /// Read the exact previously allocated observation attempt.
    #[must_use]
    pub const fn observation_attempt(&self) -> u32 {
        self.observation_attempt
    }
    /// Read the same canonical observation digest reserved in the original journal.
    #[must_use]
    pub fn digest(&self) -> ArtifactChecksumRecord {
        self.intent.digest()
    }
    /// Recheck exact current retained reservation before admitting any result.
    ///
    /// This pure check authenticates no receipt and never performs IO or scheduling.
    /// # Errors
    /// Rejects replaced/settled journal evidence, other attempts, request or authority.
    pub fn validate_journal(
        &self,
        journal: &AttemptJournalRecord,
    ) -> Result<(), FenceReconciliationRequestError> {
        self.intent.validate_mutation(journal)?;
        if journal.view().pending_observation != Some(self.observation_attempt) {
            return Err(FenceReconciliationRequestError::ObservationMismatch);
        }
        if journal.pending_observation_request() != Some(self.digest().hash()) {
            return Err(FenceReconciliationRequestError::RequestMismatch);
        }
        Ok(())
    }
}

/// Passive application-qualified attribution; fence appearance alone is insufficient.
#[derive(Clone, Debug)]
pub enum FenceReconciliationEvidence {
    /// Exact capture acquisition is exclusively attributed to the original request/attempt.
    AcquiredCapture {
        /// Actual Active whole-unit write/membership/timer/external-work/drain evidence.
        fence: Box<ApplicationFenceEvidence>,
        /// Original-request attribution, including exclusion of independent acquisitions.
        attribution: ArtifactChecksumRecord,
    },
    /// Exact restore acquisition is attributed with outside-snapshot/replay-safe custody.
    AcquiredRestore {
        /// Actual retained fence, whole-selection, rewind-independent and replay-safety evidence.
        fence: Box<RestoreFenceEvidence>,
        /// Exact original-request attribution; not merely matching current fence identity.
        attribution: ArtifactChecksumRecord,
    },
    /// Qualified exclusion proves this exact acquisition never applied.
    NotAcquired {
        /// Nonapplication proof includes exclusion of a transient acquire/release cycle.
        exclusion: ArtifactChecksumRecord,
    },
    /// A settled authenticated observation cannot resolve the exact original acquisition.
    Unresolved {
        /// Retained uncertainty evidence; a lost reply cannot construct this variant.
        uncertainty: ArtifactChecksumRecord,
    },
}
/// Passive actual context/selection and attribution under one exact reserved observation.
#[derive(Clone, Debug)]
pub struct FenceReconciliationObservationInput {
    /// Exact original challenge-bound observation request digest.
    pub request: ArtifactChecksumRecord,
    /// Actual associated original pending mutation attempt.
    pub mutation_attempt: u32,
    /// Actual associated already-allocated observation attempt.
    pub observation_attempt: u32,
    /// Actually authenticated network/caller/release, not copied declarations.
    pub context: PlanContextRecord,
    /// Full actual current inventory, including unselected parent metadata.
    pub inventory: InventoryRecord,
    /// Actual whole-unit coverage; model admits a bounded canonical unique set.
    pub selected_targets: Vec<String>,
    /// Actual application-qualified original-acquisition attribution or unresolved result.
    pub settlement: FenceReconciliationEvidence,
    /// Opaque evidence binding the exact request, attempts and all actual result fields.
    pub evidence: ArtifactChecksumRecord,
    /// Actual underlying remote observations; prior durable per-call accounting is mandatory.
    pub remote_observations: u32,
}
/// Canonical immutable passive observation; no serialized authority or automatic receipt.
#[derive(Clone, Debug)]
pub struct FenceReconciliationObservation {
    input: FenceReconciliationObservationInput,
}
impl FenceReconciliationObservation {
    /// Admit bounded chronological attempt IDs and canonical exact actual selected targets.
    /// # Errors
    /// Rejects empty/excessive/duplicate/unknown targets or impossible attempt IDs.
    pub fn new(
        mut input: FenceReconciliationObservationInput,
    ) -> Result<Self, FenceReconciliationObservationError> {
        if input.mutation_attempt == 0
            || input.observation_attempt <= input.mutation_attempt
            || input.observation_attempt > MAX_OPERATION_ATTEMPTS
        {
            return Err(FenceReconciliationObservationError::InvalidAttempts);
        }
        if input.selected_targets.is_empty() || input.selected_targets.len() > MAX_INVENTORY_TARGETS
        {
            return Err(FenceReconciliationObservationError::InvalidTargetCount);
        }
        for target in &mut input.selected_targets {
            *target = input.inventory.target(target)?.canister_id().into();
        }
        input.selected_targets.sort();
        if input
            .selected_targets
            .windows(2)
            .any(|pair| pair[0] == pair[1])
        {
            return Err(FenceReconciliationObservationError::DuplicateTarget);
        }
        Ok(Self { input })
    }
    /// Read immutable admitted passive fields; not authenticated by this accessor.
    #[must_use]
    pub const fn input(&self) -> &FenceReconciliationObservationInput {
        &self.input
    }
}

/// Typed original journal/reservation binding denial; never consumes or replenishes attempts.
#[derive(Debug, Error)]
pub enum FenceReconciliationRequestError {
    /// Exact original journal context, operation, request or limits differ.
    #[error("fence reconciliation original journal authority mismatch")]
    AuthorityMismatch,
    /// No original mutation is unresolved.
    #[error("fence reconciliation requires a pending acquisition mutation")]
    NoPendingMutation,
    /// Original acquisition reservation changed or settled.
    #[error("fence reconciliation pending acquisition mismatch")]
    MutationMismatch,
    /// The exact observation was not already reserved.
    #[error("fence reconciliation requires a pending observation reservation")]
    NoPendingObservation,
    /// The retained observation changed or settled.
    #[error("fence reconciliation observation attempt mismatch")]
    ObservationMismatch,
    /// The pending reservation binds another original observation digest.
    #[error("fence reconciliation reserved request mismatch")]
    RequestMismatch,
    /// Full original fence obligation is not bound to the plan.
    #[error(transparent)]
    Obligation(#[from] FenceObligationError),
    /// Original acquisition authority cannot be derived.
    #[error(transparent)]
    Plan(#[from] OperationPlanError),
}
/// Typed actual passive observation boundary denial.
#[derive(Debug, Error)]
pub enum FenceReconciliationObservationError {
    /// Attempts must be chronological and within the existing total-attempt bound.
    #[error("invalid fence reconciliation attempt identities")]
    InvalidAttempts,
    /// Actual whole-unit selection must contain 1..=1,024 members.
    #[error("invalid fence reconciliation target count")]
    InvalidTargetCount,
    /// Equivalent physical IDs cannot appear twice.
    #[error("duplicate fence reconciliation target")]
    DuplicateTarget,
    /// Physical selection cannot be admitted against actual inventory.
    #[error(transparent)]
    Inventory(#[from] InventoryRecordError),
}

#[cfg(test)]
mod tests;
