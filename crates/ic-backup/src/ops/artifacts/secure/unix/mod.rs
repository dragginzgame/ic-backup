//! Unix descriptor-based artifact directory traversal.

use super::{ArtifactChecksumRecord, ArtifactError};
use crate::ops::artifacts::{checksum_reader, checksum_relative_files};
use ic_host_fs::read::{
    OpenEntryError, OpenedEntry, open_directory_no_follow, open_entry_beneath,
    tree::{TreeVisitor, walk_tree},
};

use std::{
    fs::File,
    path::{Component, Path, PathBuf},
};

use rustix::{
    fd::{AsFd, OwnedFd},
    fs::FileType,
};

pub(super) fn checksum_directory(path: &Path) -> Result<ArtifactChecksumRecord, ArtifactError> {
    let (root, relative) = if path.is_absolute() {
        (
            Path::new("/"),
            path.strip_prefix("/").map_err(std::io::Error::other)?,
        )
    } else {
        (Path::new("."), path)
    };
    let (artifact, kind) = open_relative(root, relative)?;
    if kind != FileType::Directory {
        return Err(unsupported_entry(path, kind));
    }
    directory_checksum(&artifact, Path::new(""), path)
}

fn open_relative(root: &Path, relative: &Path) -> Result<(OwnedFd, FileType), ArtifactError> {
    let mut has_component = false;
    for component in relative.components() {
        if !matches!(component, Component::Normal(name) if name.to_str().is_some()) {
            return Err(unsupported_entry(relative, FileType::Unknown));
        }
        has_component = true;
    }
    if !has_component {
        return Err(unsupported_entry(relative, FileType::Unknown));
    }

    let root = open_directory_no_follow(root)?;
    match open_entry_beneath(root.as_fd(), relative) {
        Ok(OpenedEntry::RegularFile(file)) => Ok((file.into(), FileType::RegularFile)),
        Ok(OpenedEntry::Directory(file)) => Ok((file.into(), FileType::Directory)),
        Err(OpenEntryError::UnsupportedEntry { kind }) => Err(unsupported_entry(relative, kind)),
        Err(OpenEntryError::Io(error)) => Err(error.into()),
    }
}

fn directory_checksum(
    directory_fd: &OwnedFd,
    relative_directory: &Path,
    display_root: &Path,
) -> Result<ArtifactChecksumRecord, ArtifactError> {
    let mut visitor = ChecksumVisitor {
        checksums: Vec::new(),
    };
    walk_tree(directory_fd, relative_directory, display_root, &mut visitor)?;
    checksum_relative_files(visitor.checksums).map_err(ArtifactError::from)
}

struct ChecksumVisitor {
    checksums: Vec<(PathBuf, ArtifactChecksumRecord)>,
}

impl TreeVisitor for ChecksumVisitor {
    type Error = ArtifactError;

    fn file(
        &mut self,
        fd: &OwnedFd,
        relative: &Path,
        _display: &Path,
    ) -> Result<(), ArtifactError> {
        let checksum = checksum_reader(&mut File::from(fd.try_clone()?))?;
        self.checksums.push((relative.to_path_buf(), checksum));
        Ok(())
    }

    fn enter_directory(&mut self, _relative: &Path, _display: &Path) -> Result<(), ArtifactError> {
        Ok(())
    }

    fn leave_directory(
        &mut self,
        _fd: &OwnedFd,
        _relative: &Path,
        _display: &Path,
    ) -> Result<(), ArtifactError> {
        Ok(())
    }

    fn unsupported(&self, display: &Path, kind: FileType) -> ArtifactError {
        unsupported_entry(display, kind)
    }

    fn non_utf8(&self, display: &Path) -> ArtifactError {
        ArtifactError::NonUtf8Path {
            path: display.to_path_buf(),
        }
    }
}

fn unsupported_entry(path: &Path, kind: FileType) -> ArtifactError {
    ArtifactError::UnsupportedEntry {
        path: path.display().to_string(),
        kind: format!("{kind:?}"),
    }
}
