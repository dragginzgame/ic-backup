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
//! Explicit fresh download integrity checks bind the retained original plan and
//! exact selected set to published directory bytes without changing journals,
//! replenishing allowances or releasing dependencies. Stable byte custody and
//! backend transfer completeness remain integration-owned.
//! Immutable local download manifests reuse that exact v1 journal schema after
//! fresh byte verification. Exact replay reads original retained records only,
//! preserving snapshot/checksum provenance even when artifact trees are absent.
//! Explicit local restore-source verification joins both original plans, retained
//! safety requirement and exact local manifest before checking every source tree.
//! Borrowed same-ID selection views grant no upload/load, application or release authority.
//! Private operation-bound artifact copies check original and copied checksums;
//! explicit retained-copy verification recovers without re-reading source trees.
//! Failed copies and drop retain evidence, spending and source references without cleanup.
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
//! Bounded capture/inventory reply decoding preserves raw snapshot identity and
//! exact request/reply evidence without authenticating origin or settling effects.
//! Pure inventory comparison exposes new candidates and rejects baseline drift;
//! candidate cardinality never attributes or settles a lost capture.
//! Bounded lifecycle replies retain exact empty acknowledgements and required
//! status/controller projections without converting them into fresh authority.
//! Exact IC mutation envelopes bind already reserved original updates; a single-call
//! provider contract and bounded passive reply association preserve pending spending.
//! Existing codecs decode replies without automatic settlement, retries or release.
//! Exact staged restore copies have separate durable publication/recovery and fresh
//! canonical-copy verification, retaining original accounting and source references.
//! Exact reserved status/list observations also retain both original attempt identities;
//! passive association leaves lost observations pending and proves no effect outcome.
//! A separate membership port binds ephemeral provider results to original intent,
//! exact current context/full inventory and an integration-owned challenge.
//! Pure matching views grant neither controller authority nor application continuity.
//! The separate control port checks direct caller-controller evidence for exact
//! IC mutation payloads; its matching views still grant no dispatch or restore safety.
//! A separate snapshot-read port checks current snapshot-list visibility and exact
//! caller read paths, without granting mutation control or settling lost replies.
//! Immutable consistency requirements retain the original requested guarantee;
//! current target/fence checks acquire or release no application obligations.
//! Immutable restore safety requirements bind exact original source and same-release
//! targets; fresh application evidence checks load/start safety without effects,
//! lost-load settlement or fence/reference release.
//! Immutable fence obligations retain original scope and acquisition identity;
//! recovery joins their exact original attempt journals without new accounting,
//! automatic release or a claim of current Active custody.
//! Reserved fence-acquisition reconciliation matches original-request attribution
//! under exact pending mutation and observation identities. Passive claims and
//! late-reply admission perform no automatic settlement, redispatch or release;
//! authenticated providers and actual application effects remain integration-owned.
//! Exact application acquisition envelopes bind receiver, update mode, method and
//! opaque bytes to already reserved original mutations. Passive acknowledgement
//! association establishes no acquisition outcome or fresh dispatch permission.
//! Immutable execution settlement checkpoints bind complete original Applied
//! journals and their exact chronological histories for local replay. They prove
//! no full backup/restore completion, command quiescence or fence/reference release.
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
