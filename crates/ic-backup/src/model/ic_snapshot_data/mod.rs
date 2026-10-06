//! Metadata-bound snapshot data codecs, without dispatch or transfer completion.

mod wire;

use super::{
    artifacts::ArtifactChecksumRecord,
    ic_request::{IcRequestError, MAX_IC_ARGUMENT_BYTES, management_request_digest},
    ic_snapshot_metadata::IcSnapshotMetadataReply,
};
use candid::Principal;
use ic_management_canister_types::{ReadCanisterSnapshotDataArgs, SnapshotDataKind};
use std::fmt;
use thiserror::Error;

/// Maximum requested/decoded data bytes per call; no aggregate allowance is granted.
pub const MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES: usize = 1024 * 1024;
/// Maximum raw data reply bytes, allowing wire overhead around a 1 MiB chunk.
pub const MAX_IC_SNAPSHOT_DATA_REPLY_BYTES: usize = 2 * 1024 * 1024;

/// Exact ephemeral read for one range or chunk in retained snapshot metadata.
///
/// The borrowed metadata owns the target/raw ID, declared sizes and chunk set.
/// Construction validates bytes only, without authenticating that metadata or
/// granting a reservation, fresh read access, signing, dispatch or repeat call.
pub struct IcSnapshotDataRequest<'metadata> {
    metadata: &'metadata IcSnapshotMetadataReply<'metadata>,
    kind: SnapshotDataKind,
    target_bytes: Vec<u8>,
    arguments: Vec<u8>,
}

impl<'metadata> IcSnapshotDataRequest<'metadata> {
    /// Encode one bounded nonempty range or exact known chunk-store identity.
    ///
    /// # Errors
    /// Rejects zero/oversized ranges, overflow or ranges beyond declared metadata,
    /// malformed/unknown chunk hashes and existing argument encoding/size failures.
    pub fn new(
        metadata: &'metadata IcSnapshotMetadataReply<'metadata>,
        kind: SnapshotDataKind,
    ) -> Result<Self, IcSnapshotDataError> {
        let values = metadata.metadata();
        match &kind {
            SnapshotDataKind::WasmModule { offset, size } => {
                range(*offset, *size, values.wasm_module_size)?;
            }
            SnapshotDataKind::WasmMemory { offset, size } => {
                range(*offset, *size, values.wasm_memory_size)?;
            }
            SnapshotDataKind::StableMemory { offset, size } => {
                range(*offset, *size, values.stable_memory_size)?;
            }
            SnapshotDataKind::WasmChunk { hash } => {
                if hash.len() != 32 {
                    return Err(IcSnapshotDataError::InvalidChunkHash);
                }
                if !values
                    .wasm_chunk_store
                    .iter()
                    .any(|chunk| chunk.hash == *hash)
                {
                    return Err(IcSnapshotDataError::ChunkNotInMetadata);
                }
            }
        }
        let target = Principal::from_text(metadata.request().target())
            .map_err(|_| IcRequestError::InvalidTarget)?;
        let arguments = candid::encode_one(ReadCanisterSnapshotDataArgs {
            canister_id: target,
            snapshot_id: metadata.request().snapshot_id().to_vec(),
            kind: kind.clone(),
        })
        .map_err(|error| IcRequestError::Encoding(error.to_string()))?;
        if arguments.len() > MAX_IC_ARGUMENT_BYTES {
            return Err(IcRequestError::ArgumentsTooLarge.into());
        }
        Ok(Self {
            metadata,
            kind,
            target_bytes: target.as_slice().to_vec(),
            arguments,
        })
    }

    /// Read the retained metadata evidence used for range/hash admission.
    #[must_use]
    pub const fn metadata(&self) -> &'metadata IcSnapshotMetadataReply<'metadata> {
        self.metadata
    }

    /// Read the exact admitted upstream range or chunk identity.
    #[must_use]
    pub const fn kind(&self) -> &SnapshotDataKind {
        &self.kind
    }

    /// Read the canonical effective routing target, also encoded in the arguments.
    #[must_use]
    pub fn target(&self) -> &str {
        self.metadata.request().target()
    }

    /// Read the exact raw snapshot identity, distinct from generic backend tokens.
    #[must_use]
    pub fn snapshot_id(&self) -> &[u8] {
        self.metadata.request().snapshot_id()
    }

    /// Read exact Candid arguments, without a transport or execution permit.
    #[must_use]
    pub fn arguments(&self) -> &[u8] {
        &self.arguments
    }

    /// Read the management receiver, separate from effective routing.
    #[must_use]
    pub const fn receiver(&self) -> &'static str {
        "aaaaa-aa"
    }

    /// Read the sole data-read method, using replicated host update ingress.
    #[must_use]
    pub const fn method(&self) -> &'static str {
        "read_canister_snapshot_data"
    }

    /// Hash exact wire identity with the existing management payload owner.
    ///
    /// Metadata evidence is excluded from this nonrecursive request hash and
    /// separately included in the reply evidence digest. Spending stays external.
    #[must_use]
    pub fn digest(&self) -> ArtifactChecksumRecord {
        management_request_digest(&self.target_bytes, self.method(), &self.arguments)
    }
}

fn range(offset: u64, size: u64, total: u64) -> Result<(), IcSnapshotDataError> {
    let admitted_size = usize::try_from(size)
        .is_ok_and(|size| (1..=MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES).contains(&size));
    if !admitted_size {
        return Err(IcSnapshotDataError::InvalidRangeSize);
    }
    if offset.checked_add(size).is_none_or(|end| end > total) {
        return Err(IcSnapshotDataError::RangeOutsideMetadata);
    }
    Ok(())
}

