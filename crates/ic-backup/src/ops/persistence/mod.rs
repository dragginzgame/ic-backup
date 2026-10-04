//! Durable local publication and journal locking; no domain transitions.

mod artifact_commit;
mod file_lock;
mod journal_lock;
mod json;

pub use artifact_commit::{ArtifactCommitOutcome, commit_artifact_directory};
pub use journal_lock::{JournalLock, JournalLockError};
pub use json::{create_json_durable, read_json, write_json_durable};

use crate::{model::artifacts::ChecksumError, ops::artifacts::ArtifactError};
use std::io;
use thiserror::Error;

/// Typed local persistence failure.
#[derive(Debug, Error)]
pub enum PersistenceError {
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
