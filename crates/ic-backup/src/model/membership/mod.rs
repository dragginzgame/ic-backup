//! Ephemeral membership checks bound to original intent; no persisted fresh-authority flags.

use crate::model::{
    artifacts::ArtifactChecksumRecord,
    attempt_journal::OperationBindingRecord,
    inventory::InventoryRecord,
    operation_plan::{OperationPlanError, OperationPlanRecord, PlanContextRecord},
};
use thiserror::Error;

/// Maximum remote observations described by one membership request, without spending authority.
pub const MAX_MEMBERSHIP_REMOTE_OBSERVATIONS: u32 = 1024;

/// Explicit effect boundary being checked; neither variant admits an effect.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MembershipBoundary {
    /// Current membership before one reviewed effect.
    BeforeEffect,
    /// Current membership after that effect; not proof of uninterrupted membership.
    AfterEffect,
}

/// Immutable ephemeral request derived from the original plan and exact operation.
///
/// The integration supplies a fresh unpredictable challenge and owns its uniqueness,
/// current observation timing, coherent custody and separately approved observation
/// spending. The call ceiling is descriptive, not an allowance or journal reservation.
/// This type has no serialization, persistence or default-provider admission.
#[derive(Clone, Debug)]
pub struct MembershipObservationRequest<'a> {
    plan: &'a OperationPlanRecord,
    binding: OperationBindingRecord,
    challenge: ArtifactChecksumRecord,
    boundary: MembershipBoundary,
    max_remote_observations: u32,
}

impl<'a> MembershipObservationRequest<'a> {
    /// Bind a challenge, explicit boundary and bounded call ceiling to original intent.
    ///
    /// Zero remote calls permits a qualified local integration; it cannot authorize
    /// a paid probe. Original mutation/reconciliation allowances are never changed.
    /// # Errors
    /// Rejects unknown operations or an excessive descriptive call ceiling.
    pub fn new(
        plan: &'a OperationPlanRecord,
        operation_sequence: u64,
        challenge: ArtifactChecksumRecord,
        boundary: MembershipBoundary,
        max_remote_observations: u32,
    ) -> Result<Self, MembershipRequestError> {
        if max_remote_observations > MAX_MEMBERSHIP_REMOTE_OBSERVATIONS {
            return Err(MembershipRequestError::ObservationLimitTooLarge);
        }
        let binding = plan
            .attempt_authority(operation_sequence)?
            .binding()
            .clone();
        Ok(Self {
            plan,
            binding,
            challenge,
            boundary,
            max_remote_observations,
        })
    }
    /// Read original intent, operation, context, target and mutation-payload identity.
    #[must_use]
    pub const fn binding(&self) -> &OperationBindingRecord {
        &self.binding
    }
    /// Read the full expected declared inventory, including unselected parents.
    #[must_use]
    pub const fn inventory(&self) -> &InventoryRecord {
        self.plan.inventory()
    }
    /// Read exact canonical selected principals; this is not lifecycle order.
    #[must_use]
    pub fn selected_targets(&self) -> &[String] {
        self.plan.selected_targets()
    }
    /// Read the integration-owned challenge; its value alone establishes no freshness.
    #[must_use]
    pub const fn challenge(&self) -> &ArtifactChecksumRecord {
        &self.challenge
    }
    /// Read the explicit effect boundary.
    #[must_use]
    pub const fn boundary(&self) -> MembershipBoundary {
        self.boundary
    }
    /// Read the descriptive remote-call ceiling, not a dispatch or spending permit.
    #[must_use]
    pub const fn max_remote_observations(&self) -> u32 {
        self.max_remote_observations
    }
    /// Hash original full plan intent, operation, challenge, boundary and call ceiling.
    ///
    /// Encoding uses the NUL-terminated ASCII v1 domain, canonical 64-byte ASCII
    /// intent, big-endian u64 sequence, canonical 64-byte ASCII challenge, boundary
    /// byte (before=0, after=1), then big-endian u32 descriptive call ceiling.
    /// Context/inventory/selection/request/original allowances bind through full
    /// original intent. Observation results, revision and evidence are excluded.
    #[must_use]
    pub fn digest(&self) -> ArtifactChecksumRecord {
        let mut bytes = b"ic-backup/membership-request/v1\0".to_vec();
        bytes.extend_from_slice(self.binding.intent().as_bytes());
        bytes.extend_from_slice(&self.binding.operation_sequence().to_be_bytes());
        bytes.extend_from_slice(self.challenge.hash().as_bytes());
        bytes.push(match self.boundary {
            MembershipBoundary::BeforeEffect => 0,
            MembershipBoundary::AfterEffect => 1,
        });
        bytes.extend_from_slice(&self.max_remote_observations.to_be_bytes());
        ArtifactChecksumRecord::from_bytes(&bytes)
    }
}

/// Passive current-observation result supplied by a trusted membership integration.
///
/// No Serialize/Deserialize implementation exists: retained JSON is not a current
/// observation. The caller must qualify the provider and its actual observations;
/// these fields and opaque evidence digests are not self-authenticating.
#[derive(Clone, Debug)]
pub struct MembershipObservation {
    /// Exact digest of the current challenge/boundary-bound request.
    pub request: ArtifactChecksumRecord,
    /// Actually observed network/caller/release, not copied expected labels.
    pub context: PlanContextRecord,
    /// Complete current inventory under the existing 1,024-target forest bound.
    pub inventory: InventoryRecord,
    /// Optional opaque current authority revision; equality proves no continuity.
    pub revision: Option<ArtifactChecksumRecord>,
    /// Exact opaque evidence identifier qualified by the integration owner.
    pub evidence: ArtifactChecksumRecord,
    /// Actual remote observations; each needs separate prior approved accounting.
    pub remote_observations: u32,
}

/// Typed rejection before requesting any provider observation.
#[derive(Debug, Error)]
pub enum MembershipRequestError {
    /// Descriptive remote-call ceiling exceeds the maintained bound.
    #[error("membership call ceiling exceeds {MAX_MEMBERSHIP_REMOTE_OBSERVATIONS}")]
    ObservationLimitTooLarge,
    /// Original plan cannot derive the named operation binding.
    #[error(transparent)]
    Plan(#[from] OperationPlanError),
}

#[cfg(test)]
mod tests;
