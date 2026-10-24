//! Durable publication, journal/layout exclusion and retained local dependencies.

mod artifact_commit;
mod attempt_journal;
mod command_lifetime_lock;
mod consistency;
mod download_journal;
mod execution_progress;
mod execution_settlement;
mod execution_workflow;
mod fence_obligation;
mod immutable_record;
mod journal_lock;
mod journal_path;
pub(crate) mod json;
mod layout_lifetime;
mod operation_plan;
mod restore_reference_release;
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
    LocalRestoreSourceError, read_download_manifest,
};
#[cfg(unix)]
pub use download_journal::{
    IcSnapshotArtifactError, IcSnapshotArtifactWriter, IcSnapshotDataUploadPreparationError,
    IcSnapshotUploadArtifactError,
};
pub use execution_progress::{ExecutionProgressPersistenceError, read_execution_progress};
pub use execution_settlement::{
    ExecutionSettlementCheckpointError, ExecutionSettlementPersistenceError,
    read_execution_settlement, reconcile_execution_settlement, replay_execution_settlement,
};
pub(crate) use execution_workflow::read_execution_stage_occupancy;
pub use execution_workflow::{
    ExecutionStageCheckpointError, ExecutionStageGuard, ExecutionStagePreparationError,
    ExecutionStageResumeError, ExecutionStageSettlementView, ExecutionWorkflowPersistenceError,
    ExecutionWorkflowSettlementView, create_execution_workflow, read_complete_execution_workflow,
    read_complete_execution_workflow_locked, read_execution_workflow,
};
pub use fence_obligation::{
    FenceObligationPersistenceError, FenceObligationRequirement, create_fence_obligation,
    read_fence_obligation,
};
pub use journal_lock::{JournalLock, JournalLockError};
pub(crate) use json::read_json;
pub use layout_lifetime::{BackupLayoutGuard, MAX_RESTORE_REFERENCE_BYTES};
pub use operation_plan::{
    OperationPlanPersistenceError, create_operation_plan, read_operation_plan,
};
pub use restore_reference_release::{
    MAX_RESTORE_TERMINAL_BYTES, RestoreReferenceReleaseError, RestoreReferenceReleaseOutcome,
    RestoreTerminalEvidenceProvider,
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
    /// Parent preparation, record reads or other local filesystem IO failed.
    /// Shared JSON publication failures retain their separate structured boundary.
    #[error(transparent)]
    Io(#[from] io::Error),
    /// File publication failed, retaining producer/cleanup causes and visibility.
    /// An after-publication failure leaves output visible and requires reconciliation.
    #[error(transparent)]
    Publication(#[from] ic_host_fs::durable::NamedWriteError<io::Error>),
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
