//! Bounded Platform-schema data replies and exact scheduled byte admission.
use super::CaptureTransfer;
/// Data ordinal or bounded request encoding refused.
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
    /// Encode one exact scheduled request without iterating preceding extents.
    ///
    /// Uses the plan target, original snapshot ID and zero-based data ordinal.
    /// This pure helper grants no signing or dispatch authority.
    /// # Errors
    /// Refuses an ordinal outside the schedule or arguments exceeding 1 KiB.
    pub fn data_arguments(&self, ordinal: u32) -> Result<Vec<u8>, DataReadError> {
        if ordinal >= self.data_calls() {
            return Err(TransferDataError::Ordinal.into());
        }
        candid_codec::encode_one(
            &ReadCanisterSnapshotDataArgs {
                canister_id: self.plan.request().target.canister,
                snapshot_id: self.snapshot_id.to_vec(),
                kind: self.kind(u64::from(ordinal)),
            },
            1024,
        )
        .map_err(|_| OperationStoreError::StatusAdmission.into())
    }
}
use super::super::OperationStoreError;
use ic_host_artifacts::artifact::Sha256Digest;
use ic_platform::management_canister_types::ReadCanisterSnapshotDataArgs;
use ic_platform::{
    candid_codec::{
        self, Encode, Type, Writer,
        decoder::{Decode, Error as DecodeError, Limits, Reader, decode_one_with_skip_limit},
    },
    management_canister_types::{ReadCanisterSnapshotDataResult, SnapshotDataKind},
};
use thiserror::Error;

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
            SnapshotDataKind::WasmModule { size, .. }
            | SnapshotDataKind::WasmMemory { size, .. }
            | SnapshotDataKind::StableMemory { size, .. } => {
                if bytes.len() as u64 != size {
                    return Err(TransferDataError::Length {
                        expected: size,
                        actual: bytes.len(),
                    });
                }
            }
            SnapshotDataKind::WasmChunk { hash } => {
                let limit = self.plan.request().limits.extent_bytes as usize;
                if bytes.len() > limit {
                    return Err(TransferDataError::ChunkLimit {
                        limit,
                        actual: bytes.len(),
                    });
                }
                if Sha256Digest::compute(bytes).as_bytes().as_slice() != hash {
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
    let BoundedData(result) = decode_one_with_skip_limit(
        reply,
        Limits {
            bytes: MAX_REPLY_BYTES,
            header: 4096,
            types: 16,
            work: 8 * 1024 * 1024,
            allocation: 2 * 1024 * 1024,
        },
        0,
    )
    .map_err(TransferDataError::Decode)?;
    Ok(result.chunk)
}

// Platform owns the DTO, schema, numeric field label and codec. This mapping
// only imposes Backup's vector cap, before Platform allocates the field payload.
struct BoundedData(ReadCanisterSnapshotDataResult);
impl Encode for BoundedData {
    const TYPE: &'static Type = ReadCanisterSnapshotDataResult::TYPE;
    fn write(&self, output: &mut Writer) -> Result<(), candid_codec::Error> {
        self.0.write(output)
    }
}
impl<'a> Decode<'a> for BoundedData {
    fn read(reader: &mut Reader<'a>, wire: i64) -> Result<Self, DecodeError> {
        let schema = match Self::TYPE {
            Type::Deferred(resolve) => resolve(),
            schema => schema,
        };
        let Type::Record([(chunk_label, Type::Vec(Type::Primitive(-5)))]) = schema else {
            return Err(DecodeError::Unsupported);
        };
        let mut chunk = None;
        reader.record(wire, false, |label, field_type, reader| {
            if label != *chunk_label {
                return Ok(false);
            }
            chunk = Some(reader.bounded_vec::<u8>(field_type, MAX_DATA_BYTES)?);
            Ok(true)
        })?;
        Ok(Self(ReadCanisterSnapshotDataResult {
            chunk: chunk.ok_or(DecodeError::Type)?,
        }))
    }
}

/// Typed data-reply refusal, without raw payloads or any new attempt authority.
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

#[cfg(test)]
mod tests;
