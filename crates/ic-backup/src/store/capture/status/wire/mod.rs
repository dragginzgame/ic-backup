//! Bounded projection; Platform owns wire traversal and management field labels.
#[cfg(test)]
mod tests;

#[cfg(test)]
use super::CapturePolicy;
use super::{CaptureStatusError, StatusRequirements};
use ic_platform_protocol::{
    candid::{Error as DecodeError, Limits},
    management_snapshots::decode_capture_status,
};
use thiserror::Error;

const MAX_REPLY_BYTES: usize = 1024 * 1024;
const MAX_SKIPPED_WORK: usize = 64 * 1024;

#[cfg(test)]
impl CapturePolicy {
    /// Validate a raw status reply against the original capture preconditions.
    ///
    /// Reads only status, controllers and module hash through Platform's codec.
    /// Other fields are skipped under a separate cumulative work limit; their
    /// presence and semantics are not validated. Controller and hash sizes are
    /// checked before allocation. Exact single-argument framing is required.
    ///
    /// This applies the same policy as [`Self::validate_status`]. The caller must
    /// retain the original raw evidence and authenticate its exact request,
    /// network, target and observation timing. Success authorizes no dispatch,
    /// progress, reservation or settlement and does not authenticate a reply.
    ///
    ///
    /// # Errors
    /// Refuses excessive or malformed wire input, missing required fields,
    /// exhausted decoding/skipping budgets and failed capture preconditions.
    pub fn validate_status_reply(&self, reply: &[u8]) -> Result<(), CaptureStatusReplyError> {
        self.status_requirements().validate_status_reply(reply)
    }
}

impl StatusRequirements {
    pub(in crate::store) fn validate_status_reply(
        &self,
        reply: &[u8],
    ) -> Result<(), CaptureStatusReplyError> {
        if reply.len() > MAX_REPLY_BYTES {
            return Err(CaptureStatusReplyError::ReplyTooLarge);
        }
        let observed = decode_capture_status(
            reply,
            Limits {
                bytes: MAX_REPLY_BYTES,
                header: 64 * 1024,
                types: 64,
                work: 2 * 1024 * 1024,
                allocation: 2 * 1024 * 1024,
                vector: MAX_REPLY_BYTES,
                skipping: MAX_SKIPPED_WORK,
            },
        )
        .map_err(CaptureStatusReplyError::Decode)?;
        self.validate_status_fields(
            observed.status,
            observed.module_hash.as_deref(),
            &observed.settings.controllers,
        )?;
        Ok(())
    }
}

/// Bounded status wire refusal or an original capture precondition failure.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum CaptureStatusReplyError {
    /// The complete raw reply exceeds 1 MiB.
    #[error("capture status reply exceeds 1 MiB")]
    ReplyTooLarge,
    /// Platform's exact codec failure; raw payloads are not retained in errors.
    #[error("capture status decoding refused: {0:?}")]
    Decode(DecodeError),
    /// Decoded fields fail the same policy as typed status validation.
    #[error(transparent)]
    Status(#[from] CaptureStatusError),
}
