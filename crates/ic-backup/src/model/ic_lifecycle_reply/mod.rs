//! Bounded lifecycle acknowledgements and status/controller projections.

mod wire;

use super::{
    artifacts::ArtifactChecksumRecord,
    control_authority::{ControlObservationError, ControllerSet},
    ic_request::{IcManagementMethodRecord, IcManagementRequestRecord},
};
use ic_management_canister_types::CanisterStatusType;
use thiserror::Error;

/// Maximum raw reply bytes, checked before any status decoding.
pub const MAX_IC_LIFECYCLE_REPLY_BYTES: usize = 1024 * 1024;

/// Read-only required status and complete declared controller set.
///
/// Other upstream status fields are skipped under finite work limits; their
/// presence, values and semantics are not qualified by this projection. A decoded
/// Stopped value establishes no drained work, continuous fence or load success.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcCanisterStatusInfo {
    status: CanisterStatusType,
    controllers: ControllerSet,
}

impl IcCanisterStatusInfo {
    /// Read the upstream running, stopping or stopped variant exactly.
    #[must_use]
    pub const fn status(&self) -> CanisterStatusType {
        self.status
    }

    /// Read the canonical bounded set through its existing owning boundary.
    ///
    /// An explicitly empty set stays empty; missing settings/controllers reject.
    #[must_use]
    pub const fn controllers(&self) -> &ControllerSet {
        &self.controllers
    }
}

/// Decoded wire shape, with no receipt or current authority admission.
#[derive(Debug, Eq, PartialEq)]
pub enum IcLifecycleReplyKind {
    /// Required status/controller projection of a canister-status result.
    Status(IcCanisterStatusInfo),
    /// Exact empty Candid argument tuple for stop, start or snapshot load.
    ///
    /// This is local wire admission, not authentication or ongoing lifecycle state.
    Acknowledgement,
}

/// Local reply evidence associated with the caller's immutable exact request.
///
/// The wire carries no network, caller, target or challenge. Integrations must
/// authenticate association, qualify observation timing and consume original
/// per-call authority. No decoded kind settles pending attempts, replenishes limits,
/// admits restart/load/fence release or proves complete same-release restoration.
#[derive(Debug)]
pub struct IcLifecycleReply<'request> {
    request: &'request IcManagementRequestRecord,
    kind: IcLifecycleReplyKind,
    payload_checksum: ArtifactChecksumRecord,
}

impl<'request> IcLifecycleReply<'request> {
    /// Admit the existing status, stop, start or load reply under finite local bounds.
    ///
    /// Status requires exactly one value with required status/settings/controllers.
    /// Unprojected fields are skipped, not validated. Acknowledgements require the
    /// canonical six-byte empty tuple, with no extra type table, argument or bytes.
    ///
    /// # Errors
    /// Rejects unsupported methods, excessive raw input, invalid/bounded wire shapes
    /// and duplicate controller identity, with no raw payload in diagnostics.
    pub fn decode(
        request: &'request IcManagementRequestRecord,
        bytes: &[u8],
    ) -> Result<Self, IcLifecycleReplyError> {
        let method = request.method();
        if !matches!(
            method,
            IcManagementMethodRecord::CanisterStatus
                | IcManagementMethodRecord::StopCanister
                | IcManagementMethodRecord::StartCanister
                | IcManagementMethodRecord::LoadCanisterSnapshot
        ) {
            return Err(IcLifecycleReplyError::UnsupportedMethod { method });
        }
        if bytes.len() > MAX_IC_LIFECYCLE_REPLY_BYTES {
            return Err(IcLifecycleReplyError::ReplyTooLarge);
        }
        let kind = if method == IcManagementMethodRecord::CanisterStatus {
            IcLifecycleReplyKind::Status(wire::status(bytes)?)
        } else {
            if bytes != b"DIDL\0\0" {
                return Err(IcLifecycleReplyError::InvalidReply);
            }
            IcLifecycleReplyKind::Acknowledgement
        };
        Ok(Self {
            request,
            kind,
            payload_checksum: ArtifactChecksumRecord::from_bytes(bytes),
        })
    }

    /// Read the original method, canonical target and exact wire identity owner.
    #[must_use]
    pub const fn request(&self) -> &'request IcManagementRequestRecord {
        self.request
    }

    /// Read the method-specific local projection or acknowledgement.
    #[must_use]
    pub const fn kind(&self) -> &IcLifecycleReplyKind {
        &self.kind
    }

    /// Read SHA-256 of exact raw bytes, including all unprojected fields/order.
    #[must_use]
    pub const fn payload_checksum(&self) -> &ArtifactChecksumRecord {
        &self.payload_checksum
    }

    /// Hash existing request digest and exact raw-reply checksum in a v1 domain.
    ///
    /// Two fixed 64-byte lowercase SHA-256 strings follow the NUL-terminated domain.
    /// This binds declared association and never authenticates a transport receipt.
    #[must_use]
    pub fn digest(&self) -> ArtifactChecksumRecord {
        let mut bytes = b"ic-backup/ic-lifecycle-reply/v1\0".to_vec();
        bytes.extend_from_slice(self.request.digest().hash().as_bytes());
        bytes.extend_from_slice(self.payload_checksum.hash().as_bytes());
        ArtifactChecksumRecord::from_bytes(&bytes)
    }
}

/// Typed local wire admission failure, preserving caller-owned evidence and journals.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum IcLifecycleReplyError {
    /// The existing capture/inventory shapes have a separate reply owner.
    #[error("lifecycle reply codec does not support {method:?}")]
    UnsupportedMethod {
        /// Actual request method.
        method: IcManagementMethodRecord,
    },
    /// Raw input exceeded the bound before parsing.
    #[error("lifecycle reply exceeds {MAX_IC_LIFECYCLE_REPLY_BYTES} bytes")]
    ReplyTooLarge,
    /// Required shape/fields/counts, decoder work or exact consumption failed.
    #[error("invalid or unbounded lifecycle Candid reply")]
    InvalidReply,
    /// Canonical controller admission failed in its existing owner.
    #[error(transparent)]
    Controllers(#[from] ControlObservationError),
}

#[cfg(test)]
mod tests;