impl fmt::Debug for IcSnapshotDataRequest<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IcSnapshotDataRequest")
            .field("target", &self.target())
            .field("argument_bytes", &self.arguments.len())
            .finish_non_exhaustive()
    }
}

/// Exact bounded data bytes associated with one declared request and metadata reply.
///
/// Range replies must have the exact requested length. Chunk-store replies must
/// hash to the requested retained identity, including a valid empty chunk. Neither
/// condition authenticates origin, proves whole-snapshot coverage or settles effects.
pub struct IcSnapshotDataReply<'request, 'metadata> {
    request: &'request IcSnapshotDataRequest<'metadata>,
    chunk: Vec<u8>,
    chunk_checksum: ArtifactChecksumRecord,
    payload_checksum: ArtifactChecksumRecord,
}

impl<'request, 'metadata> IcSnapshotDataReply<'request, 'metadata> {
    /// Decode exactly one bounded data argument, then admit length/hash association.
    ///
    /// # Errors
    /// Rejects excessive raw/chunk bytes, malformed/unsupported/skipped/extra/trailing
    /// wire, an inexact range length or bytes with a different chunk-store hash.
    pub fn decode(
        request: &'request IcSnapshotDataRequest<'metadata>,
        bytes: &[u8],
    ) -> Result<Self, IcSnapshotDataError> {
        if bytes.len() > MAX_IC_SNAPSHOT_DATA_REPLY_BYTES {
            return Err(IcSnapshotDataError::ReplyTooLarge);
        }
        let chunk = wire::decode(bytes)?;
        let chunk_checksum = ArtifactChecksumRecord::from_bytes(&chunk);
        match request.kind() {
            SnapshotDataKind::WasmModule { size, .. }
            | SnapshotDataKind::WasmMemory { size, .. }
            | SnapshotDataKind::StableMemory { size, .. } => {
                if u64::try_from(chunk.len()).ok() != Some(*size) {
                    return Err(IcSnapshotDataError::LengthMismatch);
                }
            }
            SnapshotDataKind::WasmChunk { hash } => {
                if chunk_checksum.hash() != crate::hash::hex_bytes(hash) {
                    return Err(IcSnapshotDataError::ChunkHashMismatch);
                }
            }
        }
        Ok(Self {
            request,
            chunk,
            chunk_checksum,
            payload_checksum: ArtifactChecksumRecord::from_bytes(bytes),
        })
    }

    /// Read the exact request and retained metadata association.
    #[must_use]
    pub const fn request(&self) -> &'request IcSnapshotDataRequest<'metadata> {
        self.request
    }

    /// Read exact admitted data bytes, with no mutable access or persistence side effect.
    #[must_use]
    pub fn chunk(&self) -> &[u8] {
        &self.chunk
    }

    /// Read SHA-256 of actual data, distinct from raw Candid wire identity.
    #[must_use]
    pub const fn chunk_checksum(&self) -> &ArtifactChecksumRecord {
        &self.chunk_checksum
    }

    /// Read SHA-256 of exact raw reply bytes.
    #[must_use]
    pub const fn payload_checksum(&self) -> &ArtifactChecksumRecord {
        &self.payload_checksum
    }

    /// Bind exact retained metadata, request and raw reply hashes in the data domain.
    #[must_use]
    pub fn digest(&self) -> ArtifactChecksumRecord {
        let mut bytes = b"ic-backup/ic-snapshot-data-reply/v1\0".to_vec();
        bytes.extend_from_slice(self.request.metadata().digest().hash().as_bytes());
        bytes.extend_from_slice(self.request.digest().hash().as_bytes());
        bytes.extend_from_slice(self.payload_checksum.hash().as_bytes());
        ArtifactChecksumRecord::from_bytes(&bytes)
    }
}

impl fmt::Debug for IcSnapshotDataReply<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IcSnapshotDataReply")
            .field("request", &self.request)
            .field("chunk_bytes", &self.chunk.len())
            .field("payload_checksum", &self.payload_checksum)
            .finish_non_exhaustive()
    }
}

/// Typed metadata/range/data admission failures, without raw payload diagnostics.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum IcSnapshotDataError {
    /// Existing canonical IC argument boundary rejected the request.
    #[error(transparent)]
    Request(#[from] IcRequestError),
    /// Range size is zero or greater than the local single-call bound.
    #[error("snapshot range size must be 1..={MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES}")]
    InvalidRangeSize,
    /// Checked offset plus size exceeds the retained region or overflows nat64.
    #[error("snapshot range is outside retained metadata")]
    RangeOutsideMetadata,
    /// Chunk identity is not exactly 32 SHA-256 bytes.
    #[error("snapshot chunk identity must be 32 bytes")]
    InvalidChunkHash,
    /// The exact chunk identity is absent from retained metadata.
    #[error("snapshot chunk identity is absent from retained metadata")]
    ChunkNotInMetadata,
    /// Raw wire input exceeded its bound before parsing.
    #[error("snapshot data reply exceeds {MAX_IC_SNAPSHOT_DATA_REPLY_BYTES} bytes")]
    ReplyTooLarge,
    /// Wire shape, decoder work, header or actual chunk bounds failed.
    #[error("invalid or unsupported snapshot data reply")]
    InvalidReply,
    /// The decoded range is shorter or longer than the exact request.
    #[error("snapshot data length differs from the requested range")]
    LengthMismatch,
    /// Actual chunk bytes do not hash to the requested retained identity.
    #[error("snapshot data differs from the requested chunk hash")]
    ChunkHashMismatch,
}

#[cfg(test)]
mod tests;
