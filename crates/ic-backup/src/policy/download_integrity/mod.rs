//! Pure original-plan admission of durable local download declarations.

use crate::model::{
    artifacts::ArtifactChecksumRecord,
    download_journal::{ArtifactStateRecord, DownloadJournalRecord},
    operation_plan::OperationPlanRecord,
};
use thiserror::Error;

/// Require original intent, exact selected-target coverage and durable checksums.
///
/// The canonical record owners already bound/normalize entries and reject duplicates.
/// This comparison neither observes bytes nor authenticates retained declarations.
/// Snapshot tokens/timestamp/size remain exactly those retained by the journal;
/// the pre-capture plan cannot establish an eventual captured snapshot's identity.
///
/// # Errors
/// Rejects changed full-plan intent, missing/extra/different selected targets and
/// any artifact without retained durable publication and checksum.
pub fn validate<'a>(
    plan: &OperationPlanRecord,
    journal: &'a DownloadJournalRecord,
) -> Result<&'a ArtifactChecksumRecord, DownloadIntegrityPolicyError> {
    if plan.digest().hash() != journal.intent() {
        return Err(DownloadIntegrityPolicyError::IntentMismatch);
    }
    if plan.selected_targets().len() != journal.artifacts().len()
        || plan
            .selected_targets()
            .iter()
            .zip(journal.artifacts())
            .any(|(target, artifact)| target != artifact.canister_id())
    {
        return Err(DownloadIntegrityPolicyError::TargetSetMismatch);
    }
    let artifact = &journal.artifacts()[0];
    let checksum = artifact
        .checksum()
        .filter(|_| artifact.state() == ArtifactStateRecord::Durable)
        .ok_or_else(|| DownloadIntegrityPolicyError::NonDurableArtifact {
            canister_id: artifact.canister_id().to_owned(),
            state: artifact.state(),
        })?;
    Ok(checksum)
}

/// Typed structural mismatch, before any artifact IO or remote effect.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum DownloadIntegrityPolicyError {
    /// The journal is not bound to the full original plan digest.
    #[error("download journal differs from original plan intent")]
    IntentMismatch,
    /// Journal targets are not exactly the original selected physical set.
    #[error("download journal differs from the exact selected target set")]
    TargetSetMismatch,
    /// An exact artifact lacks retained durable publication and its checksum.
    #[error("artifact for {canister_id} lacks durable checksum evidence ({state:?})")]
    NonDurableArtifact {
        /// Canonical exact physical target.
        canister_id: String,
        /// Actually retained local state.
        state: ArtifactStateRecord,
    },
}

#[cfg(test)]
mod tests;
