//! Durable publication, journal/layout exclusion and retained local dependencies.

mod artifact_commit;
mod attempt_journal;
mod command_lifetime_lock;
mod consistency;
mod download_journal;
mod effect_graph;
mod execution_settlement;
mod fence_obligation;
mod file_lock;
mod inventory;
mod journal_lock;
mod json;
mod layout_lifetime;
mod operation_plan;
mod restore_safety;

pub use artifact_commit::{ArtifactCommitOutcome, commit_artifact_directory};
pub use attempt_journal::{AttemptJournalError, AttemptJournalGuard};
pub use command_lifetime_lock::{
    COMMAND_CUSTODY_DESCRIPTOR_ENV, CommandLifetimeLock, CommandLifetimeLockError,
    CommandQuiescenceGuard,
};
pub use consistency::{
    ConsistencyPersistenceError, create_consistency_requirement, read_consistency_requirement,
};
pub use download_journal::{
    DownloadIntegrityError, DownloadJournalError, DownloadJournalGuard, DownloadManifestError,
    LocalRestoreArtifactError, LocalRestoreArtifactPublicationError, LocalRestoreArtifactView,
    LocalRestoreSourceError, read_download_manifest,
};
#[cfg(unix)]
pub use download_journal::{IcSnapshotArtifactError, IcSnapshotArtifactWriter};
pub use effect_graph::{EffectGraphPersistenceError, create_effect_graph, read_effect_graph};
pub use execution_settlement::{
    ExecutionSettlementPersistenceError, create_execution_settlement, read_execution_settlement,
};
pub use fence_obligation::{
    FenceObligationPersistenceError, FenceObligationRequirement, create_fence_obligation,
    read_fence_obligation,
};
pub use inventory::{InventoryError, create_inventory, read_inventory};
pub use journal_lock::{JournalLock, JournalLockError};
pub use json::{create_json_durable, read_json, write_json_durable};
pub use layout_lifetime::{BackupLayoutGuard, MAX_RESTORE_REFERENCE_BYTES};
pub use operation_plan::{
    OperationPlanPersistenceError, create_operation_plan, read_operation_plan,
};
pub use restore_safety::{
    RestoreSafetyPersistenceError, create_restore_safety_requirement,
    read_restore_safety_requirement,
};

use crate::{model::artifacts::ChecksumError, ops::artifacts::ArtifactError};
use std::io;
use thiserror::Error;

/// Typed local persistence failure.
#[derive(Debug, Error)]
pub enum PersistenceError {
    /// An existing layout path no longer denotes the held directory.
    #[error("backup layout changed while held: {path:?}")]
    LayoutChanged {
        /// Resolved layout location.
        path: std::path::PathBuf,
    },
    /// A retained dependency document or journal location has an unsafe entry type.
    #[error("unsafe restore reference entry: {path:?}")]
    InvalidRestoreReferences {
        /// Rejected local path.
        path: std::path::PathBuf,
    },
    /// Restore dependency validation or an immutable retention transition failed.
    #[error(transparent)]
    RestoreReference(#[from] crate::model::restore_references::RestoreReferenceError),
    /// Filesystem IO failed; publication may require local reconciliation.
    #[error(transparent)]
    Io(#[from] io::Error),
    /// JSON encoding or decoding failed.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// A machine record exceeds its caller-selected byte bound.
    #[error("record exceeds byte limit {limit}")]
    RecordTooLarge {
        /// Maximum accepted bytes.
        limit: u64,
    },
    /// Artifact traversal or streaming failed.
    #[error(transparent)]
    Artifact(#[from] ArtifactError),
    /// Verified bytes do not match the expected checksum.
    #[error(transparent)]
    Checksum(#[from] ChecksumError),
    /// Publication requires distinct siblings in one directory.
    #[error("artifact commit paths must be distinct siblings: {temporary}, {canonical}")]
    ArtifactCommitPathMismatch {
        /// Retained staging path.
        temporary: String,
        /// Canonical destination.
        canonical: String,
    },
    /// Both staging and canonical trees exist; neither is overwritten.
    #[error("artifact commit has conflicting paths: {temporary}, {canonical}")]
    ArtifactCommitPathConflict {
        /// Retained staging path.
        temporary: String,
        /// Existing canonical destination.
        canonical: String,
    },
    /// Neither expected tree exists.
    #[error("artifact commit is missing both paths: {temporary}, {canonical}")]
    ArtifactCommitPathMissing {
        /// Missing staging path.
        temporary: String,
        /// Missing canonical destination.
        canonical: String,
    },
    /// Atomic no-replace directory publication is unsupported.
    #[error("durable artifact publication is unsupported on {platform}")]
    ArtifactCommitUnsupportedPlatform {
        /// Host platform name.
        platform: &'static str,
    },
    /// Publication found an unsafe tree entry.
    #[error("unsupported artifact entry at {path}: {kind}")]
    UnsupportedArtifactEntry {
        /// Entry path.
        path: String,
        /// Observed entry kind.
        kind: String,
    },
}
