//! One IC snapshot artifact per original transfer stage, with model-owned publication.

use crate::model::evidence_digest::EvidenceDigest;

use super::artifacts::{ArtifactChecksumRecord, ChecksumError, canonical_hash};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Maximum encoded journal bytes admitted by persistence operations.
pub const MAX_DOWNLOAD_JOURNAL_BYTES: u64 = 1024 * 1024;
/// Maximum ASCII bytes in an opaque backend snapshot identifier.
pub const MAX_SNAPSHOT_ID_BYTES: usize = 256;

/// Snapshot identity admitted by the caller after authoritative capture observation.
///
/// This passive request grants no remote authority and contains no signing material.
#[derive(Clone, Debug)]
pub struct DownloadArtifactRequest {
    /// Exact source canister principal, normalized on admission.
    pub canister_id: String,
    /// Exact backend snapshot token; no case folding or inferred identity.
    pub snapshot_id: String,
    /// Observed snapshot timestamp in nanoseconds.
    pub snapshot_taken_at_timestamp: u64,
    /// Observed snapshot size, not the encoded artifact directory's byte length.
    pub snapshot_total_size_bytes: u64,
}

/// Local IC artifact publication states; partial transfers retain Created.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ArtifactStateRecord {
    /// Exact snapshot identity retained before local download completion.
    Created,
    /// Local bytes verified and their canonical checksum retained.
    ChecksumVerified,
    /// Verified bytes durably published at the canonical location.
    Durable,
}

// Internal transition inputs cannot pair a state with inappropriate checksum
// evidence. Persisted fields retain their independent decoding/corruption checks.
pub(crate) enum ArtifactTransition {
    ChecksumVerified(ArtifactChecksumRecord),
    Durable,
}

impl ArtifactTransition {
    const fn state(&self) -> ArtifactStateRecord {
        match self {
            Self::ChecksumVerified(_) => ArtifactStateRecord::ChecksumVerified,
            Self::Durable => ArtifactStateRecord::Durable,
        }
    }
}

/// Immutable exact snapshot identity with validated local progress evidence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "ArtifactFields")]
pub struct DownloadArtifactRecord {
    canister_id: String,
    snapshot_id: String,
    snapshot_taken_at_timestamp: u64,
    snapshot_total_size_bytes: u64,
    staging_path: String,
    artifact_path: String,
    #[serde(flatten)]
    progress: ArtifactProgress,
}

// Keep verified evidence in the state that requires it; only boundary fields are independent.
#[derive(Clone, Debug, Eq, PartialEq)]
enum ArtifactProgress {
    Created,
    ChecksumVerified(ArtifactChecksumRecord),
    Durable(ArtifactChecksumRecord),
}
impl ArtifactProgress {
    const fn state(&self) -> ArtifactStateRecord {
        match self {
            Self::Created => ArtifactStateRecord::Created,
            Self::ChecksumVerified(_) => ArtifactStateRecord::ChecksumVerified,
            Self::Durable(_) => ArtifactStateRecord::Durable,
        }
    }
    const fn checksum(&self) -> Option<&ArtifactChecksumRecord> {
        match self {
            Self::ChecksumVerified(checksum) | Self::Durable(checksum) => Some(checksum),
            Self::Created => None,
        }
    }
}
impl Serialize for ArtifactProgress {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut fields = serializer.serialize_struct("ArtifactProgress", 2)?;
        fields.serialize_field("state", &self.state())?;
        fields.serialize_field("checksum", &self.checksum())?;
        fields.end()
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ArtifactFields {
    canister_id: String,
    snapshot_id: String,
    snapshot_taken_at_timestamp: u64,
    snapshot_total_size_bytes: u64,
    staging_path: String,
    artifact_path: String,
    state: ArtifactStateRecord,
    #[serde(deserialize_with = "crate::model::required_option::deserialize")]
    checksum: Option<ArtifactChecksumRecord>,
}

impl TryFrom<ArtifactFields> for DownloadArtifactRecord {
    type Error = DownloadJournalRecordError;
    fn try_from(fields: ArtifactFields) -> Result<Self, Self::Error> {
        let mut record = Self::new(DownloadArtifactRequest {
            canister_id: fields.canister_id,
            snapshot_id: fields.snapshot_id,
            snapshot_taken_at_timestamp: fields.snapshot_taken_at_timestamp,
            snapshot_total_size_bytes: fields.snapshot_total_size_bytes,
        })?;
        // Persisted paths must agree with normalized identity, never with a label.
        if fields.staging_path != record.staging_path
            || fields.artifact_path != record.artifact_path
        {
            return Err(DownloadJournalRecordError::ArtifactPathMismatch);
        }
        record.progress = match (fields.state, fields.checksum) {
            (ArtifactStateRecord::Created, None) => ArtifactProgress::Created,
            (ArtifactStateRecord::ChecksumVerified, Some(checksum)) => {
                ArtifactProgress::ChecksumVerified(checksum)
            }
            (ArtifactStateRecord::Durable, Some(checksum)) => ArtifactProgress::Durable(checksum),
            (state, _) => return Err(DownloadJournalRecordError::InvalidChecksumState(state)),
        };
        Ok(record)
    }
}

impl DownloadArtifactRecord {
    fn new(request: DownloadArtifactRequest) -> Result<Self, DownloadJournalRecordError> {
        let canister_id = normalize_principal(&request.canister_id)?;
        if request.snapshot_id.is_empty()
            || request.snapshot_id.len() > MAX_SNAPSHOT_ID_BYTES
            || !request
                .snapshot_id
                .bytes()
                .all(|byte| byte.is_ascii_graphic())
        {
            return Err(DownloadJournalRecordError::InvalidSnapshotId);
        }
        Ok(Self {
            staging_path: format!("artifacts/{canister_id}.tmp"),
            artifact_path: format!("artifacts/{canister_id}"),
            canister_id,
            snapshot_id: request.snapshot_id,
            snapshot_taken_at_timestamp: request.snapshot_taken_at_timestamp,
            snapshot_total_size_bytes: request.snapshot_total_size_bytes,
            progress: ArtifactProgress::Created,
        })
    }

