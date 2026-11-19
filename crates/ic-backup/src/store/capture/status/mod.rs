//! Pure status preconditions; observation authentication stays with the caller.

#[cfg(test)]
pub(in crate::store) mod tests;
mod wire;

use crate::store::BackupDigest;
use ic_platform_protocol::management_snapshots::Status;
use ic_platform_types::Principal;
use thiserror::Error;
pub use wire::CaptureStatusReplyError;

pub(in crate::store) struct StatusRequirements {
    pub module_sha256: BackupDigest,
    pub sender: Principal,
    pub sole_controller: bool,
}

const MAX_CONTROLLERS: usize = 10;

impl StatusRequirements {
    fn validate_status_fields(
        &self,
        status: Status,
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
        if status != Status::Stopped {
            return Err(CaptureStatusError::NotStopped(status));
        }
        if module_hash != Some(self.module_sha256.as_bytes().as_slice()) {
            return Err(CaptureStatusError::ModuleMismatch);
        }
        if self.sole_controller && controllers != [self.sender] {
            return Err(CaptureStatusError::ManagerAuthority);
        }
        if !controllers.contains(&self.sender) {
            return Err(CaptureStatusError::NotController);
        }
        Ok(())
    }
}

/// A decoded status fails an original capture or restore precondition; no effect is authorized.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum CaptureStatusError {
    /// A manager-controlled child must have Root as its only controller.
    #[error("child controller set differs from the approved Root")]
    ManagerAuthority,
    /// The declared controller set exceeds the supported ten-controller boundary.
    #[error("capture status exceeds ten controllers")]
    ControllerLimit,
    /// A canonical Principal appears more than once.
    #[error("capture status contains duplicate controllers")]
    DuplicateController,
    /// Running and stopping are both outside the already-stopped capture profile.
    #[error("capture requires stopped status, observed {0:?}")]
    NotStopped(Status),
    /// The installed module hash is absent, malformed or different from the plan.
    #[error("capture status module hash differs from the approved plan")]
    ModuleMismatch,
    /// The actual management sender is absent from the declared controller set.
    #[error("capture management sender is not a declared controller")]
    NotController,
}
