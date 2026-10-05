//! Immutable local download declarations using the existing v1 journal schema.

use super::{DownloadIntegrityError, DownloadJournalError, DownloadJournalGuard, check_size};
use crate::{
    model::{
        artifacts::ArtifactChecksumRecord,
        download_journal::{DownloadJournalRecord, MAX_DOWNLOAD_JOURNAL_BYTES},
        operation_plan::OperationPlanRecord,
    },
    ops::persistence::{
        BackupLayoutGuard, JournalLock, JournalLockError, PersistenceError, create_json_durable,
        read_json, read_operation_plan,
    },
    policy::download_integrity::validate,
};
use std::path::Path;
use thiserror::Error;

const MANIFEST_FILE: &str = "download-manifest.json";

impl DownloadJournalGuard<'_> {
    /// Replay exact manifest evidence while borrowing the already-held original journal.
    ///
    /// Requires the retained plan and unchanged guarded journal before/after admission,
    /// without acquiring a second journal lock or reading artifact trees. This grants
    /// no current byte, backend, application or effect authority.
    /// # Errors
    /// Rejects unsafe/missing/changed originals, wrong identity and manifest contention.
    pub fn read_download_manifest(
        &self,
        plan: &OperationPlanRecord,
        expected: &ArtifactChecksumRecord,
    ) -> Result<DownloadJournalRecord, DownloadManifestError> {
        self.check_usable()?;
        self.require_unchanged_integrity_journal()?;
        let path = self.layout.root().join(MANIFEST_FILE);
        let _lock = JournalLock::acquire(&path)?;
        let record = read_manifest_record(self.layout, plan, expected)?;
        if self.record()? != &record {
            return Err(DownloadManifestError::JournalChanged);
        }
        self.require_unchanged_integrity_journal()?;
        self.layout.check_root()?;
        Ok(record)
    }

    /// Freshly verify and immutably publish the exact original durable download set.
    ///
    /// Reuses the existing journal schema/identity owner and guarded no-follow byte
    /// verification. The private bounded file is never replaced. An existing file
    /// or lost publication reply requires explicit exact local replay. This changes
    /// no journal, spending or references and invokes no provider. Stable byte
    /// custody, complete transfer, authenticated snapshots and consistency remain
    /// integration-owned; this is not the full product backup manifest/terminal proof.
    /// # Errors
    /// Rejects original/evidence/byte drift, incomplete sets, contention and publication failures.
    pub fn publish_download_manifest(
        &self,
        plan: &OperationPlanRecord,
    ) -> Result<ArtifactChecksumRecord, DownloadManifestError> {
        self.publish_manifest_with(plan, create_json_durable)
    }

    fn publish_manifest_with(
        &self,
        plan: &OperationPlanRecord,
        writer: impl FnOnce(&Path, &DownloadJournalRecord) -> Result<(), PersistenceError>,
    ) -> Result<ArtifactChecksumRecord, DownloadManifestError> {
        self.check_usable()?;
        let path = self.layout.root().join(MANIFEST_FILE);
        let _lock = JournalLock::acquire(&path)?;
        self.verify_durable_artifacts(plan)?;
        writer(&path, self.record()?)?;
        Ok(self.record()?.digest())
    }
}

/// Replay exact immutable download evidence under its retained original plan/journal.
///
/// This reads bounded machine records only; artifact trees may be absent or changed.
/// It never rechecks bytes or remote state, recreates evidence, repairs a mismatch,
/// resets allowances or releases references. Drop active download guards first.
/// Fresh local byte verification remains an explicit separate operation.
/// # Errors
/// Rejects unsafe/missing/excessive evidence, wrong identity, incomplete selected sets,
/// original journal drift and contention. No conflicting evidence is overwritten.
pub fn read_download_manifest(
    layout: &BackupLayoutGuard,
    plan: &OperationPlanRecord,
    expected: &ArtifactChecksumRecord,
) -> Result<DownloadJournalRecord, DownloadManifestError> {
    layout.check_root()?;
    let path = layout.root().join(MANIFEST_FILE);
    let _lock = JournalLock::acquire(&path)?;
    let record = read_manifest_record(layout, plan, expected)?;
    let journal = DownloadJournalGuard::open(layout, plan.digest().hash())?;
    if journal.record()? != &record {
        return Err(DownloadManifestError::JournalChanged);
    }
    layout.check_root()?;
    Ok(record)
}

fn read_manifest_record(
    layout: &BackupLayoutGuard,
    plan: &OperationPlanRecord,
    expected: &ArtifactChecksumRecord,
) -> Result<DownloadJournalRecord, DownloadManifestError> {
    let path = layout.root().join(MANIFEST_FILE);
    let record: DownloadJournalRecord = read_json(&path, MAX_DOWNLOAD_JOURNAL_BYTES)?;
    check_size(&record)?;
    if &record.digest() != expected {
        return Err(DownloadManifestError::DigestMismatch);
    }
    read_operation_plan(layout, &plan.digest()).map_err(DownloadIntegrityError::from)?;
    validate(plan, &record).map_err(DownloadIntegrityError::from)?;
    Ok(record)
}

/// Typed immutable local declaration publication/replay failure.
#[derive(Debug, Error)]
pub enum DownloadManifestError {
    /// The supplied exact manifest fingerprint differs from retained evidence.
    #[error("download manifest digest mismatch")]
    DigestMismatch,
    /// Immutable manifest and original retained journal no longer agree.
    #[error("download manifest differs from original retained journal")]
    JournalChanged,
    /// Original-plan, selected-set or fresh local byte verification failed.
    #[error(transparent)]
    Integrity(#[from] DownloadIntegrityError),
    /// Original journal/layout admission failed.
    #[error(transparent)]
    Journal(#[from] DownloadJournalError),
    /// Manifest exclusion failed.
    #[error(transparent)]
    Lock(#[from] JournalLockError),
    /// Bounded records or immutable durable publication failed.
    #[error(transparent)]
    Persistence(#[from] PersistenceError),
}

#[cfg(all(test, unix))]
mod tests;
