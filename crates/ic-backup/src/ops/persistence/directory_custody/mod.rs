//! Narrow no-follow directory selection and held-inode comparison.

use std::{
    fs::{self, File},
    io,
    path::Path,
};

pub(super) fn open_directory(path: &Path) -> io::Result<File> {
    #[cfg(unix)]
    {
        use rustix::fs::{Mode, OFlags, open};
        open(
            path,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map(File::from)
        .map_err(io::Error::from)
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Err(io::Error::from(io::ErrorKind::Unsupported))
    }
}

pub(super) fn same_directory(path: &Path, held: &File) -> io::Result<bool> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let current = fs::symlink_metadata(path)?;
        let original = held.metadata()?;
        Ok(current.is_dir() && current.dev() == original.dev() && current.ino() == original.ino())
    }
    #[cfg(not(unix))]
    {
        let _ = (path, held);
        Err(io::Error::from(io::ErrorKind::Unsupported))
    }
}
