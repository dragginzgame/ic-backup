//! Bounded Platform-schema data replies and exact scheduled byte admission.

#[cfg(test)]
mod tests;

use super::CaptureTransfer;
use crate::store::BackupDigest;
use ic_platform_protocol::management_snapshots::{Extent, SnapshotChunk};
use ic_platform_protocol::{
    candid::{Error as DecodeError, Limits},
    management_snapshots::{ReadDataArgs, ReadKind, decode_snapshot_data},
};
use thiserror::Error;

///
/// DataReadError
/// Data ordinal or bounded request encoding refused.
///

#[derive(Debug, thiserror::Error)]
pub enum DataReadError {
    /// Original argument allowance refused.
    #[error(transparent)]
    Admission(#[from] super::super::OperationStoreError),
    /// The requested ordinal is outside the approved schedule.
    #[error(transparent)]
    Data(#[from] TransferDataError),
}

impl CaptureTransfer<'_> {
    /// Build one exact scheduled request without iterating preceding extents.
    ///
    /// Uses the plan target, original snapshot ID and zero-based data ordinal.
    /// This pure helper grants no signing or dispatch authority.
    /// # Errors
    /// Refuses an ordinal outside the schedule.
    pub fn data_arguments(&self, ordinal: u32) -> Result<ReadDataArgs, DataReadError> {
        if ordinal >= self.data_calls() {
            return Err(TransferDataError::Ordinal.into());
        }

        Ok(ReadDataArgs {
            canister_id: self.plan.request().target.canister,
            snapshot_id: self.snapshot_id.to_vec(),
            kind: self.kind(u64::from(ordinal)),
        })
    }
}

const MAX_DATA_BYTES: usize = 1024 * 1024;
const MAX_REPLY_BYTES: usize = 2 * 1024 * 1024;

impl CaptureTransfer<'_> {
    /// Decode one raw Candid reply against its zero-based scheduled data-read ordinal.
    ///
    /// Region replies must have exactly the requested length. Whole chunks must
    /// fit the original extent ceiling and match their metadata SHA-256, including
    /// a valid empty chunk. Input/header/type/work/allocation limits and a vector
    /// element cap apply before untrusted allocation; skipped payloads are refused.
    ///
    /// This proves byte shape and chunk integrity only. The caller must retain raw
    /// evidence and authenticate its exact request/target/snapshot association.
    /// Equal-length region bytes are not authenticated here. Success does not
    /// record progress, settle an attempt or prove complete snapshot coverage.
    ///
    /// # Errors
    /// Refuses an ordinal outside the schedule, excessive/malformed wire input,
    /// wrong region length, excessive chunks or a mismatched chunk digest.
    pub fn decode_data(&self, ordinal: u32, reply: &[u8]) -> Result<Vec<u8>, TransferDataError> {
        if ordinal >= self.data_calls {
            return Err(TransferDataError::Ordinal);
        }
        let chunk = decode_reply(reply)?;
        self.validate_data_bytes(ordinal, &chunk)?;

        Ok(chunk)
    }

    // Both captured source slices and decoded destination replies obey the same
    // immutable schedule. This validates shape/integrity, not origin or custody.
    pub(super) fn validate_data_bytes(
        &self,
        ordinal: u32,
        bytes: &[u8],
    ) -> Result<(), TransferDataError> {
        if ordinal >= self.data_calls {
            return Err(TransferDataError::Ordinal);
        }
        match self.kind(u64::from(ordinal)) {
            ReadKind::WasmModule(Extent { size, .. })
            | ReadKind::WasmMemory(Extent { size, .. })
            | ReadKind::StableMemory(Extent { size, .. }) => {
                if bytes.len() as u64 != size {
                    return Err(TransferDataError::Length {
                        expected: size,
                        actual: bytes.len(),
                    });
                }
            }
            ReadKind::WasmChunk(SnapshotChunk { hash }) => {
                let limit = self.plan.request().limits.extent_bytes as usize;
                if bytes.len() > limit {
                    return Err(TransferDataError::ChunkLimit {
                        limit,
                        actual: bytes.len(),
                    });
                }

                if BackupDigest::compute(bytes).as_bytes().as_slice() != hash {
                    return Err(TransferDataError::ChunkHash);
                }
            }
        }

        Ok(())
    }
}

pub(in crate::store) fn decode_reply(reply: &[u8]) -> Result<Vec<u8>, TransferDataError> {
    if reply.len() > MAX_REPLY_BYTES {
        return Err(TransferDataError::ReplyTooLarge);
    }
    let result = decode_snapshot_data(
        reply,
        Limits {
            bytes: MAX_REPLY_BYTES,
            header: 4096,
            types: 16,
            work: 8 * 1024 * 1024,
            allocation: 2 * 1024 * 1024,
            vector: MAX_DATA_BYTES,
            skipping: 0,
        },
    )
    .map_err(TransferDataError::Decode)?;

    Ok(result.chunk)
}

///
/// TransferDataError
/// Typed data-reply refusal, without raw payloads or any new attempt authority.
///

#[derive(Debug, Error, Eq, PartialEq)]
pub enum TransferDataError {
    /// The ordinal does not select one of the original scheduled data reads.
    #[error("data-read ordinal is outside the capture schedule")]
    Ordinal,
    /// Complete raw reply exceeds 2 MiB.
    #[error("capture data reply exceeds 2 MiB")]
    ReplyTooLarge,
    /// Platform's exact typed codec refusal; these counters are codec-specific.
    #[error("capture data reply decoding refused: {0:?}")]
    Decode(DecodeError),
    /// A region reply does not have exactly the scheduled byte length.
    #[error("capture data length {actual} differs from scheduled {expected}")]
    Length {
        /// Scheduled region length.
        expected: u64,
        /// Decoded response length.
        actual: usize,
    },
    /// A whole chunk exceeds the original extent ceiling.
    #[error("capture chunk length {actual} exceeds original ceiling {limit}")]
    ChunkLimit {
        /// Original plan's extent ceiling.
        limit: usize,
        /// Decoded chunk length.
        actual: usize,
    },
    /// Whole-chunk bytes differ from their exact metadata SHA-256.
    #[error("capture chunk does not match its original hash")]
    ChunkHash,
}
