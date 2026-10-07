//! Pure admission of independently qualified original capture settlement.

use super::{
    IcObservationAssociationError, IcObservationReplyView, IcObservationResponseView,
    validate_response,
};
use crate::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::{AttemptJournalRecord, ObservationOutcomeRecord},
        ic_observation::{
            IcCaptureAttribution, IcCaptureSettlement, IcObservationRequest, IcObservationResponse,
        },
        ic_request::IcManagementMethodRecord as Method,
        ic_snapshot_reply::{IcSnapshotInfo, IcSnapshotReply},
    },
    policy::snapshot_inventory_delta::{SnapshotInventoryDeltaError, compare_inventories},
};
use thiserror::Error;

/// Matched passive capture claim and exact inventories; never an authenticated receipt.
#[derive(Debug)]
pub struct IcCaptureSettlementView<'a> {
    observation: IcObservationResponseView<'a>,
    baseline: &'a IcSnapshotReply<'a>,
    settlement: &'a IcCaptureSettlement,
    captured_snapshot: Option<usize>,
    outcome: ObservationOutcomeRecord,
}
impl<'a> IcCaptureSettlementView<'a> {
    /// Read the independent claim; integration qualification precedes explicit recording.
    #[must_use]
    pub const fn outcome(&self) -> ObservationOutcomeRecord {
        self.outcome
    }
    /// Read the original independently retained full baseline.
    #[must_use]
    pub const fn baseline(&self) -> &'a IcSnapshotReply<'a> {
        self.baseline
    }
    /// Read the exact current reserved list association.
    #[must_use]
    pub const fn observation(&self) -> &IcObservationResponseView<'a> {
        &self.observation
    }
    /// Read the complete retained integration claims.
    #[must_use]
    pub const fn settlement(&self) -> &'a IcCaptureSettlement {
        self.settlement
    }
    /// Read only the explicitly attributed new descriptor, without singleton inference.
    #[must_use]
    pub fn captured_snapshot(&self) -> Option<&IcSnapshotInfo> {
        match self.observation.reply() {
            IcObservationReplyView::Inventory(reply) => self
                .captured_snapshot
                .map(|index| &reply.snapshots()[index]),
            IcObservationReplyView::Status(_) => None,
        }
    }
}

/// Bind independently qualified capture settlement to exact original/current inventories.
///
/// Reuses current reservations, actual claims, bounded decoding and the canonical
/// closed-baseline comparison. Every original ID and its metadata must remain unchanged.
/// Applied names an explicitly attributed new ID; zero/one/many candidates infer no
/// outcome. Negative evidence excludes transient capture/deletion. Unresolved needs
/// an actually settled authenticated list; lost replies remain pending.
///
/// Integrations own original baseline chronology, actual authentication, freshness,
/// read permission, attribution and custody. Matching digests establish none of those.
/// This performs no IO, calls, serialization, receipt transition, retries, refunds,
/// consistency/transfer qualification or terminal/fence/reference release admission.
/// # Errors
/// Rejects lifecycle/status lanes, changed reservations/claims/wire, baseline loss or
/// drift, settlement identity drift and an Applied ID outside the exact new candidates.
pub fn validate_capture_settlement<'a>(
    request: &IcObservationRequest<'a>,
    journal: &AttemptJournalRecord,
    baseline: &'a IcSnapshotReply<'a>,
    response: &'a IcObservationResponse,
    challenge: &ArtifactChecksumRecord,
    settlement: &'a IcCaptureSettlement,
) -> Result<IcCaptureSettlementView<'a>, IcCaptureSettlementError> {
    if request.mutation().method() != Method::TakeCanisterSnapshot
        || request.payload().method() != Method::ListCanisterSnapshots
    {
        return Err(IcCaptureSettlementError::UnsupportedMethod);
    }
    let observation = validate_response(request, journal, response)?;
    let IcObservationReplyView::Inventory(inventory) = observation.reply() else {
        return Err(IcCaptureSettlementError::UnsupportedMethod);
    };
    let candidates = compare_inventories(request.mutation().target(), baseline, inventory)?;
    if settlement.authority != request.authority().digest() {
        return Err(IcCaptureSettlementError::AuthorityMismatch);
    }
    if settlement.mutation_attempt != request.mutation_attempt()
        || settlement.observation_attempt != request.observation_attempt()
    {
        return Err(IcCaptureSettlementError::AttemptMismatch);
    }
    if &settlement.challenge != challenge {
        return Err(IcCaptureSettlementError::ChallengeMismatch);
    }
    if settlement.baseline != baseline.digest() {
        return Err(IcCaptureSettlementError::BaselineMismatch);
    }
    if settlement.inventory != inventory.digest() {
        return Err(IcCaptureSettlementError::InventoryMismatch);
    }
    if settlement.observation_evidence != response.input().evidence {
        return Err(IcCaptureSettlementError::ObservationEvidenceMismatch);
    }
    let (outcome, captured_snapshot) = match &settlement.attribution {
        IcCaptureAttribution::Applied { snapshot_id, .. } => {
            if !candidates
                .iter()
                .any(|snapshot| snapshot.id() == snapshot_id)
            {
                return Err(IcCaptureSettlementError::NotNewCandidate);
            }
            let index = inventory
                .snapshots()
                .binary_search_by(|snapshot| snapshot.id().cmp(snapshot_id.as_slice()))
                .map_err(|_| IcCaptureSettlementError::NotNewCandidate)?;
            (ObservationOutcomeRecord::Applied, Some(index))
        }
        IcCaptureAttribution::NotApplied { .. } => (ObservationOutcomeRecord::NotApplied, None),
        IcCaptureAttribution::Unresolved { .. } => (ObservationOutcomeRecord::Uncertain, None),
    };
    Ok(IcCaptureSettlementView {
        observation,
        baseline,
        settlement,
        captured_snapshot,
        outcome,
    })
}

/// Typed local claim denial; original allowances, obligations and references stay retained.
#[derive(Debug, Error)]
pub enum IcCaptureSettlementError {
    /// Only original non-replacing capture and a reserved list observation are supported.
    #[error("capture settlement requires original capture and list observation")]
    UnsupportedMethod,
    /// Existing association, current reservation or finite wire admission rejected.
    #[error(transparent)]
    Observation(#[from] IcObservationAssociationError),
    /// Existing closed full-baseline comparison rejected.
    #[error(transparent)]
    Baseline(#[from] SnapshotInventoryDeltaError),
    /// Complete original operation authority differs.
    #[error("capture settlement original authority differs")]
    AuthorityMismatch,
    /// Original mutation or observation attempt differs.
    #[error("capture settlement attempt identities differ")]
    AttemptMismatch,
    /// Caller-owned current qualification challenge differs.
    #[error("capture settlement challenge differs")]
    ChallengeMismatch,
    /// Exact original baseline request/raw evidence differs.
    #[error("capture settlement original baseline differs")]
    BaselineMismatch,
    /// Exact current inventory request/raw evidence differs.
    #[error("capture settlement current inventory differs")]
    InventoryMismatch,
    /// Opaque observation evidence differs even if raw reply bytes match.
    #[error("capture settlement observation evidence differs")]
    ObservationEvidenceMismatch,
    /// The attributed raw ID is not a bounded new candidate in the current inventory.
    #[error("capture settlement ID is not a new candidate")]
    NotNewCandidate,
}

#[cfg(test)]
mod tests;
