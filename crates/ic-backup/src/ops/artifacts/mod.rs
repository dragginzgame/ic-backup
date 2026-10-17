//! Stream artifact bytes through descriptor-based no-follow traversal.

#[cfg(test)]
mod regressions;
mod secure;
#[cfg(test)]
mod tests;

use crate::model::artifacts::ArtifactChecksumRecord;
use crate::model::evidence_digest::EvidenceDigest;
#[cfg(any(unix, test))]
use std::io::Read;
use std::{
    io,
    path::{Component, Path, PathBuf},
};
use thiserror::Error;

/// Checksum a directory using sorted relative-path/file-digest pairs.
///
/// # Errors
/// Rejects unsafe entry types, unsupported platforms and filesystem failures.
pub fn checksum_directory(path: &Path) -> Result<ArtifactChecksumRecord, ArtifactError> {
    secure::checksum_directory(path)
}

#[cfg(any(unix, test))]
/// Stream an already-open reader using a bounded transfer buffer.
///
/// Interrupted reads retry internally. The caller owns blocking and timeouts;
/// no network or paid-operation retry is performed.
///
/// # Errors
/// Returns other reader IO failures unchanged; impossible byte counts reject as
/// [`io::ErrorKind::InvalidData`] rather than indexing outside the transfer buffer.
pub(crate) fn checksum_reader(
    reader: &mut impl Read,
) -> Result<ArtifactChecksumRecord, ArtifactError> {
    let identity =
        ic_host_artifacts::artifact::hash_reader(reader, u64::MAX).map_err(io::Error::from)?;
    Ok(ArtifactChecksumRecord::from_digest(
        *identity.sha256.as_bytes(),
    ))
}

/// Compose the maintained directory checksum from already-admitted file checksums.
///
/// Names must be exact UTF-8 relative paths with nonempty normal components,
/// separated by `/`, without NUL bytes, normalization aliases or duplicates.
/// Unix filename bytes such as newlines and backslashes retain their exact meaning.
/// Entries sort by [`PathBuf`] ordering, then hash UTF-8 path bytes, NUL,
/// the canonical lowercase file digest and newline. An empty set hashes empty bytes.
///
/// Performs no filesystem IO. The caller owns descriptor/byte custody, completeness,
/// synchronization and publication; declared checksums prove none of these.
///
/// The checksums below stand for bytes already admitted by the caller. A
/// descriptor-owning consumer may supply its retained records instead.
///
/// ```
/// use ic_backup::{
///     model::artifacts::ArtifactChecksumRecord,
///     ops::artifacts::{DirectoryChecksumError, checksum_relative_files},
/// };
///
/// let checksum = checksum_relative_files(vec![
///     ("a.txt".into(), ArtifactChecksumRecord::from_bytes(b"a")),
///     ("nested/b.txt".into(), ArtifactChecksumRecord::from_bytes(b"b")),
/// ])?;
/// assert_eq!(
///     checksum.hash(),
///     "e4d330f138b8f1b3044e84b5dcbe4fd1cb7e043d0c20810c083d791b6de01266",
/// );
/// # Ok::<(), DirectoryChecksumError>(())
/// ```
///
/// # Errors
/// Returns a typed refusal for non-UTF-8, malformed or duplicate identities.
pub fn checksum_relative_files(
    mut files: Vec<(PathBuf, ArtifactChecksumRecord)>,
) -> Result<ArtifactChecksumRecord, DirectoryChecksumError> {
    files.sort_by(|left, right| left.0.cmp(&right.0));
    let mut hasher = EvidenceDigest::new(&[]);
    let mut previous = None;
    for (relative, checksum) in &files {
        let name = relative
            .to_str()
            .ok_or_else(|| DirectoryChecksumError::NonUtf8Path {
                path: relative.clone(),
            })?;
        if name.contains('\0')
            || name.split('/').any(|part| matches!(part, "" | "." | ".."))
            || !relative
                .components()
                .all(|part| matches!(part, Component::Normal(_)))
        {
            return Err(DirectoryChecksumError::InvalidRelativePath {
                path: relative.clone(),
            });
        }
        if previous == Some(relative) {
            return Err(DirectoryChecksumError::DuplicatePath {
                path: relative.clone(),
            });
        }
        previous = Some(relative);
        hasher.append(name.as_bytes());
        hasher.byte(0);
        hasher.append(checksum.hash().as_bytes());
        hasher.append(b"\n");
    }
    Ok(hasher.finish())
}

/// Invalid declared file identities at the I/O-free directory checksum boundary.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum DirectoryChecksumError {
    /// The supplied path cannot retain exact UTF-8 identity.
    #[error("directory checksum path is not UTF-8: {path:?}")]
    NonUtf8Path {
        /// Exact rejected path.
        path: PathBuf,
    },
    /// The path contains invalid bytes or noncanonical/unnormalized components.
    #[error("directory checksum path is not canonical and relative: {path:?}")]
    InvalidRelativePath {
        /// Exact rejected path, without normalization.
        path: PathBuf,
    },
    /// Two entries declare the same canonical path, even if their digests agree.
    #[error("duplicate directory checksum path: {path:?}")]
    DuplicatePath {
        /// Exact repeated path.
        path: PathBuf,
    },
}

impl From<DirectoryChecksumError> for ArtifactError {
    fn from(error: DirectoryChecksumError) -> Self {
        match error {
            DirectoryChecksumError::NonUtf8Path { path } => Self::NonUtf8Path { path },
            error => Self::Io(io::Error::new(io::ErrorKind::InvalidData, error)),
        }
    }
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
