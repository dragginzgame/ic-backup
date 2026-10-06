//! Exact same-target snapshot upload codecs and passive original-reservation binding.

mod attempt;
mod reply;
pub(crate) use attempt::original_authority;
pub use attempt::{IcSnapshotUploadAttempt, IcSnapshotUploadAttemptError};
pub use reply::{IcSnapshotUploadReply, IcSnapshotUploadReplyKind};

use super::{
    artifacts::{ArtifactChecksumRecord, ChecksumError},
    ic_request::{IcRequestError, MAX_IC_SNAPSHOT_ID_BYTES, management_request_digest},
    ic_snapshot_data::{IcSnapshotDataError, MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES, validate_kind},
    ic_snapshot_metadata::IcSnapshotMetadataReply,
    operation_plan::OperationPlanRecord,
};
use candid::Principal;
use ic_management_canister_types::{
    SnapshotDataKind, SnapshotDataOffset, UploadCanisterSnapshotDataArgs,
    UploadCanisterSnapshotMetadataArgs,
};
use std::fmt;
use thiserror::Error;

/// Maximum upload arguments, including finite Candid overhead around a 1 MiB chunk.
pub const MAX_IC_SNAPSHOT_UPLOAD_ARGUMENT_BYTES: usize = 2 * 1024 * 1024;
/// Maximum raw upload reply bytes before finite decoding.
pub const MAX_IC_SNAPSHOT_UPLOAD_REPLY_BYTES: usize = 4096;

/// Closed upload declaration; each branch retains exact original source association.
#[derive(Debug)]
pub enum IcSnapshotUploadKind {
    /// Allocate a new snapshot with exact representable metadata, without replacement.
    Metadata,
    /// Write exact bounded bytes into an independently qualified new upload snapshot.
    Data {
        /// Exact destination raw ID, separate from the retained source raw ID/token.
        snapshot_id: Vec<u8>,
        /// Original source extent or known hash; chunk hashes are not part of upload wire.
        source_kind: SnapshotDataKind,
        /// Hash of the actual bounded chunk bytes encoded in the arguments.
        chunk_checksum: ArtifactChecksumRecord,
        /// Exact original metadata-allocation wire request digest.
        metadata_request: ArtifactChecksumRecord,
    },
}

/// Ephemeral exact wire payload with retained source evidence; no dispatch permission.
///
/// Pure construction admits declarations only. Guarded persistence operations supply
/// fresh local byte binding. Integrations qualify authentic source/complete transfer,
/// new destination attribution, original spending, current controller/custody and
/// same-release application safety. No serialization, replacement, relocation,
/// allowance or automatic outcome is introduced.
pub struct IcSnapshotUploadRequest<'source> {
    source_plan: &'source OperationPlanRecord,
    source: &'source IcSnapshotMetadataReply<'source>,
    source_checksum: ArtifactChecksumRecord,
    kind: IcSnapshotUploadKind,
    arguments: Vec<u8>,
    target_bytes: Vec<u8>,
}

impl<'source> IcSnapshotUploadRequest<'source> {
    /// Encode exact uploadable original metadata for the same declared canister.
    ///
    /// Globals retain order and bits. Unavailable slots reject; absent timer/hook
    /// values stay absent and do not default to inactive/ready states. Replacement
    /// is always absent; this payload has no source-snapshot deletion lane.
    /// Build these nonrecursive bytes before retaining their original operation plan.
    /// # Errors
    /// Rejects unavailable globals, encoding failure and excessive arguments.
    pub fn metadata(
        source_plan: &'source OperationPlanRecord,
        source: &'source IcSnapshotMetadataReply<'source>,
        source_checksum: &ArtifactChecksumRecord,
    ) -> Result<Self, IcSnapshotUploadError> {
        if !source_plan
            .selected_targets()
            .iter()
            .any(|target| target == source.request().target())
        {
            return Err(IcSnapshotUploadError::SourceTargetMismatch);
        }
        let values = source.metadata();
        let globals = values
            .globals
            .iter()
            .cloned()
            .collect::<Option<Vec<_>>>()
            .ok_or(IcSnapshotUploadError::UnavailableGlobal)?;
        let target = Principal::from_text(source.request().target())
            .map_err(|_| IcRequestError::InvalidTarget)?;
        let arguments = candid::encode_one(UploadCanisterSnapshotMetadataArgs {
            canister_id: target,
            replace_snapshot: None,
            wasm_module_size: values.wasm_module_size,
            globals,
            wasm_memory_size: values.wasm_memory_size,
            stable_memory_size: values.stable_memory_size,
            certified_data: values.certified_data.clone(),
            global_timer: values.global_timer.clone(),
            on_low_wasm_memory_hook_status: values.on_low_wasm_memory_hook_status.clone(),
        })
        .map_err(|error| IcRequestError::Encoding(error.to_string()))?;
        Self::from_arguments(
            source_plan,
            source,
            source_checksum,
            IcSnapshotUploadKind::Metadata,
            arguments,
        )
    }

