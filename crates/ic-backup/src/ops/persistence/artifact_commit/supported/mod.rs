//! Descriptor-based synchronization and atomic no-replace publication.

use super::ArtifactCommitOutcome;
use crate::{
    model::artifacts::ArtifactChecksumRecord,
    ops::{artifacts::ArtifactError, persistence::PersistenceError},
};

use std::{fs::File, io, path::Path};

use ic_host_fs::{
    durable::tree::{TreeSyncStep, sync_checksum_directory_at},
    durable::{
        DirectoryCommitError, DirectoryCommitOutcome, DirectoryCommitStep,
        commit_existing_directory_with,
    },
    read::tree::checksum::TreeChecksumError,
};
use std::cell::RefCell;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ArtifactCommitStep {
    RegularFileSync,
    NestedDirectorySync,
    RootDirectorySync,
    Publication,
    ParentDirectorySync,
    PublicationDurable,
}

pub(super) fn commit_with_hook(
    temporary: &Path,
    canonical: &Path,
    expected_checksum: &str,
    at_step: impl FnMut(ArtifactCommitStep, &Path) -> io::Result<()>,
) -> Result<ArtifactCommitOutcome, PersistenceError> {
    // The shared engine invokes preparation and publication hooks sequentially.
    // Both retain this caller's single ordered crash/failure observer.
    let at_step = RefCell::new(at_step);
    commit_existing_directory_with(
        temporary,
        canonical,
        |directory, display| {
            let checksum = sync_tree(directory, display, &mut *at_step.borrow_mut())?;
            checksum.verify(expected_checksum)?;
            Ok::<(), PersistenceError>(())
        },
        |step, path| {
            let step = match step {
                DirectoryCommitStep::BeforePublication => ArtifactCommitStep::Publication,
                DirectoryCommitStep::BeforeParentSync => ArtifactCommitStep::ParentDirectorySync,
                DirectoryCommitStep::PublicationDurable => ArtifactCommitStep::PublicationDurable,
            };
            at_step.borrow_mut()(step, path)
        },
    )
    .map(|outcome| match outcome {
        DirectoryCommitOutcome::Published => ArtifactCommitOutcome::Published,
        DirectoryCommitOutcome::Recovered => ArtifactCommitOutcome::Recovered,
    })
    .map_err(|error| match error {
        DirectoryCommitError::InvalidPaths => PersistenceError::ArtifactCommitPathMismatch {
            temporary: temporary.display().to_string(),
            canonical: canonical.display().to_string(),
        },
        DirectoryCommitError::Conflict => PersistenceError::ArtifactCommitPathConflict {
            temporary: temporary.display().to_string(),
            canonical: canonical.display().to_string(),
        },
        DirectoryCommitError::Missing => PersistenceError::ArtifactCommitPathMissing {
            temporary: temporary.display().to_string(),
            canonical: canonical.display().to_string(),
        },
        DirectoryCommitError::Prepare(error) => error,
        DirectoryCommitError::BeforePublication(error)
        | DirectoryCommitError::AfterPublication(error) => error.into(),
    })
}

fn sync_tree(
    directory: &File,
    display_root: &Path,
    at_step: &mut impl FnMut(ArtifactCommitStep, &Path) -> io::Result<()>,
) -> Result<ArtifactChecksumRecord, PersistenceError> {
    let digest = sync_checksum_directory_at(directory, display_root, u64::MAX, |step, path| {
        let step = match step {
            TreeSyncStep::RegularFile => ArtifactCommitStep::RegularFileSync,
            TreeSyncStep::NestedDirectory => ArtifactCommitStep::NestedDirectorySync,
            TreeSyncStep::RootDirectory => ArtifactCommitStep::RootDirectorySync,
        };
        at_step(step, path)
    })
    .map_err(|error| match error {
        TreeChecksumError::Io(error) => PersistenceError::Io(error),
        TreeChecksumError::Input(error) => ArtifactError::Io(error.into()).into(),
        TreeChecksumError::Declaration(error) => ArtifactError::from(error).into(),
        TreeChecksumError::NonUtf8Path { path } => ArtifactError::NonUtf8Path { path }.into(),
        TreeChecksumError::UnsupportedEntry { path, kind } => {
            PersistenceError::UnsupportedArtifactEntry {
                path: path.display().to_string(),
                kind: format!("{kind:?}"),
            }
        }
        TreeChecksumError::InvalidPath { path } => PersistenceError::UnsupportedArtifactEntry {
            path: path.display().to_string(),
            kind: "Unknown".into(),
        },
    })?;
    Ok(ArtifactChecksumRecord::from_digest(*digest.as_bytes()))
}
