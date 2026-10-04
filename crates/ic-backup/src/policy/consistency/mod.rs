//! Pure original-guarantee and current stopped/drained/fence matching; no effects.

use crate::model::{
    artifacts::ArtifactChecksumRecord,
    consistency::{
        ApplicationFenceEvidence, ApplicationFenceState, CaptureState, ConsistencyEvidence,
        ConsistencyGuaranteeRecord, ConsistencyObservation, ConsistencyRequest,
        TargetCaptureEvidence,
    },
};
use thiserror::Error;

/// Read-only matched current evidence; no capture, restart, release or spending permit.
#[derive(Clone, Debug)]
pub struct ConsistencyView<'a> {
    request: ArtifactChecksumRecord,
    requirement: ArtifactChecksumRecord,
    observation: &'a ConsistencyObservation,
}
impl ConsistencyView<'_> {
    /// Read exact challenge/boundary/fence-bound current request digest.
    #[must_use]
    pub const fn request(&self) -> &ArtifactChecksumRecord {
        &self.request
    }
    /// Read immutable original consistency requirement digest.
    #[must_use]
    pub const fn requirement(&self) -> &ArtifactChecksumRecord {
        &self.requirement
    }
    /// Read matched exact selected stopped/drained target evidence.
    #[must_use]
    pub fn targets(&self) -> &[TargetCaptureEvidence] {
        self.observation.targets()
    }
    /// Read matched active application fence when the original guarantee requires one.
    #[must_use]
    pub fn fence(&self) -> Option<&ApplicationFenceEvidence> {
        match self.observation.consistency() {
            ConsistencyEvidence::PerCanister => None,
            ConsistencyEvidence::ApplicationCoordinated(fence) => Some(fence),
        }
    }
    /// Read opaque integration-qualified current evidence.
    #[must_use]
    pub const fn evidence(&self) -> &ArtifactChecksumRecord {
        self.observation.evidence()
    }
    /// Read actual call reporting; grants no paid-call admission/replenishment.
    #[must_use]
    pub const fn remote_observations(&self) -> u32 {
        self.observation.remote_observations()
    }
}
/// Match original guarantee, current context/inventory/selection/stopped state and exact fence.
///
/// Integrations qualify evidence authenticity, actual freshness/drain and continuous
/// fence custody. Policy cannot prove these through hashes, echoed labels or matching
/// before/after observations. It performs no IO/serialization/provider calls, plan or
/// journal changes, dispatch, restart or fence release. Restore safety remains separate.
/// # Errors
/// Rejects changed request/context/inventory/selection/lane, non-stopped targets,
/// excess calls, changed retained fence identity or unbound membership revision.
pub fn validate<'a>(
    request: &ConsistencyRequest<'_>,
    observation: &'a ConsistencyObservation,
) -> Result<ConsistencyView<'a>, ConsistencyError> {
    let digest = request.digest();
    if *observation.request() != digest {
        return Err(ConsistencyError::RequestMismatch);
    }
    let binding = request.binding();
    for (field, expected, actual) in [
        (
            "network",
            binding.network(),
            observation.context().network(),
        ),
        ("caller", binding.caller(), observation.context().caller()),
        (
            "release",
            binding.release(),
            observation.context().release(),
        ),
    ] {
        if actual != expected {
            return Err(ConsistencyError::ContextMismatch(field));
        }
    }
    if observation.inventory() != request.inventory() {
        return Err(ConsistencyError::InventoryMismatch);
    }
    if !observation
        .targets()
        .iter()
        .map(|target| &target.target)
        .eq(request.selected_targets())
    {
        return Err(ConsistencyError::SelectionMismatch);
    }
    if observation.remote_observations() > request.max_remote_observations() {
        return Err(ConsistencyError::ObservationLimitExceeded {
            limit: request.max_remote_observations(),
            reported: observation.remote_observations(),
        });
    }
    if observation
        .targets()
        .iter()
        .any(|target| target.state != CaptureState::Stopped)
    {
        return Err(ConsistencyError::TargetNotStopped);
    }
    match (request.requirement().guarantee(), observation.consistency()) {
        (ConsistencyGuaranteeRecord::PerCanister, ConsistencyEvidence::PerCanister) => {}
        (
            ConsistencyGuaranteeRecord::ApplicationCoordinated,
            ConsistencyEvidence::ApplicationCoordinated(fence),
        ) => {
            let expected = request
                .expected_fence()
                .ok_or(ConsistencyError::FenceMismatch)?;
            if fence.identity != expected.identity {
                return Err(ConsistencyError::FenceMismatch);
            }
            if fence.state != ApplicationFenceState::Active {
                return Err(ConsistencyError::FenceNotActive);
            }
            let original_revision_matches =
                observation.membership_revision() == Some(&expected.membership_revision);
            let fence_revision_matches = fence.membership_revision == expected.membership_revision;
            if !original_revision_matches || !fence_revision_matches {
                return Err(ConsistencyError::MembershipRevisionMismatch);
            }
        }
        _ => return Err(ConsistencyError::GuaranteeMismatch),
    }
    Ok(ConsistencyView {
        request: digest,
        requirement: request.requirement().digest(),
        observation,
    })
}
/// Typed current evidence denial; no failure releases fences or replenishes allowances.
#[derive(Debug, Eq, Error, PartialEq)]
pub enum ConsistencyError {
    /// Original requirement/operation/challenge/boundary/fence/ceiling differs.
    #[error("consistency request mismatch")]
    RequestMismatch,
    /// Actually observed context differs.
    #[error("consistency observed {0} mismatch")]
    ContextMismatch(&'static str),
    /// Complete actually observed inventory differs, including unselected parent metadata.
    #[error("consistency inventory mismatch")]
    InventoryMismatch,
    /// Actual target evidence differs from the exact original selected set.
    #[error("consistency selected targets mismatch")]
    SelectionMismatch,
    /// Evidence cannot upgrade or downgrade the original declared guarantee.
    #[error("consistency evidence guarantee mismatch")]
    GuaranteeMismatch,
    /// At least one actual target is running or still stopping.
    #[error("consistency target is not stopped")]
    TargetNotStopped,
    /// Actual fence differs from the retained exact obligation.
    #[error("consistency fence identity mismatch")]
    FenceMismatch,
    /// Current exact fence is known inactive; retain the obligation and deny capture.
    #[error("consistency fence is not active")]
    FenceNotActive,
    /// Missing/changed actual revision is not bound to the application fence.
    #[error("consistency fence membership revision mismatch")]
    MembershipRevisionMismatch,
    /// Actual call reporting exceeds the descriptive ceiling.
    #[error("consistency reports {reported} observations above ceiling {limit}")]
    ObservationLimitExceeded {
        /// Original descriptive ceiling, separate from allowances.
        limit: u32,
        /// Reported actual calls.
        reported: u32,
    },
}

#[cfg(test)]
mod tests;
