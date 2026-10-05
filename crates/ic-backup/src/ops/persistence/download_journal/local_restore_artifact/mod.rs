//! Private original-operation artifact copies and explicit retained-copy verification.

use super::{DownloadJournalGuard, LocalRestoreSourceError};
use crate::{
    model::{
        artifacts::ChecksumError,
        download_journal::DownloadArtifactRecord,
        operation_plan::{OperationPlanError, OperationPlanRecord, PlannedOperationRecord},
        restore_safety::RestoreSafetyRequirementRecord,
    },
    ops::{
        artifacts::{ArtifactError, checksum_directory, stage_relative_path},
        persistence::{BackupLayoutGuard, JournalLock, JournalLockError},
    },
    policy::local_restore_source::LocalRestoreSourceView,
};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Borrowed original source/operation identity and the exact freshly checked private copy.
///
/// This view retains both layout/source-journal lifetimes. It grants no future path
/// stability, durable publication, complete backend transfer, authenticated snapshot,
/// command dispatch or upload/load permission. Dropping it deletes nothing.
#[derive(Debug)]
pub struct LocalRestoreArtifactView<'a> {
    source: LocalRestoreSourceView<'a>,
    operation: &'a PlannedOperationRecord,
    artifact: &'a DownloadArtifactRecord,
    path: PathBuf,
}

impl<'a> LocalRestoreArtifactView<'a> {
    /// Read exact original source, restore and safety declarations.
    #[must_use]
    pub const fn source(&self) -> &LocalRestoreSourceView<'a> {
        &self.source
    }
    /// Read original opaque operation sequence, target/request and attempt allowances.
    #[must_use]
    pub const fn operation(&self) -> &'a PlannedOperationRecord {
        self.operation
    }
    /// Read original exact snapshot metadata, canonical source path and retained checksum.
    #[must_use]
    pub const fn artifact(&self) -> &'a DownloadArtifactRecord {
        self.artifact
    }
    /// Read the private copy location; integrations maintain byte custody before actual use.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl DownloadJournalGuard<'_> {
    /// Create an exact private artifact copy for one original selected restore operation.
    ///
    /// Complete original source verification precedes descriptor-based no-follow
    /// copying to fixed `restore-artifact-{sequence}.tmp` directly under the held
    /// restore layout. Existing destinations are never adopted, replaced or deleted.
    /// Copy hash and fresh destination hash must equal the original artifact checksum;
    /// retained declarations are re-admitted before returning. Directories/files are
    /// private 0700/0600. This is staging, without fsync/durable publication or dispatch.
    /// Failures/drop retain partial bytes and all original spending/references. After
    /// a lost reply, explicitly verify the retained copy; an invalid partial copy needs
    /// operator-owned disposition. Stable noncooperating destination custody remains
    /// integration-owned. The operation sequence associates bytes, not effect authority.
    /// # Errors
    /// Rejects unknown original operations, changed/unsafe source or copy, contention,
    /// existing destinations, lost IO replies and original record/custody mismatch.
    pub fn stage_local_restore_artifact<'a>(
        &'a self,
        restore_layout: &'a BackupLayoutGuard,
        restore: &'a OperationPlanRecord,
        source: &'a OperationPlanRecord,
        requirement: &'a RestoreSafetyRequirementRecord,
        operation_sequence: u64,
    ) -> Result<LocalRestoreArtifactView<'a>, LocalRestoreArtifactError> {
        self.stage_restore_artifact_with(
            restore_layout,
            restore,
            source,
            requirement,
            operation_sequence,
            stage_relative_path,
        )
    }

    fn stage_restore_artifact_with<'a>(
        &'a self,
        restore_layout: &'a BackupLayoutGuard,
        restore: &'a OperationPlanRecord,
        source: &'a OperationPlanRecord,
        requirement: &'a RestoreSafetyRequirementRecord,
        operation_sequence: u64,
        copy: impl FnOnce(
            &Path,
            &Path,
            &Path,
        )
            -> Result<crate::model::artifacts::ArtifactChecksumRecord, ArtifactError>,
    ) -> Result<LocalRestoreArtifactView<'a>, LocalRestoreArtifactError> {
        let operation = restore.operation(operation_sequence)?;
        let view =
            self.verify_local_restore_source(restore_layout, restore, source, requirement)?;
        let artifact = selected_artifact(&view, operation)?;
        let path = staged_path(restore_layout, operation_sequence);
        let _lock = JournalLock::acquire(&path)?;
        copy(
            self.layout.root(),
            Path::new(artifact.artifact_path()),
            &path,
        )?
        .verify(
            artifact
                .checksum()
                .ok_or(LocalRestoreArtifactError::ArtifactUnavailable)?
                .hash(),
        )?;
        self.verify_restore_artifact_at(
            restore_layout,
            restore,
            source,
            requirement,
            operation,
            path,
        )
    }

    /// Explicitly check a retained private copy against exact original declarations.
    ///
    /// Reads retained original plans/requirement/manifest/journal and the copy's bytes,
    /// without re-reading source trees or repeating a copy. Original source trees may
    /// be absent; exact retained metadata remains required. Missing/unsafe/incomplete
    /// or conflicting copies are retained, never repaired or recreated. This is fresh
    /// local copy verification, not ordinary resume/terminal replay or effect authority.
    /// # Errors
    /// Rejects original identity/custody mismatch, unknown operations, unsafe/missing
    /// copies, checksum drift and contention without altering recovery evidence.
    pub fn verify_staged_local_restore_artifact<'a>(
        &'a self,
        restore_layout: &'a BackupLayoutGuard,
        restore: &'a OperationPlanRecord,
        source: &'a OperationPlanRecord,
        requirement: &'a RestoreSafetyRequirementRecord,
        operation_sequence: u64,
    ) -> Result<LocalRestoreArtifactView<'a>, LocalRestoreArtifactError> {
        let operation = restore.operation(operation_sequence)?;
        let path = staged_path(restore_layout, operation_sequence);
        let _lock = JournalLock::acquire(&path)?;
        self.verify_restore_artifact_at(
            restore_layout,
            restore,
            source,
            requirement,
            operation,
            path,
        )
    }

    fn verify_restore_artifact_at<'a>(
        &'a self,
        restore_layout: &'a BackupLayoutGuard,
        restore: &'a OperationPlanRecord,
        source: &'a OperationPlanRecord,
        requirement: &'a RestoreSafetyRequirementRecord,
        operation: &'a PlannedOperationRecord,
        path: PathBuf,
    ) -> Result<LocalRestoreArtifactView<'a>, LocalRestoreArtifactError> {
        let view = self.admit_local_restore_source(restore_layout, restore, source, requirement)?;
        let artifact = selected_artifact(&view, operation)?;
        checksum_directory(&path)?.verify(
            artifact
                .checksum()
                .ok_or(LocalRestoreArtifactError::ArtifactUnavailable)?
                .hash(),
        )?;
        let source =
            self.admit_local_restore_source(restore_layout, restore, source, requirement)?;
        Ok(LocalRestoreArtifactView {
            source,
            operation,
            artifact,
            path,
        })
    }
}

