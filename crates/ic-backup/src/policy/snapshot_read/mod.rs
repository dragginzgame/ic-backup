//! Pure exact-result and caller snapshot-read checks; no IO, control or dispatch.

use crate::model::{
    artifacts::ArtifactChecksumRecord,
    snapshot_read::{SnapshotReadObservation, SnapshotReadRequest, SnapshotVisibility},
};
use thiserror::Error;

/// Read permission established by qualified current input; never mutation control.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotReadPath {
    /// Exact original caller is in known current controllers.
    Controller,
    /// Actually observed snapshot visibility is public.
    Public,
    /// Exact original caller is in the known snapshot viewer list.
    AllowedViewer,
}
/// Read-only matching evidence; not a dispatch, spending or snapshot-completion permit.
#[derive(Clone, Debug)]
pub struct SnapshotReadView<'a> {
    request: ArtifactChecksumRecord,
    observation: &'a SnapshotReadObservation,
    path: SnapshotReadPath,
}
impl SnapshotReadView<'_> {
    /// Read current full-intent/operation/read-payload/challenge-bound request digest.
    #[must_use]
    pub const fn request(&self) -> &ArtifactChecksumRecord {
        &self.request
    }
    /// Read actually observed matching canonical target.
    #[must_use]
    pub fn target(&self) -> &str {
        self.observation.target()
    }
    /// Read known current snapshot visibility.
    #[must_use]
    pub const fn visibility(&self) -> &SnapshotVisibility {
        self.observation.visibility()
    }
    /// Read the qualified caller read path; controller evidence takes precedence when known.
    #[must_use]
    pub const fn path(&self) -> SnapshotReadPath {
        self.path
    }
    /// Read opaque qualified evidence identifier.
    #[must_use]
    pub const fn evidence(&self) -> &ArtifactChecksumRecord {
        self.observation.evidence()
    }
    /// Read reported actual calls, without spending or replenishing allowance.
    #[must_use]
    pub const fn remote_observations(&self) -> u32 {
        self.observation.remote_observations()
    }
}
/// Match exact current request/context/target/call bound and snapshot visibility permission.
///
/// The caller qualifies input authenticity, actual fresh observations and custody.
/// Public or explicit viewer membership can admit read-only evidence without a
/// controller projection; unknown controllers cannot satisfy the controller path.
/// No status visibility, Root proxy or serialized Proven flag is considered. Policy
/// performs no IO, provider invocation, serialization, journal transitions or scheduling.
/// # Errors
/// Rejects identity/call mismatches, unobserved necessary controllers and absent caller access.
pub fn validate<'a>(
    request: &SnapshotReadRequest<'_>,
    observation: &'a SnapshotReadObservation,
) -> Result<SnapshotReadView<'a>, SnapshotReadError> {
    let digest = request.digest();
    if *observation.request() != digest {
        return Err(SnapshotReadError::RequestMismatch);
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
            return Err(SnapshotReadError::ContextMismatch(field));
        }
    }
    if observation.target() != binding.target() {
        return Err(SnapshotReadError::TargetMismatch);
    }
    if observation.remote_observations() > request.max_remote_observations() {
        return Err(SnapshotReadError::ObservationLimitExceeded {
            limit: request.max_remote_observations(),
            reported: observation.remote_observations(),
        });
    }
    let controller = observation
        .controllers()
        .is_some_and(|set| set.contains_caller(binding));
    let path = if controller {
        SnapshotReadPath::Controller
    } else {
        match observation.visibility() {
            SnapshotVisibility::Public => SnapshotReadPath::Public,
            SnapshotVisibility::AllowedViewers(viewers) if viewers.contains_caller(binding) => {
                SnapshotReadPath::AllowedViewer
            }
            SnapshotVisibility::Controllers | SnapshotVisibility::AllowedViewers(_) => {
                return Err(if observation.controllers().is_none() {
                    SnapshotReadError::ControllersUnobserved
                } else {
                    SnapshotReadError::CallerCannotRead
                });
            }
        }
    };
    Ok(SnapshotReadView {
        request: digest,
        observation,
        path,
    })
}
/// Typed denial; no error alters consumption or admits any paid effect.
#[derive(Debug, Eq, Error, PartialEq)]
pub enum SnapshotReadError {
    /// Current original intent/operation/read-payload/challenge/ceiling differs.
    #[error("snapshot read request mismatch")]
    RequestMismatch,
    /// Actually observed context differs.
    #[error("snapshot read observed {0} mismatch")]
    ContextMismatch(&'static str),
    /// Actually observed physical target differs.
    #[error("snapshot read observed target mismatch")]
    TargetMismatch,
    /// No public/viewer path applies and controller evidence is unobserved.
    #[error("snapshot read requires unobserved controller evidence")]
    ControllersUnobserved,
    /// Exact original caller is neither a known controller nor an admitted viewer.
    #[error("selected caller cannot read observed snapshots")]
    CallerCannotRead,
    /// Actual call reporting exceeds the descriptive invocation ceiling.
    #[error("snapshot read reports {reported} observations above ceiling {limit}")]
    ObservationLimitExceeded {
        /// Original descriptive ceiling, not spending authority.
        limit: u32,
        /// Reported actual calls.
        reported: u32,
    },
}

#[cfg(test)]
mod tests;
