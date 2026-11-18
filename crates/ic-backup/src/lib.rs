//! Host-side Internet Computer capture, same-canister restore and local inventory.
//!
//! The Unix [`store`] API binds immutable approvals to a direct or Root-managed route and
//! retains original signed/certified evidence with finite one-shot spending.
//! Fixed capture and source-bound restore procedures share the same recorder,
//! explicit recovery and historical replay. Canic supplies the Root adapter. Integrations
//! own application fencing, external obligations and permission to resume.

// Store custody relies on Unix directory handles and file permissions in ic-host-fs.
#[cfg(unix)]
pub mod store;
