//! Bounded metadata field mappings over Platform's schema and owned decoder.
use super::{CapturePlan, MAX_CHUNKS, MAX_GLOBALS, TransferError};
use ic_platform::{
    candid_codec::{
        self, Encode, Reserved, Type, Writer,
        decoder::{Decode, Error as DecodeError, Limits, Reader, decode_one_with_skip_limit},
    },
    management_canister_types::{
        ChunkHash, ReadCanisterSnapshotMetadataResult, SnapshotMetadataGlobal, SnapshotSource,
    },
};
use thiserror::Error;

const MAX_REPLY_BYTES: usize = 1024 * 1024;

impl CapturePlan {
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
    /// ```
    /// use ic_backup::store::{CapturePlan, TransferMetadataError};
    ///
    /// fn inspect_reply(plan: &CapturePlan, snapshot: &[u8], raw: &[u8])
    ///     -> Result<u32, TransferMetadataError>
    /// {
    ///     let metadata = plan.decode_metadata(snapshot, raw)?;
    ///     let transfer = plan.plan_transfer(snapshot, &metadata)?;
    ///     // No requests have been dispatched or attempts spent.
    ///     Ok(transfer.data_calls())
    /// }
    /// ```
    ///
    /// # Errors
    /// Refuses oversized/malformed replies, unsupported metadata and transfers
    /// outside the original plan's bounds. No failure changes the plan or ledger.
    pub fn decode_metadata(
        &self,
        snapshot_id: &[u8],
        reply: &[u8],
    ) -> Result<ReadCanisterSnapshotMetadataResult, TransferMetadataError> {
        let metadata = decode(reply)?;
        self.plan_transfer(snapshot_id, &metadata)?;
        Ok(metadata)
    }
}

pub(super) fn decode(
    reply: &[u8],
) -> Result<ReadCanisterSnapshotMetadataResult, TransferMetadataError> {
    if reply.len() > MAX_REPLY_BYTES {
        return Err(TransferMetadataError::ReplyTooLarge);
    }
    let BoundedMetadata(value) = decode_one_with_skip_limit(
        reply,
        Limits {
            bytes: MAX_REPLY_BYTES,
            header: 16 * 1024,
            types: 64,
            work: 2 * 1024 * 1024,
            allocation: 2 * 1024 * 1024,
        },
        0,
    )
    .map_err(TransferMetadataError::Decode)?;
    Ok(value)
}

struct BoundedMetadata(ReadCanisterSnapshotMetadataResult);
impl Encode for BoundedMetadata {
    const TYPE: &'static Type = ReadCanisterSnapshotMetadataResult::TYPE;
    fn write(&self, output: &mut Writer) -> Result<(), candid_codec::Error> {
        self.0.write(output)
    }
}
impl<'a> Decode<'a> for BoundedMetadata {
    fn read(reader: &mut Reader<'a>, wire: i64) -> Result<Self, DecodeError> {
        // Platform owns labels and types. Positions follow its reviewed schema;
        // independent retained wire fixtures guard this mapping during upgrades.
        let Type::Record(fields) = Self::TYPE
            .resolved()
            .map_err(|_| DecodeError::Unsupported)?
        else {
            return Err(DecodeError::Unsupported);
        };
        if fields.len() != 11 {
            return Err(DecodeError::Unsupported);
        }
        let (mut globals, mut canister_version, mut source, mut certified_data) =
            (None, None, None, None);
        let (mut global_timer, mut hook, mut module, mut stable) = (None, None, None, None);
        let (mut chunks, mut timestamp, mut heap) = (None, None, None);
        reader.record(wire, false, |label, inner, reader| {
            match fields.iter().position(|field| field.0 == label) {
                Some(0) => {
                    globals =
                        Some(reader.bounded_vec::<KnownOption<SnapshotMetadataGlobal>>(
                            inner,
                            MAX_GLOBALS,
                        )?)
                }
                Some(1) => canister_version = Some(reader.value(inner)?),
                Some(2) => {
                    source = Some(
                        reader
                            .value::<KnownOption<BoundedSource>>(inner)?
                            .0
                            .map(|source| source.0),
                    )
                }
                Some(3) => certified_data = Some(reader.bounded_vec::<u8>(inner, 32)?),
                Some(4) => global_timer = Some(reader.value::<KnownOption<_>>(inner)?.0),
                Some(5) => hook = Some(reader.value::<KnownOption<_>>(inner)?.0),
                Some(6) => module = Some(reader.value(inner)?),
                Some(7) => stable = Some(reader.value(inner)?),
                Some(8) => chunks = Some(reader.bounded_vec::<BoundedChunk>(inner, MAX_CHUNKS)?),
                Some(9) => timestamp = Some(reader.value(inner)?),
                Some(10) => heap = Some(reader.value(inner)?),
                _ => return Ok(false),
            }
            Ok(true)
        })?;
        Ok(Self(ReadCanisterSnapshotMetadataResult {
            globals: globals
                .ok_or(DecodeError::Type)?
                .into_iter()
                .map(|global| global.0)
                .collect(),
            canister_version: canister_version.ok_or(DecodeError::Type)?,
            source: source.flatten(),
            certified_data: certified_data.ok_or(DecodeError::Type)?,
            global_timer: global_timer.flatten(),
            on_low_wasm_memory_hook_status: hook.flatten(),
            wasm_module_size: module.ok_or(DecodeError::Type)?,
            stable_memory_size: stable.ok_or(DecodeError::Type)?,
            wasm_chunk_store: chunks
                .ok_or(DecodeError::Type)?
                .into_iter()
                .map(|chunk| chunk.0)
                .collect(),
            taken_at_timestamp: timestamp.ok_or(DecodeError::Type)?,
            wasm_memory_size: heap.ok_or(DecodeError::Type)?,
        }))
    }
}

