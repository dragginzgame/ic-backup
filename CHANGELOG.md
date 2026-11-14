# Changelog

## [0.101.0] — pending

Development continues as 0.101.0 after the GitHub checkpoint containing status
request binding, snapshot preflight and Host/Types adoption. The minor version
accounts for Host 0.79 public type identities. Publication remains disabled.

### ⚠️ Breaking

- Adopt published `ic-host-artifacts` and `ic-host-fs` 0.79.0. Backup exposes
  their `Sha256Digest`, `ArtifactError` and `CapabilityError` types, so consumers
  exchanging these types must select the same Host 0.79 line. Rust type identity
  changes even though both source trees are unchanged from 0.78.0.
  Persisted formats, digest bytes and recovery authority are unchanged; no data
  migration or reset is required. See the
  [Host release notes](https://github.com/dragginzgame/ic-tooling/blob/1a7b1521c56531e25b7bf75e3b79e27fe6ee93a2/releases/foundation/CHANGELOG.md).

### ✨ Features

- Add a held local payload source for bounded reads under original file custody.
  Verify every member against the trusted manifest, then check requested bytes
  against in-memory block hashes. Reject changed files, identities and ranges;
  invalidate the reader after filesystem/integrity failures. Reuse payload
  admission checks without changing bundle formats or restore authority.

- Compare bounded uploaded-snapshot metadata with admitted capture metadata.
  Preserve exact global variants/float bits, timer/hook states and chunk identities;
  retain fresh destination provenance separately. Return typed mismatches without
  changing operation state or granting load authority. Add bounded ordered payload
  comparison against caller-held source slices; refuse missing/repeated extents,
  same-length corruption and invalid chunk bytes. Completion requires every region
  and named chunk, including empty chunks. No remote readback or load is performed.

- Add bounded snapshot list/take reply validation and direct-host take preflight.
  Preserve exact IDs, wire order and full timestamp/size values; reject duplicate
  IDs, excessive inputs and a take result already in the prior inventory. Take
  arguments preserve existing snapshots and installed code. These pure helpers
  do not authenticate replies, dispatch capture or recover an uncertain call.

### 🛡️ Fixes

- Check prepared status call content against the original plan before submission
  and during recovery. Refuse mismatched callers, targets, methods, arguments,
  expiry and request IDs without re-signing, resubmitting or rewriting evidence.
  Keep the existing API and record formats; provider authentication remains required.

### 📦 Dependencies

- Retain the incoming Platform 0.33.4 lock update and qualify the native
  transfer/readback surface against it. Native management fields and codec bytes
  remain unchanged. Platform now supplies historical verification and portable
  snapshot DTOs; Backup's concrete provider and Canic integration remain pending.

- Qualify published Platform Types 0.20.4 through the existing Platform dependency.
  Principal implementation and bytes are unchanged; retain original plan, request
  and recovery fixtures. This tooling-only supplier patch does not supply historical
  certificate verification or Wasm snapshot APIs.

- Enable the existing Platform 0.33.2 protocol codecs for immutable ingress
  inspection. No Agent or HTTP runtime is added.

## [0.100.0] — GitHub checkpoint (unpublished)

### ⚠️ Breaking

- Select `0.100.0` for the incompatible rewrite, superseding the unpublished
  `0.68.0` draft. Publication remains disabled. Legacy backups and unfinished
  operations retain their original implementation and recovery authority;
  no automatic migration is provided.

### ▶️ Try it

From the repository root, prepare the pinned developer tools once (downloads
require network access), then list package versions and source line counts:

```bash
make install-tools
make deps-list
toko-cloc -d 0
```

These commands prepare local tools and print reports; they do not publish Backup.

### ✨ Features

- Add read-only status inspection for cold recovery: recover original admission,
  retained stage, reserved signing/network allowances and request identity.
  Report settled evidence without claiming authentication, success or refunds.

- Add initial direct-host status continuation with durable signing/submission
  reservations, exact retained evidence and local settled replay. Pending attempts
  never re-sign or resubmit after reopening. Preserve partial writes and orphans;
  concrete provider qualification, Canic execution and snapshots remain pending.

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
  ownership instead of resetting it. Intent creation alone enables no dispatch or completion.
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

- Adopt verified registry Host FS/Artifacts 0.77.3, including the held-directory
  update opener. Remove the temporary local overrides after consumer qualification
  so a fresh checkout needs no sibling source tree; publication remains disabled.

- Adopt published Platform 0.33.0 with the existing management-only feature
  selection and Types 0.20.3. Requalify public DTOs, exact wire fixtures and local
  recovery tests; keep unrelated dependencies and publication settings unchanged.

- Establish a fresh virtual workspace with an `ic-backup` library,
  publication disabled, Rust 1.100.0 and shared strict lints. Keep all packages
  non-publishable while the implementation matures.
- Adopt authenticated Toko Tooling 0.43.1,
  including standard setup, formatting, dependency commands and LOC reporting.
- Configure Linux and macOS scaffold validation and a separate secret scan.
- Complete the authenticated snapshot with the shared pre-commit formatter hook
  and activate it locally; verify dependency commands and PATH LOC reporting.

### 📚 Documentation

- Specify the proposed first capture call, including separate signing/submission
  reservations, aggregate evidence capacity and interruption recovery. Keep live
  dispatch authority distinct from replayed pending state. The initial status transitions are implemented; concrete provider and IC
  qualification remain pending.

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