    /// Encode one exact source extent/chunk for a separately qualified new raw ID.
    ///
    /// Requires the original metadata upload declaration. Regions are nonempty and
    /// bounded to 1 MiB; an empty known chunk requires its exact SHA-256. Actual
    /// new snapshot attribution is not inferred from a supplied ID or decoded reply.
    /// Data requests include their new ID before their own plan/reservation is retained;
    /// metadata authority never supplies data-call spending or resets prior journals.
    /// # Errors
    /// Rejects a non-metadata original, invalid/reused ID, source range/hash or byte
    /// mismatch, excessive data/arguments and encoding failure.
    pub fn data(
        metadata: &Self,
        snapshot_id: &[u8],
        source_kind: SnapshotDataKind,
        chunk: &[u8],
    ) -> Result<Self, IcSnapshotUploadError> {
        metadata.validate_data_destination(snapshot_id)?;
        validate_kind(metadata.source, &source_kind)?;
        if chunk.len() > MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES {
            return Err(IcSnapshotUploadError::ChunkTooLarge);
        }
        let checksum = ArtifactChecksumRecord::from_bytes(chunk);
        let kind = match &source_kind {
            SnapshotDataKind::WasmModule { offset, size } => {
                require_length(chunk, *size)?;
                SnapshotDataOffset::WasmModule { offset: *offset }
            }
            SnapshotDataKind::WasmMemory { offset, size } => {
                require_length(chunk, *size)?;
                SnapshotDataOffset::WasmMemory { offset: *offset }
            }
            SnapshotDataKind::StableMemory { offset, size } => {
                require_length(chunk, *size)?;
                SnapshotDataOffset::StableMemory { offset: *offset }
            }
            SnapshotDataKind::WasmChunk { hash } => {
                checksum.verify(&crate::hash::hex_bytes(hash))?;
                SnapshotDataOffset::WasmChunk
            }
        };
        let target =
            Principal::from_text(metadata.target()).map_err(|_| IcRequestError::InvalidTarget)?;
        let arguments = candid::encode_one(UploadCanisterSnapshotDataArgs {
            canister_id: target,
            snapshot_id: snapshot_id.to_vec(),
            kind,
            chunk: chunk.to_vec(),
        })
        .map_err(|error| IcRequestError::Encoding(error.to_string()))?;
        Self::from_arguments(
            metadata.source_plan,
            metadata.source,
            &metadata.source_checksum,
            IcSnapshotUploadKind::Data {
                snapshot_id: snapshot_id.to_vec(),
                source_kind,
                chunk_checksum: checksum,
                metadata_request: metadata.digest(),
            },
            arguments,
        )
    }

    fn from_arguments(
        source_plan: &'source OperationPlanRecord,
        source: &'source IcSnapshotMetadataReply<'source>,
        source_checksum: &ArtifactChecksumRecord,
        kind: IcSnapshotUploadKind,
        arguments: Vec<u8>,
    ) -> Result<Self, IcSnapshotUploadError> {
        if arguments.len() > MAX_IC_SNAPSHOT_UPLOAD_ARGUMENT_BYTES {
            return Err(IcSnapshotUploadError::ArgumentsTooLarge);
        }
        let target = Principal::from_text(source.request().target())
            .map_err(|_| IcRequestError::InvalidTarget)?;
        Ok(Self {
            source_plan,
            source,
            source_checksum: source_checksum.clone(),
            kind,
            arguments,
            target_bytes: target.as_slice().to_vec(),
        })
    }

    pub(crate) fn validate_data_destination(
        &self,
        snapshot_id: &[u8],
    ) -> Result<(), IcSnapshotUploadError> {
        if !matches!(self.kind, IcSnapshotUploadKind::Metadata) {
            return Err(IcSnapshotUploadError::MetadataRequestRequired);
        }
        validate_destination(self.source, snapshot_id)
    }

