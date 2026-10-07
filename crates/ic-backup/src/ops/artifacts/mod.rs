//! Stream artifact bytes through descriptor-based no-follow traversal.

#[cfg(test)]
mod regressions;
mod secure;
#[cfg(test)]
mod tests;

use crate::model::artifacts::ArtifactChecksumRecord;
use sha2::{Digest, Sha256};
#[cfg(unix)]
use std::io::Write;
use std::{
    io::{self, Read},
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
/// Interrupted reads retry internally. The caller owns blocking and timeouts;
/// no network or paid-operation retry is performed.
///
/// # Errors
/// Returns other reader IO failures unchanged; impossible byte counts reject as
/// [`io::ErrorKind::InvalidData`] rather than indexing outside the transfer buffer.
pub fn checksum_reader(reader: &mut impl Read) -> Result<ArtifactChecksumRecord, ArtifactError> {
    use ic_host_artifacts::artifact::ArtifactError as InputError;

    let identity =
        ic_host_artifacts::artifact::hash_reader(reader, u64::MAX).map_err(
            |error| match error {
                InputError::Io(error) => ArtifactError::Io(error),
                error => ArtifactError::Io(io::Error::other(error)),
            },
        )?;
    Ok(ArtifactChecksumRecord::from_digest(
        *identity.sha256.as_bytes(),
    ))
}

#[cfg(unix)]
pub(crate) fn copy_from_reader(
    reader: &mut impl Read,
    writer: &mut impl Write,
) -> Result<ArtifactChecksumRecord, ArtifactError> {
    use ic_host_artifacts::artifact::{ArtifactError as InputError, CopyError};

    // Artifacts have no total-size ceiling here. Descriptor admission, private
    // staging, retained checksum comparison and publication remain local.
    let identity =
        ic_host_artifacts::artifact::copy_reader(reader, writer, u64::MAX).map_err(|error| {
            match error {
                CopyError::Input(InputError::Io(error)) | CopyError::Output(error) => {
                    ArtifactError::Io(error)
                }
                CopyError::Input(error) => ArtifactError::Io(io::Error::other(error)),
            }
        })?;
    Ok(ArtifactChecksumRecord::from_digest(
        *identity.sha256.as_bytes(),
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

#[cfg(unix)]
fn require_utf8_tree_name(
    name: &std::ffi::OsStr,
    display_root: &Path,
) -> Result<(), ArtifactError> {
    if name.to_str().is_none() {
        return Err(ArtifactError::NonUtf8Path {
            path: display_root.join(name),
        });
    }
    Ok(())
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
