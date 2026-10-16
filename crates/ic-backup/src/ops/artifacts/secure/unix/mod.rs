//! Unix descriptor-based artifact traversal and staging.

use super::{ArtifactChecksumRecord, ArtifactError, ExpectedArtifactType};
use crate::ops::artifacts::{checksum_reader, checksum_relative_files, copy_from_reader};
use ic_host_fs::read::tree::{TreeVisitor, walk_tree};

use std::{
    fs::{File, OpenOptions},
    os::unix::fs::OpenOptionsExt,
    path::{Component, Path, PathBuf},
};

use rustix::{
    fd::{AsFd, OwnedFd},
    fs::{self as unix_fs, FileType, Mode, OFlags},
};

pub(super) fn checksum_path(
    path: &Path,
    expected: ExpectedArtifactType,
) -> Result<ArtifactChecksumRecord, ArtifactError> {
    let (root, relative) = if path.is_absolute() {
        (
            Path::new("/"),
            path.strip_prefix("/").map_err(std::io::Error::other)?,
        )
    } else {
        (Path::new("."), path)
    };
    let (artifact, kind) = open_relative(root, relative)?;
    match expected {
        ExpectedArtifactType::Any => {}
        ExpectedArtifactType::Directory if kind == FileType::Directory => {}
        ExpectedArtifactType::File if kind == FileType::RegularFile => {}
        ExpectedArtifactType::Directory | ExpectedArtifactType::File => {
            return Err(unsupported_entry(path, kind));
        }
    }
    checksum_opened(&artifact, kind, path)
}

pub(super) fn checksum_relative_path(
    root: &Path,
    relative: &Path,
) -> Result<ArtifactChecksumRecord, ArtifactError> {
    let (artifact, kind) = open_relative(root, relative)?;
    checksum_opened(&artifact, kind, relative)
}

pub(super) fn stage_relative_path(
    root: &Path,
    relative: &Path,
    destination: &Path,
) -> Result<ArtifactChecksumRecord, ArtifactError> {
    let (artifact, kind) = open_relative(root, relative)?;
    match kind {
        FileType::RegularFile => copy_file(&artifact, destination),
        FileType::Directory => {
            private_directory(destination)?;
            directory_checksum(
                &artifact,
                Path::new(""),
                relative,
                DirectoryAction::CopyTo(destination),
            )
        }
        kind => Err(unsupported_entry(relative, kind)),
    }
}

fn open_relative(root: &Path, relative: &Path) -> Result<(OwnedFd, FileType), ArtifactError> {
    let components = relative
        .components()
        .map(|component| match component {
            Component::Normal(name) if name.to_str().is_some() => Ok(name),
            _ => Err(unsupported_entry(relative, FileType::Unknown)),
        })
        .collect::<Result<Vec<_>, _>>()?;
    if components.is_empty() {
        return Err(unsupported_entry(relative, FileType::Unknown));
    }

    let mut current = unix_fs::open(
        root,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(std::io::Error::from)?;
    for (index, component) in components.iter().enumerate() {
        let is_leaf = index + 1 == components.len();
        let mut flags = OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC;
        if !is_leaf {
            flags |= OFlags::DIRECTORY;
        }
        current = unix_fs::openat(&current, *component, flags, Mode::empty())
            .map_err(std::io::Error::from)?;
        if !is_leaf {
            ensure_opened_type(&current, FileType::Directory, relative)?;
        }
    }
    let metadata = unix_fs::fstat(&current).map_err(std::io::Error::from)?;
    Ok((current, FileType::from_raw_mode(metadata.st_mode)))
}

fn checksum_opened(
    artifact: &OwnedFd,
    kind: FileType,
    display_path: &Path,
) -> Result<ArtifactChecksumRecord, ArtifactError> {
    match kind {
        FileType::RegularFile => {
            let mut file = File::from(artifact.try_clone()?);
            checksum_reader(&mut file)
        }
        FileType::Directory => directory_checksum(
            artifact,
            Path::new(""),
            display_path,
            DirectoryAction::Checksum,
        ),
        kind => Err(unsupported_entry(display_path, kind)),
    }
}

#[derive(Clone, Copy)]
enum DirectoryAction<'a> {
    Checksum,
    CopyTo(&'a Path),
}

fn directory_checksum(
    directory_fd: &OwnedFd,
    relative_directory: &Path,
    display_root: &Path,
    action: DirectoryAction<'_>,
) -> Result<ArtifactChecksumRecord, ArtifactError> {
    let mut visitor = ChecksumVisitor {
        action,
        checksums: Vec::new(),
    };
    walk_tree(directory_fd, relative_directory, display_root, &mut visitor)?;
    checksum_relative_files(visitor.checksums).map_err(ArtifactError::from)
}

struct ChecksumVisitor<'a> {
    action: DirectoryAction<'a>,
    checksums: Vec<(PathBuf, ArtifactChecksumRecord)>,
}

impl TreeVisitor for ChecksumVisitor<'_> {
    type Error = ArtifactError;

    fn file(
        &mut self,
        fd: &OwnedFd,
        relative: &Path,
        _display: &Path,
    ) -> Result<(), ArtifactError> {
        let checksum = match self.action {
            DirectoryAction::Checksum => checksum_reader(&mut File::from(fd.try_clone()?))?,
            DirectoryAction::CopyTo(destination_root) => {
                copy_file(fd, &destination_root.join(relative))?
            }
        };
        self.checksums.push((relative.to_path_buf(), checksum));
        Ok(())
    }

    fn enter_directory(&mut self, relative: &Path, _display: &Path) -> Result<(), ArtifactError> {
        if let DirectoryAction::CopyTo(destination_root) = self.action {
            private_directory(&destination_root.join(relative))?;
        }
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

fn copy_file(
    source: &OwnedFd,
    destination: &Path,
) -> Result<ArtifactChecksumRecord, ArtifactError> {
    let mut source = File::from(source.try_clone()?);
    let mut destination = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(destination)?;
    copy_from_reader(&mut source, &mut destination)
}

fn private_directory(path: &Path) -> Result<(), ArtifactError> {
    ic_host_fs::durable::create_private_directory(path)
        .map(|_| ())
        .map_err(|error| ArtifactError::Io(crate::ops::directory_creation_io(error)))
}

fn ensure_opened_type(
    fd: &impl AsFd,
    expected: FileType,
    path: &Path,
) -> Result<(), ArtifactError> {
    let metadata = unix_fs::fstat(fd).map_err(std::io::Error::from)?;
    let actual = FileType::from_raw_mode(metadata.st_mode);
    if actual == expected {
        Ok(())
    } else {
        Err(unsupported_entry(path, actual))
    }
}

fn unsupported_entry(path: &Path, kind: FileType) -> ArtifactError {
    ArtifactError::UnsupportedEntry {
        path: path.display().to_string(),
        kind: format!("{kind:?}"),
    }
}
