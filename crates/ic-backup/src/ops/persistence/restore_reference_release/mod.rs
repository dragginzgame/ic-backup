//! Release exact source dependencies after complete in-process restore qualification.

use super::{
    json::{encode_json, write_encoded},
    layout_lifetime::MAX_RESTORE_REFERENCE_BYTES,
};
use crate::{
    model::restore_references::RestoreReferenceRecord,
    ops::persistence::{BackupLayoutGuard, JournalLock, PersistenceError},
    view::restore_reference_release::TerminalRestoreEvidence,
};
use std::{fs, path::Path};
use thiserror::Error;

/// Maximum bytes supplied to the integration's terminal-journal validator.
pub const MAX_RESTORE_TERMINAL_BYTES: usize = 16 * 1024 * 1024;

/// Validate complete original settlement of an exclusively in-process restore.
///
/// Implementations authenticate their maintained journal schema, original intent,
/// complete application/effect obligations and that no external command was dispatched.
/// An all-Applied projection alone is insufficient. The library supplies freshly
/// read journal bytes under its held mutation lock; no cached completion flag is used.
pub trait RestoreTerminalEvidenceProvider {
    /// Product-owned typed terminal qualification failure.
    type Error: std::error::Error + 'static;

    /// Qualify current terminal bytes; reject any original external-command execution.
    ///
    /// # Errors
    /// Reject incomplete, changed, unauthenticated or unsettled original evidence.
    fn qualify_terminal(
        &self,
        journal: &Path,
        bytes: &[u8],
    ) -> Result<TerminalRestoreEvidence, Self::Error>;
}

/// Observable durable reference transition; neither outcome dispatches IC effects.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RestoreReferenceReleaseOutcome {
    /// The exact dependency was durably removed.
    Released,
    /// Current terminal evidence passes and the dependency is already absent.
    AlreadyReleased,
}

impl BackupLayoutGuard {
    /// Release one exact dependency after fresh in-process terminal qualification.
    ///
    /// Hold the original journal mutation guard. Retained command sidecars reject
    /// even after their holders exit. Their absence does not qualify command-free
    /// execution: the provider must establish that from its complete original record.
    /// Recheck journal bytes and sidecars before durable publication. Lost publication
    /// responses reconcile by this same effect-free call; no fence is released.
    /// # Errors
    /// Rejects retired command evidence, terminal admission, changed originals,
    /// unsafe layouts and publication failure, preserving unfinished dependencies.
    pub fn release_restore<P: RestoreTerminalEvidenceProvider>(
        &self,
        journal_guard: &JournalLock,
        provider: &P,
    ) -> Result<RestoreReferenceReleaseOutcome, RestoreReferenceReleaseError<P::Error>> {
        let journal = journal_guard.journal();
        self.check_root()?;
        reject_command_sidecars(journal)?;
        let bytes = read_terminal(journal)?;
        let terminal = provider
            .qualify_terminal(journal, &bytes)
            .map_err(RestoreReferenceReleaseError::Terminal)?;
        let reference =
            RestoreReferenceRecord::new(journal.to_path_buf(), terminal.authority().hash())
                .map_err(PersistenceError::from)?;
        let mut references = self.restore_references()?;
        let released = references
            .release(&reference)
            .map_err(PersistenceError::from)?;
        // The validator may perform product reads; preserve the exact bytes it admitted.
        if read_terminal(journal)? != bytes {
            return Err(RestoreReferenceReleaseError::JournalChanged);
        }
        self.check_root()?;
        reject_command_sidecars(journal)?;
        if released {
            let encoded = encode_json(&references, MAX_RESTORE_REFERENCE_BYTES)?;
            write_encoded(&self.root().join("restore-references.json"), &encoded)?;
        }
        self.sync_references()?;
        Ok(if released {
            RestoreReferenceReleaseOutcome::Released
        } else {
            RestoreReferenceReleaseOutcome::AlreadyReleased
        })
    }
}

fn reject_command_sidecars<E: std::error::Error + 'static>(
    journal: &Path,
) -> Result<(), RestoreReferenceReleaseError<E>> {
    let parent = journal
        .parent()
        .ok_or_else(|| PersistenceError::Io(std::io::ErrorKind::InvalidInput.into()))?;
    let mut prefix = journal
        .file_name()
        .ok_or_else(|| PersistenceError::Io(std::io::ErrorKind::InvalidInput.into()))?
        .to_os_string();
    prefix.push(".command-");
    for entry in fs::read_dir(parent).map_err(PersistenceError::from)? {
        let name = entry.map_err(PersistenceError::from)?.file_name();
        if name
            .as_encoded_bytes()
            .starts_with(prefix.as_encoded_bytes())
        {
            return Err(RestoreReferenceReleaseError::RetiredCommandState);
        }
    }
    Ok(())
}

fn read_terminal(path: &Path) -> Result<Vec<u8>, PersistenceError> {
    ic_host_fs::read::read_file_no_follow(path, MAX_RESTORE_TERMINAL_BYTES).map_err(|error| {
        match error {
            ic_host_artifacts::artifact::ArtifactError::LimitExceeded { .. } => {
                PersistenceError::RecordTooLarge {
                    limit: MAX_RESTORE_TERMINAL_BYTES as u64,
                }
            }
            error => PersistenceError::Io(error.into()),
        }
    })
}

/// Typed refusal preserves original dependency and custody evidence.
#[derive(Debug, Error)]
pub enum RestoreReferenceReleaseError<E: std::error::Error + 'static> {
    /// The product did not qualify its complete original terminal obligations.
    #[error("restore terminal admission failed: {0}")]
    Terminal(#[source] E),
    /// Retained external-command evidence requires its original qualified integration.
    #[error("command-backed restore requires its original qualified integration")]
    RetiredCommandState,
    /// Journal bytes changed after product qualification.
    #[error("restore terminal journal changed during qualification")]
    JournalChanged,
    /// Layout/reference/byte admission or durable publication failed.
    #[error(transparent)]
    Persistence(#[from] PersistenceError),
}

#[cfg(all(test, unix))]
mod tests;
