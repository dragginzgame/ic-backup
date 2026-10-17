//! Bind each admitted extent while hashing the same bytes under original source custody.

use super::super::super::{
    ArtifactChecksumRecord, File, IcSnapshotArtifactError, REGIONS, member,
    verification::{checksum_child_with, checksum_exact_reader, checksum_ic_tree_with},
};
use super::super::IcSnapshotUploadArtifactError;
use crate::model::{
    ic_snapshot_data::MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES, ic_snapshot_upload::IcSnapshotUploadRequest,
};
use ic_host_artifacts::artifact::HashingWriter;
use ic_management_canister_types::SnapshotDataKind;
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
    let mut remaining = kinds.iter().peekable();
    let mut bindings = Vec::with_capacity(kinds.len());
    let checksum = checksum_ic_tree_with(
        directory,
        metadata.source(),
        |directory, name, length, maximum| {
            checksum_child_with(directory, name, length, maximum, |file, length| {
                member_checksum(
                    file,
                    name,
                    length,
                    metadata,
                    destination,
                    &mut remaining,
                    &mut bindings,
                )
            })
        },
    )?;
    if remaining.next().is_some() {
        return Err(IcSnapshotArtifactError::IncompleteCoverage.into());
    }
    checksum
        .verify(metadata.source_checksum().hash())
        .map_err(IcSnapshotArtifactError::from)?;
    Ok(bindings)
}

fn member_checksum(
    file: &mut impl Read,
    name: &str,
    length: u64,
    metadata: &IcSnapshotUploadRequest<'_>,
    destination: &[u8],
    remaining: &mut Peekable<Iter<'_, SnapshotDataKind>>,
    bindings: &mut Vec<ArtifactChecksumRecord>,
) -> Result<ArtifactChecksumRecord, IcSnapshotUploadArtifactError> {
    // Fixed metadata members have no payloads; zero-length regions still hash exactly.
    if !REGIONS.contains(&name) && !name.starts_with("chunk-") {
        return Ok(checksum_exact_reader(file, length)?);
    }
    let mut digest = HashingWriter::new(io::sink(), length);
    while let Some(kind) = remaining.peek() {
        let (member_name, offset, size) = match member::select(kind)? {
            member::Member::Region {
                index,
                offset,
                size,
            } => (REGIONS[index].to_owned(), offset, size),
            member::Member::Chunk { name } => (name, 0, length),
        };
        if member_name != name {
            break;
        }
        if offset != digest.bytes_written() || size > length - digest.bytes_written() {
            return Err(IcSnapshotArtifactError::IncompleteCoverage.into());
        }
        let size = usize::try_from(size)
            .ok()
            .filter(|size| *size <= MAX_IC_SNAPSHOT_DATA_CHUNK_BYTES)
            .ok_or(IcSnapshotArtifactError::FileShape)?;
        let bytes =
            ic_host_artifacts::artifact::read_exact_reader((&mut *file).take(size as u64), size)
                .map_err(|error| match error {
                    ic_host_artifacts::artifact::ArtifactError::LimitExceeded { .. }
                    | ic_host_artifacts::artifact::ArtifactError::LengthMismatch { .. } => {
                        IcSnapshotArtifactError::FileShape
                    }
                    error => IcSnapshotArtifactError::Io(error.into()),
                })?;
        digest
            .write_all(&bytes)
            .map_err(IcSnapshotArtifactError::from)?;
        let payload =
            IcSnapshotUploadRequest::data(metadata, destination, (*kind).clone(), &bytes)?;
        bindings.push(payload.binding_digest());
        remaining.next();
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

#[cfg(test)]
mod tests;
