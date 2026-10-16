//! Descriptor-based synchronization and atomic no-replace publication.

use super::ArtifactCommitOutcome;
use crate::{
    model::artifacts::ArtifactChecksumRecord,
    ops::{
        artifacts::{ArtifactError, checksum_reader, checksum_relative_files},
        persistence::PersistenceError,
    },
};

use std::{
    fs::File,
    io::{self, Seek, SeekFrom},
    path::{Path, PathBuf},
};

use ic_host_fs::{
    durable::{
        DirectoryCommitError, DirectoryCommitOutcome, DirectoryCommitStep,
        commit_existing_directory_with,
    },
    read::tree::{TreeVisitor, walk_tree},
};
use rustix::{
    fd::{AsFd, OwnedFd},
    fs::{self as unix_fs, FileType},
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
    let directory_fd = directory.as_fd().try_clone_to_owned()?;
    let mut visitor = SyncVisitor {
        checksums: Vec::new(),
        at_step,
    };
    walk_tree(&directory_fd, Path::new(""), display_root, &mut visitor)?;
    checksum_relative_files(visitor.checksums)
        .map_err(ArtifactError::from)
        .map_err(PersistenceError::from)
}

struct SyncVisitor<'a, F> {
    checksums: Vec<(PathBuf, ArtifactChecksumRecord)>,
    at_step: &'a mut F,
}

impl<F> TreeVisitor for SyncVisitor<'_, F>
where
    F: FnMut(ArtifactCommitStep, &Path) -> io::Result<()>,
{
    type Error = PersistenceError;

    fn file(
        &mut self,
        fd: &OwnedFd,
        relative: &Path,
        display: &Path,
    ) -> Result<(), PersistenceError> {
        let mut file = File::from(fd.try_clone()?);
        (self.at_step)(ArtifactCommitStep::RegularFileSync, display)?;
        file.sync_all()?;
        file.seek(SeekFrom::Start(0))?;
        self.checksums
            .push((relative.to_path_buf(), checksum_reader(&mut file)?));
        Ok(())
    }

    fn enter_directory(
        &mut self,
        _relative: &Path,
        _display: &Path,
    ) -> Result<(), PersistenceError> {
        Ok(())
    }

    fn leave_directory(
        &mut self,
        fd: &OwnedFd,
        relative: &Path,
        display: &Path,
    ) -> Result<(), PersistenceError> {
        let step = if relative.as_os_str().is_empty() {
            ArtifactCommitStep::RootDirectorySync
        } else {
            ArtifactCommitStep::NestedDirectorySync
        };
        (self.at_step)(step, display)?;
        // Keep the same descriptor open during the walk and sync that precedes publication.
        unix_fs::fsync(fd).map_err(io::Error::from)?;
        Ok(())
    }

    fn unsupported(&self, display: &Path, kind: FileType) -> PersistenceError {
        unsupported_entry(display, kind)
    }

    fn non_utf8(&self, display: &Path) -> PersistenceError {
        ArtifactError::NonUtf8Path {
            path: display.to_path_buf(),
        }
        .into()
    }
}

fn unsupported_entry(path: &Path, kind: FileType) -> PersistenceError {
    PersistenceError::UnsupportedArtifactEntry {
        path: path.display().to_string(),
        kind: format!("{kind:?}"),
    }
}
