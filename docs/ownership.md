# Ownership boundary and extraction cleanup

Reviewed 2026-10-18 against the pending working tree over released `ic-backup`
0.31.0, commit `3317b4e6e49c0fe59e343069f68eb539324c0612`. This review uses the vendored
[flow convergence](../audits/flow-convergence-and-duplication.md) and
[module cleanup](../audits/module-cleanup.md) methods from Shared Tooling
0.24.0 at `cab12cd3dfa4fdb34846b25397f0d72881bb9ae0`, with the local
[overlay](../AGENTS.md). The repeat review preserves the incoming uncommitted
implementation, Platform adoption and tooling work.
It inventories maintained source, manifests, tests, fixtures, scripts, CI and
active architecture guidance, with focused traces through the shared owners. Historical provenance and audit reports are retained as
evidence rather than treated as current interfaces. Platform adoption is complete;
the repeat review removes unused producer fixtures and corrects active ownership
guidance. Downstream application qualification still belongs to its owner.

## What this repository owns

IC Backup provides host-side foundations for same-release restoration to the same
existing canister IDs. It owns exact operation bindings, persisted records and
journals, bounded spending, snapshot layouts and coverage, guarded local progress,
reconciliation and workflow coordination. These mechanisms do not establish fresh
application authority or complete application backup/restore safety.

The [Core](../crates/ic-backup/Cargo.toml) and
[Agent](../crates/ic-backup-agent/Cargo.toml) manifests publish two libraries.
Reusable capture/restore operator composition uses the original workflow and journal
owners, bounded advancement and typed inspection. There is no standalone CLI,
canister, application runtime, general scheduler or default application provider.

The ownership table follows current dependency adoptions. Historical cleanup
findings below retain their original review sources and qualification scope.

| Entry or capability | Authoritative owner and relationship |
| --- | --- |
| `ic_backup::{model, policy, view}` | Backup records, validation and read-only projections; policy performs no IO |
| `ic_backup::{ops, workflow}` | Guarded product persistence and ordered single-step coordination over explicit ports |
| `ic_backup::ports` | Integration contracts for fresh membership, permissions, consistency, restore safety and separately accounted IC effects |
| `ic_backup_agent::AgentTransport` and providers | Separate package depending on Core and Platform’s native Agent feature; uses Platform for all SDK exports and keeps explicit Reqwest policy; signs and submits one original reserved update after mandatory retention |
| Artifact hash primitives and directory framing | `ic-host-artifacts`; Backup translates raw digests into its persisted checksum records |
| Descriptor IO, publication and filesystem identity | `ic-host-fs`; Backup selects record budgets, closed snapshot layouts and recovery transitions |
| Inherited process custody | `ic-host-process`; Backup binds custody to its original journal and quiescence requirements |
| Candid decoder setup and exact-result mechanics | Published `ic-platform` with its `candid` feature; Backup retains bounded wire projections, method quotas and redacted errors |
| Management DTOs and principal identity | Platform’s management exports over exact upstream 0.11.0; owned root `Principal` for internal admission, contract/external principal exports for Candid and SDK boundaries |
| Simulator process/assets and PocketIC SDK | Development-only Testkit owns managed servers/assets and harness extensions; Platform's development-only `pocket-ic` feature owns clients/DTOs. Fixture intent, bytes, logs and application assertions remain here |
| Common developer/release tooling | Exact reviewed Shared Tooling snapshot; consumer package, registry, delivery and resume policy remains local |
| Fleet membership, routing, release identity, quiescence and external obligations | Canic or another application integration, through Core ports; Core has no Canic runtime/control-plane dependency |

Core does not depend on Agent. Applications can use the public Core ports with
another explicitly qualified implementation. Agent's public write entry is
`AgentUpdateProvider`, implementing mutation and source-bound upload ports.
Transfer reads, recovery observations and fence acquisition have their own
providers and retain their distinct request/error/evidence contracts.
`PreparedUpdate` is a passive retention view, not a public preparation or reissue
API. The [transport contract](agent-transport.md) owns its detailed limits.

## Cleanup supported by current evidence

**LOW — parallel region hashing state.** The [snapshot artifact writer](../crates/ic-backup/src/ops/persistence/download_journal/ic_snapshot_artifact/mod.rs) kept files
and separate SHA-256 states in matching arrays, then manually updated both after
writes. The fields trace to the original 2026-10-06 implementation (`9478fcae`),
before the currently selected Host writer inspection API. Published Host Artifacts 0.27.1 already supplies `HashingWriter`, including
borrowed sink access and accepted-byte identities. Regions now use that writer
without another adapter. Coverage still owns contiguous ranges and metadata
ceilings; Backup retains descriptor/path/extent checks, closed-tree verification,
exact v1 framing, journal transitions and publication. Hash identity is consumed
only after complete coverage and successful writes. `sha2` is now a direct test
dependency only, retaining independent digest expectations. It remains available
transitively through the canonical cryptographic owners.

