//! Provider-driven steps under retained original plans and spending.
//!
//! Integrations supply fresh admission and authenticated providers. These bounded
//! steps compose through the original workflow coordinator with bounded advancement
//! and typed inspection. Terminal/reference release retains its separate guarded owner.

#[cfg(unix)]
pub mod execution_workflow;
pub mod fence_acquisition;
pub mod fence_reconciliation;
mod ic_mutation;
pub mod ic_snapshot_capture;
#[cfg(unix)]
pub mod ic_snapshot_download;
pub mod ic_snapshot_metadata;
#[cfg(unix)]
pub mod ic_snapshot_restore;
pub mod ic_snapshot_transfer_read;
pub mod ic_snapshot_upload;
mod qualified_receipt;