    /// Read the exact original source plan; later upload intent must retain its context.
    #[must_use]
    pub const fn source_plan(&self) -> &'source OperationPlanRecord {
        self.source_plan
    }

    /// Read original metadata/request evidence; it authenticates no source.
    #[must_use]
    pub const fn source(&self) -> &'source IcSnapshotMetadataReply<'source> {
        self.source
    }
    /// Read declared retained tree checksum; pure construction verifies no files.
    #[must_use]
    pub const fn source_checksum(&self) -> &ArtifactChecksumRecord {
        &self.source_checksum
    }
    /// Read the closed kind and exact source/destination association.
    #[must_use]
    pub const fn kind(&self) -> &IcSnapshotUploadKind {
        &self.kind
    }
    /// Read the same canonical source/destination canister target.
    #[must_use]
    pub fn target(&self) -> &str {
        self.source.request().target()
    }
    /// Read fixed management receiver, separate from effective routing.
    #[must_use]
    pub const fn receiver(&self) -> &'static str {
        "aaaaa-aa"
    }
    /// Read the exact replicated host-update method; no query mode exists.
    #[must_use]
    pub const fn method(&self) -> &'static str {
        match self.kind {
            IcSnapshotUploadKind::Metadata => "upload_canister_snapshot_metadata",
            IcSnapshotUploadKind::Data { .. } => "upload_canister_snapshot_data",
        }
    }
    /// Read exact canonical encoded bytes, without signing or spending.
    #[must_use]
    pub fn arguments(&self) -> &[u8] {
        &self.arguments
    }
    /// Hash wire payload only, before plan/authority derivation, using its canonical owner.
    #[must_use]
    pub fn digest(&self) -> ArtifactChecksumRecord {
        management_request_digest(&self.target_bytes, self.method(), &self.arguments)
    }
    /// Bind original source evidence/tree and metadata origin separately from wire intent.
    #[must_use]
    pub fn binding_digest(&self) -> ArtifactChecksumRecord {
        let mut bytes = b"ic-backup/ic-snapshot-upload-binding/v1\0".to_vec();
        bytes.extend_from_slice(self.source_plan.digest().hash().as_bytes());
        bytes.extend_from_slice(self.source.digest().hash().as_bytes());
        bytes.extend_from_slice(self.source_checksum.hash().as_bytes());
        if let IcSnapshotUploadKind::Data {
            metadata_request, ..
        } = &self.kind
        {
            bytes.push(1);
            bytes.extend_from_slice(metadata_request.hash().as_bytes());
        } else {
            bytes.push(0);
        }
        bytes.extend_from_slice(self.digest().hash().as_bytes());
        ArtifactChecksumRecord::from_bytes(&bytes)
    }
}

pub(super) fn validate_destination(
    source: &IcSnapshotMetadataReply<'_>,
    snapshot_id: &[u8],
) -> Result<(), IcSnapshotUploadError> {
    if snapshot_id.is_empty() || snapshot_id.len() > MAX_IC_SNAPSHOT_ID_BYTES {
        return Err(IcSnapshotUploadError::InvalidDestination);
    }
    if snapshot_id == source.request().snapshot_id() {
        return Err(IcSnapshotUploadError::DestinationReusesSource);
    }
    Ok(())
}

fn require_length(chunk: &[u8], length: u64) -> Result<(), IcSnapshotUploadError> {
    if u64::try_from(chunk.len()).ok() != Some(length) {
        return Err(IcSnapshotUploadError::ChunkLengthMismatch);
    }
    Ok(())
}

impl fmt::Debug for IcSnapshotUploadRequest<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IcSnapshotUploadRequest")
            .field("target", &self.target())
            .field("method", &self.method())
            .field("argument_bytes", &self.arguments.len())
            .finish_non_exhaustive()
    }
}

/// Typed bounded upload denial; no error changes retained spending or source evidence.
#[derive(Debug, Error)]
pub enum IcSnapshotUploadError {
    /// Exact original source target is outside its retained plan selection.
    #[error("snapshot upload source target differs from the original selection")]
    SourceTargetMismatch,
    /// A missing original global cannot be represented by the upstream upload type.
    #[error("snapshot upload requires every original global value")]
    UnavailableGlobal,
    /// Data upload must retain its exact original metadata-allocation declaration.
    #[error("snapshot upload requires an original metadata request")]
    MetadataRequestRequired,
    /// Destination raw bytes violate the existing 1..=256 ID bounds.
    #[error("snapshot upload destination ID is invalid")]
    InvalidDestination,
    /// The new upload declaration attempts to reuse its retained original source ID.
    #[error("snapshot upload destination reuses the original source ID")]
    DestinationReusesSource,
    /// Actual bounded bytes differ from the requested source range length.
    #[error("snapshot upload chunk length differs from the source range")]
    ChunkLengthMismatch,
    /// Actual chunk exceeds 1 MiB before cloning or encoding.
    #[error("snapshot upload chunk exceeds bound")]
    ChunkTooLarge,
    /// Encoded arguments exceed the separate 2 MiB upload ceiling.
    #[error("snapshot upload arguments exceed bound")]
    ArgumentsTooLarge,
    /// Raw reply exceeds 4 KiB before decoding.
    #[error("snapshot upload reply exceeds bound")]
    ReplyTooLarge,
    /// Finite exact method-specific reply decoding rejected bytes.
    #[error("snapshot upload reply is invalid")]
    InvalidReply,
    /// Existing range/known-hash owner rejected the source kind.
    #[error(transparent)]
    Data(#[from] IcSnapshotDataError),
    /// The exact original chunk digest differs.
    #[error(transparent)]
    Checksum(#[from] ChecksumError),
    /// Canonical principal or upstream Candid encoding failed.
    #[error(transparent)]
    Request(#[from] IcRequestError),
}

#[cfg(test)]
mod tests;
