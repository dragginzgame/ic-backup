//! Exact local artifact identity and model-owned download lifecycle transitions.

mod view;
pub use view::{DownloadArtifactView, DownloadJournalView, ResumeAction};

use super::artifacts::{ArtifactChecksumRecord, ChecksumError, canonical_hash};
use serde::{Deserialize, Deserializer, Serialize, de};
use std::{collections::BTreeSet, fmt};
use thiserror::Error;

/// Maximum artifact entries in a local download journal.
pub const MAX_DOWNLOAD_ARTIFACTS: usize = 1024;
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

/// Ordered local artifact lifecycle adapted from Canic's download journal.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ArtifactStateRecord {
    /// Exact snapshot identity retained before local download completion.
    Created,
    /// Caller attested a complete staged backend artifact.
    Downloaded,
    /// Local bytes verified and their canonical checksum retained.
    ChecksumVerified,
    /// Verified bytes durably published at the canonical location.
    Durable,
}

impl ArtifactStateRecord {
    const fn can_advance_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Created, Self::Downloaded)
                | (Self::Downloaded, Self::ChecksumVerified)
                | (Self::ChecksumVerified, Self::Durable)
        )
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
    state: ArtifactStateRecord,
    checksum: Option<ArtifactChecksumRecord>,
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
    #[serde(deserialize_with = "required_checksum")]
    checksum: Option<ArtifactChecksumRecord>,
}

fn required_checksum<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<ArtifactChecksumRecord>, D::Error> {
    Option::deserialize(deserializer)
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
        let requires_checksum = matches!(
            fields.state,
            ArtifactStateRecord::ChecksumVerified | ArtifactStateRecord::Durable
        );
        if requires_checksum != fields.checksum.is_some() {
            return Err(DownloadJournalRecordError::InvalidChecksumState(
                fields.state,
            ));
        }
        record.state = fields.state;
        record.checksum = fields.checksum;
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
            state: ArtifactStateRecord::Created,
            checksum: None,
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
        self.state
    }
    /// Return the retained exact checksum when verification has completed.
    #[must_use]
    pub const fn checksum(&self) -> Option<&ArtifactChecksumRecord> {
        self.checksum.as_ref()
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
    artifacts: Vec<DownloadArtifactRecord>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct JournalFields {
    version: u16,
    intent: String,
    #[serde(deserialize_with = "read_bounded_artifacts")]
    artifacts: Vec<DownloadArtifactRecord>,
}

impl TryFrom<JournalFields> for DownloadJournalRecord {
    type Error = DownloadJournalRecordError;
    fn try_from(mut fields: JournalFields) -> Result<Self, Self::Error> {
        if fields.version != 1 {
            return Err(DownloadJournalRecordError::UnsupportedVersion(
                fields.version,
            ));
        }
        validate_entries(&fields.artifacts)?;
        fields
            .artifacts
            .sort_by(|left, right| left.canister_id.cmp(&right.canister_id));
        Ok(Self {
            version: 1,
            intent: canonical_hash(&fields.intent)?,
            artifacts: fields.artifacts,
        })
    }
}

impl DownloadJournalRecord {
    /// Admit a nonempty exact physical set before any local transfer completion.
    ///
    /// # Errors
    /// Rejects invalid identities/digests, duplicate principals and exceeded count bounds.
    pub fn new(
        intent: &str,
        requests: Vec<DownloadArtifactRequest>,
    ) -> Result<Self, DownloadJournalRecordError> {
        if requests.len() > MAX_DOWNLOAD_ARTIFACTS {
            return Err(DownloadJournalRecordError::TooManyArtifacts);
        }
        let artifacts = requests
            .into_iter()
            .map(DownloadArtifactRecord::new)
            .collect::<Result<Vec<_>, _>>()?;
        JournalFields {
            version: 1,
            intent: intent.to_owned(),
            artifacts,
        }
        .try_into()
    }

    /// Return the canonical caller-supplied immutable intent digest.
    #[must_use]
    pub fn intent(&self) -> &str {
        &self.intent
    }
    /// Read entries in canonical principal-text order.
    #[must_use]
    pub fn artifacts(&self) -> &[DownloadArtifactRecord] {
        &self.artifacts
    }

    /// Hash canonical original intent, exact snapshot metadata, paths and local evidence.
    ///
    /// The NUL-terminated v1 domain precedes 64 ASCII intent bytes and a big-endian
    /// u64 entry count. Each canonical entry has four u32-length-prefixed UTF-8
    /// strings (principal, snapshot token, staging path, artifact path), two u64
    /// metadata values, a state byte (Created=0 through Durable=3), then a checksum
    /// presence byte and its 64 ASCII hash when present. Integers are big-endian.
    /// This grants no transfer, snapshot authenticity or terminal proof.
    #[must_use]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "record admission bounds principals, snapshot tokens and identity-derived paths below u32"
    )]
    pub fn digest(&self) -> ArtifactChecksumRecord {
        let mut bytes = b"ic-backup/download-journal/v1\0".to_vec();
        bytes.extend_from_slice(self.intent.as_bytes());
        bytes.extend_from_slice(&(self.artifacts.len() as u64).to_be_bytes());
        for entry in &self.artifacts {
            for value in [
                entry.canister_id(),
                entry.snapshot_id(),
                entry.staging_path(),
                entry.artifact_path(),
            ] {
                bytes.extend_from_slice(&(value.len() as u32).to_be_bytes());
                bytes.extend_from_slice(value.as_bytes());
            }
            bytes.extend_from_slice(&entry.snapshot_taken_at_timestamp.to_be_bytes());
            bytes.extend_from_slice(&entry.snapshot_total_size_bytes.to_be_bytes());
            bytes.push(match entry.state {
                ArtifactStateRecord::Created => 0,
                ArtifactStateRecord::Downloaded => 1,
                ArtifactStateRecord::ChecksumVerified => 2,
                ArtifactStateRecord::Durable => 3,
            });
            bytes.push(u8::from(entry.checksum.is_some()));
            if let Some(checksum) = &entry.checksum {
                bytes.extend_from_slice(checksum.hash().as_bytes());
            }
        }
        ArtifactChecksumRecord::from_bytes(&bytes)
    }

    pub(crate) fn artifact(
        &self,
        canister: &str,
        snapshot: &str,
    ) -> Result<&DownloadArtifactRecord, DownloadJournalRecordError> {
        let canister = normalize_principal(canister)?;
        let entry = self
            .artifacts
            .iter()
            .find(|entry| entry.canister_id == canister)
            .ok_or(DownloadJournalRecordError::UnknownArtifact)?;
        if entry.snapshot_id != snapshot {
            return Err(DownloadJournalRecordError::SnapshotMismatch);
        }
        Ok(entry)
    }

    pub(crate) fn advance(
        &mut self,
        canister: &str,
        snapshot: &str,
        next: ArtifactStateRecord,
        checksum: Option<ArtifactChecksumRecord>,
    ) -> Result<(), DownloadJournalRecordError> {
        let entry = self.artifact(canister, snapshot)?;
        let from = entry.state;
        if !from.can_advance_to(next) {
            return Err(DownloadJournalRecordError::InvalidStateTransition { from, to: next });
        }
        let checksum = match next {
            ArtifactStateRecord::Downloaded if checksum.is_none() => None,
            ArtifactStateRecord::ChecksumVerified if checksum.is_some() => checksum,
            ArtifactStateRecord::Durable if checksum.is_none() => entry.checksum.clone(),
            _ => return Err(DownloadJournalRecordError::InvalidChecksumState(next)),
        };
        let canister = entry.canister_id.clone();
        let entry = self
            .artifacts
            .iter_mut()
            .find(|entry| entry.canister_id == canister)
            .ok_or(DownloadJournalRecordError::UnknownArtifact)?;
        entry.state = next;
        entry.checksum = checksum;
        Ok(())
    }
}

