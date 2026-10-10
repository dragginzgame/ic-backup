//! Provider-driven steps under retained original plans and spending.
//!
//! Integrations supply fresh admission and authenticated providers. These bounded
//! steps do not implement complete backup/restore or terminal/reference release.

pub mod ic_snapshot_capture;
#[cfg(unix)]
pub mod ic_snapshot_download;
pub mod ic_snapshot_metadata;
#[cfg(unix)]
pub mod ic_snapshot_restore;
pub mod ic_snapshot_transfer_read;
pub mod ic_snapshot_upload;
