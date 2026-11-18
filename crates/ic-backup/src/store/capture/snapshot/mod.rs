//! Bounded list/take replies and non-destructive capture arguments; no IC effects.
#[cfg(test)]
mod tests;

use super::CapturePolicy;
use ic_platform_protocol::{
    candid::{Error as DecodeError, Limits},
    management_snapshots::{
        Snapshot, TakeSnapshotArgs, decode_list_snapshots, decode_take_snapshot,
    },
};
use std::collections::BTreeSet;
use thiserror::Error;

const MAX_REPLY_BYTES: usize = 1024 * 1024;
const MAX_SNAPSHOTS: usize = 1024;
const MAX_ID_BYTES: usize = 256;

/// Decode an exact snapshot list, retaining its original order and metadata.
///
/// Limits are 1 MiB raw input, 1,024 entries and 1..=256 bytes per opaque ID.
/// Duplicate IDs refuse even when their metadata differs. Size and timestamp
/// retain the full u64 range; reported size is not local payload length.
/// The caller must retain and authenticate the original network/target/request
/// association. This performs no IO and supplies no capture or recovery authority.
///
/// # Errors
/// Refuses malformed/excessive replies, unknown/missing fields and duplicate IDs.
pub fn decode_snapshot_list(reply: &[u8]) -> Result<Vec<Snapshot>, SnapshotReplyError> {
    let snapshots =
        decode_list_snapshots(reply, limits(reply)?).map_err(SnapshotReplyError::Decode)?;
    validate_inventory(&snapshots)?;
    Ok(snapshots)
}

/// Decode the snapshot returned by the original take call, against its prior list.
///
/// A returned ID already present before capture refuses. Only the take reply
/// identifies its result: this never derives a successful take from a later list
/// difference. The caller must authenticate both original replies and retain them;
/// this helper neither settles an attempt nor grants a retry after a lost reply.
///
/// # Errors
/// Refuses invalid prior inventory, malformed/excessive replies and reused IDs.
pub fn decode_taken_snapshot(
    reply: &[u8],
    prior: &[Snapshot],
) -> Result<Snapshot, SnapshotReplyError> {
    validate_inventory(prior)?;
    let snapshot =
        decode_take_snapshot(reply, limits(reply)?).map_err(SnapshotReplyError::Decode)?;
    validate_inventory(std::slice::from_ref(&snapshot))?;
    if prior.iter().any(|existing| existing.id == snapshot.id) {
        return Err(SnapshotReplyError::AlreadyExisted);
    }
    Ok(snapshot)
}

impl CapturePolicy {
    /// Build direct-host take arguments after a bounded capacity preflight.
    ///
    /// `capacity` is the integration's selected remote capacity (1..=1,024), not
    /// a capacity discovered or guaranteed by this library. The original workflow
    /// must bind it and the authenticated inventory before spending. A list is
    /// never a reservation: remote capacity can still reject the later take.
    ///
    /// Arguments bind the plan's exact target, replace no existing snapshot,
    /// explicitly keep installed code and omit a sender canister version. Canic
    /// execution remains unsupported until its sender-version/receipt contract exists.
    /// This pure helper checks neither status nor application safety and grants
    /// no signing, submission, deletion, funding or retry authority.
    ///
    ///
    /// # Errors
    /// Refuses Canic routing, invalid inventory/capacity and an already full list.
    pub fn take_snapshot_arguments(
        &self,
        inventory: &[Snapshot],
        capacity: u32,
    ) -> Result<TakeSnapshotArgs, SnapshotReplyError> {
        if !(1..=MAX_SNAPSHOTS as u32).contains(&capacity) {
            return Err(SnapshotReplyError::CapacityLimit);
        }
        validate_inventory(inventory)?;
        if inventory.len() >= capacity as usize {
            return Err(SnapshotReplyError::NoCapacity);
        }
        Ok(TakeSnapshotArgs {
            canister_id: self.request().target.canister,
            replace_snapshot: None,
            uninstall_code: Some(false),
            sender_canister_version: None,
        })
    }
}

fn validate_inventory(snapshots: &[Snapshot]) -> Result<(), SnapshotReplyError> {
    if snapshots.len() > MAX_SNAPSHOTS {
        return Err(SnapshotReplyError::InventoryLimit);
    }
    let mut ids = BTreeSet::new();
    for snapshot in snapshots {
        if !(1..=MAX_ID_BYTES).contains(&snapshot.id.len()) {
            return Err(SnapshotReplyError::SnapshotId);
        }
        if !ids.insert(snapshot.id.as_slice()) {
            return Err(SnapshotReplyError::DuplicateSnapshotId);
        }
    }
    Ok(())
}

fn limits(reply: &[u8]) -> Result<Limits, SnapshotReplyError> {
    if reply.len() > MAX_REPLY_BYTES {
        return Err(SnapshotReplyError::ReplyTooLarge);
    }
    Ok(Limits {
        bytes: MAX_REPLY_BYTES,
        header: 64 * 1024,
        types: 16,
        work: 2 * 1024 * 1024,
        allocation: 2 * 1024 * 1024,
        vector: MAX_REPLY_BYTES,
        skipping: 0,
    })
}

/// Snapshot wire or non-destructive preflight refusal; no raw bytes in diagnostics.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum SnapshotReplyError {
    /// Raw input exceeds 1 MiB before decoding.
    #[error("snapshot reply exceeds 1 MiB")]
    ReplyTooLarge,
    /// Platform's bounded codec rejected the exact reply.
    #[error("snapshot decoding refused: {0:?}")]
    Decode(DecodeError),
    /// More than 1,024 inventory entries were supplied.
    #[error("snapshot inventory exceeds 1024 entries")]
    InventoryLimit,
    /// A supplied snapshot ID is empty or longer than 256 bytes.
    #[error("snapshot ID must contain 1..=256 bytes")]
    SnapshotId,
    /// The same opaque ID appears twice, irrespective of metadata.
    #[error("duplicate snapshot ID")]
    DuplicateSnapshotId,
    /// Capacity must be explicitly supplied within 1..=1,024.
    #[error("snapshot capacity must be within 1..=1024")]
    CapacityLimit,
    /// The observed list already fills or exceeds the selected capacity.
    #[error("no capacity for a new snapshot")]
    NoCapacity,
    /// The take response identifies a snapshot present in the prior inventory.
    #[error("returned snapshot already existed before capture")]
    AlreadyExisted,
}
