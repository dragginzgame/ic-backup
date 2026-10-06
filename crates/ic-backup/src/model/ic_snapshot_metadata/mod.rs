//! Bounded snapshot metadata reads; decoded declarations do not prove transfer.

mod wire;

use super::{
    artifacts::ArtifactChecksumRecord,
    ic_request::{
        IcRequestError, MAX_IC_ARGUMENT_BYTES, MAX_IC_SNAPSHOT_ID_BYTES, management_request_digest,
    },
};
use candid::Principal;
use ic_management_canister_types::{
    ReadCanisterSnapshotMetadataArgs, ReadCanisterSnapshotMetadataResult,
};
use std::fmt;
use thiserror::Error;

/// Maximum raw metadata reply bytes, checked before decoding.
pub const MAX_IC_SNAPSHOT_METADATA_BYTES: usize = 1024 * 1024;
/// Local bound on ordered exported globals, including unavailable slots.
pub const MAX_IC_SNAPSHOT_GLOBALS: usize = 4096;
/// Local bound on distinct SHA-256 chunk-store entries.
pub const MAX_IC_SNAPSHOT_CHUNKS: usize = 1024;

/// Exact ephemeral metadata-read payload for one retained raw snapshot identity.
///
/// This separate transfer read leaves the six-method lifecycle/recovery record
/// unchanged. It carries no plan, reservation, signing, freshness or permissions.
/// Integrations own authenticated snapshot association and prior per-call accounting.
#[derive(Clone)]
pub struct IcSnapshotMetadataRequest {
    target: String,
    target_bytes: Vec<u8>,
    snapshot_id: Vec<u8>,
    arguments: Vec<u8>,
}

impl IcSnapshotMetadataRequest {
    /// Normalize the target and encode the pinned upstream argument shape.
    ///
    /// # Errors
    /// Rejects malformed principals, empty/oversized raw identifiers, encoding
    /// failure or arguments exceeding the existing 4 KiB request bound.
    pub fn new(target: &str, snapshot_id: &[u8]) -> Result<Self, IcRequestError> {
        if snapshot_id.is_empty() || snapshot_id.len() > MAX_IC_SNAPSHOT_ID_BYTES {
            return Err(IcRequestError::InvalidSnapshotId);
        }
        let target =
            super::principal::canonical_text(target).ok_or(IcRequestError::InvalidTarget)?;
        let principal = Principal::from_text(&target).map_err(|_| IcRequestError::InvalidTarget)?;
        let arguments = candid::encode_one(ReadCanisterSnapshotMetadataArgs {
            canister_id: principal,
            snapshot_id: snapshot_id.to_vec(),
        })
        .map_err(|error| IcRequestError::Encoding(error.to_string()))?;
        if arguments.len() > MAX_IC_ARGUMENT_BYTES {
            return Err(IcRequestError::ArgumentsTooLarge);
        }
        Ok(Self {
            target,
            target_bytes: principal.as_slice().to_vec(),
            snapshot_id: snapshot_id.to_vec(),
            arguments,
        })
    }

    /// Read the canonical effective routing target, also encoded in the arguments.
    #[must_use]
    pub fn target(&self) -> &str {
        &self.target
    }

    /// Read exact raw snapshot bytes, distinct from a generic backend token.
    #[must_use]
    pub fn snapshot_id(&self) -> &[u8] {
        &self.snapshot_id
    }

    /// Read exact Candid arguments; no transport or dispatch is installed.
    #[must_use]
    pub fn arguments(&self) -> &[u8] {
        &self.arguments
    }

    /// Read the fixed management receiver, separate from the routing target.
    #[must_use]
    pub const fn receiver(&self) -> &'static str {
        "aaaaa-aa"
    }

    /// Read the sole method, using replicated host update ingress.
    #[must_use]
    pub const fn method(&self) -> &'static str {
        "read_canister_snapshot_metadata"
    }

    /// Hash exact receiver/routing/update-mode/method/arguments with the existing owner.
    #[must_use]
    pub fn digest(&self) -> ArtifactChecksumRecord {
        management_request_digest(&self.target_bytes, self.method(), &self.arguments)
    }
}

