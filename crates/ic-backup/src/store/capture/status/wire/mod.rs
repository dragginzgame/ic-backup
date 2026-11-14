//! Bounded projection; Platform owns wire traversal and management field labels.
use super::{CapturePlan, CaptureStatusError, MAX_CONTROLLERS};
use ic_platform::{
    Principal,
    candid_codec::{
        Encode, Type,
        arguments::DecodeArgs,
        decoder::{Arity, Error as DecodeError, Limits, Reader, decode_args_with_skip_limit},
    },
    management_canister_types::{
        CanisterStatusResult, CanisterStatusType, DefiniteCanisterSettings,
    },
};
use thiserror::Error;

const MAX_REPLY_BYTES: usize = 1024 * 1024;
const MAX_SKIPPED_WORK: usize = 64 * 1024;

impl CapturePlan {
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
    /// ```
    /// use ic_backup::store::{CapturePlan, CaptureStatusReplyError};
    ///
    /// fn inspect_status(plan: &CapturePlan, raw: &[u8]) -> Result<(), CaptureStatusReplyError> {
    ///     plan.validate_status_reply(raw)
    /// }
    /// ```
    ///
    /// # Errors
    /// Refuses excessive or malformed wire input, missing required fields,
    /// exhausted decoding/skipping budgets and failed capture preconditions.
    pub fn validate_status_reply(&self, reply: &[u8]) -> Result<(), CaptureStatusReplyError> {
        if reply.len() > MAX_REPLY_BYTES {
            return Err(CaptureStatusReplyError::ReplyTooLarge);
        }
        let observed: StatusFields = decode_args_with_skip_limit(
            reply,
            Limits {
                bytes: MAX_REPLY_BYTES,
                header: 64 * 1024,
                types: 64,
                work: 2 * 1024 * 1024,
                allocation: 2 * 1024 * 1024,
            },
            Arity::Exact,
            MAX_SKIPPED_WORK,
        )
        .map_err(CaptureStatusReplyError::Decode)?;
        self.validate_status_fields(
            observed.status,
            observed.module_hash.as_ref().map(<[u8; 32]>::as_slice),
            &observed.controllers,
        )?;
        Ok(())
    }
}

// A private projection, not an alternative management DTO or persisted record.
// DecodeArgs avoids inventing a wire schema/encoder for a partial observation.
struct StatusFields {
    status: CanisterStatusType,
    controllers: Vec<Principal>,
    module_hash: Option<[u8; 32]>,
}
impl<'a> DecodeArgs<'a> for StatusFields {
    const COUNT: usize = 1;
    fn read_args(reader: &mut Reader<'a>, types: &[i64]) -> Result<Self, DecodeError> {
        let [wire] = types else {
            return Err(DecodeError::Arity);
        };
        let Type::Record(fields) = CanisterStatusResult::TYPE
            .resolved()
            .map_err(|_| DecodeError::Unsupported)?
        else {
            return Err(DecodeError::Unsupported);
        };
        if fields.len() != 11 {
            return Err(DecodeError::Unsupported);
        }
        let (mut status, mut controllers, mut module_hash) = (None, None, None);
        reader.record(*wire, false, |label, inner, reader| {
            // Positions follow the reviewed Platform schema. Tests use complete
            // supplier DTOs and independent wire labels to guard this mapping.
            if label == fields[1].0 {
                status = Some(reader.value(inner)?);
            } else if label == fields[6].0 {
                controllers = Some(read_controllers(reader, inner)?);
            } else if label == fields[9].0 {
                module_hash = reader.value::<Option<[u8; 32]>>(inner)?;
            } else {
                return Ok(false);
            }
            Ok(true)
        })?;
        Ok(Self {
            status: status.ok_or(DecodeError::Type)?,
            controllers: controllers.ok_or(DecodeError::Type)?,
            module_hash,
        })
    }
}

fn read_controllers(reader: &mut Reader<'_>, wire: i64) -> Result<Vec<Principal>, DecodeError> {
    let Type::Record(fields) = DefiniteCanisterSettings::TYPE
        .resolved()
        .map_err(|_| DecodeError::Unsupported)?
    else {
        return Err(DecodeError::Unsupported);
    };
    if fields.len() != 13 {
        return Err(DecodeError::Unsupported);
    }
    let mut controllers = None;
    reader.record(wire, false, |label, inner, reader| {
        if label != fields[3].0 {
            return Ok(false);
        }
        controllers = Some(reader.bounded_vec(inner, MAX_CONTROLLERS)?);
        Ok(true)
    })?;
    controllers.ok_or(DecodeError::Type)
}

/// Bounded status wire refusal or an original capture precondition failure.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
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

#[cfg(test)]
mod tests;
