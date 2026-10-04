//! Locked durable local lifecycle updates and verified artifact publication.

use super::{
    BackupLayoutGuard, JournalLock, JournalLockError, PersistenceError, commit_artifact_directory,
    create_json_durable, read_json, write_json_durable,
};
use crate::{
    model::{
        artifacts::ArtifactChecksumRecord,
        download_journal::{
            ArtifactStateRecord, DownloadArtifactRequest, DownloadJournalRecord,
            DownloadJournalRecordError, MAX_DOWNLOAD_JOURNAL_BYTES,
        },
    },
    ops::artifacts::{ArtifactError, checksum_directory},
};
use std::{fs, io, path::PathBuf};
use thiserror::Error;

const JOURNAL_FILE: &str = "download-journal.json";

/// Exclusive local lifecycle access borrowing the stable backup layout guard.
///
/// The caller owns backend artifact completeness and fresh remote authority.
/// These operations never invoke a transport, remove staging or release references.
#[derive(Debug)]
pub struct DownloadJournalGuard<'a> {
    layout: &'a BackupLayoutGuard,
    _lock: JournalLock,
    record: DownloadJournalRecord,
    usable: bool,
}

impl<'a> DownloadJournalGuard<'a> {
    /// Exclusively create exact intent and snapshot identities without replacing evidence.
    ///
    /// # Errors
    /// Rejects existing/unsafe journals, locked or replaced layouts and invalid/bounded records.
    pub fn create(
        layout: &'a BackupLayoutGuard,
        intent: &str,
        artifacts: Vec<DownloadArtifactRequest>,
    ) -> Result<Self, DownloadJournalError> {
        layout.check_root()?;
        let path = layout.root().join(JOURNAL_FILE);
        let lock = JournalLock::acquire(&path)?;
        let record = DownloadJournalRecord::new(intent, artifacts)?;
        check_size(&record)?;
        create_json_durable(&path, &record)?;
        Ok(Self {
            layout,
            _lock: lock,
            record,
            usable: true,
        })
    }

    /// Open retained bounded v1 evidence under exact caller-supplied intent.
    ///
    /// Reads only local journal evidence; it does not reverify artifacts or remote state.
    /// # Errors
    /// Rejects missing/unsafe/corrupt journals, intent mismatch and locked/replaced layouts.
    pub fn open(
        layout: &'a BackupLayoutGuard,
        expected_intent: &str,
    ) -> Result<Self, DownloadJournalError> {
        layout.check_root()?;
        let expected = ArtifactChecksumRecord::from_hash(expected_intent)
            .map_err(DownloadJournalRecordError::from)?;
        let path = layout.root().join(JOURNAL_FILE);
        let lock = JournalLock::acquire(&path)?;
        let record: DownloadJournalRecord = read_json(&path, MAX_DOWNLOAD_JOURNAL_BYTES)?;
        check_size(&record)?;
        if record.intent() != expected.hash() {
            return Err(DownloadJournalError::IntentMismatch);
        }
        Ok(Self {
            layout,
            _lock: lock,
            record,
            usable: true,
        })
    }

    /// Read retained progress; failed publication requires reopening before further use.
    ///
    /// # Errors
    /// Rejects an indeterminate write outcome or a replaced layout.
    pub fn record(&self) -> Result<&DownloadJournalRecord, DownloadJournalError> {
        self.check_usable()?;
        Ok(&self.record)
    }

    /// Return the canonical journal location whose sidecar this guard owns.
    #[must_use]
    pub fn path(&self) -> PathBuf {
        self.layout.root().join(JOURNAL_FILE)
    }

    /// Retain the caller's complete-download attestation for the exact snapshot.
    ///
    /// Requires a safe existing staging directory. The caller must already have
    /// validated complete backend metadata/extent coverage and command quiescence;
    /// traversability alone does not establish IC transfer completeness.
    /// # Errors
    /// Rejects identity/state conflicts, unsafe or missing staging and failed persistence.
    pub fn record_downloaded(
        &mut self,
        canister: &str,
        snapshot: &str,
    ) -> Result<(), DownloadJournalError> {
        let next = self.next(canister, snapshot, ArtifactStateRecord::Downloaded, None)?;
        self.check_artifact_parent()?;
        let entry = next.artifact(canister, snapshot)?;
        checksum_directory(&self.layout.root().join(entry.staging_path()))?;
        self.store(next, write_json_durable)
    }

    /// Verify staged bytes and durably retain their canonical checksum.
    ///
    /// # Errors
    /// Rejects wrong identity/state, unsafe or missing bytes and failed persistence.
    pub fn verify_artifact(
        &mut self,
        canister: &str,
        snapshot: &str,
    ) -> Result<(), DownloadJournalError> {
        self.check_usable()?;
        let entry = self.record.artifact(canister, snapshot)?;
        if entry.state() != ArtifactStateRecord::Downloaded {
            return Err(DownloadJournalRecordError::InvalidStateTransition {
                from: entry.state(),
                to: ArtifactStateRecord::ChecksumVerified,
            }
            .into());
        }
        self.check_artifact_parent()?;
        let checksum = checksum_directory(&self.layout.root().join(entry.staging_path()))?;
        let next = self.next(
            canister,
            snapshot,
            ArtifactStateRecord::ChecksumVerified,
            Some(checksum),
        )?;
        self.store(next, write_json_durable)
    }

