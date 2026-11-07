//! Product-qualified terminal intent for an exclusively in-process restore.

use crate::model::artifacts::ArtifactChecksumRecord;

/// Fresh complete terminal evidence supplied by the owning integration.
///
/// The provider must authenticate the original journal, settle all application and
/// external-effect obligations, and establish that no external commands were ever
/// dispatched. Absence of command sidecars is insufficient. Command-backed restores
/// require their original qualified integration; this API cannot release them.
#[derive(Debug)]
pub struct TerminalRestoreEvidence {
    authority: ArtifactChecksumRecord,
}

impl TerminalRestoreEvidence {
    /// Declare complete qualification of an exclusively in-process restore.
    #[must_use]
    pub const fn for_in_process_restore(authority: ArtifactChecksumRecord) -> Self {
        Self { authority }
    }

    /// Read original intent, independently compared with retained references.
    #[must_use]
    pub const fn authority(&self) -> &ArtifactChecksumRecord {
        &self.authority
    }
}
