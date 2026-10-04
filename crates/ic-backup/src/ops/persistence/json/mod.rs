//! Module: `persistence::json`
//!
//! Responsibility: read and durably create or replace JSON persistence documents.
//! Does not own: document validation, layout paths, or integrity checks.
//! Boundary: provides explicit create-only and replace filesystem primitives.

use crate::ops::persistence::PersistenceError;

use std::{
    ffi::OsString,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use serde::{Serialize, de::DeserializeOwned};
#[cfg(unix)]
use std::io::Read;

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Durably replace a machine record using a sibling temporary and rename.
///
/// # Errors
/// Returns encoding or IO failures; a lost post-rename response requires reconciliation.
pub fn write_json_durable<T>(path: &Path, value: &T) -> Result<(), PersistenceError>
where
    T: Serialize,
{
    let bytes = serde_json::to_vec_pretty(value)?;
    replace_bytes_at_barriers(path, &bytes, |_| {}).map_err(PersistenceError::from)
}

/// Publish a new machine record without replacing an existing entry.
///
/// # Errors
/// Returns encoding, existing-destination or IO failures.
pub fn create_json_durable<T>(path: &Path, value: &T) -> Result<(), PersistenceError>
where
    T: Serialize,
{
    let bytes = serde_json::to_vec_pretty(value)?;
    create_bytes_at_barriers(path, &bytes, || {}, || {}).map_err(PersistenceError::from)
}

/// Read one regular no-follow JSON file within an explicit byte limit.
///
/// # Errors
/// Rejects unsafe files, excessive bytes, invalid JSON and filesystem failures.
pub fn read_json<T>(path: &Path, max_bytes: u64) -> Result<T, PersistenceError>
where
    T: DeserializeOwned,
{
    #[cfg(unix)]
    {
        let file = {
            use rustix::fs::{FileType, Mode, OFlags, fstat, open};
            let fd = open(
                path,
                OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .map_err(|error| io::Error::from_raw_os_error(error.raw_os_error()))?;
            let metadata =
                fstat(&fd).map_err(|error| io::Error::from_raw_os_error(error.raw_os_error()))?;
            if !FileType::from_raw_mode(metadata.st_mode).is_file() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "record must be a regular file",
                )
                .into());
            }
            File::from(fd)
        };
        let mut bytes = Vec::new();
        file.take(max_bytes.saturating_add(1))
            .read_to_end(&mut bytes)?;
        if bytes.len() > usize::try_from(max_bytes).unwrap_or(usize::MAX) {
            return Err(PersistenceError::RecordTooLarge { limit: max_bytes });
        }
        Ok(serde_json::from_slice(&bytes)?)
    }
    #[cfg(not(unix))]
    {
        let _ = (path, max_bytes);
        Err(io::Error::from(io::ErrorKind::Unsupported).into())
    }
}

fn create_bytes_at_barriers(
    path: &Path,
    bytes: &[u8],
    mut before_publication: impl FnMut(),
    mut after_directory_sync: impl FnMut(),
) -> io::Result<()> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    create_private_parents(parent)?;

    let (temp_path, mut temp_file) = create_sibling_temp(path, parent)?;
    if let Err(error) = temp_file
        .write_all(bytes)
        .and_then(|()| temp_file.sync_all())
    {
        drop(temp_file);
        let _ = fs::remove_file(&temp_path);
        return Err(error);
    }
    drop(temp_file);
    before_publication();

    if let Err(error) = fs::hard_link(&temp_path, path) {
        let _ = fs::remove_file(&temp_path);
        return Err(error);
    }
    fs::remove_file(&temp_path)?;
    File::open(parent)?.sync_all()?;
    after_directory_sync();
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DurableWriteBarrier {
    BeforeRename,
    AfterDirectorySync,
}

fn replace_bytes_at_barriers(
    path: &Path,
    bytes: &[u8],
    mut barrier: impl FnMut(DurableWriteBarrier),
) -> io::Result<()> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    create_private_parents(parent)?;

    let (temp_path, mut temp_file) = create_sibling_temp(path, parent)?;
    if let Err(error) = temp_file
        .write_all(bytes)
        .and_then(|()| temp_file.sync_all())
    {
        drop(temp_file);
        let _ = fs::remove_file(&temp_path);
        return Err(error);
    }
    drop(temp_file);
    barrier(DurableWriteBarrier::BeforeRename);

    if let Err(error) = fs::rename(&temp_path, path) {
        let _ = fs::remove_file(&temp_path);
        return Err(error);
    }

    File::open(parent)?.sync_all()?;
    barrier(DurableWriteBarrier::AfterDirectorySync);
    Ok(())
}

#[cfg(test)]
pub(crate) fn write_json_durable_at_barriers<T>(
    path: &Path,
    value: &T,
    barrier: impl FnMut(DurableWriteBarrier),
) -> Result<(), PersistenceError>
where
    T: Serialize,
{
    let bytes = serde_json::to_vec_pretty(value)?;
    replace_bytes_at_barriers(path, &bytes, barrier).map_err(PersistenceError::from)
}

#[cfg(test)]
pub(crate) fn create_json_durable_at_barriers<T>(
    path: &Path,
    value: &T,
    before_publication: impl FnMut(),
    after_directory_sync: impl FnMut(),
) -> Result<(), PersistenceError>
where
    T: Serialize,
{
    let bytes = serde_json::to_vec_pretty(value)?;
    create_bytes_at_barriers(path, &bytes, before_publication, after_directory_sync)
        .map_err(PersistenceError::from)
}

fn create_sibling_temp(path: &Path, parent: &Path) -> io::Result<(PathBuf, File)> {
    let file_name = path.file_name().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("durable write target has no file name: {}", path.display()),
        )
    })?;

    for _ in 0..64 {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let mut temp_name = OsString::from(".");
        temp_name.push(file_name);
        temp_name.push(format!(".ic-backup-tmp-{}-{sequence}", std::process::id()));
        let temp_path = parent.join(temp_name);
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        match options.open(&temp_path) {
            Ok(file) => return Ok((temp_path, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }

    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        format!(
            "could not allocate a unique sibling temporary file for {}",
            path.display()
        ),
    ))
}

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod regressions;
#[cfg(test)]
mod tests;

fn create_private_parents(parent: &Path) -> io::Result<()> {
    let mut missing = Vec::new();
    let mut current = parent;
    while !current.try_exists()? {
        missing.push(current.to_path_buf());
        current = current
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
    }
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(parent)?;
    // Persist each newly created directory and its link from the existing root.
    for directory in missing {
        File::open(&directory)?.sync_all()?;
        let ancestor = directory
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        File::open(ancestor)?.sync_all()?;
    }
    Ok(())
}
