//! Immutable bounded inventory publication and exact local declared-identity admission.

use super::json::check_json_size;
use super::{
    BackupLayoutGuard, JournalLock, JournalLockError, PersistenceError, create_json_durable,
    read_json,
};
use crate::model::{
    artifacts::ArtifactChecksumRecord,
    inventory::{InventoryRecord, MAX_INVENTORY_BYTES},
};
use thiserror::Error;

/// Durably create `inventory.json` under held layout exclusion without replacing evidence.
///
/// # Errors
/// Rejects excessive canonical bytes, existing/unsafe entries, replaced roots and IO/lock failures.
pub fn create_inventory(
    layout: &BackupLayoutGuard,
    record: &InventoryRecord,
) -> Result<(), InventoryError> {
    layout.check_root()?;
    let path = layout.root().join("inventory.json");
    let _lock = JournalLock::acquire(&path)?;
    check_json_size(record, MAX_INVENTORY_BYTES)?;
    create_json_durable(&path, record)?;
    Ok(())
}

/// Read bounded validated local inventory under its original expected canonical digest.
///
/// This observes no remote membership or state. A failed/lost creation response
/// may be reconciled by reading the exact declared digest, never overwriting it.
///
/// # Errors
/// Rejects unsafe/missing/oversized records, invalid declarations, digest mismatch and locks.
pub fn read_inventory(
    layout: &BackupLayoutGuard,
    expected: &ArtifactChecksumRecord,
) -> Result<InventoryRecord, InventoryError> {
    layout.check_root()?;
    let path = layout.root().join("inventory.json");
    let _lock = JournalLock::acquire(&path)?;
    let record: InventoryRecord = read_json(&path, MAX_INVENTORY_BYTES)?;
    check_json_size(&record, MAX_INVENTORY_BYTES)?;
    if &record.digest() != expected {
        return Err(InventoryError::DigestMismatch);
    }
    Ok(record)
}

/// Typed exact declared-identity or durable local inventory admission failure.
#[derive(Debug, Error)]
pub enum InventoryError {
    /// Canonical retained declaration differs from its original selected digest.
    #[error("inventory digest mismatch")]
    DigestMismatch,
    /// Exclusive layout/journal ownership failed.
    #[error(transparent)]
    Lock(#[from] JournalLockError),
    /// Bounded JSON, model admission or durable filesystem access failed.
    #[error(transparent)]
    Persistence(#[from] PersistenceError),
}

#[cfg(all(test, unix))]
mod tests;
