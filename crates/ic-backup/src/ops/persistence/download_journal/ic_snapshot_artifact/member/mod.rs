//! Exact v1 snapshot member names and data-kind mapping.

use super::{
    ArtifactChecksumRecord, FORMAT, IcSnapshotArtifactError, IcSnapshotMetadataReply,
    MAX_IC_SNAPSHOT_METADATA_BYTES, REGIONS,
};
use crate::model::ic_snapshot_data::{
    DataExtent, MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES, SnapshotRegion,
};
use ic_host_artifacts::artifact::Sha256Digest;
use ic_platform::management_canister_types::SnapshotDataKind;

pub(super) enum Member {
    Region {
        region: SnapshotRegion,
        offset: u64,
        size: u64,
    },
    Chunk {
        name: String,
    },
}

pub(super) fn select(kind: &SnapshotDataKind) -> Result<Member, IcSnapshotArtifactError> {
    Ok(match DataExtent::from(kind) {
        DataExtent::Region {
            region,
            offset,
            size,
        } => Member::Region {
            region,
            offset,
            size,
        },
        DataExtent::Chunk { hash } => Member::Chunk {
            name: chunk_name(Sha256Digest::from_bytes(
                hash.try_into()
                    .map_err(|_| IcSnapshotArtifactError::OriginalMismatch)?,
            )),
        },
    })
}

pub(super) fn chunk_name(hash: Sha256Digest) -> String {
    format!("chunk-{hash}.bin")
}

/// One exact tree member; declaration digests grant no byte custody.
#[derive(Clone, Copy)]
pub(super) enum PayloadMember {
    Region(SnapshotRegion),
    Chunk(Sha256Digest),
}

pub(super) struct ExpectedMember {
    pub name: String,
    kind: MemberKind,
}

enum MemberKind {
    Fixed {
        length: u64,
        checksum: ArtifactChecksumRecord,
    },
    Metadata {
        checksum: ArtifactChecksumRecord,
    },
    Region {
        region: SnapshotRegion,
        length: u64,
    },
    Chunk {
        hash: Sha256Digest,
        checksum: ArtifactChecksumRecord,
    },
}

impl ExpectedMember {
    // All expected-tree consumers open with the name and bounds owned by this declaration.
    pub(super) fn open(
        &self,
        directory: &super::File,
    ) -> Result<(super::File, std::fs::Metadata), IcSnapshotArtifactError> {
        super::verification::open_regular_child(
            directory,
            &self.name,
            self.length(),
            self.maximum(),
        )
    }
    fn length(&self) -> Option<u64> {
        match self.kind {
            MemberKind::Fixed { length, .. } | MemberKind::Region { length, .. } => Some(length),
            MemberKind::Metadata { .. } | MemberKind::Chunk { .. } => None,
        }
    }
    fn maximum(&self) -> u64 {
        match self.kind {
            MemberKind::Fixed { length, .. } | MemberKind::Region { length, .. } => length,
            MemberKind::Metadata { .. } => MAX_IC_SNAPSHOT_METADATA_BYTES as u64,
            MemberKind::Chunk { .. } => MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES as u64,
        }
    }
    pub(super) fn checksum(&self) -> Option<&ArtifactChecksumRecord> {
        match &self.kind {
            MemberKind::Fixed { checksum, .. }
            | MemberKind::Metadata { checksum }
            | MemberKind::Chunk { checksum, .. } => Some(checksum),
            MemberKind::Region { .. } => None,
        }
    }
    pub(super) fn payload(&self) -> Option<PayloadMember> {
        match self.kind {
            MemberKind::Region { region, .. } => Some(PayloadMember::Region(region)),
            MemberKind::Chunk { hash, .. } => Some(PayloadMember::Chunk(hash)),
            MemberKind::Fixed { .. } | MemberKind::Metadata { .. } => None,
        }
    }
}

pub(super) fn expected(
    metadata: &IcSnapshotMetadataReply<'_>,
) -> Result<Vec<ExpectedMember>, IcSnapshotArtifactError> {
    let mut members = vec![
        ExpectedMember {
            name: "format".into(),
            kind: MemberKind::Fixed {
                length: FORMAT.len() as u64,
                checksum: ArtifactChecksumRecord::from_bytes(FORMAT),
            },
        },
        ExpectedMember {
            name: "metadata.candid".into(),
            kind: MemberKind::Metadata {
                checksum: metadata.payload_checksum().clone(),
            },
        },
        ExpectedMember {
            name: "metadata-arguments.candid".into(),
            kind: MemberKind::Fixed {
                length: metadata.request().arguments().len() as u64,
                checksum: ArtifactChecksumRecord::from_bytes(metadata.request().arguments()),
            },
        },
    ];
    let values = metadata.metadata();
    members.extend(
        SnapshotRegion::ALL
            .into_iter()
            .zip(metadata.region_sizes())
            .map(|(region, length)| ExpectedMember {
                name: REGIONS[region.index()].into(),
                kind: MemberKind::Region { region, length },
            }),
    );
    for chunk in &values.wasm_chunk_store {
        let digest = Sha256Digest::from_bytes(
            chunk
                .hash
                .as_slice()
                .try_into()
                .map_err(|_| IcSnapshotArtifactError::OriginalMismatch)?,
        );
        members.push(ExpectedMember {
            name: chunk_name(digest),
            kind: MemberKind::Chunk {
                hash: digest,
                checksum: ArtifactChecksumRecord::from_digest(*digest.as_bytes()),
            },
        });
    }
    Ok(members)
}