    /// Return the normalized exact source principal.
    #[must_use]
    pub fn canister_id(&self) -> &str {
        &self.canister_id
    }
    /// Return the exact backend snapshot token.
    #[must_use]
    pub fn snapshot_id(&self) -> &str {
        &self.snapshot_id
    }
    /// Return the observed snapshot timestamp in nanoseconds.
    #[must_use]
    pub const fn snapshot_taken_at_timestamp(&self) -> u64 {
        self.snapshot_taken_at_timestamp
    }
    /// Return the observed snapshot size; encoded directory size may differ.
    #[must_use]
    pub const fn snapshot_total_size_bytes(&self) -> u64 {
        self.snapshot_total_size_bytes
    }
    /// Return the fixed relative staging directory for this principal.
    #[must_use]
    pub fn staging_path(&self) -> &str {
        &self.staging_path
    }
    /// Return the fixed relative canonical directory for this principal.
    #[must_use]
    pub fn artifact_path(&self) -> &str {
        &self.artifact_path
    }
    /// Return retained local lifecycle state.
    #[must_use]
    pub const fn state(&self) -> ArtifactStateRecord {
        self.progress.state()
    }
    /// Return the retained exact checksum when verification has completed.
    #[must_use]
    pub const fn checksum(&self) -> Option<&ArtifactChecksumRecord> {
        self.progress.checksum()
    }
}

/// Maintained v1 local artifact journal bound to a caller-supplied intent digest.
///
/// This record neither authenticates that intent nor proves current remote authority.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "JournalFields")]
pub struct DownloadJournalRecord {
    version: u16,
    intent: String,
    artifacts: [DownloadArtifactRecord; 1],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct JournalFields {
    version: u16,
    intent: String,
    artifacts: [DownloadArtifactRecord; 1],
}

impl TryFrom<JournalFields> for DownloadJournalRecord {
    type Error = DownloadJournalRecordError;
    fn try_from(fields: JournalFields) -> Result<Self, Self::Error> {
        if fields.version != 1 {
            return Err(DownloadJournalRecordError::UnsupportedVersion(
                fields.version,
            ));
        }
        Ok(Self {
            version: 1,
            intent: canonical_hash(&fields.intent)?,
            artifacts: fields.artifacts,
        })
    }
}

impl DownloadJournalRecord {
    /// Admit one snapshot artifact under its original transfer intent.
    /// # Errors
    /// Rejects invalid source identities, tokens and intent digests.
    pub fn new(
        intent: &str,
        request: DownloadArtifactRequest,
    ) -> Result<Self, DownloadJournalRecordError> {
        JournalFields {
            version: 1,
            intent: intent.to_owned(),
            artifacts: [DownloadArtifactRecord::new(request)?],
        }
        .try_into()
    }

    /// Return the canonical caller-supplied immutable intent digest.
    #[must_use]
    pub fn intent(&self) -> &str {
        &self.intent
    }
    /// Read the sole artifact in the retained v1 singleton wire envelope.
    #[must_use]
    pub const fn artifacts(&self) -> &[DownloadArtifactRecord; 1] {
        &self.artifacts
    }

