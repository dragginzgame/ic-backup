//! Shared local sequence for immutable bounded declarations.

use super::json::{check_json_size, create_encoded, encode_json};
use super::{BackupLayoutGuard, JournalLock, JournalLockError, PersistenceError, read_json};
use serde::{Serialize, de::DeserializeOwned};

pub(super) fn create<T, E>(
    layout: &BackupLayoutGuard,
    name: &str,
    limit: u64,
    record: &T,
) -> Result<(), E>
where
    T: Serialize,
    E: From<JournalLockError> + From<PersistenceError>,
{
    create_with(layout, name, limit, record, || Ok(()), create_encoded)
}

// Validation runs after encoding, under the same lock that protects publication.
pub(super) fn create_with<T, E>(
    layout: &BackupLayoutGuard,
    name: &str,
    limit: u64,
    record: &T,
    validate: impl FnOnce() -> Result<(), E>,
    write: impl FnOnce(&std::path::Path, &[u8]) -> Result<(), PersistenceError>,
) -> Result<(), E>
where
    T: Serialize,
    E: From<JournalLockError> + From<PersistenceError>,
{
    layout.check_root()?;
    let path = layout.root().join(name);
    let _lock = JournalLock::acquire(&path)?;
    let bytes = encode_json(record, limit)?;
    validate()?;
    write(&path, &bytes)?;
    Ok(())
}

/// Publish bytes already encoded and admitted before a caller's path allocation.
pub(super) fn create_encoded_record<E>(
    layout: &BackupLayoutGuard,
    name: &str,
    bytes: &[u8],
) -> Result<(), E>
where
    E: From<JournalLockError> + From<PersistenceError>,
{
    layout.check_root()?;
    let path = layout.root().join(name);
    let _lock = JournalLock::acquire(&path)?;
    create_encoded(&path, bytes)?;
    Ok(())
}

pub(super) fn read<T, E>(
    layout: &BackupLayoutGuard,
    name: &str,
    limit: u64,
    matches_original: impl FnOnce(&T) -> bool,
    mismatch: impl FnOnce() -> E,
) -> Result<T, E>
where
    T: DeserializeOwned + Serialize,
    E: From<JournalLockError> + From<PersistenceError>,
{
    read_with(layout, name, limit, |record| {
        if matches_original(record) {
            Ok(())
        } else {
            Err(mismatch())
        }
    })
}

pub(super) fn read_with<T, E>(
    layout: &BackupLayoutGuard,
    name: &str,
    limit: u64,
    validate: impl FnOnce(&T) -> Result<(), E>,
) -> Result<T, E>
where
    T: DeserializeOwned + Serialize,
    E: From<JournalLockError> + From<PersistenceError>,
{
    layout.check_root()?;
    let path = layout.root().join(name);
    let _lock = JournalLock::acquire(&path)?;
    let record: T = read_json(&path, limit)?;
    check_json_size(&record, limit)?;
    validate(&record)?;
    Ok(record)
}
