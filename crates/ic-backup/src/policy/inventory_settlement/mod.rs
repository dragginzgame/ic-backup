//! Common exact identity and explicit new-candidate checks for inventory settlements.

use super::{
    settlement_identity::{SettlementIdentity, SettlementIdentityMismatch},
    snapshot_inventory_delta::{SnapshotInventoryDeltaError, compare_inventories},
};
use crate::model::{
    artifacts::ArtifactChecksumRecord,
    attempt_journal::ObservationOutcomeRecord,
    ic_observation::IcCaptureAttribution,
    ic_snapshot_reply::{IcSnapshotInfo, IcSnapshotReply},
    ic_snapshot_upload_observation::IcSnapshotUploadAttribution,
};

pub(crate) struct InventorySettlementIdentity<'a> {
    pub authority: &'a ArtifactChecksumRecord,
    pub mutation_attempt: u32,
    pub observation_attempt: u32,
    pub challenge: &'a ArtifactChecksumRecord,
    pub baseline: &'a ArtifactChecksumRecord,
    pub inventory: &'a ArtifactChecksumRecord,
    pub observation_evidence: &'a ArtifactChecksumRecord,
}

impl InventorySettlementIdentity<'_> {
    pub fn compare(&self, expected: &Self) -> Result<(), InventorySettlementMismatch> {
        SettlementIdentity {
            authority: self.authority,
            mutation_attempt: self.mutation_attempt,
            observation_attempt: self.observation_attempt,
            challenge: self.challenge,
        }
        .compare(&SettlementIdentity {
            authority: expected.authority,
            mutation_attempt: expected.mutation_attempt,
            observation_attempt: expected.observation_attempt,
            challenge: expected.challenge,
        })?;
        if self.baseline != expected.baseline {
            return Err(InventorySettlementMismatch::Baseline);
        }
        if self.inventory != expected.inventory {
            return Err(InventorySettlementMismatch::Inventory);
        }
        if self.observation_evidence != expected.observation_evidence {
            return Err(InventorySettlementMismatch::ObservationEvidence);
        }
        Ok(())
    }
}

pub(crate) enum InventorySettlementMismatch {
    Authority,
    Attempt,
    Challenge,
    Baseline,
    Inventory,
    ObservationEvidence,
}

impl From<SettlementIdentityMismatch> for InventorySettlementMismatch {
    fn from(value: SettlementIdentityMismatch) -> Self {
        match value {
            SettlementIdentityMismatch::Authority => Self::Authority,
            SettlementIdentityMismatch::Attempt => Self::Attempt,
            SettlementIdentityMismatch::Challenge => Self::Challenge,
        }
    }
}

pub(crate) fn validate_pair<'a, E>(
    target: &str,
    baseline: &IcSnapshotReply<'_>,
    inventory: &'a IcSnapshotReply<'_>,
    claimed: &InventorySettlementIdentity<'_>,
    expected: &InventorySettlementIdentity<'_>,
) -> Result<Vec<&'a IcSnapshotInfo>, E>
where
    E: From<SnapshotInventoryDeltaError> + From<InventorySettlementMismatch>,
{
    let candidates = compare_inventories(target, baseline, inventory)?;
    claimed.compare(expected)?;
    Ok(candidates)
}

#[derive(Clone, Copy)]
pub(crate) enum InventoryAttribution<'a> {
    Applied(&'a [u8]),
    NotApplied,
    Unresolved,
}

impl<'a> From<&'a IcCaptureAttribution> for InventoryAttribution<'a> {
    fn from(value: &'a IcCaptureAttribution) -> Self {
        match value {
            IcCaptureAttribution::Applied { snapshot_id, .. } => Self::Applied(snapshot_id),
            IcCaptureAttribution::NotApplied { .. } => Self::NotApplied,
            IcCaptureAttribution::Unresolved { .. } => Self::Unresolved,
        }
    }
}

impl<'a> From<&'a IcSnapshotUploadAttribution> for InventoryAttribution<'a> {
    fn from(value: &'a IcSnapshotUploadAttribution) -> Self {
        match value {
            IcSnapshotUploadAttribution::Applied { snapshot_id, .. } => Self::Applied(snapshot_id),
            IcSnapshotUploadAttribution::NotApplied { .. } => Self::NotApplied,
            IcSnapshotUploadAttribution::Unresolved { .. } => Self::Unresolved,
        }
    }
}

pub(crate) fn admitted_outcome(
    inventory: &IcSnapshotReply<'_>,
    candidates: &[&IcSnapshotInfo],
    attribution: InventoryAttribution<'_>,
) -> Result<(ObservationOutcomeRecord, Option<usize>), ()> {
    match attribution {
        InventoryAttribution::Applied(snapshot_id) => {
            new_candidate_index(inventory, candidates, snapshot_id)
                .map(|index| (ObservationOutcomeRecord::Applied, Some(index)))
                .ok_or(())
        }
        InventoryAttribution::NotApplied => Ok((ObservationOutcomeRecord::NotApplied, None)),
        InventoryAttribution::Unresolved => Ok((ObservationOutcomeRecord::Uncertain, None)),
    }
}

pub(crate) fn new_candidate_index(
    inventory: &IcSnapshotReply<'_>,
    candidates: &[&IcSnapshotInfo],
    snapshot_id: &[u8],
) -> Option<usize> {
    if !candidates
        .iter()
        .any(|snapshot| snapshot.id() == snapshot_id)
    {
        return None;
    }
    inventory
        .snapshots()
        .binary_search_by(|snapshot| snapshot.id().cmp(snapshot_id))
        .ok()
}
