//! Pure admission of independently qualified original capture settlement.

use super::{
    IcObservationAssociationError, IcObservationReplyView, IcObservationResponseView,
    validate_response,
};
use crate::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::{AttemptJournalRecord, ObservationOutcomeRecord},
        ic_observation::{IcCaptureSettlement, IcObservationRequest, IcObservationResponse},
        ic_request::IcManagementMethodRecord as Method,
        ic_snapshot_reply::{IcSnapshotInfo, IcSnapshotReply},
    },
    policy::{
        inventory_settlement::{
            InventoryAttribution, InventorySettlementIdentity, InventorySettlementMismatch,
            InventorySettlementOutcome, admitted_outcome, validate_pair,
        },
        settlement_identity::SettlementIdentity,
        snapshot_inventory_delta::SnapshotInventoryDeltaError,
    },
};
use thiserror::Error;

#[cfg(test)]
use crate::model::ic_observation::IcCaptureAttribution;

/// Matched passive capture claim and exact inventories; never an authenticated receipt.
#[derive(Debug)]
pub struct IcCaptureSettlementView<'a> {
    observation: IcObservationResponseView<'a>,
    baseline: &'a IcSnapshotReply<'a>,
    settlement: &'a IcCaptureSettlement,
    resolution: InventorySettlementOutcome,
}
impl<'a> IcCaptureSettlementView<'a> {
    /// Read the independent claim; integration qualification precedes explicit recording.
    #[must_use]
    pub const fn outcome(&self) -> ObservationOutcomeRecord {
        self.resolution.outcome()
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
        self.observation
            .inventory_snapshot(self.resolution.snapshot_index()?)
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
    let authority = request.authority().digest();
    let candidates = validate_pair::<IcCaptureSettlementError>(
        request.mutation().target(),
        baseline,
        inventory,
        &InventorySettlementIdentity {
            operation: SettlementIdentity {
                authority: &settlement.authority,
                mutation_attempt: settlement.mutation_attempt,
                observation_attempt: settlement.observation_attempt,
                challenge: &settlement.challenge,
            },
            baseline: &settlement.baseline,
            inventory: &settlement.inventory,
            observation_evidence: &settlement.observation_evidence,
        },
        &SettlementIdentity {
            authority: &authority,
            mutation_attempt: request.mutation_attempt(),
            observation_attempt: request.observation_attempt(),
            challenge,
        },
        &response.input().evidence,
    )?;
    let resolution = admitted_outcome(
        inventory,
        &candidates,
        InventoryAttribution::from(&settlement.attribution),
    )
    .map_err(|()| IcCaptureSettlementError::NotNewCandidate)?;
    Ok(IcCaptureSettlementView {
        observation,
        baseline,
        settlement,
        resolution,
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
    /// Exact original settlement identity or retained evidence differs.
    #[error(transparent)]
    Identity(#[from] InventorySettlementMismatch),
    /// The attributed raw ID is not a bounded new candidate in the current inventory.
    #[error("capture settlement ID is not a new candidate")]
    NotNewCandidate,
}

#[cfg(test)]
mod tests;