fn normalize_principal(value: &str) -> Result<String, DownloadJournalRecordError> {
    super::principal::canonical_text(value).ok_or(DownloadJournalRecordError::InvalidPrincipal)
}

fn validate_entries(entries: &[DownloadArtifactRecord]) -> Result<(), DownloadJournalRecordError> {
    if entries.is_empty() {
        return Err(DownloadJournalRecordError::EmptyArtifacts);
    }
    if entries.len() > MAX_DOWNLOAD_ARTIFACTS {
        return Err(DownloadJournalRecordError::TooManyArtifacts);
    }
    let mut identities = BTreeSet::new();
    for entry in entries {
        if !identities.insert(entry.canister_id()) {
            return Err(DownloadJournalRecordError::DuplicateCanister);
        }
    }
    Ok(())
}

fn read_bounded_artifacts<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<DownloadArtifactRecord>, D::Error> {
    struct ArtifactsVisitor;
    impl<'de> de::Visitor<'de> for ArtifactsVisitor {
        type Value = Vec<DownloadArtifactRecord>;
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a bounded artifact list")
        }
        fn visit_seq<A: de::SeqAccess<'de>>(
            self,
            mut sequence: A,
        ) -> Result<Self::Value, A::Error> {
            let mut entries = Vec::new();
            while entries.len() < MAX_DOWNLOAD_ARTIFACTS {
                match sequence.next_element()? {
                    Some(entry) => entries.push(entry),
                    None => return Ok(entries),
                }
            }
            if sequence.next_element::<de::IgnoredAny>()?.is_some() {
                return Err(de::Error::custom(
                    DownloadJournalRecordError::TooManyArtifacts,
                ));
            }
            Ok(entries)
        }
    }
    deserializer.deserialize_seq(ArtifactsVisitor)
}

/// Typed local journal identity, schema or transition rejection.
#[derive(Debug, Error)]
pub enum DownloadJournalRecordError {
    /// Only v1 records are maintained.
    #[error("unsupported download journal version {0}")]
    UnsupportedVersion(u16),
    /// An empty physical set cannot claim completion.
    #[error("download journal artifacts must not be empty")]
    EmptyArtifacts,
    /// The bounded artifact count was exceeded.
    #[error("download journal artifact count exceeds {MAX_DOWNLOAD_ARTIFACTS}")]
    TooManyArtifacts,
    /// More than one snapshot was selected for the same physical canister.
    #[error("duplicate download canister identity")]
    DuplicateCanister,
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
