//! Pure exact-result and direct caller-controller admission; no IO or dispatch authority.

use crate::model::{
    artifacts::ArtifactChecksumRecord,
    control_authority::{ControlObservation, ControlObservationRequest, ControllerSet},
};
use thiserror::Error;

/// Read-only matching controller evidence; not a dispatch permit or complete preflight.
#[derive(Clone, Debug)]
pub struct ControlAuthorityView<'a> {
    request: ArtifactChecksumRecord,
    observation: &'a ControlObservation,
}
impl ControlAuthorityView<'_> {
    /// Read exact current original-intent/payload/challenge-bound request identity.
    #[must_use]
    pub const fn request(&self) -> &ArtifactChecksumRecord {
        &self.request
    }
    /// Read matching canonical actual target.
    #[must_use]
    pub fn target(&self) -> &str {
        self.observation.target()
    }
    /// Read complete known controller evidence.
    #[must_use]
    pub const fn controllers(&self) -> &ControllerSet {
        self.observation.controllers()
    }
    /// Read opaque integration-owned evidence identifier.
    #[must_use]
    pub const fn evidence(&self) -> &ArtifactChecksumRecord {
        self.observation.evidence()
    }
    /// Read reported calls without reserving, refunding or replenishing authority.
    #[must_use]
    pub const fn remote_observations(&self) -> u32 {
        self.observation.remote_observations()
    }
}
/// Match current request/context/target/call bound and exact caller-controller membership.
///
/// The caller qualifies actual provider authenticity, freshness and coherent custody.
/// Policy performs no IO, record serialization, journal transition or scheduling.
/// Other controllers, public/read visibility, Root paths and subnet-admin exceptions
/// never substitute for the selected caller. Load still needs qualified snapshot-origin
/// permissions and same-ID safety; controller membership is only one preflight component.
/// # Errors
/// Rejects changed request/context/target, excess calls and missing exact caller control.
pub fn validate<'a>(
    request: &ControlObservationRequest<'_>,
    observation: &'a ControlObservation,
) -> Result<ControlAuthorityView<'a>, ControlAuthorityError> {
    let digest = request.digest();
    if *observation.request() != digest {
        return Err(ControlAuthorityError::RequestMismatch);
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
            return Err(ControlAuthorityError::ContextMismatch(field));
        }
    }
    if observation.target() != binding.target() {
        return Err(ControlAuthorityError::TargetMismatch);
    }
    if observation.remote_observations() > request.max_remote_observations() {
        return Err(ControlAuthorityError::ObservationLimitExceeded {
            limit: request.max_remote_observations(),
            reported: observation.remote_observations(),
        });
    }
    if !observation.controllers().contains_caller(binding) {
        return Err(ControlAuthorityError::CallerNotController);
    }
    Ok(ControlAuthorityView {
        request: digest,
        observation,
    })
}
/// Typed denial; no mismatch changes a plan/journal or admits any paid effect.
#[derive(Debug, Eq, Error, PartialEq)]
pub enum ControlAuthorityError {
    /// Original intent/operation/payload/challenge/ceiling differs.
    #[error("control observation request mismatch")]
    RequestMismatch,
    /// Actually observed context differs.
    #[error("control observed {0} mismatch")]
    ContextMismatch(&'static str),
    /// Actually observed target differs.
    #[error("control observed target mismatch")]
    TargetMismatch,
    /// Exact selected caller is absent; other access does not grant this lane.
    #[error("selected caller is not an observed controller")]
    CallerNotController,
    /// Actual reported remote calls exceed the descriptive ceiling.
    #[error("control reports {reported} observations above ceiling {limit}")]
    ObservationLimitExceeded {
        /// Original descriptive ceiling, not spending authority.
        limit: u32,
        /// Reported actual calls.
        reported: u32,
    },
}

#[cfg(test)]
mod tests;
