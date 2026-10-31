//! Exact retained fixture bytes; these helpers grant no receipt or effect authority.

use ic_backup::{
    model::artifacts::ArtifactChecksumRecord,
    model::ic_snapshot_metadata::{
        IcSnapshotMetadataError, IcSnapshotMetadataReply, IcSnapshotMetadataRequest,
    },
    ops::persistence::BackupLayoutGuard,
};
use serde::{Deserialize, Serialize};
use std::{fs, io::Write};

pub(super) struct MetadataEvidence {
    pub request: IcSnapshotMetadataRequest,
    pub raw: Vec<u8>,
}
impl MetadataEvidence {
    pub fn decode(&self) -> Result<IcSnapshotMetadataReply<'_>, IcSnapshotMetadataError> {
        IcSnapshotMetadataReply::decode(&self.request, &self.raw)
    }
}

pub(super) fn retain_bytes(layout: &BackupLayoutGuard, name: &str, bytes: &[u8]) {
    let file = fs::File::create(layout.root().join(name)).unwrap();
    (&file).write_all(bytes).unwrap();
    file.sync_all().unwrap();
    fs::File::open(layout.root()).unwrap().sync_all().unwrap();
}

// Preserve exact JSON checksum spelling at the fixture acceptance boundary;
// domain checksum admission would normalize altered hash text before comparison.
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ExactChecksumRecord {
    algorithm: String,
    hash: String,
}
impl From<&ArtifactChecksumRecord> for ExactChecksumRecord {
    fn from(checksum: &ArtifactChecksumRecord) -> Self {
        Self {
            algorithm: "sha256".into(),
            hash: checksum.hash().into(),
        }
    }
}
impl ExactChecksumRecord {
    fn matches(&self, checksum: &ArtifactChecksumRecord) -> bool {
        self.algorithm == "sha256" && self.hash == checksum.hash()
    }
}

// Diagnostic verification sequence/ID and extra top-level fields remain outside
// terminal acceptance. Missing or altered acceptance fields still refuse.
#[derive(Deserialize, Serialize)]
pub(super) struct PostLoadAcceptanceRecord {
    complete_state_evidence: ExactChecksumRecord,
    loaded_destination: Vec<u8>,
    original_source_metadata: ExactChecksumRecord,
}
impl PostLoadAcceptanceRecord {
    pub fn matches(
        &self,
        metadata: &ArtifactChecksumRecord,
        destination: &[u8],
        accepted: &ArtifactChecksumRecord,
    ) -> bool {
        self.original_source_metadata.matches(metadata)
            && self.loaded_destination == destination
            && self.complete_state_evidence.matches(accepted)
    }
}

#[derive(Serialize)]
pub(super) struct PostLoadVerificationRecord {
    // Alphabetical JSON field order preserves the preceding Value writer's bytes.
    #[serde(flatten)]
    acceptance: PostLoadAcceptanceRecord,
    verification_operation_sequence: u64,
    verification_snapshot_id: Vec<u8>,
}
impl PostLoadVerificationRecord {
    pub fn new(
        metadata: &ArtifactChecksumRecord,
        destination: &[u8],
        sequence: u64,
        snapshot_id: &[u8],
        evidence: &ArtifactChecksumRecord,
    ) -> Self {
        Self {
            acceptance: PostLoadAcceptanceRecord {
                complete_state_evidence: evidence.into(),
                loaded_destination: destination.into(),
                original_source_metadata: metadata.into(),
            },
            verification_operation_sequence: sequence,
            verification_snapshot_id: snapshot_id.into(),
        }
    }
}

#[cfg(test)]
mod tests;
