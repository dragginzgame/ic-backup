//! Module: `artifacts::secure`
//!
//! Responsibility: traverse and stage artifact trees without following symlinks.
//! Does not own: backup-root selection, manifest authority, or restore execution.
//! Boundary: returns checksums for the exact descriptor-read bytes.

use super::{ArtifactChecksumRecord, ArtifactError};

use std::path::Path;

#[derive(Clone, Copy)]
pub(super) enum ExpectedArtifactType {
    Any,
    Directory,
    File,
}

pub(super) fn checksum_path(
    path: &Path,
    expected: ExpectedArtifactType,
) -> Result<ArtifactChecksumRecord, ArtifactError> {
    #[cfg(unix)]
    {
        unix::checksum_path(path, expected)
    }

    #[cfg(not(unix))]
    {
        let _ = (path, expected);
        Err(ArtifactError::UnsupportedPlatform(std::env::consts::OS))
    }
}

pub(super) fn checksum_relative_path(
    root: &Path,
    relative: &Path,
) -> Result<ArtifactChecksumRecord, ArtifactError> {
    #[cfg(unix)]
    {
        unix::checksum_relative_path(root, relative)
    }

    #[cfg(not(unix))]
    {
        let _ = (root, relative);
        Err(ArtifactError::UnsupportedPlatform(std::env::consts::OS))
    }
}

pub(super) fn stage_relative_path(
    root: &Path,
    relative: &Path,
    destination: &Path,
) -> Result<ArtifactChecksumRecord, ArtifactError> {
    #[cfg(unix)]
    {
        unix::stage_relative_path(root, relative, destination)
    }

    #[cfg(not(unix))]
    {
        let _ = (root, relative, destination);
        Err(ArtifactError::UnsupportedPlatform(std::env::consts::OS))
    }
}

#[cfg(unix)]
mod unix;
