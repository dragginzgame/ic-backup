//! Bounded list/take replies and non-destructive capture arguments; no IC effects.
use super::CapturePolicy;
use ic_platform::{
    candid_codec::{
        self, Encode, Type, Writer,
        decoder::{Decode, Error as DecodeError, Limits, Reader, decode_one_with_skip_limit},
    },
    management_canister_types::{Snapshot, TakeCanisterSnapshotArgs},
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
    let Inventory(snapshots) = decode(reply)?;
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
    let BoundedSnapshot(snapshot) = decode(reply)?;
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
    ) -> Result<TakeCanisterSnapshotArgs, SnapshotReplyError> {
        if !(1..=MAX_SNAPSHOTS as u32).contains(&capacity) {
            return Err(SnapshotReplyError::CapacityLimit);
        }
        validate_inventory(inventory)?;
        if inventory.len() >= capacity as usize {
            return Err(SnapshotReplyError::NoCapacity);
        }
        Ok(TakeCanisterSnapshotArgs {
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

fn decode<'a, T: Decode<'a>>(reply: &'a [u8]) -> Result<T, SnapshotReplyError> {
    if reply.len() > MAX_REPLY_BYTES {
        return Err(SnapshotReplyError::ReplyTooLarge);
    }
    decode_one_with_skip_limit(
        reply,
        Limits {
            bytes: MAX_REPLY_BYTES,
            header: 64 * 1024,
            types: 16,
            work: 2 * 1024 * 1024,
            allocation: 2 * 1024 * 1024,
        },
        0,
    )
    .map_err(SnapshotReplyError::Decode)
}

// Private bounds over the supplier's schema, not replacement management DTOs.
struct BoundedSnapshot(Snapshot);
impl Encode for BoundedSnapshot {
    const TYPE: &'static Type = Snapshot::TYPE;
    fn write(&self, output: &mut Writer) -> Result<(), candid_codec::Error> {
        self.0.write(output)
    }
}
impl<'a> Decode<'a> for BoundedSnapshot {
    fn read(reader: &mut Reader<'a>, wire: i64) -> Result<Self, DecodeError> {
        let Type::Record(fields) = Snapshot::TYPE
            .resolved()
            .map_err(|_| DecodeError::Unsupported)?
        else {
            return Err(DecodeError::Unsupported);
        };
        if fields.len() != 3 {
            return Err(DecodeError::Unsupported);
        }
        let (mut id, mut size, mut timestamp) = (None, None, None);
        reader.record(wire, false, |label, inner, reader| {
            if label == fields[0].0 {
                let bytes = reader.bounded_vec(inner, MAX_ID_BYTES)?;
                if bytes.is_empty() {
                    return Err(DecodeError::Type);
                }
                id = Some(bytes);
            } else if label == fields[1].0 {
                size = Some(reader.value(inner)?);
            } else if label == fields[2].0 {
                timestamp = Some(reader.value(inner)?);
            } else {
                return Ok(false);
            }
            Ok(true)
        })?;
        Ok(Self(Snapshot {
            id: id.ok_or(DecodeError::Type)?,
            total_size: size.ok_or(DecodeError::Type)?,
            taken_at_timestamp: timestamp.ok_or(DecodeError::Type)?,
        }))
    }
}
struct Inventory(Vec<Snapshot>);
impl Encode for Inventory {
    const TYPE: &'static Type = <Vec<Snapshot> as Encode>::TYPE;
    fn write(&self, output: &mut Writer) -> Result<(), candid_codec::Error> {
        self.0.write(output)
    }
}
impl<'a> Decode<'a> for Inventory {
    fn read(reader: &mut Reader<'a>, wire: i64) -> Result<Self, DecodeError> {
        let snapshots = reader.bounded_vec::<BoundedSnapshot>(wire, MAX_SNAPSHOTS)?;
        Ok(Self(
            snapshots.into_iter().map(|snapshot| snapshot.0).collect(),
        ))
    }
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

#[cfg(test)]
mod tests;
