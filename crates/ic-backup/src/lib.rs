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
//! Bounded physical inventories retain canonical declared parent forests; pure
//! selection policy expands exact principals without live discovery or authority.
//! Explicit effect graphs retain validated operation dependencies and project
//! deterministic planning order/readiness without authorizing dispatch.
//! Immutable operation plans bind these declarations to exact target/request digests
//! and original attempt allowances, deriving journal authority under the full plan digest.
//! Pure execution progress joins exact retained journals to the original plan and
//! projects causal Applied evidence, pending attempts and exhaustion without dispatch.
//! The IC request boundary encodes closed host-ingress management operations and
//! binds exact method/routing/Candid bytes to original mutation or observation digests.
//! Codec qualification does not establish IC effects or fresh execution authority.
//! A separate membership port binds ephemeral provider results to original intent,
//! exact current context/full inventory and an integration-owned challenge.
//! Pure matching views grant neither controller authority nor application continuity.
//! The separate control port checks direct caller-controller evidence for exact
//! IC mutation payloads; its matching views still grant no dispatch or restore safety.
//!
//! Applications own membership, release identity, control routing, quiescence
//! and external-effect settlement. Capture/restore runners and an IC transport
//! have not been extracted yet. Filesystem access and credentials remain on the
//! operator host.

mod hash;
pub mod model;
pub mod ops;
pub mod policy;
pub mod ports;

#[cfg(test)]
mod test_support;
