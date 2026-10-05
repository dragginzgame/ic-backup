//! Ephemeral snapshot-list permission evidence; no control or paid-call authority.

use crate::model::{
    artifacts::ArtifactChecksumRecord,
    attempt_journal::OperationBindingRecord,
    control_authority::ControllerSet,
    ic_request::{IcManagementMethodRecord, IcManagementRequestRecord, IcRequestError},
    operation_plan::{OperationPlanError, OperationPlanRecord, PlanContextRecord},
};
use thiserror::Error;

/// Maximum principals in a maintained IC snapshot allowed-viewer list.
pub const MAX_SNAPSHOT_VIEWERS: usize = 10;
/// Maximum descriptive remote observations per invocation; never spending authority.
pub const MAX_SNAPSHOT_READ_REMOTE_OBSERVATIONS: u32 = 1024;

/// Known canonical snapshot viewers; equivalent duplicates reject.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotViewerSet {
    principals: Vec<String>,
}
impl SnapshotViewerSet {
    /// Normalize and sort a bounded exact principal set, including known empty.
    /// # Errors
    /// Rejects excessive counts, invalid principals and canonical duplicates.
    pub fn new(mut principals: Vec<String>) -> Result<Self, SnapshotReadObservationError> {
        if principals.len() > MAX_SNAPSHOT_VIEWERS {
            return Err(SnapshotReadObservationError::TooManyViewers);
        }
        for principal in &mut principals {
            *principal = super::principal::canonical_text(principal)
                .ok_or(SnapshotReadObservationError::InvalidPrincipal)?;
        }
        principals.sort();
        if principals.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(SnapshotReadObservationError::DuplicateViewer);
        }
        Ok(Self { principals })
    }
    /// Read canonical viewers; ordering confers no control routing.
    #[must_use]
    pub fn principals(&self) -> &[String] {
        &self.principals
    }
    /// Test the exact original canonical caller.
    #[must_use]
    pub fn contains_caller(&self, binding: &OperationBindingRecord) -> bool {
        self.principals
            .binary_search_by(|principal| principal.as_str().cmp(binding.caller()))
            .is_ok()
    }
}

/// Actually observed snapshot visibility, distinct from status/log visibility.
///
/// No default, unknown variant or Serde admission exists. An integration unable
/// to qualify this exact setting must fail at its provider boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SnapshotVisibility {
    /// Only current controllers may read snapshots.
    Controllers,
    /// Anyone may read snapshots; this grants no mutation control.
    Public,
    /// These exact viewers and current controllers may read snapshots.
    AllowedViewers(SnapshotViewerSet),
}

/// Original mutation identity and independently declared exact snapshot-list payload.
///
/// This request does not inspect journals or reserve calls. The caller supplies
/// the observation digest from its own exact retained intent/reservation owner;
/// digest equality alone proves neither retention nor spending admission.
/// Challenge freshness, authenticated context and prior per-call accounting stay
/// integration-owned. Only list is implemented; metadata/data codecs remain pending.
#[derive(Clone, Debug)]
pub struct SnapshotReadRequest<'a> {
    binding: OperationBindingRecord,
    wire: &'a IcManagementRequestRecord,
    challenge: ArtifactChecksumRecord,
    max_remote_observations: u32,
}
impl<'a> SnapshotReadRequest<'a> {
    /// Bind exact list bytes to the original operation target and separate observation digest.
    /// # Errors
    /// Rejects unsupported methods, unknown operations, changed target/payload and excess ceiling.
    pub fn new(
        plan: &OperationPlanRecord,
        sequence: u64,
        wire: &'a IcManagementRequestRecord,
        observation: &ArtifactChecksumRecord,
        challenge: ArtifactChecksumRecord,
        max_remote_observations: u32,
    ) -> Result<Self, SnapshotReadRequestError> {
        if max_remote_observations > MAX_SNAPSHOT_READ_REMOTE_OBSERVATIONS {
            return Err(SnapshotReadRequestError::ObservationLimitTooLarge);
        }
        if wire.method() != IcManagementMethodRecord::ListCanisterSnapshots {
            return Err(SnapshotReadRequestError::UnsupportedMethod);
        }
        let binding = plan.attempt_authority(sequence)?.binding().clone();
        wire.validate_observation_binding(&binding, observation)?;
        Ok(Self {
            binding,
            wire,
            challenge,
            max_remote_observations,
        })
    }
    /// Read exact original mutation intent/context/target/request identity.
    #[must_use]
    pub const fn binding(&self) -> &OperationBindingRecord {
        &self.binding
    }
    /// Read validated exact snapshot-list routing/method/Candid bytes.
    #[must_use]
    pub const fn wire(&self) -> &IcManagementRequestRecord {
        self.wire
    }
    /// Read caller-owned challenge; its value does not establish freshness.
    #[must_use]
    pub const fn challenge(&self) -> &ArtifactChecksumRecord {
        &self.challenge
    }
    /// Read descriptive invocation ceiling, distinct from spending allowances.
    #[must_use]
    pub const fn max_remote_observations(&self) -> u32 {
        self.max_remote_observations
    }
    /// Hash full original intent/sequence, separate exact read payload, challenge and ceiling.
    ///
    /// Encoding: NUL-terminated ASCII v1 domain, 64 ASCII intent bytes, u64
    /// big-endian sequence, 64 ASCII list wire-digest bytes, 64 ASCII challenge
    /// bytes and u32 big-endian ceiling. Observed visibility/evidence are excluded.
    #[must_use]
    pub fn digest(&self) -> ArtifactChecksumRecord {
        let mut bytes = b"ic-backup/snapshot-read/v1\0".to_vec();
        bytes.extend_from_slice(self.binding.intent().as_bytes());
        bytes.extend_from_slice(&self.binding.operation_sequence().to_be_bytes());
        bytes.extend_from_slice(self.wire.digest().hash().as_bytes());
        bytes.extend_from_slice(self.challenge.hash().as_bytes());
        bytes.extend_from_slice(&self.max_remote_observations.to_be_bytes());
        ArtifactChecksumRecord::from_bytes(&bytes)
    }
}

