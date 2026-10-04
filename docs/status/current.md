# Current handoff — 2026-10-04

The maintainer completed `0.1.2` at `575d7d0` (Release 0.1.2) and requested
continued extraction grouped under `0.1.3`. This session extended the retained
uncommitted download-journal batch with durable per-operation attempt accounting.
Source, regressions, schemas and documentation remain uncommitted for review.
Canic and siblings remain read-only. No live IC effects, release/version
transactions, commits, pushes or package uploads ran.

`DownloadJournalRecord` retains v1 immutable intent and a nonempty exact physical
snapshot set: canonical principal, opaque snapshot token, observed timestamp/size,
fixed relative paths, four-state progress and required nullable checksum. Model
owns immediate Created → Downloaded → ChecksumVerified → Durable transitions;
views derive local resume state without granting remote retries. The borrowed
`DownloadJournalGuard` exclusively locks bounded records, verifies staging bytes
and durably publishes/adopts the matching artifact tree. Failed writes/publication
stop the guard until reopen. Backend complete-transfer attestation stays caller-owned.
The existing 1,024 artifact, 256 ASCII snapshot-token and 1 MiB read/write bounds,
no-overwrite creation and nonsymlink artifact-parent contracts remain enforced.

`AttemptJournalRecord` now binds exact intent, operation sequence, network
fingerprint, caller/target principals, release and mutating-request digests through
immutable `AttemptAuthorityRecord`. Original mutation and observation limits have
a checked combined maximum of 1,024 and never refund or replenish. Reservations
and receipts form append-only chronology; derived counters/flags are not persisted.
Global attempt numbers and exact request matching reject stale receipts. A pending
mutation blocks another mutation; a separately reserved observation can resolve it
after mutation exhaustion. Applied stops new reservations for that operation.
Uncertain requires a qualified settled observation unable to resolve mutation;
a lost observation reply remains pending until qualified settlement.

`AttemptJournalGuard` borrows layout exclusion and locks
`attempt-<operation_sequence>.json`. Creation preserves prior evidence; opening
requires the full exact original binding and budgets. A reservation is durably
published before returning its number. Any write failure disables the guard until
drop/reopen; recovery validates consumption instead of blindly retrying. History
admits at most 2,048 events; bounded input and canonical pretty-output each admit
at most 1 MiB. Canonical authority hashing uses a specified domain-separated binary
encoding and excludes mutable events and JSON formatting. Equivalent principal/hash
text normalizes at admission; both journals share the pure principal boundary.

These are local mechanisms, not authenticated intent, fresh IC authority, a
subprocess dispatch permit or qualified remote reconciliation. Receipt owners
retain evidence and qualify current authority, command custody and paid-effect
settlement. This ledger counts explicit caller reservations; opaque backend
retries, cycles and calls within commands require their own executor bounds.
Local replay performs no remote observations/effects. Applied operation evidence
and Durable artifact progress do not establish full terminal run completion,
verified manifest or permission to release restore references. See
[the maintained boundary](../extraction-boundary.md),
[download schema](../contracts/download-journal.schema.json) and
[attempt schema](../contracts/attempt-journal.schema.json).

Fresh targeted Linux checks passed: 72 unit tests and three public-API integration
journeys, warning-denied Clippy/rustdoc, formatting, Rust 1.91.0 all-target/all-feature
compilation and standalone Cargo package verification. Fresh regressions cover
exact identity/hash bytes, exhausted budgets, stale/mismatched receipts, closed
schema/bounds, locking, unsafe/replaced paths and lost writes on both publication
sides. A real acknowledged owner dies before reservation writing, after writing
and after its returned number; reopening retains the correct consumed allowance.
Existing artifact, download/publication, JSON, lock, layout/reference and command
custody regressions also passed. Schemas/examples and independently encoded
canonical authority golden bytes validate. This is native filesystem/process
qualification only; no PocketIC or real IC backend was used. No broad gate ran.

Retained fresh logs are `target/attempt-journal-tests.log`,
`target/attempt-journal-clippy.log`, `target/attempt-journal-msrv.log`,
`target/attempt-journal-docs.log` and `target/attempt-journal-package.log`.
Package inspection verifies all archived Rust source bytes and the exact regular
MIT license contents. The member `LICENSE` links to the maintained root contributor
notices. Current source/archive remain under `target/package/`; prior package
evidence is retained under `target/attempt-journal-package-evidence.kqp9ti8e/` and
`target/download-journal-package-evidence.45iah_5j/`. Earlier logs and recovery/
release evidence remain retained. No package upload occurred.

[Download provenance](../download-journal-source.json) retains 14 inspected Canic
files, 57 consumer references and published principal dependency inspection.
[Attempt provenance](../attempt-journal-source.json) adds 14 inspected files,
56 consumer references and dispositions for existing engine/CLI owners. Canic
HEAD remains `3978e02d28fea9022c7a6e84a7ec6d6e4a0d4af3`; its dirty working-tree
source hashes identify inspected bytes, not fresh qualification. No consumer or
source was removed. The original [planning baseline](../source-baseline.json),
[first extraction](../extraction-source.json), [layout](../layout-source.json) and
[command custody](../command-custody-source.json) provenance remain unchanged.

Cargo stays at `0.1.2`, Rust 2024, development Rust 1.99.0 and MSRV 1.91.0.
The previous download batch adds published `ic_principal` 0.1.5 conversion with
default features disabled and six lockfile packages, preserving previous locked
versions/checksums. Attempt accounting adds no dependencies. No Canic dependencies,
sibling patches, unsafe Rust or shared target exist. The sole populated changelog
draft is undated `0.1.3` beneath empty Unreleased; dated `0.1.2`, older notes and
`docs/release.json` remain unchanged. Maintainers own release preparation.
See [development](../development.md) and [releasing](../releasing.md).

Full B1/B2 completion and independently usable canister backup/restore remain
unestablished. Complete backend metadata/transfer extents, authenticated generic
plans, fresh authority/consistency and executor reconciliation of lost create/
upload/load responses still precede runner extraction. Full execution/restore
journals, completion manifests, terminal reference release, prune, transport and
CLI remain proposed. Canic's backup executor preflight still rejects. Read
[the design](../extraction-design.md) for the maintained sequence. Canic adoption
and live IC effects need their own instructions; the no-commit rule remains in force.
