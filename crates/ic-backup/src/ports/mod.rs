//! Fallible integration contracts; implementations own qualified observations and effects.

pub mod consistency;
pub mod control_authority;
pub mod fence_acquisition;
pub mod fence_reconciliation;
pub mod ic_mutation;
pub mod ic_observation;
pub mod ic_snapshot_transfer_read;
pub mod ic_snapshot_upload;
pub mod ic_snapshot_upload_data_observation;
pub mod ic_snapshot_upload_observation;
pub mod membership;
pub mod restore_safety;
pub mod snapshot_read;
