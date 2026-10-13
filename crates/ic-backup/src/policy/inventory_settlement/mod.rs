//! Common exact identity and explicit new-candidate checks for inventory settlements.

use crate::model::{
    artifacts::ArtifactChecksumRecord,
    ic_snapshot_reply::{IcSnapshotInfo, IcSnapshotReply},
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
        if self.authority != expected.authority {
            return Err(InventorySettlementMismatch::Authority);
        }
        if self.mutation_attempt != expected.mutation_attempt
            || self.observation_attempt != expected.observation_attempt
        {
            return Err(InventorySettlementMismatch::Attempt);
        }
        if self.challenge != expected.challenge {
            return Err(InventorySettlementMismatch::Challenge);
        }
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