/// Passive current provider data; no serialized authority or permissive defaults.
#[derive(Clone, Debug)]
pub struct SnapshotReadObservationInput {
    /// Exact current request digest.
    pub request: ArtifactChecksumRecord,
    /// Actually observed canonical network/caller/release, not echoed labels.
    pub context: PlanContextRecord,
    /// Actually observed physical target; normalized at model admission.
    pub target: String,
    /// Qualified current snapshot visibility; status visibility never substitutes.
    pub visibility: SnapshotVisibility,
    /// Complete known controllers, or None when not observed.
    ///
    /// None cannot establish controller access. Independently known public access
    /// or exact viewer membership needs no controller projection. Some(empty) is
    /// known empty, distinct from unknown. No fallback guesses a controller set.
    pub controllers: Option<ControllerSet>,
    /// Opaque qualified evidence identifier, not a signature or dispatch permit.
    pub evidence: ArtifactChecksumRecord,
    /// Actual remote calls, separately accounted before each call by the integration.
    pub remote_observations: u32,
}
/// Immutable model-admitted current evidence, without persisted authority admission.
#[derive(Clone, Debug)]
pub struct SnapshotReadObservation {
    input: SnapshotReadObservationInput,
}
impl SnapshotReadObservation {
    /// Normalize the actually observed physical target.
    /// # Errors
    /// Rejects malformed or oversized target principals.
    pub fn new(
        mut input: SnapshotReadObservationInput,
    ) -> Result<Self, SnapshotReadObservationError> {
        input.target = super::principal::canonical_text(&input.target)
            .ok_or(SnapshotReadObservationError::InvalidPrincipal)?;
        Ok(Self { input })
    }
    /// Read exact current request identity.
    #[must_use]
    pub const fn request(&self) -> &ArtifactChecksumRecord {
        &self.input.request
    }
    /// Read actually observed canonical context.
    #[must_use]
    pub const fn context(&self) -> &PlanContextRecord {
        &self.input.context
    }
    /// Read actually observed canonical target.
    #[must_use]
    pub fn target(&self) -> &str {
        &self.input.target
    }
    /// Read known current snapshot visibility.
    #[must_use]
    pub const fn visibility(&self) -> &SnapshotVisibility {
        &self.input.visibility
    }
    /// Read complete known controllers or explicitly unobserved evidence.
    #[must_use]
    pub const fn controllers(&self) -> Option<&ControllerSet> {
        self.input.controllers.as_ref()
    }
    /// Read opaque integration-qualified evidence identifier.
    #[must_use]
    pub const fn evidence(&self) -> &ArtifactChecksumRecord {
        &self.input.evidence
    }
    /// Read reported calls, without consuming or replenishing any allowance.
    #[must_use]
    pub const fn remote_observations(&self) -> u32 {
        self.input.remote_observations
    }
}

/// Typed model observation admission failure; diagnostics retain no raw identities.
#[derive(Debug, Eq, Error, PartialEq)]
pub enum SnapshotReadObservationError {
    /// Principal text is malformed or exceeds its owning boundary.
    #[error("invalid snapshot read principal")]
    InvalidPrincipal,
    /// Viewer list exceeds the maintained IC limit.
    #[error("snapshot viewer set exceeds {MAX_SNAPSHOT_VIEWERS}")]
    TooManyViewers,
    /// Equivalent principals cannot create duplicate viewer entries.
    #[error("duplicate snapshot viewer")]
    DuplicateViewer,
}
/// Typed exact original-plan/read-payload admission failure.
#[derive(Debug, Error)]
pub enum SnapshotReadRequestError {
    /// The descriptive ceiling exceeds its maintained bound.
    #[error("snapshot read observation ceiling exceeds {MAX_SNAPSHOT_READ_REMOTE_OBSERVATIONS}")]
    ObservationLimitTooLarge,
    /// Only the implemented snapshot-list codec is admitted.
    #[error("snapshot read contract requires list_canister_snapshots")]
    UnsupportedMethod,
    /// Original operation plan rejects binding derivation.
    #[error(transparent)]
    Plan(#[from] OperationPlanError),
    /// Exact read target or separate observation digest differs.
    #[error(transparent)]
    Payload(#[from] IcRequestError),
}

#[cfg(test)]
mod tests;
