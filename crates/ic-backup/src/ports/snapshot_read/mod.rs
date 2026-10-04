//! Fallible current snapshot-read permission observation; no installed provider.

use crate::model::snapshot_read::{SnapshotReadObservation, SnapshotReadRequest};
use thiserror::Error;

/// Integration-owned authenticated snapshot visibility and optional complete controller evidence.
///
/// Implementations qualify actual context/target/snapshot visibility, exact viewer
/// identities, controller evidence when used, challenge freshness, coherent custody
/// and prior approved per-call accounting. Status/log visibility and Root-configured
/// declarations cannot stand in for current snapshot settings. Unknown visibility
/// must fail. A result does not perform the requested list or prove effect settlement,
/// transfer completeness, mutation control, lifecycle safety or application fencing.
/// No default provider or paid-call workflow is installed. Terminal replay must not
/// invoke this port; fresh live verification is a separate operation.
pub trait SnapshotReadProvider {
    /// Observe permissions for exact original-operation-bound snapshot-list bytes.
    /// # Errors
    /// Unavailable/unsupported reject before effects; indeterminate retains consumed
    /// spending/evidence and stops without retry. A missing required provider fails unavailable.
    fn observe_snapshot_read(
        &mut self,
        request: &SnapshotReadRequest<'_>,
    ) -> Result<SnapshotReadObservation, SnapshotReadProviderError>;
}
/// Redacted provider failure; no error grants dispatch or replenishment.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum SnapshotReadProviderError {
    /// No qualified provider; reject before remote effects.
    #[error("snapshot read provider unavailable")]
    Unavailable,
    /// Cannot satisfy the contract; reject before remote effects.
    #[error("snapshot read contract unsupported")]
    Unsupported,
    /// Actual observation/reply/accounting uncertain; retain evidence and stop.
    #[error("snapshot read observation indeterminate")]
    Indeterminate,
}
