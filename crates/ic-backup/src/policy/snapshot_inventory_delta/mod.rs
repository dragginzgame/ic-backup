//! Pure snapshot inventory comparison, without capture attribution or settlement.

use crate::model::{
    ic_request::{IcManagementMethodRecord, IcManagementRequestRecord},
    ic_snapshot_reply::{IcSnapshotInfo, IcSnapshotReply},
};
use thiserror::Error;

/// Read-only new-snapshot candidates under exact capture and inventory declarations.
///
/// Even one candidate is not proof that the capture created it: another controller
/// may have acted. No result authorizes a receipt, retry, load, deletion or dispatch.
/// Integrations own authenticated association, baseline custody, observation timing
/// and exclusive attribution. An empty delta does not prove the capture failed.
#[derive(Debug)]
pub struct SnapshotInventoryDeltaView<'a> {
    capture: &'a IcManagementRequestRecord,
    baseline: &'a IcSnapshotReply<'a>,
    observed: &'a IcSnapshotReply<'a>,
    candidates: Vec<&'a IcSnapshotInfo>,
}

impl<'a> SnapshotInventoryDeltaView<'a> {
    /// Read the exact capture declaration, including its existing wire digest.
    #[must_use]
    pub const fn capture(&self) -> &'a IcManagementRequestRecord {
        self.capture
    }

    /// Read the original inventory declaration and exact raw-payload evidence.
    #[must_use]
    pub const fn baseline(&self) -> &'a IcSnapshotReply<'a> {
        self.baseline
    }

    /// Read the subsequent inventory declaration and exact raw-payload evidence.
    #[must_use]
    pub const fn observed(&self) -> &'a IcSnapshotReply<'a> {
        self.observed
    }

    /// Read zero, one or multiple new descriptors in exact raw-ID order.
    ///
    /// Cardinality is descriptive. A singleton grants no capture attribution;
    /// multiple candidates remain explicit rather than selecting an arbitrary ID.
    #[must_use]
    pub fn candidates(&self) -> &[&'a IcSnapshotInfo] {
        &self.candidates
    }
}

/// Compare admitted inventories for the exact declared new-snapshot capture target.
///
/// Both replies must be inventory replies for the same exact canonical target as
/// the capture. Every baseline ID must remain present with unchanged timestamp and
/// size. New IDs are projected in the reply owner's canonical order, retaining its
/// existing 1,024-entry and 256-ID-byte bounds without re-encoding or remote IO.
///
/// There is no clock, network/caller validation, effect receipt or spending change.
/// The integration must retain the original baseline before the attempted capture;
/// this function cannot establish chronology or authenticate supplied reply bytes.
///
/// # Errors
/// Rejects wrong request methods, mismatched targets, lost baseline entries and
/// changed metadata for a retained exact ID. No input or journal is mutated.
pub fn compare<'a>(
    capture: &'a IcManagementRequestRecord,
    baseline: &'a IcSnapshotReply<'a>,
    observed: &'a IcSnapshotReply<'a>,
) -> Result<SnapshotInventoryDeltaView<'a>, SnapshotInventoryDeltaError> {
    if capture.method() != IcManagementMethodRecord::TakeCanisterSnapshot {
        return Err(SnapshotInventoryDeltaError::WrongCaptureMethod);
    }
    let candidates = compare_inventories(capture.target(), baseline, observed)?;
    Ok(SnapshotInventoryDeltaView {
        capture,
        baseline,
        observed,
        candidates,
    })
}

/// Shared closed-baseline admission; candidates remain descriptive only.
pub(crate) fn compare_inventories<'a>(
    target: &str,
    baseline: &IcSnapshotReply<'_>,
    observed: &'a IcSnapshotReply<'_>,
) -> Result<Vec<&'a IcSnapshotInfo>, SnapshotInventoryDeltaError> {
    for reply in [baseline, observed] {
        if reply.request().method() != IcManagementMethodRecord::ListCanisterSnapshots {
            return Err(SnapshotInventoryDeltaError::WrongInventoryMethod);
        }
        if reply.request().target() != target {
            return Err(SnapshotInventoryDeltaError::TargetMismatch);
        }
    }
    let mut candidates = Vec::new();
    let mut previous = baseline.snapshots().iter().peekable();
    for snapshot in observed.snapshots() {
        match previous.peek() {
            Some(old) if old.id() < snapshot.id() => {
                return Err(SnapshotInventoryDeltaError::LostBaseline);
            }
            Some(old) if old.id() == snapshot.id() => {
                if *old != snapshot {
                    return Err(SnapshotInventoryDeltaError::ChangedBaselineMetadata);
                }
                previous.next();
            }
            _ => candidates.push(snapshot),
        }
    }
    if previous.next().is_some() {
        return Err(SnapshotInventoryDeltaError::LostBaseline);
    }
    Ok(candidates)
}

/// Typed comparison rejection, with no raw identifiers or payload diagnostics.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum SnapshotInventoryDeltaError {
    /// The supplied intent is not the closed non-replacing capture method.
    #[error("inventory comparison requires a new-snapshot capture request")]
    WrongCaptureMethod,
    /// A decoded singleton capture reply cannot substitute for an inventory.
    #[error("inventory comparison requires two snapshot-list replies")]
    WrongInventoryMethod,
    /// One inventory is declared under another canonical target.
    #[error("snapshot inventory target differs from capture target")]
    TargetMismatch,
    /// At least one retained baseline ID disappeared from the subsequent inventory.
    #[error("snapshot inventory lost a baseline identifier")]
    LostBaseline,
    /// A retained exact ID changed its declared timestamp or total size.
    #[error("snapshot inventory changed baseline metadata")]
    ChangedBaselineMetadata,
}

#[cfg(test)]
mod tests;
