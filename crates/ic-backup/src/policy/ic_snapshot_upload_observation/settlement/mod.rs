//! Pure original metadata-allocation settlement admission.
use super::validate_response;
use crate::{
    model::{
        artifacts::ArtifactChecksumRecord,
        attempt_journal::{AttemptJournalRecord, ObservationOutcomeRecord},
        ic_observation::IcObservationResponse,
        ic_snapshot_reply::{IcSnapshotInfo, IcSnapshotReply},
        ic_snapshot_upload::IcSnapshotUploadError,
        ic_snapshot_upload_observation::{
            IcSnapshotUploadAttribution, IcSnapshotUploadObservationRequest,
            IcSnapshotUploadSettlement,
        },
    },
    policy::{
        ic_observation::{
            IcObservationAssociationError, IcObservationReplyView, IcObservationResponseView,
        },
        snapshot_inventory_delta::{SnapshotInventoryDeltaError, compare_inventories},
    },
};
use thiserror::Error;

/// Exact local evidence and passive attribution; never an authenticated receipt.
#[derive(Debug)]
pub struct IcSnapshotUploadSettlementView<'a> {
    observation: IcObservationResponseView<'a>,
    baseline: &'a IcSnapshotReply<'a>,
    settlement: &'a IcSnapshotUploadSettlement,
    allocated_snapshot: Option<usize>,
    outcome: ObservationOutcomeRecord,
}
impl<'a> IcSnapshotUploadSettlementView<'a> {
    /// Read the passive claim; only a qualified integration may record it.
    #[must_use]
    pub const fn outcome(&self) -> ObservationOutcomeRecord {
        self.outcome
    }
    /// Read the original independently retained baseline.
    #[must_use]
    pub const fn baseline(&self) -> &'a IcSnapshotReply<'a> {
        self.baseline
    }
    /// Read the exact bounded current list association.
    #[must_use]
    pub const fn observation(&self) -> &IcObservationResponseView<'a> {
        &self.observation
    }
    /// Read the complete retained integration claims.
    #[must_use]
    pub const fn settlement(&self) -> &'a IcSnapshotUploadSettlement {
        self.settlement
    }
    /// Read only the explicitly attributed new descriptor, with no singleton inference.
    #[must_use]
    pub fn allocated_snapshot(&self) -> Option<&IcSnapshotInfo> {
        match self.observation.reply() {
            IcObservationReplyView::Inventory(reply) => self
                .allocated_snapshot
                .map(|index| &reply.snapshots()[index]),
            IcObservationReplyView::Status(_) => None,
        }
    }
}

/// Admit separately qualified allocation claims against exact original and current lists.
///
/// Every baseline ID and its metadata must remain unchanged. Applied requires an
/// explicitly attributed new ID distinct from the retained source; several candidates
/// may exist. Zero/one/many candidates imply no outcome. Negative evidence must exclude
/// transient allocation/deletion. Unresolved requires a settled authenticated list,
/// never a lost response. Integrations own original baseline chronology, actual
/// authentication, freshness, attribution and stable custody; matching digests prove
/// none of these. No calls, IO, receipts, retries, refunds or release admission occur.
/// # Errors
/// Rejects changed reservations/claims/wire, baseline drift, settlement identity drift
/// and an Applied ID outside the exact new candidate set or original destination bounds.
pub fn validate_settlement<'a>(
    request: &IcSnapshotUploadObservationRequest<'a, '_>,
    journal: &AttemptJournalRecord,
    baseline: &'a IcSnapshotReply<'a>,
    response: &'a IcObservationResponse,
    challenge: &ArtifactChecksumRecord,
    settlement: &'a IcSnapshotUploadSettlement,
) -> Result<IcSnapshotUploadSettlementView<'a>, IcSnapshotUploadSettlementError> {
    let observation = validate_response(request, journal, response)?;
    let IcObservationReplyView::Inventory(inventory) = observation.reply() else {
        return Err(IcSnapshotUploadSettlementError::WrongInventory);
    };
    let candidates = compare_inventories(request.payload().target(), baseline, inventory)?;
    if settlement.authority != request.authority().digest() {
        return Err(IcSnapshotUploadSettlementError::AuthorityMismatch);
    }
    if settlement.mutation_attempt != request.mutation_attempt()
        || settlement.observation_attempt != request.observation_attempt()
    {
        return Err(IcSnapshotUploadSettlementError::AttemptMismatch);
    }
    if &settlement.challenge != challenge {
        return Err(IcSnapshotUploadSettlementError::ChallengeMismatch);
    }
    if settlement.baseline != baseline.digest() {
        return Err(IcSnapshotUploadSettlementError::BaselineMismatch);
    }
    if settlement.inventory != inventory.digest() {
        return Err(IcSnapshotUploadSettlementError::InventoryMismatch);
    }
    if settlement.observation_evidence != response.input().evidence {
        return Err(IcSnapshotUploadSettlementError::ObservationEvidenceMismatch);
    }
    let (outcome, allocated_snapshot) = match &settlement.attribution {
        IcSnapshotUploadAttribution::Applied { snapshot_id, .. } => {
            request.mutation().validate_data_destination(snapshot_id)?;
            if !candidates
                .iter()
                .any(|snapshot| snapshot.id() == snapshot_id)
            {
                return Err(IcSnapshotUploadSettlementError::NotNewCandidate);
            }
            let index = inventory
                .snapshots()
                .binary_search_by(|snapshot| snapshot.id().cmp(snapshot_id.as_slice()))
                .map_err(|_| IcSnapshotUploadSettlementError::NotNewCandidate)?;
            (ObservationOutcomeRecord::Applied, Some(index))
        }
        IcSnapshotUploadAttribution::NotApplied { .. } => {
            (ObservationOutcomeRecord::NotApplied, None)
        }
        IcSnapshotUploadAttribution::Unresolved { .. } => {
            (ObservationOutcomeRecord::Uncertain, None)
        }
    };
    Ok(IcSnapshotUploadSettlementView {
        observation,
        baseline,
        settlement,
        allocated_snapshot,
        outcome,
    })
}

/// Local admission denial; original spending and obligations remain retained.
#[derive(Debug, Error)]
pub enum IcSnapshotUploadSettlementError {
    /// Existing exact reservation, claim or bounded wire admission failed.
    #[error(transparent)]
    Observation(#[from] IcObservationAssociationError),
    /// Existing canonical closed-baseline comparison failed.
    #[error(transparent)]
    Baseline(#[from] SnapshotInventoryDeltaError),
    /// Existing raw destination bounds or source-ID exclusion failed.
    #[error(transparent)]
    Destination(#[from] IcSnapshotUploadError),
    /// A status projection cannot substitute for the original list.
    #[error("metadata settlement requires snapshot inventory")]
    WrongInventory,
    /// Full original authority differs.
    #[error("metadata settlement authority differs")]
    AuthorityMismatch,
    /// Original attempt identities differ.
    #[error("metadata settlement attempts differ")]
    AttemptMismatch,
    /// Current challenge differs.
    #[error("metadata settlement challenge differs")]
    ChallengeMismatch,
    /// Exact original baseline evidence differs.
    #[error("metadata settlement baseline differs")]
    BaselineMismatch,
    /// Exact current inventory evidence differs.
    #[error("metadata settlement inventory differs")]
    InventoryMismatch,
    /// Opaque observation evidence differs.
    #[error("metadata settlement observation evidence differs")]
    ObservationEvidenceMismatch,
    /// The attributed ID is not a new candidate in the exact current inventory.
    #[error("metadata settlement ID is not a new candidate")]
    NotNewCandidate,
}

#[cfg(test)]
mod tests;