    /// Publish exact verified bytes or adopt a matching tree after a lost response.
    ///
    /// Leaves staging and retained intent intact on rejection. Durable state does
    /// not silently trigger fresh artifact verification; that is a distinct action.
    /// # Errors
    /// Rejects wrong identity/state, changed bytes, unsafe paths and uncertain publication.
    pub fn finalize_artifact(
        &mut self,
        canister: &str,
        snapshot: &str,
    ) -> Result<(), DownloadJournalError> {
        self.finalize_with(canister, snapshot, write_json_durable)
    }

    fn finalize_with(
        &mut self,
        canister: &str,
        snapshot: &str,
        write: impl FnOnce(&std::path::Path, &DownloadJournalRecord) -> Result<(), PersistenceError>,
    ) -> Result<(), DownloadJournalError> {
        let next = self.next(canister, snapshot, ArtifactStateRecord::Durable, None)?;
        check_size(&next)?;
        self.check_artifact_parent()?;
        let entry = next.artifact(canister, snapshot)?;
        let checksum = entry
            .checksum()
            .ok_or(DownloadJournalRecordError::InvalidChecksumState(
                ArtifactStateRecord::Durable,
            ))?;
        // A failed commit may have published bytes. Stop this guard until its
        // durable journal is reopened and those exact bytes are reconciled.
        self.usable = false;
        commit_artifact_directory(
            &self.layout.root().join(entry.staging_path()),
            &self.layout.root().join(entry.artifact_path()),
            checksum.hash(),
        )?;
        self.store(next, write)
    }

    fn next(
        &self,
        canister: &str,
        snapshot: &str,
        state: ArtifactStateRecord,
        checksum: Option<ArtifactChecksumRecord>,
    ) -> Result<DownloadJournalRecord, DownloadJournalError> {
        self.check_usable()?;
        let mut next = self.record.clone();
        next.advance(canister, snapshot, state, checksum)?;
        Ok(next)
    }

    fn check_usable(&self) -> Result<(), DownloadJournalError> {
        if !self.usable {
            return Err(DownloadJournalError::IndeterminateWrite);
        }
        self.layout.check_root()?;
        Ok(())
    }

    fn check_artifact_parent(&self) -> Result<(), DownloadJournalError> {
        let path = self.layout.root().join("artifacts");
        if !fs::symlink_metadata(&path)?.is_dir() {
            return Err(DownloadJournalError::UnsafeArtifactParent { path });
        }
        Ok(())
    }

    fn store(
        &mut self,
        next: DownloadJournalRecord,
        write: impl FnOnce(&std::path::Path, &DownloadJournalRecord) -> Result<(), PersistenceError>,
    ) -> Result<(), DownloadJournalError> {
        check_size(&next)?;
        self.layout.check_root()?;
        self.usable = false;
        write(&self.path(), &next)?;
        self.record = next;
        self.usable = true;
        Ok(())
    }
}

fn check_size(record: &DownloadJournalRecord) -> Result<(), PersistenceError> {
    if serde_json::to_vec_pretty(record)?.len() as u64 > MAX_DOWNLOAD_JOURNAL_BYTES {
        return Err(PersistenceError::RecordTooLarge {
            limit: MAX_DOWNLOAD_JOURNAL_BYTES,
        });
    }
    Ok(())
}

/// Typed local journal admission or durable lifecycle failure.
#[derive(Debug, Error)]
pub enum DownloadJournalError {
    /// The caller's exact intent digest differs from retained evidence.
    #[error("download journal immutable intent mismatch")]
    IntentMismatch,
    /// A publication may have completed; reopen and reconcile retained evidence.
    #[error("download journal outcome is indeterminate; reopen retained evidence")]
    IndeterminateWrite,
    /// The fixed artifact parent is not an existing regular directory.
    #[error("unsafe download artifact parent: {path:?}")]
    UnsafeArtifactParent {
        /// Rejected location.
        path: PathBuf,
    },
    /// Model identity, schema or transition admission failed.
    #[error(transparent)]
    Record(#[from] DownloadJournalRecordError),
    /// Local journal or layout exclusion failed.
    #[error(transparent)]
    Lock(#[from] JournalLockError),
    /// Durable local record or artifact publication failed.
    #[error(transparent)]
    Persistence(#[from] PersistenceError),
    /// Secure artifact traversal failed.
    #[error(transparent)]
    Artifact(#[from] ArtifactError),
    /// Local fixture-independent filesystem access failed.
    #[error(transparent)]
    Io(#[from] io::Error),
}

#[cfg(all(test, unix))]
mod tests;
