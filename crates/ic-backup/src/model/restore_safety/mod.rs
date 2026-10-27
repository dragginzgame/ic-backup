//! Original restore/source requirements and ephemeral application-owned safety evidence.

mod requirement;
pub use requirement::{
    MAX_RESTORE_SAFETY_REQUIREMENT_BYTES, RestoreFenceBindingRecord, RestoreSafetyLaneRecord,
    RestoreSafetyRequirementError, RestoreSafetyRequirementRecord, RestoreSafetyRequirementRequest,
};

use crate::model::{
    artifacts::ArtifactChecksumRecord,
    attempt_journal::OperationBindingRecord,
    consistency::ApplicationFenceState,
    evidence_digest::EvidenceDigest,
    ic_request::{IcManagementMethodRecord, IcManagementRequestRecord, IcRequestError},
    inventory::{InventoryRecord, MAX_INVENTORY_TARGETS},
    operation_plan::{OperationPlanError, OperationPlanRecord, PlanContextRecord},
};
use ic_platform::management_canister_types::CanisterStatusType;
use thiserror::Error;

/// Maximum descriptive remote observations per invocation; no spending allowance.
pub const MAX_RESTORE_SAFETY_REMOTE_OBSERVATIONS: u32 = 1024;
/// Passive current parameters; the exact load/start boundary is derived from original wire bytes.
#[derive(Clone, Debug)]
pub struct RestoreSafetyRequestInput {
    /// Exact original load/start operation sequence.
    pub operation_sequence: u64,
    /// Integration-owned fresh challenge, qualified independently of hash equality.
    pub challenge: ArtifactChecksumRecord,
    /// Descriptive call ceiling; every actual call needs separate prior accounting.
    pub max_remote_observations: u32,
}
/// Ephemeral original-source/safety/challenge request; no dispatch or paid-call permit.
#[derive(Clone, Debug)]
pub struct RestoreSafetyRequest<'a> {
    plan: &'a OperationPlanRecord,
    requirement: &'a RestoreSafetyRequirementRecord,
    binding: OperationBindingRecord,
    wire: &'a IcManagementRequestRecord,
    input: RestoreSafetyRequestInput,
}
impl<'a> RestoreSafetyRequest<'a> {
    /// Validate exact original plans, requirement and load/start mutation bytes.
    ///
    /// The integration admits the retained requirement before effect-boundary
    /// use and retains original source artifacts and fence obligations. This
    /// constructor performs no IO and cannot prove that custody.
    /// # Errors
    /// Rejects changed source/plan/payload, other methods or excessive descriptive calls.
    pub fn new(
        plan: &'a OperationPlanRecord,
        source: &OperationPlanRecord,
        requirement: &'a RestoreSafetyRequirementRecord,
        wire: &'a IcManagementRequestRecord,
        input: RestoreSafetyRequestInput,
    ) -> Result<Self, RestoreSafetyRequestError> {
        requirement.validate_plans(plan, source)?;
        if input.max_remote_observations > MAX_RESTORE_SAFETY_REMOTE_OBSERVATIONS {
            return Err(RestoreSafetyRequestError::ObservationLimitTooLarge);
        }
        if !matches!(
            wire.method(),
            IcManagementMethodRecord::LoadCanisterSnapshot
                | IcManagementMethodRecord::StartCanister
        ) {
            return Err(RestoreSafetyRequestError::UnsupportedMethod);
        }
        let binding = plan.operation_binding(input.operation_sequence)?;
        wire.validate_mutation_binding(&binding)?;
        Ok(Self {
            plan,
            requirement,
            binding,
            wire,
            input,
        })
    }
    /// Read exact original operation, context, target and request identity.
    #[must_use]
    pub const fn binding(&self) -> &OperationBindingRecord {
        &self.binding
    }
    /// Read original full inventory, including unselected metadata.
    #[must_use]
    pub const fn inventory(&self) -> &InventoryRecord {
        self.plan.inventory()
    }
    /// Read exact original selected IDs in canonical order, not dispatch order.
    #[must_use]
    pub fn selected_targets(&self) -> &[String] {
        self.plan.selected_targets()
    }
    /// Read original immutable source and safety requirement.
    #[must_use]
    pub const fn requirement(&self) -> &RestoreSafetyRequirementRecord {
        self.requirement
    }
    /// Read exact original load/start wire request; codecs confer no load settlement.
    #[must_use]
    pub const fn wire(&self) -> &IcManagementRequestRecord {
        self.wire
    }
    /// Read challenge; matching values alone prove no freshness.
    #[must_use]
    pub const fn challenge(&self) -> &ArtifactChecksumRecord {
        &self.input.challenge
    }
    /// Read descriptive call ceiling without spending/replenishment.
    #[must_use]
    pub const fn max_remote_observations(&self) -> u32 {
        self.input.max_remote_observations
    }
    /// Hash v1 NUL-terminated ASCII domain, 64 ASCII requirement-digest bytes,
    /// u64 BE sequence, 64 ASCII wire-digest bytes, 64 ASCII challenge bytes and
    /// u32 BE descriptive ceiling. Wire digest distinguishes load from start.
    #[must_use]
    pub fn digest(&self) -> ArtifactChecksumRecord {
        let mut framing = EvidenceDigest::new(b"ic-backup/restore-safety-request/v1\0");
        framing.checksum(&self.requirement.digest());
        framing.u64_be(self.binding.operation_sequence());
        framing.checksum(&self.wire.digest());
        framing.checksum(self.challenge());
        framing.u32_be(self.max_remote_observations());
        framing.finish()
    }
}
/// Passive actual lifecycle and integration-qualified source-specific restored acceptance.
#[derive(Clone, Debug)]
pub struct TargetRestoreEvidence {
    /// Actual physical target, normalized at model admission.
    pub target: String,
    /// Actual upstream lifecycle state; never inferred from a process exit status.
    pub state: CanisterStatusType,
    /// Evidence qualifying lifecycle and, when stopped, drained application work.
    pub lifecycle_evidence: ArtifactChecksumRecord,
    /// Current application acceptance of this exact source's restored state, required before start.
    /// A module hash or load acknowledgement alone cannot supply this evidence.
    pub restored_acceptance: Option<ArtifactChecksumRecord>,
}
impl super::observed_targets::ObservedTargetRow for TargetRestoreEvidence {
    fn target(&self) -> &str {
        &self.target
    }

