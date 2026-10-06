//! Exclusive durable attempt reservations and exact receipt retention; no transport.

use super::{
    BackupLayoutGuard, JournalLock, JournalLockError, PersistenceError, create_json_durable,
    read_json, write_json_durable,
};
use crate::model::attempt_journal::{
    AttemptAuthorityRecord, AttemptJournalRecord, AttemptJournalRecordError,
    MAX_ATTEMPT_JOURNAL_BYTES, MutationReceiptRequest, ObservationReceiptRequest,
};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Exclusive local accounting borrowing stable layout exclusion.
///
/// Successful reservation persists consumption before returning its number.
/// It grants no fresh IC authority, backend settlement or subprocess dispatch permission.
#[derive(Debug)]
pub struct AttemptJournalGuard<'a> {
    layout: &'a BackupLayoutGuard,
    _lock: JournalLock,
    record: AttemptJournalRecord,
    usable: bool,
}

impl<'a> AttemptJournalGuard<'a> {
    /// Create original exact identity and limits without replacing retained evidence.
    ///
    /// # Errors
    /// Rejects unsafe/existing journals, lock contention, replaced layouts and failed persistence.
    pub fn create(
        layout: &'a BackupLayoutGuard,
        authority: AttemptAuthorityRecord,
    ) -> Result<Self, AttemptJournalError> {
        layout.check_root()?;
        let path = journal_path(layout, &authority);
        let lock = JournalLock::acquire(&path)?;
        let record = AttemptJournalRecord::new(authority);
        check_size(&record)?;
        create_json_durable(&path, &record)?;
        Ok(Self {
            layout,
            _lock: lock,
            record,
            usable: true,
        })
    }
    /// Open exact retained identity and original budgets, replaying only local events.
    ///
    /// # Errors
    /// Rejects identity/budget changes, invalid chronology, missing/unsafe/oversized records and locks.
    pub fn open(
        layout: &'a BackupLayoutGuard,
        expected: &AttemptAuthorityRecord,
    ) -> Result<Self, AttemptJournalError> {
        layout.check_root()?;
        let path = journal_path(layout, expected);
        let lock = JournalLock::acquire(&path)?;
        let record: AttemptJournalRecord = read_json(&path, MAX_ATTEMPT_JOURNAL_BYTES)?;
        check_size(&record)?;
        if record.authority() != expected {
            return Err(AttemptJournalError::AuthorityMismatch);
        }
        Ok(Self {
            layout,
            _lock: lock,
            record,
            usable: true,
        })
    }
    /// Read validated retained accounting without observing remote state.
    ///
    /// # Errors
    /// Rejects indeterminate writes or replaced layouts.
    pub fn record(&self) -> Result<&AttemptJournalRecord, AttemptJournalError> {
        self.check_usable()?;
        Ok(&self.record)
    }
    /// Return the exact journal location for this operation sequence.
    #[must_use]
    pub fn path(&self) -> PathBuf {
        journal_path(self.layout, self.record.authority())
    }
    /// Durably consume one mutation attempt before any admitted caller-owned call.
    ///
    /// # Errors
    /// Rejects unresolved attempts, exhaustion, applied operations and persistence failures.
    pub fn reserve_mutation(&mut self) -> Result<u32, AttemptJournalError> {
        self.reserve_with(AttemptJournalRecord::reserve_mutation, write_json_durable)
    }
    /// Durably consume one observation bound to an exact unresolved mutation and request.
    ///
    /// Fresh authority, paid-effect settlement and ended command custody remain caller-owned.
    /// # Errors
    /// Rejects wrong attempts/requests, unresolved observations, exhaustion and failed writes.
    pub fn reserve_observation(
        &mut self,
        mutation: u32,
        request: &str,
    ) -> Result<u32, AttemptJournalError> {
        self.reserve_with(
            |record| record.reserve_observation(mutation, request),
            write_json_durable,
        )
    }
    /// Retain an exact integration-qualified direct mutation reply.
    ///
    /// # Errors
    /// Rejects unmatched identity, absent/unsettled attempts, invalid evidence and failed persistence.
    pub fn record_mutation(
        &mut self,
        receipt: MutationReceiptRequest,
    ) -> Result<(), AttemptJournalError> {
        self.reserve_with(|record| record.record_mutation(receipt), write_json_durable)
    }
    /// Retain an exact observation; unresolved evidence never authorizes another mutation.
    ///
    /// The integration qualifies completed observation and paid-effect settlement.
    /// A lost observation response stays pending until qualified settlement.
    ///
    /// # Errors
    /// Rejects unmatched identity, absent/unsettled attempts, invalid evidence and failed persistence.
    pub fn record_observation(
        &mut self,
        receipt: ObservationReceiptRequest,
    ) -> Result<(), AttemptJournalError> {
        self.reserve_with(
            |record| record.record_observation(receipt),
            write_json_durable,
        )
    }

    fn reserve_with<T>(
        &mut self,
        transition: impl FnOnce(&mut AttemptJournalRecord) -> Result<T, AttemptJournalRecordError>,
        write: impl FnOnce(&Path, &AttemptJournalRecord) -> Result<(), PersistenceError>,
    ) -> Result<T, AttemptJournalError> {
        self.check_usable()?;
        let mut next = self.record.clone();
        let result = transition(&mut next)?;
        check_size(&next)?;
        self.usable = false;
        write(&self.path(), &next)?;
        self.record = next;
        self.usable = true;
        Ok(result)
    }
    fn check_usable(&self) -> Result<(), AttemptJournalError> {
        if !self.usable {
            return Err(AttemptJournalError::IndeterminateWrite);
        }
        self.layout.check_root()?;
        Ok(())
    }
}

fn journal_path(layout: &BackupLayoutGuard, authority: &AttemptAuthorityRecord) -> PathBuf {
    layout.root().join(format!(
        "attempt-{}.json",
        authority.binding().operation_sequence()
    ))
}
fn check_size(record: &AttemptJournalRecord) -> Result<(), PersistenceError> {
    super::json::check_json_size(record, MAX_ATTEMPT_JOURNAL_BYTES)
}

/// Typed exact admission, local accounting or durable publication failure.
#[derive(Debug, Error)]
pub enum AttemptJournalError {
    /// Retained identity or original ceilings differ from current exact selection.
    #[error("attempt journal authority mismatch")]
    AuthorityMismatch,
    /// A write may have completed; drop and reopen exact retained evidence.
    #[error("attempt journal write outcome indeterminate; reopen retained evidence")]
    IndeterminateWrite,
    /// Model identity, chronology or allowance admission failed.
    #[error(transparent)]
    Record(#[from] AttemptJournalRecordError),
    /// Journal/layout exclusion failed.
    #[error(transparent)]
    Lock(#[from] JournalLockError),
    /// Bounded durable record access failed.
    #[error(transparent)]
    Persistence(#[from] PersistenceError),
}

#[cfg(all(test, unix))]
mod tests;
