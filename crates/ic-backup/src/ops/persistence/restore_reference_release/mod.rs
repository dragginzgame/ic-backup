//! Release exact source dependencies under terminal journal and command custody.

use super::{
    json::{encode_json, write_encoded},
    layout_lifetime::MAX_RESTORE_REFERENCE_BYTES,
};
use crate::{
    model::{effect_graph::MAX_EFFECT_OPERATIONS, restore_references::RestoreReferenceRecord},
    ops::persistence::{
        BackupLayoutGuard, CommandLifetimeLockError, CommandQuiescenceGuard, JournalLock,
        PersistenceError,
    },
    view::restore_reference_release::TerminalRestoreEvidence,
};
use std::{collections::BTreeSet, path::Path};
use thiserror::Error;

/// Maximum bytes supplied to the integration's terminal-journal validator.
pub const MAX_RESTORE_TERMINAL_BYTES: usize = 16 * 1024 * 1024;

/// Validate complete original restore settlement and return its exact command set.
///
/// Implementations authenticate their maintained journal schema, original intent,
/// complete application/effect obligations and every external command identity.
/// An all-Applied projection alone is insufficient. The library supplies freshly
/// read journal bytes under its held mutation lock; no cached completion flag is used.
pub trait RestoreTerminalEvidenceProvider {
    /// Product-owned typed terminal qualification failure.
    type Error: std::error::Error + 'static;

    /// Qualify the current terminal bytes and complete retained command-custody set.
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
    /// Current terminal/custody evidence passes and the dependency is already absent.
    AlreadyReleased,
}

impl BackupLayoutGuard {
    /// Release one exact restore dependency after fresh terminal and command checks.
    ///
    /// Keep the original journal mutation guard held. Every declared command sidecar
    /// must already exist with its retained inode/device identity and be exclusively
    /// quiescent. Guards remain held through reference publication. Failures leave
    /// dependencies retained; lost publication responses reconcile by the same call.
    /// This performs no application fence release, retry or paid effect.
    ///
    /// # Errors
    /// Refuses changed layouts/journals, terminal admission, changed intent, invalid
    /// or conflicting custody identities, active descendants and publication failures.
    pub fn release_restore<P: RestoreTerminalEvidenceProvider>(
        &self,
        journal_guard: &JournalLock,
        provider: &P,
    ) -> Result<RestoreReferenceReleaseOutcome, RestoreReferenceReleaseError<P::Error>> {
        let journal = journal_guard.journal();
        self.check_root()?;
        let bytes = read_terminal(journal)?;
        let terminal = provider
            .qualify_terminal(journal, &bytes)
            .map_err(RestoreReferenceReleaseError::Terminal)?;
        let reference =
            RestoreReferenceRecord::new(journal.to_path_buf(), terminal.authority().hash())
                .map_err(PersistenceError::from)?;
        if terminal.commands().len() > MAX_EFFECT_OPERATIONS {
            return Err(RestoreReferenceReleaseError::InvalidCommandSet);
        }
        let mut operations = BTreeSet::new();
        for command in terminal.commands() {
            if command.journal() != journal || !operations.insert(command.operation_sequence()) {
                return Err(RestoreReferenceReleaseError::InvalidCommandSet);
            }
        }
        let mut quiescence = Vec::with_capacity(terminal.commands().len());
        for command in terminal.commands() {
            quiescence.push(CommandQuiescenceGuard::acquire(command)?);
        }
        let mut references = self.restore_references()?;
        let released = references
            .release(&reference)
            .map_err(PersistenceError::from)?;
        // The validator may perform product reads; preserve the exact bytes it admitted.
        if read_terminal(journal)? != bytes {
            return Err(RestoreReferenceReleaseError::JournalChanged);
        }
        self.check_root()?;
        if released {
            let encoded = encode_json(&references, MAX_RESTORE_REFERENCE_BYTES)?;
            write_encoded(&self.root().join("restore-references.json"), &encoded)?;
        }
        self.sync_references()?;
        drop(quiescence);
        Ok(if released {
            RestoreReferenceReleaseOutcome::Released
        } else {
            RestoreReferenceReleaseOutcome::AlreadyReleased
        })
    }
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
    /// A command belongs to another journal or repeats an original identity.
    #[error("restore terminal command set is invalid")]
    InvalidCommandSet,
    /// Journal bytes changed after product qualification.
    #[error("restore terminal journal changed during qualification")]
    JournalChanged,
    /// Actual command exclusion or retained sidecar identity could not be established.
    #[error(transparent)]
    Custody(#[from] CommandLifetimeLockError),
    /// Layout/reference/byte admission or durable publication failed.
    #[error(transparent)]
    Persistence(#[from] PersistenceError),
}

#[cfg(all(test, unix))]
mod tests;
