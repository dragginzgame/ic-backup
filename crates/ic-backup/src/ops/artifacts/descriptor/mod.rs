//! Shared Unix descriptor admission for artifact traversal and publication.

use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use rustix::{
    fd::{AsFd, OwnedFd},
    fs::{self, AtFlags, Dir, FileType, Mode, OFlags},
};

/// Caller-selected actions around a no-follow descriptor walk.
pub(crate) trait TreeVisitor {
    type Error: From<io::Error>;

    fn directory_open_requires_type(&self) -> bool;
    fn file(&mut self, fd: &OwnedFd, relative: &Path, display: &Path) -> Result<(), Self::Error>;
    fn enter_directory(&mut self, relative: &Path, display: &Path) -> Result<(), Self::Error>;
    fn leave_directory(
        &mut self,
        fd: &OwnedFd,
        relative: &Path,
        display: &Path,
    ) -> Result<(), Self::Error>;
    fn unsupported(&self, display: &Path, kind: FileType) -> Self::Error;
    fn non_utf8(&self, display: &Path) -> Self::Error;
}

pub(crate) fn walk_tree<V: TreeVisitor>(
    directory_fd: &OwnedFd,
    relative_directory: &Path,
    display_root: &Path,
    visitor: &mut V,
) -> Result<(), V::Error> {
    let mut directory = Dir::read_from(directory_fd).map_err(io::Error::from)?;
    while let Some(entry) = directory.read() {
        let entry = entry.map_err(io::Error::from)?;
        let name_bytes = entry.file_name().to_bytes();
        if matches!(name_bytes, b"." | b"..") {
            continue;
        }
        let name = std::ffi::OsStr::from_bytes(name_bytes);
        if name.to_str().is_none() {
            return Err(visitor.non_utf8(&display_root.join(name)));
        }
        let relative_path: PathBuf = relative_directory.join(name);
        let display_path = display_root.join(&relative_path);
        let kind = entry_type(directory_fd, entry.file_name())?;
        match kind {
            FileType::RegularFile | FileType::Directory => {
                let child = open_child(
                    directory_fd,
                    entry.file_name(),
                    kind == FileType::Directory && visitor.directory_open_requires_type(),
                )?;
                let opened = opened_type(&child)?;
                if opened != kind {
                    return Err(visitor.unsupported(&display_path, opened));
                }
                if kind == FileType::RegularFile {
                    visitor.file(&child, &relative_path, &display_path)?;
                } else {
                    visitor.enter_directory(&relative_path, &display_path)?;
                    walk_tree(&child, &relative_path, display_root, visitor)?;
                }
            }
            kind => return Err(visitor.unsupported(&display_path, kind)),
        }
    }
    let display_path = if relative_directory.as_os_str().is_empty() {
        display_root.to_path_buf()
    } else {
        display_root.join(relative_directory)
    };
    visitor.leave_directory(directory_fd, relative_directory, &display_path)
}

pub(crate) fn entry_type(parent: &impl AsFd, name: &std::ffi::CStr) -> io::Result<FileType> {
    let metadata = fs::statat(parent, name, AtFlags::SYMLINK_NOFOLLOW).map_err(io::Error::from)?;
    Ok(FileType::from_raw_mode(metadata.st_mode))
}

pub(crate) fn open_child(
    parent: &impl AsFd,
    name: &std::ffi::CStr,
    directory: bool,
) -> io::Result<OwnedFd> {
    let mut flags = OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC;
    if directory {
        flags |= OFlags::DIRECTORY;
    }
    fs::openat(parent, name, flags, Mode::empty()).map_err(io::Error::from)
}

pub(crate) fn opened_type(fd: &impl AsFd) -> io::Result<FileType> {
    let metadata = fs::fstat(fd).map_err(io::Error::from)?;
    Ok(FileType::from_raw_mode(metadata.st_mode))
}
