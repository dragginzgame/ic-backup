//! Read-only resume projections; no scheduling, IO or transitions.

use super::{ArtifactStateRecord, DownloadJournalRecord};
use serde::Serialize;

/// Next local action derived from retained artifact state.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum ResumeAction {
    /// Transfer is not locally complete; backend-specific recovery must decide dispatch.
    Download,
    /// Verify complete staged bytes and retain their checksum.
    VerifyChecksum,
    /// Publish the checksum-bound staging directory or adopt its matching canonical tree.
    Finalize,
    /// Local publication is retained; this is not fresh byte or remote verification.
    Skip,
}

/// Owned read-only projection for one retained artifact.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DownloadArtifactView {
    /// Normalized source principal.
    pub canister_id: String,
    /// Exact backend snapshot token.
    pub snapshot_id: String,
    /// Retained local state.
    pub state: ArtifactStateRecord,
    /// Next local action, never automatic remote retry permission.
    pub resume_action: ResumeAction,
}

/// Owned read-only journal projection with progress derived from actual entries.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DownloadJournalView {
    /// Canonical immutable intent digest.
    pub intent: String,
    /// Number of artifacts not yet locally durable.
    pub pending_artifacts: usize,
    /// Whether every selected artifact has retained durable publication.
    pub is_complete: bool,
    /// Entry-derived resume actions in canonical principal order.
    pub artifacts: Vec<DownloadArtifactView>,
}

impl DownloadJournalRecord {
    /// Project retained local progress without observing bytes or remote state.
    #[must_use]
    pub fn resume_view(&self) -> DownloadJournalView {
        let artifacts: Vec<_> = self
            .artifacts()
            .iter()
            .map(|artifact| DownloadArtifactView {
                canister_id: artifact.canister_id().to_owned(),
                snapshot_id: artifact.snapshot_id().to_owned(),
                state: artifact.state(),
                resume_action: match artifact.state() {
                    ArtifactStateRecord::Created => ResumeAction::Download,
                    ArtifactStateRecord::Downloaded => ResumeAction::VerifyChecksum,
                    ArtifactStateRecord::ChecksumVerified => ResumeAction::Finalize,
                    ArtifactStateRecord::Durable => ResumeAction::Skip,
                },
            })
            .collect();
        let pending_artifacts = artifacts
            .iter()
            .filter(|entry| entry.state != ArtifactStateRecord::Durable)
            .count();
        DownloadJournalView {
            intent: self.intent().to_owned(),
            pending_artifacts,
            is_complete: pending_artifacts == 0,
            artifacts,
        }
    }
}
