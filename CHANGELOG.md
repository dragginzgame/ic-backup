# Changelog

## [0.100.0] — pending

### ⚠️ Breaking

- Select `0.100.0` for the incompatible rewrite, superseding the unpublished
  `0.68.0` draft. Publication remains disabled. Legacy backups and unfinished
  operations retain their original implementation and recovery authority;
  no automatic migration is provided.

### ✨ Features

- Add bounded raw status validation through Platform, sharing the existing capture
  preconditions. Bound controllers and module hashes before allocation and meter
  skipped fields cumulatively; preserve original evidence and operation state.

- Add pure capture status checks for stopped state, the approved module hash and
  direct-sender or Canic-executor controller membership. Reject duplicate/excessive
  controllers; status visibility alone grants no capture authority.

- Add bounded raw snapshot metadata decoding and original-plan admission. Preserve
  exact float bits and recognized optional meanings; refuse excessive collections,
  unknown variants and unsupported capture profiles without changing operation state.

- Add bounded raw data-reply decoding through Platform's schema and codec. Enforce
  exact scheduled region lengths, original chunk size limits and SHA-256; refuse
  malformed, excessive or skipped wire content without advancing an operation.

- Add pure capture transfer planning through Platform management DTOs: lazy
  ordered extents, sorted whole chunks, checked payload budgets and whole-capture
  request/signing minima. Reject unsupported round-trip metadata without defaults.
- Add immutable approved capture plans, durable target claims and exclusive
  create/open/inspect operations. Retain partial creation and refuse missing
  ownership instead of resetting it. No attempt dispatch or completion is enabled.
- Add private local-store initialization/opening and bounded completion inventory,
  including latest captured backup per network/canister. Preserve interrupted
  initialization and refuse corrupt records or incomplete scans.
- Add read-only payload verification against a caller-trusted manifest hash and
  explicit byte limit. Stream-check complete member coverage, exact lengths and
  hashes; refuse replacement, corruption and unsafe files without repair. Full
  operation-evidence verification and restore admission remain unimplemented.
- Use published Host filesystem/artifact primitives and Platform identity. Add
  native failure-path tests and a compiled usage example; IC capture/restore and
  completion writing remain unimplemented.

### 🐛 Fixes

- Retain original ledger-file custody and reject identical-byte replacement.
  Validate chained frames with bounds before payload allocation, refuse partial
  tails and enforce the original evidence-byte ceiling before claiming a target.
- Apply legacy verification lessons: recheck file identity/length, complete payload
  membership, manifest bytes and child-directory identity before returning success.
  Re-authored regressions reproduced same-byte replacement and post-scan extra-file
  acceptance before the fix; add closing-drift, FIFO and maximum-chunk coverage.

### 🔧 Tooling

- Adopt published Platform 0.33.0 with the existing management-only feature
  selection and Types 0.20.3. Requalify public DTOs, exact wire fixtures and local
  recovery tests; keep unrelated dependencies and publication settings unchanged.

- Establish a fresh virtual workspace with an `ic-backup` library,
  publication disabled, Rust 1.100.0 and shared strict lints. Keep all packages
  non-publishable while the implementation matures.
- Adopt the same authenticated Toko Tooling 0.43.0 foundation as IC Query,
  including standard setup, formatting, dependency commands and LOC reporting.
- Configure Linux and macOS scaffold validation and a separate secret scan.
- Complete the authenticated snapshot with the shared pre-commit formatter hook
  and activate it locally; verify dependency commands and PATH LOC reporting.

### 📚 Documentation

- Record standalone-library and future integration boundaries without importing
  legacy implementations, release state or recovery obligations into new schemas.
- Add a single-canister library design proposal covering stopped capture/restore,
  durable operation custody and bounded recovery.
- Ground the design in current sibling APIs and required Canic integration,
  including external signing, manager-controlled targets and recovery boundaries.
- Specify proposed operation/provider contracts, lossless snapshot comparison,
  deterministic transfer scheduling and bounded Canic executor accounting.
- Evaluate legacy reuse: retain recovery guarantees and failure scenarios, replace
  stage/journal composition, and state excluded features and recovery tradeoffs.
- Select local bundles and manifest-based backup history for v1, with Canic-owned
  concurrency across targets; defer Blob integration to v2 and specify the minimal
  local operation store, completion publication and target ownership.
