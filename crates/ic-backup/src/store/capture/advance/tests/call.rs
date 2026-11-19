//! Finite capture calls; no arbitrary management dispatch.

use super::super::{CapturePolicy, OperationStoreError, arguments};
use serde::{Deserialize, Serialize};

/// The management calls currently implemented by the capture workflow.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum CaptureCall {
    /// Check the original stopped/module/controller preconditions.
    Status,
    /// Observe the original snapshot inventory before allocating.
    ListSnapshots,
    /// Take one new snapshot, without replacement or uninstalling code.
    TakeSnapshot,
}
impl CaptureCall {
    /// Exact external method name. This projection grants no dispatch authority.
    #[must_use]
    pub const fn method_name(self) -> &'static str {
        match self {
            Self::Status => "canister_status",
            Self::ListSnapshots => "list_canister_snapshots",
            Self::TakeSnapshot => "take_canister_snapshot",
        }
    }

    pub(in crate::store::capture) fn arguments(
        self,
        plan: &CapturePolicy,
    ) -> Result<Vec<u8>, OperationStoreError> {
        match self {
            Self::Status | Self::ListSnapshots => arguments(plan),
            Self::TakeSnapshot => {
                // Only build canonical bytes here. The workflow independently admits
                // its authenticated inventory and original capacity before reservation.
                let args = plan
                    .take_snapshot_arguments(&[], 1)
                    .map_err(|_| OperationStoreError::StatusAdmission)?;
                (args)
                    .encode(512)
                    .map_err(|_| OperationStoreError::StatusAdmission)
            }
        }
    }
}
