//! Immutable bounded graph publication and exact original local graph admission.

use super::json::check_json_size;
use super::{
    BackupLayoutGuard, JournalLock, JournalLockError, PersistenceError, create_json_durable,
    read_json,
};
use crate::model::{
    artifacts::ArtifactChecksumRecord,
    effect_graph::{EffectGraphRecord, MAX_EFFECT_GRAPH_BYTES},
};
use thiserror::Error;

/// Durably create `effect-graph.json` under layout exclusion without replacing evidence.
///
/// # Errors
/// Rejects excessive canonical bytes, existing/unsafe entries, replaced roots and IO/locks.
pub fn create_effect_graph(
    layout: &BackupLayoutGuard,
    record: &EffectGraphRecord,
) -> Result<(), EffectGraphPersistenceError> {
    layout.check_root()?;
    let path = layout.root().join("effect-graph.json");
    let _lock = JournalLock::acquire(&path)?;
    check_json_size(record, MAX_EFFECT_GRAPH_BYTES)?;
    create_json_durable(&path, record)?;
    Ok(())
}

/// Admit retained bounded graph under its original exact expected digest, using local IO only.
///
/// Lost creation replies reconcile by reading the exact graph. This neither checks
/// remote completion nor grants effects, fresh authority or plan acceptance.
///
/// # Errors
/// Rejects unsafe/missing/oversized/invalid graphs, digest mismatch, replaced roots and locks.
pub fn read_effect_graph(
    layout: &BackupLayoutGuard,
    expected: &ArtifactChecksumRecord,
) -> Result<EffectGraphRecord, EffectGraphPersistenceError> {
    layout.check_root()?;
    let path = layout.root().join("effect-graph.json");
    let _lock = JournalLock::acquire(&path)?;
    let record: EffectGraphRecord = read_json(&path, MAX_EFFECT_GRAPH_BYTES)?;
    check_json_size(&record, MAX_EFFECT_GRAPH_BYTES)?;
    if &record.digest() != expected {
        return Err(EffectGraphPersistenceError::DigestMismatch);
    }
    Ok(record)
}

/// Typed declared graph identity or immutable local persistence rejection.
#[derive(Debug, Error)]
pub enum EffectGraphPersistenceError {
    /// Retained graph differs from the original exact declared digest.
    #[error("effect graph digest mismatch")]
    DigestMismatch,
    /// Cooperating layout/journal ownership failed.
    #[error(transparent)]
    Lock(#[from] JournalLockError),
    /// Bounded JSON, graph admission or durable filesystem access failed.
    #[error(transparent)]
    Persistence(#[from] PersistenceError),
}

#[cfg(all(test, unix))]
mod tests;
