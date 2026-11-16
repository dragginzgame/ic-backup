//! Pure status preconditions; observation authentication stays with the caller.
use super::CapturePolicy;
use crate::store::BackupDigest;
#[cfg(test)]
use ic_platform::management_canister_types::CanisterStatusResult;
use ic_platform::{Principal, management_canister_types::CanisterStatusType};
mod wire;
use thiserror::Error;
pub use wire::CaptureStatusReplyError;

pub(in crate::store) struct StatusRequirements {
    pub module_sha256: BackupDigest,
    pub sender: Principal,
}

const MAX_CONTROLLERS: usize = 10;

impl CapturePolicy {
    /// Check decoded status against the original capture's module and management sender.
    ///
    /// Requires stopped state, the exact approved 32-byte module hash, and the
    /// direct sender or Canic executor in a duplicate-free set of at most ten
    /// controllers. The approving initiator and other controllers cannot stand in
    /// for the actual management caller. Input order and original values are unchanged.
    ///
    /// This is pure validation of an already-decoded Platform DTO. The caller
    /// must bound decoding and authenticate the status reply's exact network,
    /// target, request and observation timing. Status contains no target identity.
    /// Success proves neither continuous application custody nor settled external
    /// obligations, and grants no reservation, dispatch or completion authority.
    /// Resource statistics, migration readiness and canister-version continuity
    /// are not evaluated by this check. It never stops or changes a canister.
    ///
    ///
    /// # Errors
    /// Refuses excessive/duplicate controllers, running/stopping state, an absent
    /// or different module hash, or a controller set lacking the original sender.
    #[cfg(test)]
    pub fn validate_status(
        &self,
        observed: &CanisterStatusResult,
    ) -> Result<(), CaptureStatusError> {
        self.status_requirements().validate_status(observed)
    }

    fn status_requirements(&self) -> StatusRequirements {
        StatusRequirements {
            module_sha256: self.request().module_sha256,
            sender: self.request().sender,
        }
    }
}

impl StatusRequirements {
    #[cfg(test)]
    pub(in crate::store) fn validate_status(
        &self,
        observed: &CanisterStatusResult,
    ) -> Result<(), CaptureStatusError> {
        self.validate_status_fields(
            observed.status,
            observed.module_hash.as_deref(),
            &observed.settings.controllers,
        )
    }

    fn validate_status_fields(
        &self,
        status: CanisterStatusType,
        module_hash: Option<&[u8]>,
        controllers: &[Principal],
    ) -> Result<(), CaptureStatusError> {
        if controllers.len() > MAX_CONTROLLERS {
            return Err(CaptureStatusError::ControllerLimit);
        }
        for (index, controller) in controllers.iter().enumerate() {
            if controllers[..index].contains(controller) {
                return Err(CaptureStatusError::DuplicateController);
            }
        }
        if status != CanisterStatusType::Stopped {
            return Err(CaptureStatusError::NotStopped(status));
        }
        if module_hash != Some(self.module_sha256.as_bytes().as_slice()) {
            return Err(CaptureStatusError::ModuleMismatch);
        }
        if !controllers.contains(&self.sender) {
            return Err(CaptureStatusError::NotController);
        }
        Ok(())
    }
}

/// A decoded status fails an original capture or restore precondition; no effect is authorized.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum CaptureStatusError {
    /// The declared controller set exceeds the supported ten-controller boundary.
    #[error("capture status exceeds ten controllers")]
    ControllerLimit,
    /// A canonical Principal appears more than once.
    #[error("capture status contains duplicate controllers")]
    DuplicateController,
    /// Running and stopping are both outside the already-stopped capture profile.
    #[error("capture requires stopped status, observed {0:?}")]
    NotStopped(CanisterStatusType),
    /// The installed module hash is absent, malformed or different from the plan.
    #[error("capture status module hash differs from the approved plan")]
    ModuleMismatch,
    /// The actual management sender is absent from the declared controller set.
    #[error("capture management sender is not a declared controller")]
    NotController,
}

#[cfg(test)]
pub(in crate::store) mod tests;
