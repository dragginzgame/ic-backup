//! Bind each admitted extent while hashing the same bytes under original source custody.

use super::super::super::{
    ArtifactChecksumRecord, File, IcSnapshotArtifactError, member,
    verification::{checksum_child_with, checksum_exact_reader, checksum_ic_tree_with},
};
use super::super::{IcSnapshotUploadArtifactError, read_chunk_bytes};
use crate::model::{ic_snapshot_data::DataExtent, ic_snapshot_upload::IcSnapshotUploadRequest};
use ic_host_artifacts::artifact::HashingWriter;
use ic_platform::management_canister_types::SnapshotDataKind;
use std::{
    io::{self, Read, Write},
    iter::Peekable,
    slice::Iter,
};

pub(super) fn bindings(
    directory: &File,
    metadata: &IcSnapshotUploadRequest<'_>,
    destination: &[u8],
    kinds: &[SnapshotDataKind],
) -> Result<Vec<ArtifactChecksumRecord>, IcSnapshotUploadArtifactError> {
    let mut scan = BindingScan {
        metadata,
        destination,
        remaining: kinds.iter().peekable(),
        bindings: Vec::with_capacity(kinds.len()),
    };
    let checksum = checksum_ic_tree_with(directory, metadata.source(), |directory, member| {
        checksum_child_with(directory, member, |file, length| {
            scan.member_checksum(file, member, length)
        })
    })?;
    if scan.remaining.next().is_some() {
        return Err(IcSnapshotArtifactError::IncompleteCoverage.into());
    }
    checksum
        .verify(metadata.source_checksum().hash())
        .map_err(IcSnapshotArtifactError::from)?;
    Ok(scan.bindings)
}

// One ephemeral scan owns the original extent cursor and accumulated bindings.
// Filesystem admission and exact member hashing remain with the existing owners.
struct BindingScan<'a, 'source> {
    metadata: &'a IcSnapshotUploadRequest<'source>,
    destination: &'a [u8],
    remaining: Peekable<Iter<'a, SnapshotDataKind>>,
    bindings: Vec<ArtifactChecksumRecord>,
}

impl BindingScan<'_, '_> {
    fn member_checksum(
        &mut self,
        file: &mut impl Read,
        member: &member::ExpectedMember,
        length: u64,
    ) -> Result<ArtifactChecksumRecord, IcSnapshotUploadArtifactError> {
        // Fixed metadata members have no payloads; zero-length regions still hash exactly.
        let Some(payload) = member.payload() else {
            return Ok(checksum_exact_reader(file, length)?);
        };
        let mut digest = HashingWriter::new(io::sink(), length);
        while let Some(kind) = self.remaining.peek() {
            let (offset, size) = match (payload, DataExtent::from(*kind)) {
                (
                    member::PayloadMember::Region(expected),
                    DataExtent::Region {
                        region,
                        offset,
                        size,
                    },
                ) if expected == region => (offset, size),
                (member::PayloadMember::Chunk(expected), DataExtent::Chunk { hash })
                    if expected.as_bytes() == hash =>
                {
                    (0, length)
                }
                _ => break,
            };
            if offset != digest.bytes_written() || size > length - digest.bytes_written() {
                return Err(IcSnapshotArtifactError::IncompleteCoverage.into());
            }
            let bytes = read_chunk_bytes(&mut *file, size)?;
            digest
                .write_all(&bytes)
                .map_err(IcSnapshotArtifactError::from)?;
            let payload = IcSnapshotUploadRequest::data(
                self.metadata,
                self.destination,
                (*kind).clone(),
                &bytes,
            )?;
            self.bindings.push(payload.binding_digest());
            self.remaining.next();
        }
        if digest.bytes_written() != length
            || file.read(&mut [0]).map_err(IcSnapshotArtifactError::from)? != 0
        {
            return Err(IcSnapshotArtifactError::FileShape.into());
        }
        let (_, identity) = digest.into_parts();
        Ok(ArtifactChecksumRecord::from_digest(
            *identity.sha256.as_bytes(),
        ))
    }
}

#[cfg(test)]
mod tests;
