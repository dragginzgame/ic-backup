//! Bounded projection; Platform owns wire traversal and management field labels.

#[cfg(test)]
mod tests;

use super::{CaptureStatusError, StatusRequirements};
use ic_platform_protocol::{
    candid::{Error as DecodeError, Limits},
    management_snapshots::decode_capture_status,
};
use thiserror::Error;

const MAX_REPLY_BYTES: usize = 1024 * 1024;
const MAX_SKIPPED_WORK: usize = 64 * 1024;

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
