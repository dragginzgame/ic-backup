//! Pure decisions over validated declarations; no IO, scheduling or persisted mutations.

pub mod consistency;
pub mod control_authority;
pub mod download_integrity;
pub mod effect_order;
pub mod execution_progress;
pub mod execution_settlement;
pub mod fence_acquisition;
pub mod fence_obligation;
pub mod fence_reconciliation;
pub mod ic_mutation;
pub mod ic_observation;
pub mod local_restore_source;
pub mod membership;
pub mod restore_safety;
pub mod selection;
pub mod snapshot_inventory_delta;
pub mod snapshot_read;
