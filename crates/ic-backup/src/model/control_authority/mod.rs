//! Ephemeral direct-controller observations for exact IC mutation payloads; no dispatch permits.

use crate::model::{
    artifacts::ArtifactChecksumRecord,
    attempt_journal::OperationBindingRecord,
    ic_request::{IcManagementRequestRecord, IcRequestError},
    operation_plan::{OperationPlanError, OperationPlanRecord, PlanContextRecord},
};
use thiserror::Error;

/// Maximum controllers in the maintained IC controller-set boundary.
pub const MAX_CONTROLLERS: usize = 10;
/// Maximum descriptive remote observations per request; never spending authority.
pub const MAX_CONTROL_REMOTE_OBSERVATIONS: u32 = 1024;

/// Canonical bounded known controller set, including an explicitly known empty set.
///
/// This validates identities, not observation authenticity. Missing/unknown controller
/// evidence must fail at the provider boundary, never default into a success result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ControllerSet {
    principals: Vec<String>,
}
impl ControllerSet {
    /// Normalize and sort exact principals; equivalent duplicate entries reject.
    /// # Errors
    /// Rejects excessive counts, malformed principals and canonical duplicates.
    pub fn new(mut principals: Vec<String>) -> Result<Self, ControlObservationError> {
        if principals.len() > MAX_CONTROLLERS {
            return Err(ControlObservationError::TooManyControllers);
        }
        for principal in &mut principals {
            *principal = super::principal::canonical_text(principal)
                .ok_or(ControlObservationError::InvalidPrincipal)?;
        }
        principals.sort();
        if principals.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(ControlObservationError::DuplicateController);
        }
        Ok(Self { principals })
    }
    /// Read canonical controllers; ordering grants no routing or lifecycle semantics.
    #[must_use]
    pub fn principals(&self) -> &[String] {
        &self.principals
    }
    /// Check the exact canonical original caller, without delegating through other controllers.
    #[must_use]
    pub fn contains_caller(&self, binding: &OperationBindingRecord) -> bool {
        self.principals
            .binary_search_by(|principal| principal.as_str().cmp(binding.caller()))
            .is_ok()
    }
}

/// Ephemeral immutable request bound to original intent and exact host-ingress mutation bytes.
///
/// The integration owns challenge freshness, authenticated observations and prior
/// per-call accounting. This request is only for direct caller-controller checks;
/// read visibility, Root proxies and subnet-admin exceptions grant no admission here.
#[derive(Clone, Debug)]
pub struct ControlObservationRequest<'a> {
    binding: OperationBindingRecord,
    wire: &'a IcManagementRequestRecord,
    challenge: ArtifactChecksumRecord,
    max_remote_observations: u32,
}
impl<'a> ControlObservationRequest<'a> {
    /// Derive original binding and validate exact mutation target/method/payload bytes.
    /// # Errors
    /// Rejects unknown operations, observation methods, changed payload/target or excessive ceiling.
    pub fn new(
        plan: &OperationPlanRecord,
        sequence: u64,
        wire: &'a IcManagementRequestRecord,
        challenge: ArtifactChecksumRecord,
        max_remote_observations: u32,
    ) -> Result<Self, ControlRequestError> {
        if max_remote_observations > MAX_CONTROL_REMOTE_OBSERVATIONS {
            return Err(ControlRequestError::ObservationLimitTooLarge);
        }
        let binding = plan.attempt_authority(sequence)?.binding().clone();
        wire.validate_mutation_binding(&binding)?;
        Ok(Self {
            binding,
            wire,
            challenge,
            max_remote_observations,
        })
    }
    /// Read exact original intent/context/target/payload identity.
    #[must_use]
    pub const fn binding(&self) -> &OperationBindingRecord {
        &self.binding
    }
    /// Read validated exact method/routing/Candid mutation bytes.
    #[must_use]
    pub const fn wire(&self) -> &IcManagementRequestRecord {
        self.wire
    }
    /// Read integration-owned challenge; its value alone proves no freshness.
    #[must_use]
    pub const fn challenge(&self) -> &ArtifactChecksumRecord {
        &self.challenge
    }
    /// Read descriptive call ceiling, separate from original spending allowances.
    #[must_use]
    pub const fn max_remote_observations(&self) -> u32 {
        self.max_remote_observations
    }
    /// Hash original intent/sequence, exact wire digest, challenge and descriptive call ceiling.
    ///
    /// Encoding: NUL-terminated ASCII v1 domain, 64 ASCII intent bytes, u64
    /// big-endian sequence, 64 ASCII wire-digest bytes, 64 ASCII challenge bytes,
    /// then u32 big-endian ceiling. Observed evidence/controllers are excluded.
    #[must_use]
    pub fn digest(&self) -> ArtifactChecksumRecord {
        let mut bytes = b"ic-backup/control-observation/v1\0".to_vec();
        bytes.extend_from_slice(self.binding.intent().as_bytes());
        bytes.extend_from_slice(&self.binding.operation_sequence().to_be_bytes());
        bytes.extend_from_slice(self.wire.digest().hash().as_bytes());
        bytes.extend_from_slice(self.challenge.hash().as_bytes());
        bytes.extend_from_slice(&self.max_remote_observations.to_be_bytes());
        ArtifactChecksumRecord::from_bytes(&bytes)
    }
}

