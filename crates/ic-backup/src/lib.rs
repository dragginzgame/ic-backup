//! Host-side Internet Computer capture, same-canister restore and local inventory.
//!
//! The Unix [`store`] API binds immutable approvals to one direct connection and
//! retains original signed/certified evidence with finite one-shot spending.
//! Fixed capture and source-bound restore procedures share the same recorder,
//! explicit recovery and historical replay. Integrations own continuous application
//! fencing, external obligations and later acceptance/start. Canic adapters remain
//! unqualified; production depends on published Platform and Host boundaries.

#[cfg(unix)]
pub mod store;