    /// Hash canonical original intent, exact snapshot metadata, paths and local evidence.
    ///
    /// The NUL-terminated v1 domain precedes 64 ASCII intent bytes and a big-endian
    /// u64 entry count. Each canonical entry has four u32-length-prefixed UTF-8
    /// strings (principal, snapshot token, staging path, artifact path), two u64
    /// metadata values, a state byte (Created=0, ChecksumVerified=2, Durable=3), then a checksum
    /// presence byte and its 64 ASCII hash when present. Integers are big-endian.
    /// This grants no transfer, snapshot authenticity or terminal proof.
    #[must_use]
    pub fn digest(&self) -> ArtifactChecksumRecord {
        let mut bytes = EvidenceDigest::new(b"ic-backup/download-journal/v1\0");
        bytes.append(self.intent.as_bytes());
        bytes.u64_be(self.artifacts.len() as u64);
        for entry in &self.artifacts {
            for value in [
                entry.canister_id(),
                entry.snapshot_id(),
                entry.staging_path(),
                entry.artifact_path(),
            ] {
                bytes.bytes_u32(value.as_bytes());
            }
            bytes.u64_be(entry.snapshot_taken_at_timestamp);
            bytes.u64_be(entry.snapshot_total_size_bytes);
            bytes.byte(match entry.state() {
                ArtifactStateRecord::Created => 0,
                ArtifactStateRecord::ChecksumVerified => 2,
                ArtifactStateRecord::Durable => 3,
            });
            bytes.byte(u8::from(entry.checksum().is_some()));
            if let Some(checksum) = entry.checksum() {
                bytes.checksum(checksum);
            }
        }
        bytes.finish()
    }

    pub(crate) fn artifact(
        &self,
        canister: &str,
        snapshot: &str,
    ) -> Result<&DownloadArtifactRecord, DownloadJournalRecordError> {
        self.admit_identity(canister, snapshot)?;
        Ok(&self.artifacts[0])
    }

    fn admit_identity(
        &self,
        canister: &str,
        snapshot: &str,
    ) -> Result<(), DownloadJournalRecordError> {
        let canister = normalize_principal(canister)?;
        if self.artifacts[0].canister_id != canister {
            return Err(DownloadJournalRecordError::UnknownArtifact);
        }
        if self.artifacts[0].snapshot_id != snapshot {
            return Err(DownloadJournalRecordError::SnapshotMismatch);
        }
        Ok(())
    }

    pub(crate) fn advance(
        &mut self,
        canister: &str,
        snapshot: &str,
        transition: ArtifactTransition,
    ) -> Result<(), DownloadJournalRecordError> {
        self.admit_identity(canister, snapshot)?;
        let entry = &mut self.artifacts[0];
        entry.progress = match (&entry.progress, transition) {
            (ArtifactProgress::Created, ArtifactTransition::ChecksumVerified(checksum)) => {
                ArtifactProgress::ChecksumVerified(checksum)
            }
            (ArtifactProgress::ChecksumVerified(checksum), ArtifactTransition::Durable) => {
                ArtifactProgress::Durable(checksum.clone())
            }
            (_, transition) => {
                return Err(DownloadJournalRecordError::InvalidStateTransition {
                    from: entry.state(),
                    to: transition.state(),
                });
            }
        };
        Ok(())
    }
}

fn normalize_principal(value: &str) -> Result<String, DownloadJournalRecordError> {
    super::principal::canonical_text(value).ok_or(DownloadJournalRecordError::InvalidPrincipal)
}

/// Typed local journal identity, schema or transition rejection.
#[derive(Debug, Error)]
pub enum DownloadJournalRecordError {
    /// Only v1 records are maintained.
    #[error("unsupported download journal version {0}")]
    UnsupportedVersion(u16),
    /// The supplied principal failed canonical admission.
    #[error("invalid download canister principal")]
    InvalidPrincipal,
    /// The exact snapshot token is empty, excessive or contains nongraphic bytes.
    #[error("invalid download snapshot token")]
    InvalidSnapshotId,
    /// A persisted path differs from its immutable identity-derived location.
    #[error("download artifact path does not match source identity")]
    ArtifactPathMismatch,
    /// The requested physical target is not retained.
    #[error("unknown download artifact")]
    UnknownArtifact,
    /// The supplied snapshot differs from the retained exact token.
    #[error("download snapshot identity mismatch")]
    SnapshotMismatch,
    /// Progress cannot be reset, repeated or advanced over an intermediate state.
    #[error("invalid download state transition from {from:?} to {to:?}")]
    InvalidStateTransition {
        /// Retained state.
        from: ArtifactStateRecord,
        /// Rejected next state.
        to: ArtifactStateRecord,
    },
    /// Checksum evidence disagrees with lifecycle state.
    #[error("invalid checksum evidence for download state {0:?}")]
    InvalidChecksumState(ArtifactStateRecord),
    /// The immutable intent or checksum is malformed.
    #[error(transparent)]
    Checksum(#[from] ChecksumError),
}

#[cfg(test)]
mod tests;
