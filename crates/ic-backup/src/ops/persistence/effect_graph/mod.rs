//! Immutable bounded graph publication and exact original local graph admission.

use super::{BackupLayoutGuard, JournalLockError, PersistenceError, immutable_record};
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
    immutable_record::create(layout, "effect-graph.json", MAX_EFFECT_GRAPH_BYTES, record)
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
    immutable_record::read(
        layout,
        "effect-graph.json",
        MAX_EFFECT_GRAPH_BYTES,
        |record: &EffectGraphRecord| &record.digest() == expected,
        || EffectGraphPersistenceError::DigestMismatch,
    )
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
