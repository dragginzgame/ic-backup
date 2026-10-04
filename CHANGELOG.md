# Changelog

## [Unreleased]

## [0.1.4]

- Adapt Canic phase and restore ordering into bounded explicit operation dependency
  graphs with deterministic planning order and pure causal readiness views. Reject
  duplicate/missing dependencies, cycles and inconsistent declared progress; retain
  immutable graphs with exact-digest admission and independent input/output bounds.
- Adapt Canic topology hashing and target expansion into canonical bounded v1
  physical inventories and pure exact/direct-child/subtree selection. Reject
  normalized duplicates, missing parents and cycles; retain immutable inventories
  under layout exclusion with exact-digest replay and explicit byte bounds.

## [0.1.3] - 2026-10-04

- Adapt Canic's pending claims and exact receipts into immutable per-operation
  identity and separate finite mutation/observation allowances. Persist append-only
  reservations before returning, block blind retries and qualify lost writes,
  exhausted budgets, stale receipts and acknowledged process death.
- Extract the local download journal lifecycle from Canic with immutable snapshot
  identities, model-owned transitions and derived resume views. Add locked bounded
  persistence, staged-byte verification and durable publication recovery; qualify
  lost writes, changed bytes and acknowledged owner death before journal advancement.
- Include the workspace MIT license and contributor notices in the standalone
  crate archive through a member license link to the maintained root file.

## [0.1.2] - 2026-10-04

- Extract command lifetime custody with owned descriptor inheritance and exact
  v1 sidecar identity records. Add one-spawn guards, bounded finish and fresh
  exclusive quiescence probes; qualify owner/direct-child/descendant death,
  failed spawns and replaced or missing sidecars without Rust unsafe code.

- Extract stable layout lifetime locks and durable v1 restore dependencies.
  Validate exact journal/intent binding, bounded records and replaced roots;
  qualify process-death recovery and conservative missing-journal retention.

- Enable crates.io publication for `ic-backup` through inherited workspace
  metadata, fixing the Cargo rejection from `make publish`.
- Delegate publication and its dry run to Cargo for the current package;
  keep receipt/tag checks within the repository release workflow.

## [0.1.1] - 2026-10-04

- Extract local artifact hashing, no-follow traversal/staging, durable verified
  directory publication, JSON persistence and journal locks from Canic without
  framework dependencies. Separate checksum records from host IO operations.
- Normalize checksum records at decode, reject ambiguous non-UTF-8 artifact
  names, bound record reads and create private record files/directories.
- Requalify copied regressions and add real process-death publication checks,
  staging/byte-limit cases and a public-API local recovery journey. Retain fresh
  source hashes and trace the existing Canic consumers.

## [0.1.0]

- Document the extraction of host-side backup/restore from Canic, including
  ownership boundaries, same-release recovery to the same canister IDs,
  source provenance and the implementation sequence.
- Establish an independent Rust 2024 workspace with the `ic-backup` library,
  inherited package metadata and lints, Rust 1.99.0 development tooling and
  Rust 1.91.0 minimum support.
- Add native build, lint, test, documentation and standalone package commands,
  Linux CI and a repository-local build directory.
- Add the pre-commit formatter with unstaged-source protection and explicit
  review/restaging of formatting changes.
- Provide `make release-patch`, `make release-minor` and `make release-major`
  to validate, prepare, commit, tag and atomically push a release, alongside
  preview and individual-step commands. Retain release-file hash receipts,
  rollback, build artifacts and isolated tooling regressions.
- Preserve MIT contributor attribution and source hashes for adapted sibling
  tooling. This initial foundation has no backup/restore APIs or CLI;
  registry publication remains disabled.
