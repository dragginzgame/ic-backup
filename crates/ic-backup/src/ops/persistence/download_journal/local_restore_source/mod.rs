//! Explicit fresh original local source verification under both layout guards.

use super::{DownloadIntegrityError, DownloadJournalGuard, DownloadManifestError};
use crate::{
    model::{operation_plan::OperationPlanRecord, restore_safety::RestoreSafetyRequirementRecord},
    ops::persistence::{
        BackupLayoutGuard, RestoreSafetyPersistenceError, read_restore_safety_requirement,
    },
    policy::local_restore_source::{
        LocalRestoreSourcePolicyError, LocalRestoreSourceView, validate,
    },
};
use thiserror::Error;

impl DownloadJournalGuard<'_> {
    /// Freshly verify the complete original local source and project exact restore artifacts.
    ///
    /// The source journal borrows its source layout; the returned view also borrows
    /// the restore layout, both plans and original safety requirement. Exact retained
    /// requirement/plans, manifest and journal are admitted before and after fresh
    /// no-follow verification of every original source artifact, including any outside
    /// a restore subset. Replay/ordinary resume never invoke this separate operation.
    /// No records, allowances, references or fences change; no provider is invoked.
    /// Integrations own stable noncooperating bytes, authenticated snapshot/transfer
    /// completeness and application subset safety. Success grants no upload/load,
    /// current permissions, signing, fence/reference release or terminal authority.
    /// # Errors
    /// Rejects absent/unsafe/changed originals, another source digest, non-durable or
    /// incomplete selected sets, changed bytes, replaced custody and contention.
    pub fn verify_local_restore_source<'a>(
        &'a self,
        restore_layout: &'a BackupLayoutGuard,
        restore: &'a OperationPlanRecord,
        source: &'a OperationPlanRecord,
        requirement: &'a RestoreSafetyRequirementRecord,
    ) -> Result<LocalRestoreSourceView<'a>, LocalRestoreSourceError> {
        self.admit_local_restore_source(restore_layout, restore, source, requirement)?;
        self.verify_durable_artifacts(source)?;
        self.admit_local_restore_source(restore_layout, restore, source, requirement)
    }

    pub(super) fn admit_local_restore_source<'a>(
        &'a self,
        restore_layout: &'a BackupLayoutGuard,
        restore: &'a OperationPlanRecord,
        source: &'a OperationPlanRecord,
        requirement: &'a RestoreSafetyRequirementRecord,
    ) -> Result<LocalRestoreSourceView<'a>, LocalRestoreSourceError> {
        read_restore_safety_requirement(
            restore_layout,
            self.layout,
            restore,
            source,
            &requirement.digest(),
        )?;
        self.read_download_manifest(source, requirement.source_artifacts())?;
        restore_layout
            .check_root()
            .map_err(DownloadIntegrityError::from)?;
        Ok(validate(
            restore,
            source,
            requirement,
            self.record().map_err(DownloadIntegrityError::from)?,
        )?)
    }
}

/// Typed fresh original local source denial, preserving every original obligation.
#[derive(Debug, Error)]
pub enum LocalRestoreSourceError {
    /// Original requirement and both retained plans cannot be admitted.
    #[error(transparent)]
    Requirement(#[from] RestoreSafetyPersistenceError),
    /// Exact original immutable manifest or guarded journal differs.
    #[error(transparent)]
    Manifest(#[from] DownloadManifestError),
    /// Pure same-ID local source admission failed.
    #[error(transparent)]
    Policy(#[from] LocalRestoreSourcePolicyError),
    /// Fresh guarded local byte/custody verification failed.
    #[error(transparent)]
    Integrity(#[from] DownloadIntegrityError),
}

#[cfg(all(test, unix))]
mod tests;
