//! Explicit fresh local artifact checks, borrowing journal and layout exclusion.

use super::{DownloadJournalError, DownloadJournalGuard, check_size};
use crate::{
    model::{
        artifacts::ChecksumError,
        download_journal::{DownloadJournalRecord, MAX_DOWNLOAD_JOURNAL_BYTES},
        operation_plan::OperationPlanRecord,
    },
    ops::{
        artifacts::{ArtifactError, checksum_directory},
        persistence::{
            OperationPlanPersistenceError, PersistenceError, read_json, read_operation_plan,
        },
    },
    policy::download_integrity::{DownloadIntegrityPolicyError, DurableDownloadView, validate},
};
use thiserror::Error;

impl DownloadJournalGuard<'_> {
    /// Explicitly reverify every published artifact under the retained original plan.
    ///
    /// Requires the exact persisted plan and unchanged held journal before and after
    /// no-follow checksumming. The returned view borrows journal/layout custody.
    /// This reads local bytes; ordinary journal reopen/resume remains effect-free
    /// and does not trigger verification. Nothing is written, pruned or released.
    ///
    /// File checks are sequential observations, not an atomic filesystem snapshot.
    /// Integrations retain stable byte custody and qualify complete backend transfer,
    /// authentic snapshot/receipt identity and terminal/reference-release evidence.
    ///
    /// # Errors
    /// Rejects unusable/replaced custody, missing/changed plans or journals, incomplete
    /// exact selected coverage, non-durable entries, unsafe/missing trees and changed bytes.
    pub fn verify_durable_artifacts<'a>(
        &'a self,
        plan: &'a OperationPlanRecord,
    ) -> Result<DurableDownloadView<'a>, DownloadIntegrityError> {
        self.check_usable()?;
        read_operation_plan(self.layout, &plan.digest())?;
        self.require_unchanged_integrity_journal()?;
        let view = validate(plan, &self.record)?;
        self.check_artifact_parent()?;
        for artifact in view.artifacts() {
            let path = self.layout.root().join(artifact.artifact().artifact_path());
            checksum_directory(&path)?.verify(artifact.checksum().hash())?;
        }
        // Detect changed retained declarations or a replaced root during traversal.
        // This does not turn individual byte reads into an atomic whole-set snapshot.
        self.check_usable()?;
        read_operation_plan(self.layout, &plan.digest())?;
        self.require_unchanged_integrity_journal()?;
        Ok(view)
    }

    fn require_unchanged_integrity_journal(&self) -> Result<(), DownloadIntegrityError> {
        let retained: DownloadJournalRecord = read_json(&self.path(), MAX_DOWNLOAD_JOURNAL_BYTES)?;
        check_size(&retained)?;
        if retained != self.record {
            return Err(DownloadIntegrityError::JournalChanged);
        }
        Ok(())
    }
}

/// Typed fresh local verification failure; original evidence remains retained.
#[derive(Debug, Error)]
pub enum DownloadIntegrityError {
    /// Retained journal differs from the exact declaration held by its guard.
    #[error("retained download journal changed during integrity verification")]
    JournalChanged,
    /// Guard/layout custody is unusable, replaced or unsafe.
    #[error(transparent)]
    Journal(#[from] DownloadJournalError),
    /// Retained original plan cannot be admitted under its exact expected digest.
    #[error(transparent)]
    Plan(#[from] OperationPlanPersistenceError),
    /// Original-plan selected-set or durable-checksum declaration mismatch.
    #[error(transparent)]
    Policy(#[from] DownloadIntegrityPolicyError),
    /// Bounded retained record admission failed.
    #[error(transparent)]
    Persistence(#[from] PersistenceError),
    /// No-follow directory traversal or streaming failed.
    #[error(transparent)]
    Artifact(#[from] ArtifactError),
    /// Current local bytes differ from the exact retained checksum.
    #[error(transparent)]
    Checksum(#[from] ChecksumError),
}

#[cfg(all(test, unix))]
mod tests;