impl fmt::Debug for IcSnapshotMetadataRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IcSnapshotMetadataRequest")
            .field("target", &self.target)
            .field("snapshot_id_bytes", &self.snapshot_id.len())
            .finish_non_exhaustive()
    }
}

/// Read-only pinned metadata and exact raw evidence under a declared request.
///
/// Optional source, unavailable globals and optional timer/hook fields retain
/// their actual presence. Missing information is not a default or upload input.
/// Floating globals retain their bits; globals and chunk rows retain wire order.
/// This evidence grants no complete transfer, authentic origin or effect outcome.
pub struct IcSnapshotMetadataReply<'request> {
    request: &'request IcSnapshotMetadataRequest,
    metadata: ReadCanisterSnapshotMetadataResult,
    payload_checksum: ArtifactChecksumRecord,
}

impl<'request> IcSnapshotMetadataReply<'request> {
    /// Decode one bounded metadata argument without skipped fields or extra bytes.
    ///
    /// # Errors
    /// Rejects excessive raw bytes, malformed/unsupported wire values, sequence
    /// bounds, certified data above 32 bytes, non-SHA-256/duplicate chunk hashes
    /// and v128 globals outside their unsigned 128-bit range.
    pub fn decode(
        request: &'request IcSnapshotMetadataRequest,
        bytes: &[u8],
    ) -> Result<Self, IcSnapshotMetadataError> {
        if bytes.len() > MAX_IC_SNAPSHOT_METADATA_BYTES {
            return Err(IcSnapshotMetadataError::ReplyTooLarge);
        }
        let metadata = wire::decode(bytes)?;
        Ok(Self {
            request,
            metadata,
            payload_checksum: ArtifactChecksumRecord::from_bytes(bytes),
        })
    }

    /// Read the exact declared request; the reply wire authenticates no target.
    #[must_use]
    pub const fn request(&self) -> &'request IcSnapshotMetadataRequest {
        self.request
    }

    /// Read admitted pinned DTO values without mutation access.
    #[must_use]
    pub const fn metadata(&self) -> &ReadCanisterSnapshotMetadataResult {
        &self.metadata
    }

    /// Read SHA-256 of exact raw bytes, including optional fields and wire ordering.
    #[must_use]
    pub const fn payload_checksum(&self) -> &ArtifactChecksumRecord {
        &self.payload_checksum
    }

    /// Bind original request and raw reply checksums in the metadata evidence domain.
    #[must_use]
    pub fn digest(&self) -> ArtifactChecksumRecord {
        let mut bytes = b"ic-backup/ic-snapshot-metadata-reply/v1\0".to_vec();
        bytes.extend_from_slice(self.request.digest().hash().as_bytes());
        bytes.extend_from_slice(self.payload_checksum.hash().as_bytes());
        ArtifactChecksumRecord::from_bytes(&bytes)
    }
}

impl fmt::Debug for IcSnapshotMetadataReply<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IcSnapshotMetadataReply")
            .field("request", &self.request)
            .field("payload_checksum", &self.payload_checksum)
            .finish_non_exhaustive()
    }
}

/// Typed metadata admission errors, excluding raw bytes and decoder diagnostics.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum IcSnapshotMetadataError {
    /// Input exceeded the local raw-byte bound before parsing.
    #[error("snapshot metadata exceeds {MAX_IC_SNAPSHOT_METADATA_BYTES} bytes")]
    ReplyTooLarge,
    /// Wire shape, finite work, sequence or certified-data bounds failed.
    #[error("invalid or unsupported snapshot metadata reply")]
    InvalidReply,
    /// A v128 value exceeded its unsigned 128-bit representation.
    #[error("snapshot v128 global exceeds 128 bits")]
    InvalidGlobal,
    /// Chunk identities must be distinct exact SHA-256 hashes.
    #[error("snapshot chunk hash is not a unique 32-byte identity")]
    InvalidChunkHash,
}

#[cfg(test)]
mod tests;
