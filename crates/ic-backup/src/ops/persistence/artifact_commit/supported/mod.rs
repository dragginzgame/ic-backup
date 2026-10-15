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
    ffi::OsStr,
    fs::{self, File},
    io::{self, Seek, SeekFrom},
    path::{Path, PathBuf},
};

use ic_host_fs::read::tree::{TreeVisitor, walk_tree};
use rustix::{
    fd::{AsFd, OwnedFd},
    fs::{self as unix_fs, FileType, Mode, OFlags, RenameFlags},
};

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
    mut at_step: impl FnMut(ArtifactCommitStep, &Path) -> io::Result<()>,
) -> Result<ArtifactCommitOutcome, PersistenceError> {
    let (parent, temporary_name, canonical_name) = sibling_paths(temporary, canonical)?;
    let temporary_exists = path_exists_without_following(temporary)?;
    let canonical_exists = path_exists_without_following(canonical)?;
    let parent_fd = open_parent_directory(parent)?;

    match (temporary_exists, canonical_exists) {
        (true, false) => {
            let checksum = sync_tree(&parent_fd, temporary_name, temporary, &mut at_step)?;
            checksum.verify(expected_checksum)?;
            at_step(ArtifactCommitStep::Publication, canonical)?;
            unix_fs::renameat_with(
                &parent_fd,
                temporary_name,
                &parent_fd,
                canonical_name,
                RenameFlags::NOREPLACE,
            )
            .map_err(io::Error::from)?;
            at_step(ArtifactCommitStep::ParentDirectorySync, parent)?;
            unix_fs::fsync(&parent_fd).map_err(io::Error::from)?;
            at_step(ArtifactCommitStep::PublicationDurable, canonical)?;
            Ok(ArtifactCommitOutcome::Published)
        }
        (false, true) => {
            let checksum = sync_tree(&parent_fd, canonical_name, canonical, &mut at_step)?;
            checksum.verify(expected_checksum)?;
            at_step(ArtifactCommitStep::ParentDirectorySync, parent)?;
            unix_fs::fsync(&parent_fd).map_err(io::Error::from)?;
            at_step(ArtifactCommitStep::PublicationDurable, canonical)?;
            Ok(ArtifactCommitOutcome::Recovered)
        }
        (true, true) => Err(PersistenceError::ArtifactCommitPathConflict {
            temporary: temporary.display().to_string(),
            canonical: canonical.display().to_string(),
        }),
        (false, false) => Err(PersistenceError::ArtifactCommitPathMissing {
            temporary: temporary.display().to_string(),
            canonical: canonical.display().to_string(),
        }),
    }
}

fn sibling_paths<'a>(
    temporary: &'a Path,
    canonical: &'a Path,
) -> Result<(&'a Path, &'a OsStr, &'a OsStr), PersistenceError> {
    let invalid_paths = || PersistenceError::ArtifactCommitPathMismatch {
        temporary: temporary.display().to_string(),
        canonical: canonical.display().to_string(),
    };
    let temporary_parent = temporary.parent().ok_or_else(invalid_paths)?;
    let canonical_parent = canonical.parent().ok_or_else(invalid_paths)?;
    let temporary_name = temporary.file_name().ok_or_else(invalid_paths)?;
    let canonical_name = canonical.file_name().ok_or_else(invalid_paths)?;
    if temporary == canonical || temporary_parent != canonical_parent {
        return Err(invalid_paths());
    }

    Ok((temporary_parent, temporary_name, canonical_name))
}

fn path_exists_without_following(path: &Path) -> io::Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

fn open_parent_directory(path: &Path) -> io::Result<OwnedFd> {
    unix_fs::open(
        path,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(io::Error::from)
}

fn sync_tree(
    parent_fd: &impl AsFd,
    directory_name: &OsStr,
    display_root: &Path,
    at_step: &mut impl FnMut(ArtifactCommitStep, &Path) -> io::Result<()>,
) -> Result<ArtifactChecksumRecord, PersistenceError> {
    let directory_fd = unix_fs::openat(
        parent_fd,
        directory_name,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(io::Error::from)?;
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
