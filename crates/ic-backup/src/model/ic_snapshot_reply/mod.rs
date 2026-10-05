//! Bounded capture/inventory reply decoding; transport authenticity stays external.

mod wire;

use super::{
    artifacts::ArtifactChecksumRecord,
    ic_request::{IcManagementMethodRecord, IcManagementRequestRecord},
};
use thiserror::Error;

/// Maximum raw Candid reply bytes admitted before decoding.
pub const MAX_IC_SNAPSHOT_REPLY_BYTES: usize = 1024 * 1024;
/// Maximum retained inventory entries; this is a local bound, not an IC capacity claim.
pub const MAX_IC_SNAPSHOT_REPLY_ENTRIES: usize = 1024;

/// Exact decoded snapshot identity and declared timestamp/size.
///
/// Raw identifiers remain opaque bytes. Timestamp and size may be zero or any
/// nat64 value; neither proves artifact completeness or unique effect settlement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcSnapshotInfo {
    id: Vec<u8>,
    taken_at_timestamp: u64,
    total_size: u64,
}

impl IcSnapshotInfo {
    /// Read exact raw snapshot bytes, without interpreting a backend token.
    #[must_use]
    pub fn id(&self) -> &[u8] {
        &self.id
    }

    /// Read the reply's declared snapshot timestamp in nanoseconds.
    #[must_use]
    pub const fn taken_at_timestamp(&self) -> u64 {
        self.taken_at_timestamp
    }

    /// Read the reply's declared snapshot size, distinct from local artifact bytes.
    #[must_use]
    pub const fn total_size(&self) -> u64 {
        self.total_size
    }
}

/// Read-only decoded reply associated with the caller's exact request.
///
/// Capture has one entry; inventory has 0..=1,024 unique entries in raw-ID order.
/// The wire contains no network, caller or target, so this association and digest
/// do not authenticate response origin, freshness, permissions or execution.
/// No Serde admission, completion receipt, reservation or settlement is provided.
#[derive(Debug)]
pub struct IcSnapshotReply<'request> {
    request: &'request IcManagementRequestRecord,
    snapshots: Vec<IcSnapshotInfo>,
    payload_checksum: ArtifactChecksumRecord,
}

impl<'request> IcSnapshotReply<'request> {
    /// Decode the pinned capture or inventory result under finite local bounds.
    ///
    /// Exactly one Candid argument with the required snapshot fields is admitted.
    /// Extra arguments, skipped fields, trailing bytes and duplicate IDs reject.
    /// The integration must independently qualify the transport/request association.
    ///
    /// # Errors
    /// Returns typed unsupported-method, raw-size, invalid-reply or duplicate-ID
    /// failures. No journal or caller-owned state changes on any outcome.
    pub fn decode(
        request: &'request IcManagementRequestRecord,
        bytes: &[u8],
    ) -> Result<Self, IcSnapshotReplyError> {
        let method = request.method();
        if !matches!(
            method,
            IcManagementMethodRecord::TakeCanisterSnapshot
                | IcManagementMethodRecord::ListCanisterSnapshots
        ) {
            return Err(IcSnapshotReplyError::UnsupportedMethod { method });
        }
        if bytes.len() > MAX_IC_SNAPSHOT_REPLY_BYTES {
            return Err(IcSnapshotReplyError::ReplyTooLarge);
        }
        let mut snapshots = wire::decode(method, bytes)?;
        snapshots.sort_unstable_by(|left, right| left.id.cmp(&right.id));
        if snapshots.windows(2).any(|pair| pair[0].id == pair[1].id) {
            return Err(IcSnapshotReplyError::DuplicateSnapshotId);
        }
        Ok(Self {
            request,
            snapshots,
            payload_checksum: ArtifactChecksumRecord::from_bytes(bytes),
        })
    }

    /// Read the original request; its owner supplies exact target/method/byte identity.
    #[must_use]
    pub const fn request(&self) -> &'request IcManagementRequestRecord {
        self.request
    }

    /// Read the capture singleton or canonical inventory without mutation access.
    #[must_use]
    pub fn snapshots(&self) -> &[IcSnapshotInfo] {
        &self.snapshots
    }

    /// Read the SHA-256 checksum of the exact raw reply, including its wire ordering.
    #[must_use]
    pub const fn payload_checksum(&self) -> &ArtifactChecksumRecord {
        &self.payload_checksum
    }

    /// Hash the exact request digest and raw-reply checksum in a separate v1 domain.
    ///
    /// Two fixed 64-byte lowercase SHA-256 strings follow the domain. A canonical
    /// inventory view does not erase raw wire differences or request identity.
    /// This is local evidence identity, not an authenticated IC receipt.
    #[must_use]
    pub fn digest(&self) -> ArtifactChecksumRecord {
        let mut bytes = b"ic-backup/ic-snapshot-reply/v1\0".to_vec();
        bytes.extend_from_slice(self.request.digest().hash().as_bytes());
        bytes.extend_from_slice(self.payload_checksum.hash().as_bytes());
        ArtifactChecksumRecord::from_bytes(&bytes)
    }
}

/// Typed snapshot wire admission failure, with no raw payload in diagnostics.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum IcSnapshotReplyError {
    /// Only the existing capture and inventory request methods have this reply shape.
    #[error("snapshot reply codec does not support {method:?}")]
    UnsupportedMethod {
        /// Actual request method, as admitted by the existing request owner.
        method: IcManagementMethodRecord,
    },
    /// Raw input exceeded the local bound before Candid parsing.
    #[error("snapshot reply exceeds {MAX_IC_SNAPSHOT_REPLY_BYTES} bytes")]
    ReplyTooLarge,
    /// Shape, field/count/ID bounds, decoding quota or exact consumption failed.
    #[error("invalid or unbounded snapshot Candid reply")]
    InvalidReply,
    /// Identical raw identifiers appeared more than once, even with different metadata.
    #[error("snapshot inventory contains duplicate raw identifiers")]
    DuplicateSnapshotId,
}

#[cfg(test)]
mod tests;
