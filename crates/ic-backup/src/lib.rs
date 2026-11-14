//! Local inventory, payload integrity and capture intent for Internet Computer backups.
//!
//! The Unix [`store`] API initializes a private store and reads bounded completion
//! metadata. It can check payload bytes against a caller-trusted manifest hash.
//! It also persists approved capture intent and durable target ownership, with
//! read-only reopening under an exclusive operation lock. Initial direct-host status
//! continuation durably reserves effects and delegates authentication and application
//! admission to a caller-qualified provider. A bounded one-shot HTTP helper handles
//! retained status ingress and authenticates direct-key signatures and certificate evidence. No complete
//! provider, snapshot capture,
//! completion or restore implementation is supplied.

#[cfg(unix)]
pub mod store;
