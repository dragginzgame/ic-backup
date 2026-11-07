//! Explicit fresh local artifact checks, borrowing journal and layout exclusion.

use super::super::json::read_record;
use super::{DownloadJournalError, DownloadJournalGuard};
use crate::{
    model::{
        artifacts::ChecksumError,
        download_journal::{DownloadJournalRecord, MAX_DOWNLOAD_JOURNAL_BYTES},
        operation_plan::OperationPlanRecord,
    },
    ops::{
        artifacts::{ArtifactError, checksum_directory},
        persistence::{OperationPlanPersistenceError, PersistenceError, read_operation_plan},
    },
    policy::download_integrity::{DownloadIntegrityPolicyError, validate},
};
use thiserror::Error;

impl DownloadJournalGuard<'_> {
    // Manifest publication rechecks original records around its fresh tree hash.
    pub(super) fn verify_manifest_source(
        &self,
        plan: &OperationPlanRecord,
    ) -> Result<(), DownloadIntegrityError> {
        self.admit_integrity_original(plan)?;
        let expected = validate(plan, &self.record)?;
        self.check_artifact_parent()?;
        let path = self
            .layout
            .root()
            .join(self.record.artifacts()[0].artifact_path());
        checksum_directory(&path)?.verify(expected.hash())?;
        // Detect changed retained declarations or a replaced root during traversal.
        // This does not turn individual byte reads into an atomic whole-set snapshot.
        self.admit_integrity_original(plan)?;
        Ok(())
    }

    // Both generic and IC verification re-admit original records around byte IO.
    // Method-specific policy remains with each caller, in its original order.
    pub(super) fn admit_integrity_original(
        &self,
        plan: &OperationPlanRecord,
    ) -> Result<(), DownloadIntegrityError> {
        self.check_usable()?;
        read_operation_plan(self.layout, &plan.digest())?;
        self.require_unchanged_integrity_journal()
    }

    pub(super) fn require_unchanged_integrity_journal(&self) -> Result<(), DownloadIntegrityError> {
        let retained: DownloadJournalRecord =
            read_record(&self.path(), MAX_DOWNLOAD_JOURNAL_BYTES)?;
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
