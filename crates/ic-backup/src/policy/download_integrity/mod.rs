//! Pure original-plan admission of durable local download declarations.

use crate::model::{
    artifacts::ArtifactChecksumRecord,
    download_journal::{ArtifactStateRecord, DownloadArtifactRecord, DownloadJournalRecord},
    operation_plan::OperationPlanRecord,
};
use thiserror::Error;

/// Borrowed durable artifact declaration with its required exact checksum.
///
/// This is a structural projection, not a fresh byte verification or remote receipt.
#[derive(Debug)]
pub struct DurableDownloadArtifactView<'a> {
    artifact: &'a DownloadArtifactRecord,
    checksum: &'a ArtifactChecksumRecord,
}

impl<'a> DurableDownloadArtifactView<'a> {
    /// Read exact retained canister/snapshot identity, metadata and canonical path.
    #[must_use]
    pub const fn artifact(&self) -> &'a DownloadArtifactRecord {
        self.artifact
    }

    /// Read the retained required checksum without reconstructing its metadata.
    #[must_use]
    pub const fn checksum(&self) -> &'a ArtifactChecksumRecord {
        self.checksum
    }
}

/// Read-only exact selected set under the original plan and durable journal.
///
/// No IO, fresh verification flag, terminal proof or reference-release admission
/// is provided. Transfer completeness and actual capture identity remain with
/// the integration; durable state is only a retained local declaration.
#[derive(Debug)]
pub struct DurableDownloadView<'a> {
    plan: &'a OperationPlanRecord,
    journal: &'a DownloadJournalRecord,
    artifacts: Vec<DurableDownloadArtifactView<'a>>,
}

impl<'a> DurableDownloadView<'a> {
    /// Read the full original plan, including original selection and spending limits.
    #[must_use]
    pub const fn plan(&self) -> &'a OperationPlanRecord {
        self.plan
    }

    /// Read exact retained journal identity and snapshot declarations.
    #[must_use]
    pub const fn journal(&self) -> &'a DownloadJournalRecord {
        self.journal
    }

    /// Read the complete selected artifact set in canonical principal order.
    #[must_use]
    pub fn artifacts(&self) -> &[DurableDownloadArtifactView<'a>] {
        &self.artifacts
    }
}

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
    plan: &'a OperationPlanRecord,
    journal: &'a DownloadJournalRecord,
) -> Result<DurableDownloadView<'a>, DownloadIntegrityPolicyError> {
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
    let artifacts = journal
        .artifacts()
        .iter()
        .map(|artifact| {
            let checksum = artifact
                .checksum()
                .filter(|_| artifact.state() == ArtifactStateRecord::Durable)
                .ok_or_else(|| DownloadIntegrityPolicyError::NonDurableArtifact {
                    canister_id: artifact.canister_id().to_owned(),
                    state: artifact.state(),
                })?;
            Ok(DurableDownloadArtifactView { artifact, checksum })
        })
        .collect::<Result<Vec<_>, DownloadIntegrityPolicyError>>()?;
    Ok(DurableDownloadView {
        plan,
        journal,
        artifacts,
    })
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
