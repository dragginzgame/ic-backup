//! Exact original local source manifest admission; no current restore/effect authority.

use crate::{
    model::{
        download_journal::DownloadJournalRecord,
        operation_plan::OperationPlanRecord,
        restore_safety::{RestoreSafetyRequirementError, RestoreSafetyRequirementRecord},
    },
    policy::download_integrity::{
        self, DownloadIntegrityPolicyError, DurableDownloadArtifactView, DurableDownloadView,
    },
};
use thiserror::Error;

/// Read-only original source declaration and exact same-ID restore selection.
///
/// Pure admission proves no fresh bytes, transfer completeness, authenticated snapshot,
/// application subset safety, control/lifecycle authority, upload/load or terminal proof.
#[derive(Debug)]
pub struct LocalRestoreSourceView<'a> {
    restore: &'a OperationPlanRecord,
    requirement: &'a RestoreSafetyRequirementRecord,
    source: DurableDownloadView<'a>,
}

impl<'a> LocalRestoreSourceView<'a> {
    /// Read the exact original restore plan and its unchanged attempt allowances.
    #[must_use]
    pub const fn restore(&self) -> &'a OperationPlanRecord {
        self.restore
    }
    /// Read original source, application safety lane and fence obligations.
    #[must_use]
    pub const fn requirement(&self) -> &'a RestoreSafetyRequirementRecord {
        self.requirement
    }
    /// Read the complete original source's durable declarations, including unselected artifacts.
    #[must_use]
    pub const fn source(&self) -> &DurableDownloadView<'a> {
        &self.source
    }
    /// Borrow exact selected artifacts in canonical principal order, without identity rebinding.
    pub fn selected_artifacts(&self) -> impl Iterator<Item = &DurableDownloadArtifactView<'a>> {
        self.source.artifacts().iter().filter(|artifact| {
            self.restore
                .selected_targets()
                .binary_search_by(|target| target.as_str().cmp(artifact.artifact().canister_id()))
                .is_ok()
        })
    }
}

/// Admit exact original plans and the local manifest digest retained in their requirement.
///
/// This opt-in local manifest binding does not reinterpret generic opaque integration
/// artifact digests. The existing requirement owner checks same network/release/IDs;
/// the durable download owner checks full original source intent and complete coverage.
/// A restore subset needs separate application qualification. No IO, transitions,
/// serialization, remote observations or new spending occur here.
/// # Errors
/// Rejects changed originals, another artifact binding or incomplete/non-durable source evidence.
pub fn validate<'a>(
    restore: &'a OperationPlanRecord,
    source: &'a OperationPlanRecord,
    requirement: &'a RestoreSafetyRequirementRecord,
    manifest: &'a DownloadJournalRecord,
) -> Result<LocalRestoreSourceView<'a>, LocalRestoreSourcePolicyError> {
    requirement.validate_plans(restore, source)?;
    if &manifest.digest() != requirement.source_artifacts() {
        return Err(LocalRestoreSourcePolicyError::ManifestMismatch);
    }
    let source = download_integrity::validate(source, manifest)?;
    Ok(LocalRestoreSourceView {
        restore,
        requirement,
        source,
    })
}

/// Typed original local source binding denial; all spending/obligations remain retained.
#[derive(Debug, Error)]
pub enum LocalRestoreSourcePolicyError {
    /// Exact original plans or same-network/release/ID source admission failed.
    #[error(transparent)]
    Requirement(#[from] RestoreSafetyRequirementError),
    /// The exact local manifest digest differs from original source artifact retention.
    #[error("local restore source manifest mismatch")]
    ManifestMismatch,
    /// Full original source intent, selected coverage or durable evidence differs.
    #[error(transparent)]
    Download(#[from] DownloadIntegrityPolicyError),
}

#[cfg(test)]
mod tests;
