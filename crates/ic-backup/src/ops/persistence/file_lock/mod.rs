//! Module: `persistence::file_lock`
//!
//! Responsibility: open and exclusively lock one regular no-follow sidecar file.
//! Does not own: journal or command-lifetime semantics, paths, or error projection.

use std::{fs, io, path::Path};

#[cfg(unix)]
use ic_host_fs::durable::{
    RegularFileLockError, open_private_regular_lock_file, try_lock_exclusive,
};

#[derive(Debug)]
pub(super) enum FileLockError {
    Locked,
    UnsafeEntry { kind: String },
    Io(io::Error),
}

pub(super) fn acquire(path: &Path) -> Result<fs::File, FileLockError> {
    #[cfg(unix)]
    {
        acquire_supported(path)
    }

    #[cfg(not(unix))]
    {
        Err(FileLockError::Io(io::Error::new(
            io::ErrorKind::Unsupported,
            format!(
                "file locking is unsupported on this host: {}",
                path.display()
            ),
        )))
    }
}

#[cfg(unix)]
fn acquire_supported(path: &Path) -> Result<fs::File, FileLockError> {
    let file = open_private_regular_lock_file(path).map_err(|error| match error {
        RegularFileLockError::NotRegular => FileLockError::UnsafeEntry {
            kind: match fs::symlink_metadata(path) {
                Ok(metadata) if metadata.file_type().is_symlink() => "Symlink",
                Ok(metadata) if metadata.is_dir() => "Directory",
                _ => "Special",
            }
            .to_owned(),
        },
        error => FileLockError::Io(error.into()),
    })?;
    try_lock_exclusive(&file).map_err(|error| {
        if error.kind() == io::ErrorKind::WouldBlock {
            FileLockError::Locked
        } else {
            FileLockError::Io(error)
        }
    })?;
    Ok(file)
}
