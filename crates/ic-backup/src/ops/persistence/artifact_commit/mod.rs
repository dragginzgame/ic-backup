//! Module: `persistence::artifact_commit`
//!
//! Responsibility: durably publish one verified snapshot artifact directory.
//! Does not own: snapshot download, journal transitions, or manifest creation.
//! Boundary: accepts journal-bound sibling paths and expected checksum bytes.

use crate::ops::persistence::PersistenceError;

use std::path::Path;

/// Result of publishing a temporary artifact or recovering its published tree.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArtifactCommitOutcome {
    /// The verified staging tree was published.
    Published,
    /// The verified canonical tree was adopted after a lost publication reply.
    Recovered,
}

/// Durably publish or recover one checksum-verified snapshot directory.
///
/// The caller owns the sibling parent and excludes other writers. A complete
/// canonical tree may remain after an IO failure; reconcile it on the next call.
///
/// # Errors
/// Rejects conflicting paths, changed bytes, unsafe entries and IO failures.
pub fn commit_artifact_directory(
    temporary: &Path,
    canonical: &Path,
    expected_checksum: &str,
) -> Result<ArtifactCommitOutcome, PersistenceError> {
    #[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
    {
        supported::commit_with_hook(temporary, canonical, expected_checksum, |_, _| Ok(()))
    }

    #[cfg(not(any(target_os = "linux", target_os = "android", target_vendor = "apple")))]
    {
        let _ = (temporary, canonical, expected_checksum);
        Err(PersistenceError::ArtifactCommitUnsupportedPlatform {
            platform: std::env::consts::OS,
        })
    }
}

#[cfg(any(target_os = "linux", target_os = "android", target_vendor = "apple"))]
mod supported;

#[cfg(all(
    test,
    any(target_os = "linux", target_os = "android", target_vendor = "apple")
))]
mod regressions;
#[cfg(all(
    test,
    any(target_os = "linux", target_os = "android", target_vendor = "apple")
))]
mod tests;
