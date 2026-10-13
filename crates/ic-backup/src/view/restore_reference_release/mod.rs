//! Product-qualified terminal intent and its complete original command identities.

use crate::model::{artifacts::ArtifactChecksumRecord, command_custody::CommandCustodyRecord};

/// Read-only result of a product's fresh complete terminal-journal validation.
///
/// Construction declares no terminal permission by itself. The guarded release
/// boundary checks the current journal binding and actual quiescence of every
/// retained command, while product qualification owns complete effect/application
/// obligations and the completeness of this original command set.
#[derive(Debug)]
pub struct TerminalRestoreEvidence {
    authority: ArtifactChecksumRecord,
    commands: Vec<CommandCustodyRecord>,
}

impl TerminalRestoreEvidence {
    /// Declare the validated original restore intent and complete original command set.
    #[must_use]
    pub const fn new(
        authority: ArtifactChecksumRecord,
        commands: Vec<CommandCustodyRecord>,
    ) -> Self {
        Self {
            authority,
            commands,
        }
    }

    /// Return immutable original intent, independently compared with retained references.
    #[must_use]
    pub const fn authority(&self) -> &ArtifactChecksumRecord {
        &self.authority
    }

    /// Return original command identities; no sidecar is created or inferred.
    #[must_use]
    pub fn commands(&self) -> &[CommandCustodyRecord] {
        &self.commands
    }
}
