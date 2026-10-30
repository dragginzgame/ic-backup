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
    pub operation: SettlementIdentity<'a>,
    pub baseline: &'a ArtifactChecksumRecord,
    pub inventory: &'a ArtifactChecksumRecord,
    pub observation_evidence: &'a ArtifactChecksumRecord,
}

impl InventorySettlementIdentity<'_> {
    pub fn compare(&self, expected: &Self) -> Result<(), InventorySettlementMismatch> {
        self.operation.compare(&expected.operation)?;
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

/// Passive settlement claims differ from their independently qualified original evidence.
#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum InventorySettlementMismatch {
    /// Full original operation authority differs.
    #[error("inventory settlement authority differs")]
    Authority,
    /// Original mutation or observation attempt differs.
    #[error("inventory settlement attempts differ")]
    Attempt,
    /// Caller-owned qualification challenge differs.
    #[error("inventory settlement challenge differs")]
    Challenge,
    /// Exact retained baseline request and raw evidence differ.
    #[error("inventory settlement baseline differs")]
    Baseline,
    /// Exact current inventory request and raw evidence differ.
    #[error("inventory settlement inventory differs")]
    Inventory,
    /// Retained opaque observation evidence differs.
    #[error("inventory settlement observation evidence differs")]
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
    operation: &SettlementIdentity<'_>,
    observation_evidence: &ArtifactChecksumRecord,
) -> Result<Vec<&'a IcSnapshotInfo>, E>
where
    E: From<SnapshotInventoryDeltaError> + From<InventorySettlementMismatch>,
{
    let candidates = compare_inventories(target, baseline, inventory)?;
    claimed.compare(&InventorySettlementIdentity {
        operation: SettlementIdentity {
            authority: operation.authority,
            mutation_attempt: operation.mutation_attempt,
            observation_attempt: operation.observation_attempt,
            challenge: operation.challenge,
        },
        baseline: &baseline.digest(),
        inventory: &inventory.digest(),
        observation_evidence,
    })?;
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

// A snapshot index exists only for an explicitly attributed Applied claim.
#[derive(Debug)]
pub(crate) enum InventorySettlementOutcome {
    Applied(usize),
    NotApplied,
    Uncertain,
}

impl InventorySettlementOutcome {
    pub const fn outcome(&self) -> ObservationOutcomeRecord {
        match self {
            Self::Applied(_) => ObservationOutcomeRecord::Applied,
            Self::NotApplied => ObservationOutcomeRecord::NotApplied,
            Self::Uncertain => ObservationOutcomeRecord::Uncertain,
        }
    }

    pub const fn snapshot_index(&self) -> Option<usize> {
        match self {
            Self::Applied(index) => Some(*index),
            Self::NotApplied | Self::Uncertain => None,
        }
    }
}

pub(crate) fn admitted_outcome(
    inventory: &IcSnapshotReply<'_>,
    candidates: &[&IcSnapshotInfo],
    attribution: InventoryAttribution<'_>,
) -> Result<InventorySettlementOutcome, ()> {
    match attribution {
        InventoryAttribution::Applied(snapshot_id) => {
            new_candidate_index(inventory, candidates, snapshot_id)
                .map(InventorySettlementOutcome::Applied)
                .ok_or(())
        }
        InventoryAttribution::NotApplied => Ok(InventorySettlementOutcome::NotApplied),
        InventoryAttribution::Unresolved => Ok(InventorySettlementOutcome::Uncertain),
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
