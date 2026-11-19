//! Bounded metadata field mappings over Platform's schema and owned decoder.

#[cfg(test)]
mod tests;

use super::{CapturePolicy, TransferError};
use ic_platform_protocol::{
    candid::{Error as DecodeError, Limits},
    management_snapshots::{SnapshotMetadata, decode_snapshot_metadata},
};
use thiserror::Error;

const MAX_REPLY_BYTES: usize = 1024 * 1024;

impl CapturePolicy {
    /// Decode bounded raw metadata and admit its transfer under this original plan.
    ///
    /// Preserves wire chunk order and exact global float bits. Collection caps
    /// apply before allocation; unknown fields/variants and non-reserved source
    /// payloads refuse. Recognized optional promotion and absence retain their
    /// meaning, then the same capture profile and budgets as [`Self::plan_transfer`]
    /// apply. Missing round-trip values are unsupported, never filled in.
    ///
    /// The caller must retain raw evidence and authenticate its association with
    /// the exact target, snapshot and request. Decoding supplies no authentication,
    /// durable schedule binding, spending, progress or dispatch authority.
    ///
    ///
    /// # Errors
    /// Refuses oversized/malformed replies, unsupported metadata and transfers
    /// outside the original plan's bounds. No failure changes the plan or ledger.
    pub fn decode_metadata(
        &self,
        snapshot_id: &[u8],
        reply: &[u8],
    ) -> Result<SnapshotMetadata, TransferMetadataError> {
        let metadata = decode(reply)?;
        self.plan_transfer(snapshot_id, &metadata)?;
        Ok(metadata)
    }
}

pub(super) fn decode(reply: &[u8]) -> Result<SnapshotMetadata, TransferMetadataError> {
    if reply.len() > MAX_REPLY_BYTES {
        return Err(TransferMetadataError::ReplyTooLarge);
    }
    let value = decode_snapshot_metadata(
        reply,
        Limits {
            bytes: MAX_REPLY_BYTES,
            header: 16 * 1024,
            types: 64,
            work: 2 * 1024 * 1024,
            allocation: 2 * 1024 * 1024,
            vector: MAX_REPLY_BYTES,
            skipping: 0,
        },
    )
    .map_err(TransferMetadataError::Decode)?;
    Ok(value)
}

/// Bounded metadata decoding or original-plan admission refusal.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum TransferMetadataError {
    /// The complete raw reply exceeds 1 MiB.
    #[error("capture metadata reply exceeds 1 MiB")]
    ReplyTooLarge,
    /// Exact Platform codec failure without raw payload diagnostics.
    #[error("capture metadata decoding refused: {0:?}")]
    Decode(DecodeError),
    /// Decoded metadata cannot form an admitted transfer under the original plan.
    #[error(transparent)]
    Transfer(#[from] TransferError),
}
