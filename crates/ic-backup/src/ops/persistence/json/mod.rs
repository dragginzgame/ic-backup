//! Module: `persistence::json`
//!
//! Responsibility: read and durably create or replace JSON persistence documents.
//! Does not own: document validation, layout paths, or integrity checks.
//! Boundary: provides explicit create-only and replace filesystem primitives.

use crate::ops::persistence::PersistenceError;

use std::{
    fs::File,
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
    encode_into(value, io::sink(), max_bytes).map(|_| ())
}

/// Encode and admit the exact publication bytes before any filesystem effect.
pub(crate) fn encode_json(
    value: &impl Serialize,
    max_bytes: u64,
) -> Result<Vec<u8>, PersistenceError> {
    encode_into(value, Vec::new(), max_bytes)
}

// Candidates are encoded before entry. A failed or lost publication must leave
// either journal guard unusable, without installing its unconfirmed cached record.
pub(super) fn replace_cached_record<T>(
    record: &mut T,
    usable: &mut bool,
    next: T,
    publish: impl FnOnce() -> Result<(), PersistenceError>,
) -> Result<(), PersistenceError> {
    *usable = false;
    publish()?;
    *record = next;
    *usable = true;
    Ok(())
}

fn encode_into<W: Write>(
    value: &impl Serialize,
    sink: W,
    max_bytes: u64,
) -> Result<W, PersistenceError> {
    let mut writer = ic_host_artifacts::artifact::BoundedWriter::new(sink, max_bytes);
    let result = serde_json::to_writer_pretty(&mut writer, value);
    if writer.limit_exceeded() {
        return Err(PersistenceError::RecordTooLarge { limit: max_bytes });
    }
    result?;
    Ok(writer.into_inner())
}

pub(crate) fn write_encoded(path: &Path, bytes: &[u8]) -> Result<(), PersistenceError> {
    publish_bytes_at_barriers(path, bytes, PublicationMode::Replace, |_| {})
}

pub(crate) fn create_encoded(path: &Path, bytes: &[u8]) -> Result<(), PersistenceError> {
    publish_bytes_at_barriers(path, bytes, PublicationMode::CreateNew, |_| {})
}

/// Read one regular no-follow JSON file within an explicit byte limit.
///
/// Reuses the shared bounded regular-file reader; caller-selected parents and
/// concurrent byte custody remain local. Only the final symlink is rejected.
///
/// # Errors
/// Rejects unsafe files, excessive bytes, invalid JSON and filesystem failures.
pub(crate) fn read_json<T>(path: &Path, max_bytes: u64) -> Result<T, PersistenceError>
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

// Existing record owners require both bounded input and bounded canonical output.
// Keep model decoding before size admission and leave locking/identity to callers.
pub(super) fn read_record<T>(path: &Path, max_bytes: u64) -> Result<T, PersistenceError>
where
    T: DeserializeOwned + Serialize,
{
    let record = read_json(path, max_bytes)?;
    check_json_size(&record, max_bytes)?;
    Ok(record)
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
pub(super) fn publish_bytes_at_barriers(
    path: &Path,
    bytes: &[u8],
    mode: PublicationMode,
    mut barrier: impl FnMut(DurableWriteBarrier),
) -> Result<(), PersistenceError> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    ic_host_fs::durable::prepare_private_parents(parent)?;
    let options = WriteOptions {
        mode,
        permissions: 0o600,
    };
    let produce = |file: &mut File| file.write_all(bytes);
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
        ic_host_fs::durable::write_at_validated_with(
            directory.as_fd(),
            name,
            options,
            produce,
            |()| {
                barrier(DurableWriteBarrier::BeforeRename);
                Ok(())
            },
        )
    };
    #[cfg(not(unix))]
    let result = ic_host_fs::durable::write_validated_with(path, options, produce, |_, ()| {
        barrier(DurableWriteBarrier::BeforeRename);
        Ok(())
    });
    result?;
    // Successful Host completion includes publication and the held-parent sync.
    barrier(DurableWriteBarrier::AfterDirectorySync);
    Ok(())
}

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod regressions;
#[cfg(test)]
mod tests;
