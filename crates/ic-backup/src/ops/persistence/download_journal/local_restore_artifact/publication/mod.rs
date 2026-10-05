//! Durable original-copy publication reusing the canonical artifact commit owner.

use super::{LocalRestoreArtifactError, LocalRestoreArtifactView, selected_artifact, staged_path};
use crate::{
    model::{operation_plan::OperationPlanRecord, restore_safety::RestoreSafetyRequirementRecord},
    ops::persistence::{
        ArtifactCommitOutcome, BackupLayoutGuard, DownloadJournalGuard, JournalLock,
        PersistenceError, commit_artifact_directory,
    },
};
use std::path::{Path, PathBuf};
use thiserror::Error;

impl DownloadJournalGuard<'_> {
    /// Durably publish an exact staged restore artifact, or recover its canonical copy.
    ///
    /// Re-admit the exact retained original plans/requirement/manifest/source journal,
    /// then reuse no-follow synchronization, checksum verification and atomic no-replace
    /// publication from `restore-artifact-{sequence}.tmp` to `restore-artifact-{sequence}`.
    /// The operation lock is shared with staging and verification. Both layouts and the
    /// source journal remain borrowed; original records are re-admitted after publication.
    ///
    /// Returns the freshly checked canonical view and explicit Published/Recovered outcome.
    /// Recovery verifies and synchronizes the existing canonical tree, without copying or
    /// reading source artifact trees. Both paths present, neither present, changed/unsafe
    /// bytes or original metadata mismatch reject without replacement, repair or cleanup.
    /// Failure may leave publication complete; recover the exact paths before other work.
    /// No journal, attempt, fence, obligation or source reference changes.
    ///
    /// Stable noncooperating parent/byte custody remains integration-owned. This local
    /// publication grants no complete backend transfer, authentic snapshot, signing,
    /// upload/load/start, application safety, terminal or fence/reference-release authority.
    /// Ordinary resume and terminal replay never invoke this explicit fresh operation.
    /// # Errors
    /// Rejects unknown operation, original custody/identity drift, contention, missing or
    /// conflicting paths, unsafe/changed bytes, synchronization failure or lost IO replies.
    pub fn publish_staged_local_restore_artifact<'a>(
        &'a self,
        restore_layout: &'a BackupLayoutGuard,
        restore: &'a OperationPlanRecord,
        source: &'a OperationPlanRecord,
        requirement: &'a RestoreSafetyRequirementRecord,
        operation_sequence: u64,
    ) -> Result<
        (LocalRestoreArtifactView<'a>, ArtifactCommitOutcome),
        LocalRestoreArtifactPublicationError,
    > {
        self.publish_restore_artifact_with(
            restore_layout,
            restore,
            source,
            requirement,
            operation_sequence,
            commit_artifact_directory,
        )
    }

    fn publish_restore_artifact_with<'a>(
        &'a self,
        restore_layout: &'a BackupLayoutGuard,
        restore: &'a OperationPlanRecord,
        source: &'a OperationPlanRecord,
        requirement: &'a RestoreSafetyRequirementRecord,
        operation_sequence: u64,
        publish: impl FnOnce(&Path, &Path, &str) -> Result<ArtifactCommitOutcome, PersistenceError>,
    ) -> Result<
        (LocalRestoreArtifactView<'a>, ArtifactCommitOutcome),
        LocalRestoreArtifactPublicationError,
    > {
        let operation = restore
            .operation(operation_sequence)
            .map_err(LocalRestoreArtifactError::from)?;
        // Admit held directory identities before creating an operation-lock sidecar.
        let view = self
            .admit_local_restore_source(restore_layout, restore, source, requirement)
            .map_err(LocalRestoreArtifactError::from)?;
        let temporary = staged_path(restore_layout, operation_sequence);
        let canonical = published_path(restore_layout, operation_sequence);
        let _lock = JournalLock::acquire(&temporary).map_err(LocalRestoreArtifactError::from)?;
        let artifact = selected_artifact(&view, operation)?;
        let expected = artifact
            .checksum()
            .ok_or(LocalRestoreArtifactError::ArtifactUnavailable)?;
        let outcome = publish(&temporary, &canonical, expected.hash())?;
        let view = self.verify_restore_artifact_at(
            restore_layout,
            restore,
            source,
            requirement,
            operation,
            canonical,
        )?;
        Ok((view, outcome))
    }

    /// Freshly verify an exact retained canonical restore copy without republishing it.
    ///
    /// Checks retained original metadata and canonical bytes under the same operation
    /// lock as staging/publication, without source-tree reads, copying or fsync. A path
    /// alone proves no earlier durable publication; this view is fresh local integrity.
    /// Ordinary resume/terminal replay performs no such verification. Failures retain
    /// all bytes, original spending, obligations and references without repair/cleanup.
    /// # Errors
    /// Rejects missing/unsafe/changed canonical copies, original drift or contention.
    pub fn verify_published_local_restore_artifact<'a>(
        &'a self,
        restore_layout: &'a BackupLayoutGuard,
        restore: &'a OperationPlanRecord,
        source: &'a OperationPlanRecord,
        requirement: &'a RestoreSafetyRequirementRecord,
        operation_sequence: u64,
    ) -> Result<LocalRestoreArtifactView<'a>, LocalRestoreArtifactError> {
        let operation = restore.operation(operation_sequence)?;
        self.admit_local_restore_source(restore_layout, restore, source, requirement)?;
        let _lock = JournalLock::acquire(&staged_path(restore_layout, operation_sequence))?;
        self.verify_restore_artifact_at(
            restore_layout,
            restore,
            source,
            requirement,
            operation,
            published_path(restore_layout, operation_sequence),
        )
    }
}

fn published_path(layout: &BackupLayoutGuard, sequence: u64) -> PathBuf {
    layout.root().join(format!("restore-artifact-{sequence}"))
}

/// Publication/recovery denial; a failure can retain an already published canonical copy.
#[derive(Debug, Error)]
pub enum LocalRestoreArtifactPublicationError {
    /// Original source/copy admission, operation identity or lock failed.
    #[error(transparent)]
    Admission(#[from] LocalRestoreArtifactError),
    /// Canonical artifact synchronization/publication failed; retain both exact paths.
    #[error(transparent)]
    Publication(#[from] PersistenceError),
}

#[cfg(all(
    test,
    any(target_os = "linux", target_os = "android", target_vendor = "apple")
))]
mod tests;
