//! Stream artifact bytes through descriptor-based no-follow traversal.

#[cfg(test)]
mod regressions;
#[cfg(test)]
mod tests;

use crate::model::artifacts::ArtifactChecksumRecord;
use ic_host_artifacts::artifact::tree::DirectoryChecksumError;
#[cfg(any(unix, test))]
use std::io::Read;
use std::{
    io,
    path::{Path, PathBuf},
};
use thiserror::Error;

pub(crate) fn checksum_directory(path: &Path) -> Result<ArtifactChecksumRecord, ArtifactError> {
    #[cfg(unix)]
    {
        use ic_host_fs::read::tree::checksum::{TreeChecksumError, checksum_directory};
        let digest = checksum_directory(path, u64::MAX).map_err(|error| match error {
            TreeChecksumError::Io(error) => ArtifactError::Io(error),
            TreeChecksumError::Input(error) => ArtifactError::Io(error.into()),
            TreeChecksumError::Declaration(error) => ArtifactError::from(error),
            TreeChecksumError::NonUtf8Path { path } => ArtifactError::NonUtf8Path { path },
            TreeChecksumError::InvalidPath { path } => ArtifactError::UnsupportedEntry {
                path: path.display().to_string(),
                kind: "Unknown".into(),
            },
            TreeChecksumError::UnsupportedEntry { path, kind } => ArtifactError::UnsupportedEntry {
                path: path.display().to_string(),
                kind: format!("{kind:?}"),
            },
        })?;
        Ok(ArtifactChecksumRecord::from_digest(*digest.as_bytes()))
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Err(ArtifactError::UnsupportedPlatform(std::env::consts::OS))
    }
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

pub(crate) fn checksum_relative_files(
    files: Vec<(PathBuf, ArtifactChecksumRecord)>,
) -> Result<ArtifactChecksumRecord, DirectoryChecksumError> {
    let digest = ic_host_artifacts::artifact::tree::checksum_relative_files(
        files
            .into_iter()
            .map(|(path, record)| (path, record.sha256()))
            .collect(),
    )?;
    Ok(ArtifactChecksumRecord::from_digest(*digest.as_bytes()))
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
