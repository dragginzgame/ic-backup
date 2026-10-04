//! Original declared consistency and ephemeral current capture/fence evidence.

mod requirement;
pub use requirement::{
    ConsistencyGuaranteeRecord, ConsistencyRequirementError, ConsistencyRequirementRecord,
    MAX_CONSISTENCY_REQUIREMENT_BYTES,
};

use crate::model::{
    artifacts::ArtifactChecksumRecord,
    attempt_journal::OperationBindingRecord,
    inventory::{InventoryRecord, MAX_INVENTORY_TARGETS},
    operation_plan::{OperationPlanError, OperationPlanRecord, PlanContextRecord},
};
use thiserror::Error;

/// Maximum descriptive remote observations per invocation; no paid-call allowance.
pub const MAX_CONSISTENCY_REMOTE_OBSERVATIONS: u32 = 1024;
/// Explicit capture boundary; before/after equality alone establishes no continuity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConsistencyBoundary {
    /// Check current evidence before capture under independently qualified effect admission.
    BeforeCapture,
    /// Check current evidence after capture; not a completion receipt.
    AfterCapture,
}
/// Exact integration-retained application fence and original membership revision.
///
/// Integrations recover this binding from durable obligation evidence; constructing
/// it proves neither retention nor current active custody and grants no release authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationFenceBinding {
    /// Exact retained fence identity, never a release token.
    pub identity: ArtifactChecksumRecord,
    /// Original membership revision bound when the retained fence was established.
    pub membership_revision: ArtifactChecksumRecord,
}
/// Passive request parameters; no default capture or fence authority.
#[derive(Clone, Debug)]
pub struct ConsistencyRequestInput {
    /// Exact original operation sequence; request payload remains bound through full intent.
    pub operation_sequence: u64,
    /// Fresh integration-owned challenge; digest binding alone proves no freshness.
    pub challenge: ArtifactChecksumRecord,
    /// Explicit capture boundary.
    pub boundary: ConsistencyBoundary,
    /// Retained exact fence/revision binding required only for coordinated capture.
    pub expected_fence: Option<ApplicationFenceBinding>,
    /// Descriptive call ceiling, independent of any original or preflight spending allowance.
    pub max_remote_observations: u32,
}
/// Ephemeral current request under the exact original consistency declaration and plan.
#[derive(Clone, Debug)]
pub struct ConsistencyRequest<'a> {
    plan: &'a OperationPlanRecord,
    requirement: &'a ConsistencyRequirementRecord,
    binding: OperationBindingRecord,
    input: ConsistencyRequestInput,
}
impl<'a> ConsistencyRequest<'a> {
    /// Bind original requirement/operation to an explicit challenge/boundary/fence identity.
    ///
    /// The integration recovers the expected fence from its durable obligation owner.
    /// This constructor cannot acquire/recover/release a fence or inspect that evidence.
    /// # Errors
    /// Rejects plan mismatch, unknown operation, inappropriate/missing fence or excess ceiling.
    pub fn new(
        plan: &'a OperationPlanRecord,
        requirement: &'a ConsistencyRequirementRecord,
        input: ConsistencyRequestInput,
    ) -> Result<Self, ConsistencyRequestError> {
        requirement.validate_plan(plan)?;
        if input.max_remote_observations > MAX_CONSISTENCY_REMOTE_OBSERVATIONS {
            return Err(ConsistencyRequestError::ObservationLimitTooLarge);
        }
        match (requirement.guarantee(), input.expected_fence.as_ref()) {
            (ConsistencyGuaranteeRecord::ApplicationCoordinated, None) => {
                return Err(ConsistencyRequestError::FenceRequired);
            }
            (ConsistencyGuaranteeRecord::PerCanister, Some(_)) => {
                return Err(ConsistencyRequestError::UnexpectedFence);
            }
            _ => {}
        }
        let binding = plan
            .attempt_authority(input.operation_sequence)?
            .binding()
            .clone();
        Ok(Self {
            plan,
            requirement,
            binding,
            input,
        })
    }
    /// Read exact original mutation identity; no codec, dispatch or current permission admission.
    #[must_use]
    pub const fn binding(&self) -> &OperationBindingRecord {
        &self.binding
    }
    /// Read full original inventory, including unselected parents/metadata.
    #[must_use]
    pub const fn inventory(&self) -> &InventoryRecord {
        self.plan.inventory()
    }
    /// Read exact original selected principals in canonical order, not dispatch order.
    #[must_use]
    pub fn selected_targets(&self) -> &[String] {
        self.plan.selected_targets()
    }
    /// Read immutable original requested guarantee.
    #[must_use]
    pub const fn requirement(&self) -> &ConsistencyRequirementRecord {
        self.requirement
    }
    /// Read caller-owned challenge; matching digests alone are not freshness proof.
    #[must_use]
    pub const fn challenge(&self) -> &ArtifactChecksumRecord {
        &self.input.challenge
    }
    /// Read explicit capture boundary.
    #[must_use]
    pub const fn boundary(&self) -> ConsistencyBoundary {
        self.input.boundary
    }
    /// Read retained exact expected fence/revision binding; not proof of current active custody.
    #[must_use]
    pub const fn expected_fence(&self) -> Option<&ApplicationFenceBinding> {
        self.input.expected_fence.as_ref()
    }
    /// Read descriptive call ceiling; grants no spending/retry admission.
    #[must_use]
    pub const fn max_remote_observations(&self) -> u32 {
        self.input.max_remote_observations
    }
    /// Hash original requirement, operation, challenge, boundary, expected fence and ceiling.
    ///
    /// Domain is NUL-terminated ASCII v1, then 64 ASCII requirement-digest bytes,
    /// u64 BE sequence, 64 ASCII challenge bytes, boundary byte (before=0/after=1),
    /// fence presence byte (0/1), optional 64 ASCII identity and 64 ASCII original
    /// membership-revision bytes, then u32 BE ceiling.
    #[must_use]
    pub fn digest(&self) -> ArtifactChecksumRecord {
        let mut bytes = b"ic-backup/consistency-request/v1\0".to_vec();
        bytes.extend_from_slice(self.requirement.digest().hash().as_bytes());
        bytes.extend_from_slice(&self.binding.operation_sequence().to_be_bytes());
        bytes.extend_from_slice(self.challenge().hash().as_bytes());
        bytes.push(match self.boundary() {
            ConsistencyBoundary::BeforeCapture => 0,
            ConsistencyBoundary::AfterCapture => 1,
        });
        match self.expected_fence() {
            None => bytes.push(0),
            Some(fence) => {
                bytes.push(1);
                bytes.extend_from_slice(fence.identity.hash().as_bytes());
                bytes.extend_from_slice(fence.membership_revision.hash().as_bytes());
            }
        }
        bytes.extend_from_slice(&self.max_remote_observations().to_be_bytes());
        ArtifactChecksumRecord::from_bytes(&bytes)
    }
}
/// Actual observed lifecycle at the capture boundary; not a management reply codec.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CaptureState {
    /// Target is currently running; deny capture consistency admission.
    Running,
    /// Target has not completed stopping; deny capture consistency admission.
    Stopping,
    /// Target is observed stopped; integration must also qualify drained-work evidence.
    Stopped,
}
/// Passive exact target state and opaque stopped/drained evidence qualified by its owner.
#[derive(Clone, Debug)]
pub struct TargetCaptureEvidence {
    /// Actually observed physical target, canonicalized at observation admission.
    pub target: String,
    /// Actually observed lifecycle; process success cannot substitute.
    pub state: CaptureState,
    /// Required integration-qualified stopped/drained-work evidence; digest alone is no proof.
    pub stopped_and_drained: ArtifactChecksumRecord,
}
/// Actually observed application fence state; not a serialized accepted/proven flag.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationFenceState {
    /// Exact retained fence is qualified currently active with continuous custody.
    Active,
    /// Exact retained fence is known inactive; deny coordinated consistency.
    Inactive,
}
/// Passive whole-selection fence evidence qualified by the application integration.
///
/// All evidence applies to the exact current request/context/full inventory/selection.
/// The integration proves continuous retained custody across the capture window and
/// interruption. Matching identities/revisions alone cannot prove that property.
#[derive(Clone, Debug)]
pub struct ApplicationFenceEvidence {
    /// Actually observed active/inactive state; unknown custody must fail at the provider.
    pub state: ApplicationFenceState,
    /// Exact active retained fence identity; never a release token.
    pub identity: ArtifactChecksumRecord,
    /// Qualified current membership authority revision bound to this active fence.
    pub membership_revision: ArtifactChecksumRecord,
    /// Evidence application writes are fenced for the whole selected unit.
    pub writes: ArtifactChecksumRecord,
    /// Evidence membership changes are fenced for the whole selected unit.
    pub membership: ArtifactChecksumRecord,
    /// Evidence relevant application timers are fenced.
    pub timers: ArtifactChecksumRecord,
    /// Evidence relevant external work is fenced; not restore/payment settlement.
    pub external_work: ArtifactChecksumRecord,
    /// Evidence application work has drained under retained fence custody.
    pub drained_work: ArtifactChecksumRecord,
}
/// Current declared evidence lane; neither variant is self-authenticating.
#[derive(Clone, Debug)]
pub enum ConsistencyEvidence {
    /// Only each exact stopped target is qualified; no atomic application claim.
    PerCanister,
    /// Application integration qualifies this exact currently active whole-selection fence.
    ApplicationCoordinated(Box<ApplicationFenceEvidence>),
}
/// Passive provider data; accepted flags, expiry/defaults and JSON authority admission are absent.
#[derive(Clone, Debug)]
pub struct ConsistencyObservationInput {
    /// Exact current request digest.
    pub request: ArtifactChecksumRecord,
    /// Actually observed canonical network/caller/release.
    pub context: PlanContextRecord,
    /// Complete actually observed current inventory, including unselected parent metadata.
    pub inventory: InventoryRecord,
    /// Exact actual selected target states/evidence; admitted canonically by the model.
    pub targets: Vec<TargetCaptureEvidence>,
    /// Actually observed optional membership revision; coordinated evidence requires a match.
    pub membership_revision: Option<ArtifactChecksumRecord>,
    /// Actual current evidence lane; policy requires the original declared guarantee.
    pub consistency: ConsistencyEvidence,
    /// Required opaque integration-qualified observation evidence.
    pub evidence: ArtifactChecksumRecord,
    /// Actual remote observations, each requiring separate prior approved accounting.
    pub remote_observations: u32,
}
/// Immutable canonical current evidence; not persisted proof or a fence-release capability.
#[derive(Clone, Debug)]
pub struct ConsistencyObservation {
    input: ConsistencyObservationInput,
}
impl ConsistencyObservation {
    /// Normalize/sort exact targets, require a nonempty bounded unique inventory-backed set.
    /// # Errors
    /// Rejects target count, malformed identities, canonical duplicates or unknown targets.
    pub fn new(
        mut input: ConsistencyObservationInput,
    ) -> Result<Self, ConsistencyObservationError> {
        if input.targets.is_empty() || input.targets.len() > MAX_INVENTORY_TARGETS {
            return Err(ConsistencyObservationError::InvalidTargetCount);
        }
        for target in &mut input.targets {
            target.target = super::principal::canonical_text(&target.target)
                .ok_or(ConsistencyObservationError::InvalidPrincipal)?;
            if input.inventory.target(&target.target).is_err() {
                return Err(ConsistencyObservationError::TargetAbsentFromInventory);
            }
        }
        input.targets.sort_by(|a, b| a.target.cmp(&b.target));
        if input
            .targets
            .windows(2)
            .any(|pair| pair[0].target == pair[1].target)
        {
            return Err(ConsistencyObservationError::DuplicateTarget);
        }
        Ok(Self { input })
    }
    /// Read current request identity.
    #[must_use]
    pub const fn request(&self) -> &ArtifactChecksumRecord {
        &self.input.request
    }
    /// Read actually observed canonical context.
    #[must_use]
    pub const fn context(&self) -> &PlanContextRecord {
        &self.input.context
    }
    /// Read complete actual current inventory.
    #[must_use]
    pub const fn inventory(&self) -> &InventoryRecord {
        &self.input.inventory
    }
    /// Read exact canonical actual target state/evidence rows.
    #[must_use]
    pub fn targets(&self) -> &[TargetCaptureEvidence] {
        &self.input.targets
    }
    /// Read actual membership revision; equality alone grants no continuity.
    #[must_use]
    pub const fn membership_revision(&self) -> Option<&ArtifactChecksumRecord> {
        self.input.membership_revision.as_ref()
    }
    /// Read current evidence lane.
    #[must_use]
    pub const fn consistency(&self) -> &ConsistencyEvidence {
        &self.input.consistency
    }
    /// Read opaque qualified observation evidence.
    #[must_use]
    pub const fn evidence(&self) -> &ArtifactChecksumRecord {
        &self.input.evidence
    }
    /// Read actual call reporting without spending/replenishment.
    #[must_use]
    pub const fn remote_observations(&self) -> u32 {
        self.input.remote_observations
    }
}
/// Typed current canonical evidence admission failure.
#[derive(Debug, Eq, Error, PartialEq)]
pub enum ConsistencyObservationError {
    /// Exact actual targets must fit the inventory bound and be nonempty.
    #[error("consistency targets must contain 1..={MAX_INVENTORY_TARGETS} entries")]
    InvalidTargetCount,
    /// Target principal is invalid or exceeds its own bound.
    #[error("invalid consistency target principal")]
    InvalidPrincipal,
    /// Equivalent target identities cannot appear twice.
    #[error("duplicate consistency target")]
    DuplicateTarget,
    /// Actual target is not in the actually observed inventory.
    #[error("consistency target absent from actual inventory")]
    TargetAbsentFromInventory,
}
/// Typed original requirement/request denial; no failure acquires/releases a fence.
#[derive(Debug, Error)]
pub enum ConsistencyRequestError {
    /// Original requirement does not match the original plan.
    #[error(transparent)]
    Requirement(#[from] ConsistencyRequirementError),
    /// Original plan rejects exact operation derivation.
    #[error(transparent)]
    Plan(#[from] OperationPlanError),
    /// Coordinated capture must name the retained exact fence obligation.
    #[error("coordinated consistency requires retained fence identity")]
    FenceRequired,
    /// Per-canister guarantee cannot silently acquire/upgrade an application fence.
    #[error("per-canister consistency does not admit expected fence identity")]
    UnexpectedFence,
    /// Descriptive call ceiling exceeds its maintained bound.
    #[error("consistency observation ceiling exceeds {MAX_CONSISTENCY_REMOTE_OBSERVATIONS}")]
    ObservationLimitTooLarge,
}

#[cfg(test)]
mod tests;
