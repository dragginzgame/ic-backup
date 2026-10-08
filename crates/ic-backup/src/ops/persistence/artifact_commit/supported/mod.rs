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
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};

use rustix::{
    fd::{AsFd, OwnedFd},
    fs::{self as unix_fs, AtFlags, Dir, FileType, Mode, OFlags, RenameFlags},
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
    let mut checksums = Vec::new();
    sync_directory(
        &directory_fd,
        Path::new(""),
        display_root,
        &mut checksums,
        at_step,
    )?;
    checksum_relative_files(checksums)
        .map_err(ArtifactError::from)
        .map_err(PersistenceError::from)
}

fn sync_directory(
    directory_fd: &OwnedFd,
    relative_directory: &Path,
    display_root: &Path,
    checksums: &mut Vec<(PathBuf, ArtifactChecksumRecord)>,
    at_step: &mut impl FnMut(ArtifactCommitStep, &Path) -> io::Result<()>,
) -> Result<(), PersistenceError> {
    let mut directory = Dir::read_from(directory_fd).map_err(io::Error::from)?;
    while let Some(entry) = directory.read() {
        let entry = entry.map_err(io::Error::from)?;
        let name_bytes = entry.file_name().to_bytes();
        if matches!(name_bytes, b"." | b"..") {
            continue;
        }

        let name = OsStr::from_bytes(name_bytes);
        if name.to_str().is_none() {
            return Err(crate::ops::artifacts::ArtifactError::NonUtf8Path {
                path: display_root.join(name),
            }
            .into());
        }
        let relative_path = relative_directory.join(name);
        let display_path = display_root.join(&relative_path);
        let observed = unix_fs::statat(directory_fd, entry.file_name(), AtFlags::SYMLINK_NOFOLLOW)
            .map_err(io::Error::from)?;
        let observed_type = FileType::from_raw_mode(observed.st_mode);
        match observed_type {
            FileType::RegularFile => {
                let child_fd = open_child(directory_fd, entry.file_name())?;
                ensure_opened_type(&child_fd, FileType::RegularFile, &display_path)?;
                let mut file = File::from(child_fd);
                at_step(ArtifactCommitStep::RegularFileSync, &display_path)?;
                file.sync_all()?;
                file.seek(SeekFrom::Start(0))?;
                checksums.push((relative_path, checksum_reader(&mut file)?));
            }
            FileType::Directory => {
                let child_fd = open_child(directory_fd, entry.file_name())?;
                ensure_opened_type(&child_fd, FileType::Directory, &display_path)?;
                sync_directory(&child_fd, &relative_path, display_root, checksums, at_step)?;
            }
            kind => return Err(unsupported_entry(&display_path, kind)),
        }
    }

    let (step, display_path) = if relative_directory.as_os_str().is_empty() {
        (
            ArtifactCommitStep::RootDirectorySync,
            display_root.to_path_buf(),
        )
    } else {
        (
            ArtifactCommitStep::NestedDirectorySync,
            display_root.join(relative_directory),
        )
    };
    at_step(step, &display_path)?;
    unix_fs::fsync(directory_fd).map_err(io::Error::from)?;
    Ok(())
}

fn open_child(parent: &impl AsFd, name: &std::ffi::CStr) -> io::Result<OwnedFd> {
    unix_fs::openat(
        parent,
        name,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(io::Error::from)
}

fn ensure_opened_type(
    fd: &impl AsFd,
    expected: FileType,
    path: &Path,
) -> Result<(), PersistenceError> {
    let opened = unix_fs::fstat(fd).map_err(io::Error::from)?;
    let actual = FileType::from_raw_mode(opened.st_mode);
    if actual == expected {
        Ok(())
    } else {
        Err(unsupported_entry(path, actual))
    }
}

fn unsupported_entry(path: &Path, kind: FileType) -> PersistenceError {
    PersistenceError::UnsupportedArtifactEntry {
        path: path.display().to_string(),
        kind: format!("{kind:?}"),
    }
}
