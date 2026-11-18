//! Fixed metadata request arguments; no independent acquisition workflow.
use super::{CapturePolicy, OperationStoreError};
use ic_platform_protocol::management_snapshots::{Snapshot, SnapshotArgs};
use thiserror::Error;
/// Metadata identity or bounded request encoding refused.
#[derive(Debug, Error)]
pub enum MetadataReadError {
    /// Empty or oversized snapshot identity.
    #[error("invalid metadata snapshot identity")]
    SnapshotId,
    /// Original argument encoding allowance refused.
    #[error(transparent)]
    Admission(#[from] OperationStoreError),
}
impl CapturePolicy {
    /// Build the fixed metadata-read arguments for this target and exact snapshot ID.
    ///
    /// The integration must authenticate the original take result and preserve its
    /// custody. This pure helper neither proves that provenance nor reserves a call.
    /// Receiver is the management canister; effective routing is this plan's target.
    /// # Errors
    /// Refuses empty/oversized snapshot IDs.
    pub fn metadata_read_arguments(
        &self,
        snapshot: &Snapshot,
    ) -> Result<SnapshotArgs, MetadataReadError> {
        if !(1..=256).contains(&snapshot.id.len()) {
            return Err(MetadataReadError::SnapshotId);
        }
        Ok(SnapshotArgs {
            canister_id: self.request().target.canister,
            snapshot_id: snapshot.id.clone(),
        })
    }
}
