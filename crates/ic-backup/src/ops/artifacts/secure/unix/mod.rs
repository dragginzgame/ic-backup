//! Unix descriptor-based artifact traversal and staging.

use super::{ArtifactChecksumRecord, ArtifactError, ExpectedArtifactType};
use crate::ops::artifacts::{
    checksum_reader, checksum_relative_files, copy_from_reader, descriptor, require_utf8_tree_name,
};

use std::{
    ffi::OsStr,
    fs::{DirBuilder, File, OpenOptions},
    os::unix::{
        ffi::OsStrExt,
        fs::{DirBuilderExt, OpenOptionsExt},
    },
    path::{Component, Path, PathBuf},
};

use rustix::{
    fd::{AsFd, OwnedFd},
    fs::{self as unix_fs, Dir, FileType, Mode, OFlags},
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
            create_private_directory(destination)?;
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
    let mut checksums = Vec::new();
    collect_directory_entries(
        directory_fd,
        relative_directory,
        display_root,
        action,
        &mut checksums,
    )?;
    checksum_relative_files(checksums).map_err(ArtifactError::from)
}

fn collect_directory_entries(
    directory_fd: &OwnedFd,
    relative_directory: &Path,
    display_root: &Path,
    action: DirectoryAction<'_>,
    checksums: &mut Vec<(PathBuf, ArtifactChecksumRecord)>,
) -> Result<(), ArtifactError> {
    let mut directory = Dir::read_from(directory_fd).map_err(std::io::Error::from)?;
    while let Some(entry) = directory.read() {
        let entry = entry.map_err(std::io::Error::from)?;
        let name_bytes = entry.file_name().to_bytes();
        if matches!(name_bytes, b"." | b"..") {
            continue;
        }
        let name = OsStr::from_bytes(name_bytes);
        require_utf8_tree_name(name, display_root)?;
        let relative_path = relative_directory.join(name);
        let display_path = display_root.join(&relative_path);
        let kind = descriptor::entry_type(directory_fd, entry.file_name())?;
        match kind {
            FileType::RegularFile => {
                let child = descriptor::open_child(directory_fd, entry.file_name(), false)?;
                ensure_opened_type(&child, FileType::RegularFile, &display_path)?;
                let checksum = match action {
                    DirectoryAction::Checksum => checksum_reader(&mut File::from(child))?,
                    DirectoryAction::CopyTo(destination_root) => {
                        copy_file(&child, &destination_root.join(&relative_path))?
                    }
                };
                checksums.push((relative_path, checksum));
            }
            FileType::Directory => {
                let child = descriptor::open_child(directory_fd, entry.file_name(), true)?;
                ensure_opened_type(&child, FileType::Directory, &display_path)?;
                if let DirectoryAction::CopyTo(destination_root) = action {
                    create_private_directory(&destination_root.join(&relative_path))?;
                }
                collect_directory_entries(&child, &relative_path, display_root, action, checksums)?;
            }
            kind => return Err(unsupported_entry(&display_path, kind)),
        }
    }
    Ok(())
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

fn create_private_directory(path: &Path) -> Result<(), ArtifactError> {
    DirBuilder::new().mode(0o700).create(path)?;
    Ok(())
}

fn ensure_opened_type(
    fd: &impl AsFd,
    expected: FileType,
    path: &Path,
) -> Result<(), ArtifactError> {
    let actual = descriptor::opened_type(fd)?;
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