fn staged_path(layout: &BackupLayoutGuard, sequence: u64) -> PathBuf {
    layout
        .root()
        .join(format!("restore-artifact-{sequence}.tmp"))
}
fn selected_artifact<'a>(
    view: &LocalRestoreSourceView<'a>,
    operation: &PlannedOperationRecord,
) -> Result<&'a DownloadArtifactRecord, LocalRestoreArtifactError> {
    view.selected_artifacts()
        .find(|artifact| artifact.artifact().canister_id() == operation.target())
        .map(crate::policy::download_integrity::DurableDownloadArtifactView::artifact)
        .ok_or(LocalRestoreArtifactError::ArtifactUnavailable)
}

/// Typed local copy/source admission denial; no error deletes or dispatches anything.
#[derive(Debug, Error)]
pub enum LocalRestoreArtifactError {
    /// Original local source, safety requirement or retained custody failed admission.
    #[error(transparent)]
    Source(#[from] LocalRestoreSourceError),
    /// Supplied opaque operation is absent from the exact original restore plan.
    #[error(transparent)]
    Operation(#[from] OperationPlanError),
    /// Original selected artifact/checksum could not be projected.
    #[error("original selected restore artifact unavailable")]
    ArtifactUnavailable,
    /// Descriptor copying or no-follow local traversal failed; partial bytes remain.
    #[error(transparent)]
    Artifact(#[from] ArtifactError),
    /// Actual copied or retained bytes differ from the original checksum.
    #[error(transparent)]
    Checksum(#[from] ChecksumError),
    /// Original-operation staging exclusion failed.
    #[error(transparent)]
    Lock(#[from] JournalLockError),
}

#[cfg(all(test, unix))]
mod tests;
