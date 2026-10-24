//! Host-side snapshot backup and same-release recovery for Internet Computer canisters.
//!
//! [`model`] owns bounded records, exact identities, IC wire formats and valid state
//! transitions. [`policy`] validates evidence and projects progress without IO.
//! [`ops`] owns guarded persistence, artifact staging and original-journal transitions.
//! [`workflow`] coordinates snapshot capture, transfer, upload, restore and recovery
//! through integration-supplied async [`ports`] and mandatory admission callbacks.
//! [`view`] exposes read-only projections across those boundaries.
//!
//! Complete operation plans retain inventory, dependency graphs and original attempt
//! limits. Progress derives from the complete original journals and qualified receipts.
//! Lost replies preserve pending spending; they never authorize blind retries, reset
//! allowances or replacement plans. Same-release restore targets existing canister IDs.
//! Artifact verification retains exact source bindings and requires stable byte custody.
//!
//! Applications own fresh membership, control/read permissions, consistency, fencing,
//! restore safety and external-effect settlement. Admission uses the existing passive
//! evidence models and pure validators; no default application authority is installed.
//! Records, matching hashes and decoded replies alone do not authorize effects or
//! establish authenticated outcomes. Terminal replay is effect-free, and failures
//! retain unfinished journals, artifacts, source references and fence obligations.
//!
//! The separate `ic-backup-agent` crate implements configured single-update transport.
//! Complete application runners and terminal/release evidence remain integration-owned.
//! Host supplies generic filesystem, process and hash capabilities; Platform supplies
//! external IC identities, codecs and SDK boundaries. Product contracts remain local,
//! with filesystem access and credentials on the operator host.

pub mod model;
pub mod ops;
pub mod policy;
pub mod ports;
pub mod view;
pub mod workflow;

#[cfg(test)]
mod test_support;