    fn target_mut(&mut self) -> &mut String {
        &mut self.target
    }
}
/// Passive exact fence evidence; every opaque field requires application qualification.
#[derive(Clone, Debug)]
pub struct RestoreFenceEvidence {
    /// Actually observed active/inactive custody; unknown custody fails at the provider.
    pub state: ApplicationFenceState,
    /// Actually observed fence and current authority revisions, matched to original retention.
    pub binding: RestoreFenceBindingRecord,
    /// Whole-selection write, membership, timer, external-work fencing and drained-work evidence.
    pub whole_selection: ArtifactChecksumRecord,
    /// Fence and obligation custody outside the source's rewindable state, across failure/recovery.
    pub rewind_independent_custody: ArtifactChecksumRecord,
    /// Qualified external-obligation disposition and prevention of replayed irreversible work.
    /// Settlement alone is insufficient if restored intent can replay that work.
    pub external_obligations_and_replay: ArtifactChecksumRecord,
    /// Current allowance for isolated restored execution under the retained fence, required before start.
    /// This is application evidence, not a release token, future guarantee or dispatch permit.
    pub controlled_execution: Option<ArtifactChecksumRecord>,
}
/// Actual application safety lane; no generic default or serialized Proven admission.
#[derive(Clone, Debug)]
pub enum RestoreSafetyEvidence {
    /// Qualified absence of irreversible external effects, including restored intent/timers.
    NoIrreversibleEffects(ArtifactChecksumRecord),
    /// Qualified whole-selection custody outside rewindable state and external-work replay safety.
    ApplicationFenced(Box<RestoreFenceEvidence>),
    /// Known unsafe or unresolved external obligations; pure policy rejects this evidence.
    Unresolved(ArtifactChecksumRecord),
}
/// Passive provider data; exact original context/source and current evidence are independent inputs.
#[derive(Clone, Debug)]
pub struct RestoreSafetyObservationInput {
    /// Exact current challenge-bound request digest.
    pub request: ArtifactChecksumRecord,
    /// Actual current canonical network/caller/release, not echoed expected labels.
    pub context: PlanContextRecord,
    /// Complete actual current inventory, including unselected parent metadata.
    pub inventory: InventoryRecord,
    /// Qualified exact original source plan identity.
    pub source_plan_intent: ArtifactChecksumRecord,
    /// Qualified complete original source artifacts in stable retained custody,
    /// including exact target-local load snapshot association when checking load.
    pub source_artifacts: ArtifactChecksumRecord,
    /// Actual lifecycle and restored acceptance for the exact selected set.
    pub targets: Vec<TargetRestoreEvidence>,
    /// Actual application safety lane and evidence for this exact source/selection/boundary.
    pub safety: RestoreSafetyEvidence,
    /// Opaque integration-qualified evidence identity; no signature or permission.
    pub evidence: ArtifactChecksumRecord,
    /// Actual remote observations, each needing separate prior spending authority.
    pub remote_observations: u32,
}
/// Canonical immutable ephemeral observation; no Serde, expiry or persisted authority flag.
#[derive(Clone, Debug)]
pub struct RestoreSafetyObservation {
    input: RestoreSafetyObservationInput,
}
impl RestoreSafetyObservation {
    /// Normalize/sort nonempty bounded unique actual targets under the actual full inventory.
    /// # Errors
    /// Rejects excessive/empty rows, malformed principals, duplicates or unknown targets.
    pub fn new(
        mut input: RestoreSafetyObservationInput,
    ) -> Result<Self, RestoreSafetyObservationError> {
        input.targets = super::observed_targets::canonicalize(
            input.targets,
            &input.inventory,
            || RestoreSafetyObservationError::InvalidTargetCount,
            || RestoreSafetyObservationError::InvalidPrincipal,
            || RestoreSafetyObservationError::TargetAbsentFromInventory,
            || RestoreSafetyObservationError::DuplicateTarget,
        )?;
        Ok(Self { input })
    }
    /// Read current request identity.
    #[must_use]
    pub const fn request(&self) -> &ArtifactChecksumRecord {
        &self.input.request
    }
    /// Read actual canonical current context.
    #[must_use]
    pub const fn context(&self) -> &PlanContextRecord {
        &self.input.context
    }
    /// Read complete actual inventory.
    #[must_use]
    pub const fn inventory(&self) -> &InventoryRecord {
        &self.input.inventory
    }
    /// Read actual qualified source plan identity.
    #[must_use]
    pub const fn source_plan_intent(&self) -> &ArtifactChecksumRecord {
        &self.input.source_plan_intent
    }
    /// Read actual qualified source artifact binding.
    #[must_use]
    pub const fn source_artifacts(&self) -> &ArtifactChecksumRecord {
        &self.input.source_artifacts
    }
    /// Read canonical actual selected rows.
    #[must_use]
    pub fn targets(&self) -> &[TargetRestoreEvidence] {
        &self.input.targets
    }
    /// Read current application-qualified safety lane.
    #[must_use]
    pub const fn safety(&self) -> &RestoreSafetyEvidence {
        &self.input.safety
    }
    /// Read opaque qualified evidence.
    #[must_use]
    pub const fn evidence(&self) -> &ArtifactChecksumRecord {
        &self.input.evidence
    }
    /// Read actual descriptive calls, without new authority or accounting.
    #[must_use]
    pub const fn remote_observations(&self) -> u32 {
        self.input.remote_observations
    }
}
/// Typed owning-boundary current evidence admission denial.
#[derive(Debug, Eq, Error, PartialEq)]
pub enum RestoreSafetyObservationError {
    /// Actual rows must be nonempty and within the inventory bound.
    #[error("restore safety targets must contain 1..={MAX_INVENTORY_TARGETS} entries")]
    InvalidTargetCount,
    /// Target principal failed bounded normalization.
    #[error("invalid restore safety target principal")]
    InvalidPrincipal,
    /// Canonical equivalent targets cannot appear twice.
    #[error("duplicate restore safety target")]
    DuplicateTarget,
    /// Actual row is absent from the actual full inventory.
    #[error("restore safety target absent from inventory")]
    TargetAbsentFromInventory,
}
/// Typed original request denial; changes no source, fence, spending or journal.
#[derive(Debug, Error)]
pub enum RestoreSafetyRequestError {
    /// Original source/safety/plan declarations differ.
    #[error(transparent)]
    Requirement(#[from] RestoreSafetyRequirementError),
    /// Exact original operation cannot be derived.
    #[error(transparent)]
    Plan(#[from] OperationPlanError),
    /// Exact original mutation bytes/target differ.
    #[error(transparent)]
    Payload(#[from] IcRequestError),
    /// Only load/start are within this safety boundary.
    #[error("restore safety requires exact load or start request")]
    UnsupportedMethod,
    /// Descriptive observation ceiling is excessive.
    #[error("restore safety observation ceiling exceeds {MAX_RESTORE_SAFETY_REMOTE_OBSERVATIONS}")]
    ObservationLimitTooLarge,
}

#[cfg(test)]
mod tests;
