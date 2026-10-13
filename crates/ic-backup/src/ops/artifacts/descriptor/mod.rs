//! Shared Unix descriptor admission for artifact traversal and publication.

use std::io;

use rustix::{
    fd::{AsFd, OwnedFd},
    fs::{self, AtFlags, FileType, Mode, OFlags},
};

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
