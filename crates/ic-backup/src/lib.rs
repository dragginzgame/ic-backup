//! Host-side snapshot backup and same-release recovery for Internet Computer canisters.
//!
//! The library provides artifact checksums, no-follow traversal and staging,
//! verified durable directory publication, bounded JSON persistence and journal
//! locking, layout lifetime exclusion, durable restore dependencies and
//! inherited command custody. Local download journals retain exact snapshot
//! identity and verified publication progress. Local attempt journals bind exact
//! declared identity and finite mutation/observation allowances, retaining durable
//! reservations and qualified receipts. These mechanisms do not authorize canister
//! effects.
//!
//! Applications own membership, release identity, control routing, quiescence
//! and external-effect settlement. Capture/restore runners and an IC transport
//! have not been extracted yet. Filesystem access and credentials remain on the
//! operator host.

mod hash;
pub mod model;
pub mod ops;

#[cfg(test)]
mod test_support;