/// Passive provider data; no permissive default or automatic JSON admission.
#[derive(Clone, Debug)]
pub struct ControlObservationInput {
    /// Exact current challenge-bound request digest.
    pub request: ArtifactChecksumRecord,
    /// Actually observed canonical network/caller/release, not echoed expected labels.
    pub context: PlanContextRecord,
    /// Actually observed physical target; normalized at model admission.
    pub target: String,
    /// Complete known controller set; public/read access is never a controller entry.
    pub controllers: ControllerSet,
    /// Opaque evidence digest qualified by the integration, not a signature or permission.
    pub evidence: ArtifactChecksumRecord,
    /// Actual calls, requiring separately approved prior per-call accounting.
    pub remote_observations: u32,
}
/// Immutable canonical ephemeral provider result; has no Serde or persisted authority lane.
#[derive(Clone, Debug)]
pub struct ControlObservation {
    input: ControlObservationInput,
}
impl ControlObservation {
    /// Admit canonical observed target; other fields are validated by their owning types.
    /// # Errors
    /// Rejects malformed observed target principals.
    pub fn new(mut input: ControlObservationInput) -> Result<Self, ControlObservationError> {
        input.target = super::principal::canonical_text(&input.target)
            .ok_or(ControlObservationError::InvalidPrincipal)?;
        Ok(Self { input })
    }
    /// Read exact current request identity.
    #[must_use]
    pub const fn request(&self) -> &ArtifactChecksumRecord {
        &self.input.request
    }
    /// Read actual observed context.
    #[must_use]
    pub const fn context(&self) -> &PlanContextRecord {
        &self.input.context
    }
    /// Read canonical actual target.
    #[must_use]
    pub fn target(&self) -> &str {
        &self.input.target
    }
    /// Read known controllers; an empty set never satisfies caller-control admission.
    #[must_use]
    pub const fn controllers(&self) -> &ControllerSet {
        &self.input.controllers
    }
    /// Read opaque integration evidence identifier.
    #[must_use]
    pub const fn evidence(&self) -> &ArtifactChecksumRecord {
        &self.input.evidence
    }
    /// Read reported calls; this changes no original allowance.
    #[must_use]
    pub const fn remote_observations(&self) -> u32 {
        self.input.remote_observations
    }
}

/// Typed owning-boundary observation rejection, without raw provider output.
#[derive(Debug, Eq, Error, PartialEq)]
pub enum ControlObservationError {
    /// Target/controller principal failed bounded canonical admission.
    #[error("invalid control observation principal")]
    InvalidPrincipal,
    /// Full set exceeds the maintained bound.
    #[error("controller set exceeds {MAX_CONTROLLERS}")]
    TooManyControllers,
    /// Equivalent principal appears twice.
    #[error("duplicate controller principal")]
    DuplicateController,
}
/// Typed request rejection before provider use or any remote effect.
#[derive(Debug, Error)]
pub enum ControlRequestError {
    /// Descriptive call ceiling exceeds the maintained bound.
    #[error("control observation ceiling exceeds {MAX_CONTROL_REMOTE_OBSERVATIONS}")]
    ObservationLimitTooLarge,
    /// Original plan cannot derive the named operation.
    #[error(transparent)]
    Plan(#[from] OperationPlanError),
    /// Actual payload is not the exact original mutation.
    #[error(transparent)]
    Payload(#[from] IcRequestError),
}

#[cfg(test)]
mod tests;