// Known null/reserved absence does not spend skipped-work allowance. All other
// option handling stays Platform-owned; zero skipped work refuses type fallback.
struct KnownOption<T>(Option<T>);
impl<T: Encode> Encode for KnownOption<T> {
    const TYPE: &'static Type = <Option<T>>::TYPE;
    fn write(&self, output: &mut Writer) -> Result<(), candid_codec::Error> {
        self.0.write(output)
    }
}
impl<'a, T: Decode<'a>> Decode<'a> for KnownOption<T> {
    fn read(reader: &mut Reader<'a>, wire: i64) -> Result<Self, DecodeError> {
        if matches!(wire, -1 | -16) {
            return Ok(Self(None));
        }
        Option::<T>::read(reader, wire).map(Self)
    }
}

struct BoundedChunk(ChunkHash);
impl Encode for BoundedChunk {
    const TYPE: &'static Type = ChunkHash::TYPE;
    fn write(&self, output: &mut Writer) -> Result<(), candid_codec::Error> {
        self.0.write(output)
    }
}
impl<'a> Decode<'a> for BoundedChunk {
    fn read(reader: &mut Reader<'a>, wire: i64) -> Result<Self, DecodeError> {
        let Type::Record([(hash_label, Type::Vec(Type::Primitive(-5)))]) = Self::TYPE
            .resolved()
            .map_err(|_| DecodeError::Unsupported)?
        else {
            return Err(DecodeError::Unsupported);
        };
        let mut hash = None;
        reader.record(wire, false, |label, inner, reader| {
            if label != *hash_label {
                return Ok(false);
            }
            hash = Some(reader.bounded_vec::<u8>(inner, 32)?);
            Ok(true)
        })?;
        Ok(Self(ChunkHash {
            hash: hash.ok_or(DecodeError::Type)?,
        }))
    }
}

struct BoundedSource(SnapshotSource);
impl Encode for BoundedSource {
    const TYPE: &'static Type = SnapshotSource::TYPE;
    fn write(&self, output: &mut Writer) -> Result<(), candid_codec::Error> {
        self.0.write(output)
    }
}
impl<'a> Decode<'a> for BoundedSource {
    fn read(reader: &mut Reader<'a>, wire: i64) -> Result<Self, DecodeError> {
        let Type::Variant([(upload, _), (taken, _)]) = Self::TYPE
            .resolved()
            .map_err(|_| DecodeError::Unsupported)?
        else {
            return Err(DecodeError::Unsupported);
        };
        reader.variant(wire, |label, inner, _| {
            // Reserved is not permission to discard a non-reserved wire payload.
            if inner != -16 {
                return Err(DecodeError::Type);
            }
            if label == *taken {
                Ok(Self(SnapshotSource::TakenFromCanister(Reserved)))
            } else if label == *upload {
                Ok(Self(SnapshotSource::MetadataUpload(Reserved)))
            } else {
                Err(DecodeError::Type)
            }
        })
    }
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

#[cfg(test)]
mod tests;
