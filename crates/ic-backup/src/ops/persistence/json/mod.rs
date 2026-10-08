//! Module: `persistence::json`
//!
//! Responsibility: read and durably create or replace JSON persistence documents.
//! Does not own: document validation, layout paths, or integrity checks.
//! Boundary: provides explicit create-only and replace filesystem primitives.

use crate::ops::persistence::PersistenceError;

use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

use serde::{Serialize, de::DeserializeOwned};

use ic_host_fs::durable::{PublicationMode, WriteOptions};

/// Check the maintained pretty-JSON budget without allocating an encoded record.
pub(super) fn check_json_size(
    value: &impl Serialize,
    max_bytes: u64,
) -> Result<(), PersistenceError> {
    let mut writer = ic_host_artifacts::artifact::BoundedWriter::new(io::sink(), max_bytes);
    let result = serde_json::to_writer_pretty(&mut writer, value);
    if writer.limit_exceeded() {
        return Err(PersistenceError::RecordTooLarge { limit: max_bytes });
    }
    result?;
    Ok(())
}

/// Durably replace a machine record using a sibling temporary and rename.
///
/// # Errors
/// Returns encoding/parent IO failures or structured shared publication failures.
/// An after-publication failure or lost response requires reconciliation.
pub fn write_json_durable<T>(path: &Path, value: &T) -> Result<(), PersistenceError>
where
    T: Serialize,
{
    let bytes = serde_json::to_vec_pretty(value)?;
    publish_bytes_at_barriers(path, &bytes, PublicationMode::Replace, |_| {})
}

/// Publish a new machine record without replacing an existing entry.
///
/// # Errors
/// Returns encoding/parent IO failures or structured shared publication failures,
/// including create-only conflicts and visible output after failed completion.
pub fn create_json_durable<T>(path: &Path, value: &T) -> Result<(), PersistenceError>
where
    T: Serialize,
{
    let bytes = serde_json::to_vec_pretty(value)?;
    publish_bytes_at_barriers(path, &bytes, PublicationMode::CreateNew, |_| {})
}

/// Read one regular no-follow JSON file within an explicit byte limit.
///
/// Reuses the shared bounded regular-file reader; caller-selected parents and
/// concurrent byte custody remain local. Only the final symlink is rejected.
///
/// # Errors
/// Rejects unsafe files, excessive bytes, invalid JSON and filesystem failures.
pub fn read_json<T>(path: &Path, max_bytes: u64) -> Result<T, PersistenceError>
where
    T: DeserializeOwned,
{
    #[cfg(unix)]
    {
        let bytes = ic_host_fs::read::read_file_no_follow(
            path,
            usize::try_from(max_bytes).unwrap_or(usize::MAX),
        )
        .map_err(|error| record_read_error(error, max_bytes))?;
        Ok(serde_json::from_slice(&bytes)?)
    }
    #[cfg(not(unix))]
    {
        let _ = (path, max_bytes);
        Err(io::Error::from(io::ErrorKind::Unsupported).into())
    }
}

#[cfg(unix)]
fn record_read_error(
    error: ic_host_artifacts::artifact::ArtifactError,
    max_bytes: u64,
) -> PersistenceError {
    use ic_host_artifacts::artifact::ArtifactError;
    match error {
        ArtifactError::NotRegularFile => PersistenceError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            "record must be a regular file",
        )),
        ArtifactError::LimitExceeded { .. } => {
            PersistenceError::RecordTooLarge { limit: max_bytes }
        }
        error => PersistenceError::Io(error.into()),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DurableWriteBarrier {
    BeforeRename,
    AfterDirectorySync,
}

// Keep one shared publication engine for ordinary writes and crash qualification.
fn publish_bytes_at_barriers(
    path: &Path,
    bytes: &[u8],
    mode: PublicationMode,
    mut barrier: impl FnMut(DurableWriteBarrier),
) -> Result<(), PersistenceError> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    create_private_parents(parent)?;
    let options = WriteOptions {
        mode,
        permissions: 0o600,
    };
    let produce = |file: &mut File| -> io::Result<()> {
        file.write_all(bytes)?;
        // Acknowledged pre-publication death must leave synchronized staging.
        // Host repeats this sync as part of its own identity/publication admission.
        file.sync_all()?;
        barrier(DurableWriteBarrier::BeforeRename);
        Ok(())
    };
    #[cfg(unix)]
    let result = {
        use std::os::fd::AsFd;
        let name = path.file_name().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "durable write target has no filename",
            )
        })?;
        let directory = File::open(parent)?;
        ic_host_fs::durable::write_at_with(directory.as_fd(), name, options, produce)
    };
    #[cfg(not(unix))]
    let result = ic_host_fs::durable::write_typed_with(path, options, produce);
    result.map_err(PersistenceError::Publication)?;
    // Successful Host completion includes publication and the held-parent sync.
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
    publish_bytes_at_barriers(path, &bytes, PublicationMode::Replace, barrier)
}

#[cfg(test)]
pub(crate) fn create_json_durable_at_barriers<T>(
    path: &Path,
    value: &T,
    mut before_publication: impl FnMut(),
    mut after_directory_sync: impl FnMut(),
) -> Result<(), PersistenceError>
where
    T: Serialize,
{
    let bytes = serde_json::to_vec_pretty(value)?;
    publish_bytes_at_barriers(
        path,
        &bytes,
        PublicationMode::CreateNew,
        |barrier| match barrier {
            DurableWriteBarrier::BeforeRename => before_publication(),
            DurableWriteBarrier::AfterDirectorySync => after_directory_sync(),
        },
    )
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