**LOW — producer-only checksum regression.** One integration case compared Host
FS checksums with Host Artifacts declarations without calling Backup. Host FS
0.27.1 already owns that exact framing regression. The redundant case is removed;
Backup's public publication/recovery case still proves changed bytes are rejected
and original declarations cannot bypass fresh verification in the
[retained consumer case](../crates/ic-backup/tests/directory_checksum.rs).
Private adapter tests
remain because they verify Backup records and error projections. Record-only
case normalization and mismatch checks stranded under filesystem ops duplicated
stronger model tests; those duplicates are removed. The retained byte-vector
regression now lives beside the checksum record in `model::artifacts`.

**LOW — obsolete active guidance.** The implemented-boundary summary still called
transport unimplemented; the transport guide instructed callers to use public
`prepare`/`submit` methods removed in 0.28. Those descriptions now match the public
providers and retention view. The long pre-release handoff is archived verbatim
in the same directory, preserving its links and receipts; the current handoff
identifies released 0.31.0 and the compatible pending batch. Remaining Canic
prefixes in newly allocated test-directory labels now use this repository's name.
No production path, record, golden wire fixture or license attribution is renamed.

**Producer qualification copied into the consumer.** Shared Tooling owns its
generic helper regression matrices. Backup had retained uncalled producer test
entrypoints, and its tooling gate ran the generic dependency-checker matrix in
addition to checking the real workspace policy. The canonical exporter retires
those roots and their unused companions at the same committed revision. Backup
keeps the actual dependency check, native tool admission, independent consumer
graphs and tests for its Make, hook, release, failure-retention and evidence
contracts. The [selection review](shared-tooling-review.json) records the exact
retirement; no retained shared payload is edited. This changes contributor
qualification selection, with no library, wire or persistence change.

## Deliberate retention and unresolved coordination

**Retained product policy.** Backup selects sequence counts, aggregate bounds and
decoder quotas. Platform supplies the bounded typed Candid sequence visitor and
strict reserved marker, including excessive-hint refusal, typed excess values and
zero-skip admission. Persisted-record sequence visitors and aggregate checks stay
local. Preserve the original wire and aggregate regressions when changing product
policy; none of these adapters supplies authentication or effect authority.

Candid remains the external management protocol through the completed
[Platform boundary](ic-platform.md). There is no direct Candid declaration.
IC Wire’s Postcard contracts remain application-owned and do not replace
management encoding.

Retain error/record adapters and fresh descriptor checks: they translate product
schemas or defend corruption, custody and recovery boundaries. Retain the Core
and Agent simulator suites: one qualifies Core effects/workflows and the other
qualifies actual signed Agent HTTP/gateway behavior. Their handwritten WAT sources
are distinct qualification inputs: Core uses two heap pages and one stable page,
Agent uses 16 pages each for full MiB transfers, and fence reconciliation exercises
separate acquisition/observation state. No generated implementation requires
consolidation. Release/package fixtures retain consumer graph, archive and exact
resume-policy assertions; snapshot-owned tooling is deliberately vendored and
must be refreshed through its exporter rather than locally rewritten. Shared IC
executables are optional manual tools, outside required setup, CI and release
preflight; they do not reintroduce the retired ICP product transport. Testkit's
authenticated server and offline admission remain required for simulator checks.

Read-only sibling inspection confirmed Host, Testkit, Wire, Platform, Shared
Tooling and Canic ownership. Selected registry source, not uncommitted sibling
code, determined usable APIs. Canic's current `crates/canic-backup/src/artifacts`
already calls Host FS for directory checksums and translates to Backup records
and errors. Its remaining application filesystem policy and runner are Canic's
responsibility; this is no longer a missing generic-helper migration. No Canic
build or live IC/application qualification is claimed by this review.

## Verification

The original artifact-writer regressions and both checksum integration cases
passed before cleanup. Focused post-change checks cover snapshot byte goldens,
empty/interleaved/maximum regions, changed descriptors and extents, corrupted
same-length content, retained journals, publication/recovery and real PocketIC
transfer/upload/restore. Independent consumers check the normal dependency graph
and Rust 1.91 without simulator feature unification. Exact commands, results and
remaining host coverage are recorded in the [current handoff](status/current.md).
No complete local release gate or sibling suite is implied by this audit.
