//! Local inventory, payload integrity and capture intent for Internet Computer backups.
//!
//! The Unix [`store`] API initializes a private store and reads bounded completion
//! metadata. It can check payload bytes against a caller-trusted manifest hash.
//! It also persists approved capture intent and durable target ownership, with
//! read-only reopening under an exclusive operation lock. It does not capture snapshots, authenticate operation evidence or restore canisters.
//! Snapshot transport and application coordination remain unimplemented.

#[cfg(unix)]
pub mod store;
