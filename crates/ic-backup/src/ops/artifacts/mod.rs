//! Stream artifact bytes through descriptor-based no-follow traversal.

#[cfg(test)]
mod regressions;
mod secure;
#[cfg(test)]
mod tests;

use crate::model::artifacts::ArtifactChecksumRecord;
use sha2::{Digest, Sha256};
use std::{
    io::{self, Read, Write},
    path::{Path, PathBuf},
};
use thiserror::Error;

/// Checksum one regular filesystem file without following path symlinks.
///
/// # Errors
/// Rejects unsafe entry types, unsupported platforms and filesystem failures.
pub fn checksum_file(path: &Path) -> Result<ArtifactChecksumRecord, ArtifactError> {
    secure::checksum_path(path, secure::ExpectedArtifactType::File)
}

/// Checksum one file or a deterministic directory listing.
///
/// # Errors
/// Rejects unsafe entry types, unsupported platforms and filesystem failures.
pub fn checksum_path(path: &Path) -> Result<ArtifactChecksumRecord, ArtifactError> {
    secure::checksum_path(path, secure::ExpectedArtifactType::Any)
}

/// Checksum a directory using sorted relative-path/file-digest pairs.
///
/// # Errors
/// Rejects unsafe entry types, unsupported platforms and filesystem failures.
pub fn checksum_directory(path: &Path) -> Result<ArtifactChecksumRecord, ArtifactError> {
    secure::checksum_path(path, secure::ExpectedArtifactType::Directory)
}

/// Stream an already-open reader using a bounded transfer buffer.
///
/// # Errors
/// Returns the reader's IO failure.
pub fn checksum_reader(reader: &mut impl Read) -> Result<ArtifactChecksumRecord, ArtifactError> {
    let mut hasher = Sha256::new();
    let mut buffer = vec![0; 64 * 1024];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(ArtifactChecksumRecord::from_digest(
        hasher.finalize().into(),
    ))
}

pub(crate) fn copy_from_reader(
    reader: &mut impl Read,
    writer: &mut impl Write,
) -> Result<ArtifactChecksumRecord, ArtifactError> {
    let mut hasher = Sha256::new();
    let mut buffer = vec![0; 64 * 1024];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        writer.write_all(&buffer[..read])?;
        hasher.update(&buffer[..read]);
    }
    Ok(ArtifactChecksumRecord::from_digest(
        hasher.finalize().into(),
    ))
}

pub(crate) fn checksum_relative_files(
    mut files: Vec<(PathBuf, ArtifactChecksumRecord)>,
) -> ArtifactChecksumRecord {
    files.sort_by(|left, right| left.0.cmp(&right.0));
    let mut hasher = Sha256::new();
    for (relative, checksum) in files {
        hasher.update(relative.to_string_lossy().as_bytes());
        hasher.update([0]);
        hasher.update(checksum.hash().as_bytes());
        hasher.update(*b"\n");
    }
    ArtifactChecksumRecord::from_digest(hasher.finalize().into())
}

/// Checksum a normal relative path beneath an operator-selected root.
///
/// # Errors
/// Rejects traversal, symlinks, special entries and IO failures.
pub fn checksum_relative_path(
    root: &Path,
    relative: &Path,
) -> Result<ArtifactChecksumRecord, ArtifactError> {
    secure::checksum_relative_path(root, relative)
}

/// Stage exact source bytes in a new private file or directory and checksum them.
///
/// The caller owns a trusted destination parent. This copy is not durable
/// publication; use the persistence operation after verifying its digest.
///
/// # Errors
/// Rejects source traversal/symlinks, existing destinations and IO failures.
pub fn stage_relative_path(
    root: &Path,
    relative: &Path,
    destination: &Path,
) -> Result<ArtifactChecksumRecord, ArtifactError> {
    secure::stage_relative_path(root, relative, destination)
}

/// Typed artifact traversal or IO failure.
#[derive(Debug, Error)]
pub enum ArtifactError {
    /// A path cannot be represented exactly in the maintained UTF-8 tree digest.
    #[error("artifact path is not UTF-8: {path:?}")]
    NonUtf8Path {
        /// Exact rejected filesystem path.
        path: PathBuf,
    },
    /// A filesystem operation or stream failed.
    #[error(transparent)]
    Io(#[from] io::Error),
    /// A tree entry is neither a regular file nor a directory.
    #[error("unsupported artifact entry at {path}: {kind}")]
    UnsupportedEntry {
        /// Entry path for diagnostics.
        path: String,
        /// Observed filesystem entry kind.
        kind: String,
    },
    /// Secure descriptor traversal is unavailable on this host.
    #[error("secure artifact traversal is unsupported on platform {0}")]
    UnsupportedPlatform(&'static str),
}
