//! Bounded exact upload results; passive association never settles an attempt.

use super::{
    IcSnapshotUploadError, IcSnapshotUploadKind, IcSnapshotUploadRequest,
    MAX_IC_SNAPSHOT_UPLOAD_REPLY_BYTES, validate_destination,
};
use crate::model::{
    artifacts::ArtifactChecksumRecord, ic_lifecycle_reply::EMPTY_CANDID_REPLY,
    ic_snapshot_reply::snapshot_id,
};
use candid::CandidType;
use ic_testkit_candid::{DecodeLimits, decode_exactly_one};
use serde::Deserialize;
use std::fmt;

#[derive(CandidType, Deserialize)]
struct WireMetadataReply {
    #[serde(deserialize_with = "snapshot_id")]
    snapshot_id: Vec<u8>,
}

/// Passive local wire result; neither branch authenticates an effect or new destination.
#[derive(Debug, Eq, PartialEq)]
pub enum IcSnapshotUploadReplyKind {
    /// Exact returned raw ID, distinct from retained source and generic backend tokens.
    Metadata {
        /// Bounded original response bytes, with no automatic authority derivation.
        snapshot_id: Vec<u8>,
    },
    /// Exact canonical empty Candid tuple; not data-transfer completeness.
    DataAcknowledgement,
}

/// Read-only decoded association with the exact source-bound upload declaration.
pub struct IcSnapshotUploadReply<'request, 'source> {
    request: &'request IcSnapshotUploadRequest<'source>,
    kind: IcSnapshotUploadReplyKind,
    payload_checksum: ArtifactChecksumRecord,
}

impl<'request, 'source> IcSnapshotUploadReply<'request, 'source> {
    /// Admit an exact metadata ID or canonical empty data reply under finite quotas.
    ///
    /// Reuses the bounded raw-ID visitor and lifecycle empty tuple owner. Exact wire
    /// association proves no target/context authentication, exclusive allocation,
    /// complete upload, new spending, retry, mutation settlement or release.
    /// # Errors
    /// Rejects excessive/invalid/extended/trailing replies and invalid/reused IDs.
    pub fn decode(
        request: &'request IcSnapshotUploadRequest<'source>,
        bytes: &[u8],
    ) -> Result<Self, IcSnapshotUploadError> {
        if bytes.len() > MAX_IC_SNAPSHOT_UPLOAD_REPLY_BYTES {
            return Err(IcSnapshotUploadError::ReplyTooLarge);
        }
        let kind = match request.kind() {
            IcSnapshotUploadKind::Metadata => {
                let reply: WireMetadataReply = decode_exactly_one(
                    bytes,
                    DecodeLimits {
                        reply_bytes: MAX_IC_SNAPSHOT_UPLOAD_REPLY_BYTES,
                        decoding_quota: 64 * 1024,
                        skipping_quota: 0,
                        type_entries: 16,
                        header_bytes: 4096,
                    },
                )
                .map_err(|_| IcSnapshotUploadError::InvalidReply)?;
                validate_destination(request.source(), &reply.snapshot_id)?;
                IcSnapshotUploadReplyKind::Metadata {
                    snapshot_id: reply.snapshot_id,
                }
            }
            IcSnapshotUploadKind::Data { .. } => {
                if bytes != EMPTY_CANDID_REPLY {
                    return Err(IcSnapshotUploadError::InvalidReply);
                }
                IcSnapshotUploadReplyKind::DataAcknowledgement
            }
        };
        Ok(Self {
            request,
            kind,
            payload_checksum: ArtifactChecksumRecord::from_bytes(bytes),
        })
    }
    /// Read the original exact source/wire declaration.
    #[must_use]
    pub const fn request(&self) -> &'request IcSnapshotUploadRequest<'source> {
        self.request
    }
    /// Read passive method-specific data without outcome or dispatch admission.
    #[must_use]
    pub const fn kind(&self) -> &IcSnapshotUploadReplyKind {
        &self.kind
    }
    /// Read exact raw-reply checksum, independent of decoded canonical values.
    #[must_use]
    pub const fn payload_checksum(&self) -> &ArtifactChecksumRecord {
        &self.payload_checksum
    }
    /// Bind exact original source/wire association and raw bytes as local evidence.
    #[must_use]
    pub fn digest(&self) -> ArtifactChecksumRecord {
        crate::model::evidence_digest::reply(
            b"ic-backup/ic-snapshot-upload-reply/v1\0",
            [&self.request.binding_digest(), &self.payload_checksum],
        )
    }
}

impl fmt::Debug for IcSnapshotUploadReply<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IcSnapshotUploadReply")
            .field("method", &self.request.method())
            .field("payload_checksum", &self.payload_checksum)
            .finish_non_exhaustive()
    }
}
