//! Shared local sequence for immutable bounded declarations.

use super::json::check_json_size;
use super::{
    BackupLayoutGuard, JournalLock, JournalLockError, PersistenceError, create_json_durable,
    read_json,
};
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
    layout.check_root()?;
    let path = layout.root().join(name);
    let _lock = JournalLock::acquire(&path)?;
    check_json_size(record, limit)?;
    create_json_durable(&path, record)?;
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
    layout.check_root()?;
    let path = layout.root().join(name);
    let _lock = JournalLock::acquire(&path)?;
    let record: T = read_json(&path, limit)?;
    check_json_size(&record, limit)?;
    if !matches_original(&record) {
        return Err(mismatch());
    }
    Ok(record)
}
