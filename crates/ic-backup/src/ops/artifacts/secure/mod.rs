//! Directory-only artifact traversal; generic copying belongs to Host Tooling.

use super::{ArtifactChecksumRecord, ArtifactError};
use std::path::Path;

pub(super) fn checksum_directory(path: &Path) -> Result<ArtifactChecksumRecord, ArtifactError> {
    #[cfg(unix)]
    {
        unix::checksum_directory(path)
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Err(ArtifactError::UnsupportedPlatform(std::env::consts::OS))
    }
}

#[cfg(unix)]
mod unix;
