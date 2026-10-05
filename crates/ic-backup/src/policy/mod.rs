//! Pure decisions over validated declarations; no IO, scheduling or persisted mutations.

pub mod consistency;
pub mod control_authority;
pub mod download_integrity;
pub mod effect_order;
pub mod execution_progress;
pub mod fence_obligation;
pub mod membership;
pub mod restore_safety;
pub mod selection;
pub mod snapshot_inventory_delta;
pub mod snapshot_read;
