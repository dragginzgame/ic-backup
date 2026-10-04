//! Pure decisions over validated declarations; no IO, scheduling or persisted mutations.

pub mod consistency;
pub mod control_authority;
pub mod effect_order;
pub mod execution_progress;
pub mod membership;
pub mod selection;
pub mod snapshot_read;
