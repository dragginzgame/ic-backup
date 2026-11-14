# Current handoff — 2026-11-14

Restart workspace at `/home/adam/projects/ic-backup`, branch `main`, with no
legacy history. Legacy source is on GitHub as `dragginzgame/ic-backup-legacy`.
The former local legacy path was absent; preserve retained journals and release
state wherever held. No legacy cleanup, import or migration was performed.

The virtual workspace contains one API-free `ic-backup` library at unpublished
0.68.0. No runtime dependencies, CLI, product schemas, providers, simulator
qualification or release automation are implemented. The user authorized initial
GitHub delivery to a separate private `dragginzgame/ic-backup` repository on `main`.
The legacy repository is preserved. This is GitHub-only delivery: no version bump,
release tag or registry publication. Cargo metadata confirms the sole member
resolves to `publish = []` from workspace `publish = false`; the local overlay
requires the same restriction for future packages until publication is authorized.

## Selected design

The user selected a host-side single-canister library: capture an already-stopped
canister and restore to the same existing ID, ending stopped. Canic is a required
consumer alongside independent Rust use. The [library design](../design/library-design.md)
owns scope, storage rationale and custody. The [operation contract](../design/operation-contract.md)
now proposes consumer/provider operations, single submission with explicit
reconciliation, execution bindings, metadata comparison and transfer budgeting.
These are design decisions, not implemented or qualified behavior.

The user confirmed local-host storage for v1 and Blob integration for v2. The
[local-store contract](../design/local-store.md) adds manifest-based listing and
latest completed backup per network/canister, plus independent concurrent targets
under a Canic-owned task limit. Inventory reports verification at capture, not
fresh payload integrity. No database, generic store trait or batch engine is planned.
It selects a single framed ledger, completion manifest and durable target owner;
exact persisted field encodings and golden vectors remain implementation work.

The contract distinguishes source/destination creation metadata, preserves float
bits, refuses unsupported absent globals/lifecycle fields for the initial
round-trip profile, and reserves conservative whole-chunk space before download.
It fixes transfer ordering and derives management-call counts, including complete
restore readback. Encoded evidence bounds and concrete provider costs remain open.

The [sibling review](../design/sibling-integration.md) records source identities
and dependency ownership. Use published Platform for external IC DTOs/codecs and
Host FS/Artifacts for local primitives. Testkit belongs only in future private
qualification. Backup must not depend on Canic runtime/control-plane crates.

Canic's current manager alone controls its worker; its host uses external ICP
signing. Full integration requires a Canic-owned durable executor receipt protocol,
Platform-owned Wasm snapshot DTOs (currently native-only), and fenced application
reopening under surviving control-plane authority. Direct-host qualification will
not establish manager-route compatibility. Blob's older-owner snapshot activation
remains unsupported. No sibling source or dependency selection was changed.

The [legacy assessment](../design/legacy-assessment.md) separates guarantees to
retain from composition to replace and features to defer. Source inspection
confirms shared transport dispatch and meaningful retained-record roles; do not
remove checks as apparent duplication. The simpler initial reconciliation rule
retains uncertainty where legacy's qualified state-based settlement may resolve it.
No legacy test suite was rerun or product source copied.

Next: implement the local-store writer/reader with exact record fields, golden
bytes and interruption/inventory tests; select a concrete application safety
provider and published suppliers for capture. Then implement the coherent
capture/verify consumer followed by
upload/readback/load through the same contract. Do not infer that DTO helpers or
an empty test suite complete the selected usable library.

## Foundation and validation

Foundation uses Rust 1.100.0 and authenticated Toko Tooling 0.43.0, confirmed as
the latest non-yanked stable sparse-index release during setup. The exporter added
the shared pre-commit entrypoint to the original eight selections: the snapshot
now verifies 51 files. `make install-hooks` activated the repository-local hook.
Dependency listing, scoped tooling update, PATH `toko-cloc -d 0`, tool admission,
formatting and documentation links passed. No dependency version changed.

The original complete offline scaffold check passed snapshot/Cargo admission,
links, formatting, strict Clippy, Rustdoc and an empty test suite. It establishes
no IC behavior or hosted macOS qualification. LOC remains zero implementation.
Setup/export evidence is retained under `target/evidence/bootstrap-20261114/`
and `target/evidence/tooling-setup-20261114/`.

Design evidence uses legacy commit `4ae087fad8c23de712aa60bf569bde4b2b932651`
and the sibling identities linked in the design documents. The operation-contract
review additionally checked Platform's local snapshot DTOs and the live management
specification. Historical or supplier qualification does not qualify this restart.
Latest documentation-only validation passed 42 local references across eleven
documents, scoped whitespace and before/after consistency review. No Rust or IC
suite ran for this prose batch.
