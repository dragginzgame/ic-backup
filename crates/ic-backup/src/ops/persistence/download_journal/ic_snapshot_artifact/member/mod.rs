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
    pub length: Option<u64>,
    pub maximum: u64,
    pub checksum: Option<ArtifactChecksumRecord>,
    pub payload: Option<PayloadMember>,
}

pub(super) fn expected(
    metadata: &IcSnapshotMetadataReply<'_>,
) -> Result<Vec<ExpectedMember>, IcSnapshotArtifactError> {
    let argument_length = metadata.request().arguments().len() as u64;
    let mut members = [
        (
            "format",
            Some(FORMAT.len() as u64),
            FORMAT.len() as u64,
            ArtifactChecksumRecord::from_bytes(FORMAT),
        ),
        (
            "metadata.candid",
            None,
            MAX_IC_SNAPSHOT_METADATA_BYTES as u64,
            metadata.payload_checksum().clone(),
        ),
        (
            "metadata-arguments.candid",
            Some(argument_length),
            argument_length,
            ArtifactChecksumRecord::from_bytes(metadata.request().arguments()),
        ),
    ]
    .into_iter()
    .map(|(name, length, maximum, checksum)| ExpectedMember {
        name: name.into(),
        length,
        maximum,
        checksum: Some(checksum),
        payload: None,
    })
    .collect::<Vec<_>>();
    let values = metadata.metadata();
    members.extend(
        SnapshotRegion::ALL
            .into_iter()
            .zip(metadata.region_sizes())
            .map(|(region, length)| ExpectedMember {
                name: REGIONS[region.index()].into(),
                length: Some(length),
                maximum: length,
                checksum: None,
                payload: Some(PayloadMember::Region(region)),
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
            length: None,
            maximum: MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES as u64,
            checksum: Some(ArtifactChecksumRecord::from_digest(*digest.as_bytes())),
            payload: Some(PayloadMember::Chunk(digest)),
        });
    }
    Ok(members)
}
