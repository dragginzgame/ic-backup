# Changelog

## [0.105.5] - 2026-11-18

### Added

- Capture and restore through a Canic Root with a caller-supplied, approval-bound
  codec. Recover lost take and load results after host restart through the
  original Root lookup. Offline replay validates both outer request identities.

### 🔧 Changed

- Adopt Toko Tooling 0.45.1 in both workspaces and reuse its bounded setup
  diagnostics helper. Keep Backup's artifact custody and per-phase status records.

- Prepare Rust in both CI jobs through the shared toolchain-file helper, so the
  compiler profile, components and targets come from `rust-toolchain.toml`.
  Preserve bounded setup diagnostics and use the prepared host JSON/TOML tools.

## [0.105.4] - 2026-11-17

### 🧪 Testing

- Adopt Testkit 0.110.0 and align qualification with Process 0.81 so retained
  server reports use one supplier type identity. PocketIC remains 16.1.

### 🔧 Changed

- Adopt Toko Tooling 0.45.0 in both workspaces through the canonical exporter,
  retaining the existing publication flow and managed executable selection.

## [0.105.3] - 2026-11-17

### 🔧 Changed

- Make publication dry-run a linted archive/registry preflight; upload runs the
  full product and installed-IC gate once. Move publication-caller regression
  checks to CI and changed-tooling qualification
  ([Backup #3 — Avoid repeating full qualification for dry-run and upload](https://github.com/dragginzgame/ic-backup/issues/3)).

## [0.105.2] - 2026-11-17

### 🧪 Testing

- Adopt Testkit 0.109.0 for private qualification, retaining the selected managed
  server boundary and PocketIC 16.1.

- Remove direct test-only Candid and management SDK dependencies while preserving
  independent malformed-wire fixtures and exact restore argument comparisons
  ([Backup #2 — Remove direct test-only SDK dependencies](https://github.com/dragginzgame/ic-backup/issues/2)).

## [0.105.1] - 2026-11-17

### 🧹 Cleanup

- Keep digest regression tests in their owning test module so source tooling can
  distinguish test code from production code.

## [0.105.0] - 2026-11-17

### ⚠️ Breaking

- Complete the public Platform type migration: identities use Types 0.21 and
  snapshot requests/metadata use Protocol 0.3. Read regions contain explicit
  extents; globals use bounded u128 and exact float-bit wrappers. See the
  [consumer migration](crates/ic-backup/README.md).
- Status/list/take/metadata decode errors now carry Protocol errors. Empty
  snapshot IDs fail snapshot policy; oversized IDs, hashes and globals fail
  codec limits. Remove the unreachable typed GlobalRange error. Status errors
  no longer implement Copy. Certificate errors separate decode and verification
  failures, and root admission validates BLS points. Stored formats are unchanged.

### 🔧 Changed

- Adopt Testkit 0.108.0 in the private qualification workspace, retaining locked
  PocketIC 16.1 and the existing authenticated server setup.

- Add qualification-workspace linting to native CI and installed-Testkit
  capture/recovery journeys on Linux. Hosted CI remains asynchronous.

- Adopt Platform 0.34.13, Protocol 0.3.5 and Types 0.21.2 throughout production
  and qualification. Remove Platform 0.33 and the private snapshot/metadata
  traversal code. Keep original request bytes, recovery bindings and finite
  budgets. Native malformed schemas use test-only independent Candid/SDK DTOs;
  normal fixtures use Platform encoders and signing primitives.

- Move the restore test module into `tests/mod.rs`, removing the overlapping
  file/directory module layout while retaining its original fixture bytes.

- Rename the private consumer directory to `qualification/ic-backup-consumer`
  to match its package name; update workspace, archive checks and documentation.

- Qualify all 11 actual-IC journeys with the new production certificate and
  management boundaries, including 20 MiB and separate-process recovery.

## [0.104.0] - 2026-11-16

### ⚠️ Breaking

- Snapshot-data and restore-reply decode errors now carry Protocol's bounded
  codec error type. Original arguments, evidence and stored formats are unchanged.

### 🔧 Changed

- Use Protocol 0.2.8 for production snapshot-data, restore allocation and empty
  replies, removing private schema decoders; partial adoption for
  [Platform #11](https://github.com/dragginzgame/ic-platform/issues/11).
- Move actual-IC external signing and proxy admission to Platform 0.34.9 for
  [Platform #12](https://github.com/dragginzgame/ic-platform/issues/12). Retain
  product 0.33.7 APIs pending the remaining coordinated migration.

- Adopt Host HTTP 0.80.0 in both graphs; keep client/TLS/no-retry policy in
  Backup and delegate complete-body limits to Host.
- Adopt Testkit 0.107.4 for private qualification, using explicit prepared
  assets, managed server/instance lifetimes and retained process evidence.
  Replace the removed Testkit CLI setup with the locked library example.

- Adopt Toko Tooling 0.44.11 in both workspaces and refresh its managed snapshot.
  Retain bounded sanitized CI setup diagnostics with original failure status and
  seven-day artifacts, following [Toko Tooling #387](https://github.com/dragginzgame/toko-tooling/issues/387).

## [0.103.0] - 2026-11-16

### ⚠️ Breaking

- Adopt registry-backed Host FS/Artifacts 0.80.2. Public commitments now use
  `ic_backup::store::BackupDigest`; filesystem, bounded-read and publication
  errors expose the new Host outcomes. Stored SHA-256 bytes, canonical text,
  intents, journals and manifests keep their existing formats.
- Publish an exact retained manifest through Host's `admit_retained` API,
  preserving its original inode and create-only recovery authority
  ([IC Tooling #2 — Publish original retained files after reopen](https://github.com/dragginzgame/ic-tooling/issues/2)).
  Use complete bounded producer hashing for materialization and direct SHA-256
  for the fixed capture module stream; remove obsolete qualification digest dependencies.

### 🐛 Fixed

- Admit original-result lookup under every supported request ceiling through
  2 MiB using the shared request limit. Preserve exact sender/path/expiry checks
  and spent observations; qualify interrupted allocation recovery at 2 MiB.
- Correct lookup authentication and Host error documentation after API migration.

### 🔧 Changed

- Verify read-only materialized payloads with the complete closing tree hash,
  removing an earlier duplicate payload read. Keep admission of all existing
  members before creating any missing file and preserve closing custody checks.
- Reuse Host's digest when reading retained operation evidence, removing a second
  full SHA-256 pass while preserving exact length, digest and closing custody checks.
- Authenticate each original request once within semantic replay, then bind its
  result certificate to those same owned bytes. Remove redundant signature checks
  without caching authentication across invocations or changing recovery authority.
- Read evidence references directly from their journal call and index held files
  by canonical name, removing repeated journal/name scans and temporary filename
  lists while retaining complete byte and directory admission.
- Adopt Toko Tooling 0.44.5 in both workspaces and refresh the managed snapshot.
  The supplier fixes declaration checks for the required tooling pin and private
  qualification versions ([Toko Tooling #386 — Align exact tooling pin and qualification checks](https://github.com/dragginzgame/toko-tooling/issues/386)).

## [0.102.1] - 2026-11-16

### ⚠️ Breaking

- First registry release of the incompatible restart described in 0.102.0 below.
  Legacy consumers must adopt the fixed capture/restore API and use a new store;
  retain original readers and authority for existing backups and unfinished work.
  Canic manager integration remains unfinished. This release supports the
  experimental direct-controller workflow and its bounded 20 MiB profile.

### 🧪 Testing

- Add an actual-IC 20 MiB capture/restore case with interrupted-transfer recovery,
  cold reopening and offline replay under explicit payload/evidence allowances.
- Verify the packaged README and an isolated archive consumer; qualify the shared
  publication caller's selection, validation and single-attempt failure behavior.
  Nested caller fixtures retain their own evidence and timing scope.
- Explicitly select blocking IO for accepted HTTP/TLS fixture and IC proxy
  connections, preserving timeouts when macOS inherits the listener's nonblocking mode.
- Recognize peer reset as well as EOF when observing HTTP cancellation; timeouts,
  other read failures and detached retries still fail the fixture.
- Start HTTP deadline tests after fixture setup and allow dispatch on loaded
  hosts while retaining a watchdog shorter than the configured transport timeout.

### 📚 Documentation

- Package consumer setup, bounded capacity, original-store recovery and legacy
  migration guidance. Add explicit publication checks; qualification packages
  remain private.

### ▶️ Try it

`make package-check` verifies the crate archive and isolated consumer.
`make publish` performs the complete local gate and one authorized upload;
see [publication and recovery](docs/development/publication.md).

## [0.102.0] - 2026-11-16

Unpublished GitHub checkpoint.

### ⚠️ Breaking

- Replace public capture phase commands with one approved direct connection and
  fixed advance loop. Common call/observation facts own signing, submission,
  original certified evidence, finite spending and explicit pending reconciliation.
- Use a distinct host-store/1 marker, capture intent and journal encoding. Older
  stores refuse intact under the replacement reader; their original reader and
  recovery evidence are preserved. No migration/reset is provided.
- Remove superseded phase engines/events, transfer-binding and assembly stages.
  Held source, authenticated succession and local restore create/open now consume
  the public capture boundary without transport/provider parameters.

Consumers must adopt the fixed capture/restore API. Retained older operations
must continue with their preserved reader, original authority and evidence;
there is no migration, reset or reinterpretation under the new formats.

### 🔧 Changed

- Reuse checked journal prefixes/results within a capture invocation, extend its
  authenticated procedure after retaining a live result, and remove duplicate
  authentication of the same owned reply. Cold replay and checks across awaits
  remain independent. Remove the unused assembled-tree digest calculation.
- Share capture/restore declaration bounds through `OperationLimits`, removing
  their duplicate public records/validators. Serialized fields and independent
  approved operation limits remain unchanged.

- Replace restore's separate status signer/adapter/events with the common recorder.
  One source-bound restore loop allocates/uploads, compares exact metadata and every
  byte, loads and verifies stopped state with distinct original status requests.
  Derive LoadedStopped from its closed journal; support fresh capture after restore.
- Retire the obsolete test-only readback cursor; move its distinct coverage,
  routing and corruption cases onto the actual restore policy.
- Share the connection, signer, action guard, progress and typed view across both
  purposes. Remove unused public proof/admission records and status-stage progress.
  New restore-intent/1 refuses older restore records intact under their preserved reader.

- Reopen restore from original operation ID and approval, reading its retained plan
  without a signer, connection or caller-side plan copy. Re-admit exact selection
  and held target lock before returning cold progress; preserve stored formats.

- Preserve bounded received certificates and original request IDs in redacted
  typed post-response errors. Import surviving observation replies locally under
  their original submitted lookup and separate time policy, without new allowance.
- Align new profiles at 2 MiB request / 4 MiB result / 2 MiB certificate maxima
  for 1 MiB payload chunks. Existing intents retain their original smaller limits.
- Name shared APIs `OperationStoreError`, `BackupHttpTransport` and
  `BackupHttpError`, without compatibility aliases or retained-domain changes.

### 🧪 Testing

- Qualify multi-extent module/heap/stable capture and empty/nonempty chunks, exact
  publication, succession, full older-source restore of changed actual IC state,
  capture after restore and offline history/replay against PocketIC.
- Recover interrupted take/data/allocation/mid-upload/load through an explicitly
  accounted observation or surviving original certificate, without resubmission.
- Retain shared protocol/corruption/custody/finite-spending/process-death cases;
  restore-specific regressions cover allocation, exact readback, sender/status and
  reused status-request refusal. Canic adapters and physical-power-loss qualification
  remain unfinished.

- Qualify full 1 MiB whole-chunk capture/restore and failed load-result retention
  recovered from the returned error after shutdown, without a proxy recovery copy.
- Build a separate normal-dependency consumer without Testkit/PocketIC and exercise
  fresh-process capture/restore continuation and offline completion replay.

## [0.101.13] — Preserved local checkpoint (unpublished)

### ➕ Added

- Bind restore preparation to the original direct transport/integration/timing
  profile, then durably reserve one status submission and retain certified results.
  Uncertain outcomes stay spent; terminal replay authenticates original evidence
  without signing, HTTP or writes. Status-policy failures remain settled.

- Qualify restore status ingress against a real IC fixture and replay it after
  server shutdown. Upload/load and pending-result reconciliation remain unfinished.

### 🔧 Changed

- Derive cumulative capture reservations once per journal event instead of
  resumming earlier admissions, preserving original allowances and wire bytes.

- Derive capture evidence membership from retained state for integrity checks,
  orphan detection and synchronization, removing repeated phase inventories.
  Exact filenames, hashes, journal bytes and no-retry behavior remain unchanged.

## [0.101.12] — GitHub checkpoint (unpublished)

### ➕ Added

- Authenticate restore status certificates against the original signed request,
  trust root, canister and historical time windows using capture’s bounded verifier.
  This local check preserves evidence and grants no submission or retry authority.

## [0.101.11] — GitHub checkpoint (unpublished)

### ➕ Added

- Reserve one restore status signing attempt durably and retain its exact signed
  request. Reopening prepared or uncertain attempts never signs again; damaged
  evidence refuses intact. This step performs no management submission.

- Authenticate exact signed restore status requests against their approved sender
  and target with bounded original records and shared direct-key verification.
  This local check preserves bytes and grants no reservation or dispatch authority.

### 🔧 Changed

- Adopt Toko Tooling 0.44.4 and place the private qualification package directly
  under `qualification/`, preserving its identity, lockfile and evidence paths.

- Flatten capture workflow tests and move restore tests beside restore code. Share
  named fixtures without nesting test groups or changing recovery assertions.

- Avoid repeating completion authentication and synchronization during restore
  creation when the source is the current completed capture owner. Preserve exact
  source admission and held custody; distinct older sources still reconcile separately.

## [0.101.10] — GitHub checkpoint (unpublished)

### ➕ Added

- Persist and reopen approved restore intent with exact source binding, a bounded
  initial journal and durable target ownership. Retain original source custody,
  refuse incomplete or changed files without repair, and admit older sources only
  after authenticating the current completed capture owner. No upload or load
  effects, reverse-reference catalog or cleanup are introduced.
- Validate restore status against its approved sender and source module using the
  existing bounded status reader and stopped/controller policy. This pure check
  grants no signing, dispatch or settlement authority.

## [0.101.9] — GitHub checkpoint (unpublished)

### ➕ Added

- Calculate bounded restore transfer schedules from authenticated held sources,
  requiring the same network, canister and release. Reuse capture extent ordering,
  account for exact whole-chunk lengths and full upload/readback call counts.
  This pure calculation creates no restore intent, source reference or IC effect.
- Encode and re-admit immutable direct-host restore plans bound to the source
  manifest/capture plan, ordered source request commitment, exact target/release,
  sender and finite limits. Approval hashes original bytes; source substitution
  and malformed records refuse. No restore operation or reference is written.

## [0.101.8] — GitHub checkpoint (unpublished)

### ➕ Added

- Verify an already-published capture against an expected manifest hash without
  writes or publication. Reuse original certified-evidence and exact payload-byte
  checks; incomplete publication and changed files refuse without repair.
  Verification returns passive evidence, not restore authority.
- Open authenticated capture sources under the original operation lock and held
  payload descriptors. Exact receipt and byte limits select the source; bounded
  reads reuse existing identity/block checks without reauthenticating the whole
  backup per extent. No restore intent or durable source reference is created.

## [0.101.7] — GitHub checkpoint (unpublished)

### ➕ Added

- Admit an exact completed capture after target ownership changes, retaining its
  original manifest, evidence and spending. Unfinished or unpublished captures
  still require their original claim. Explicit authenticated succession replaces
  only the target owner and creates fresh bounded intent while preserving the
  predecessor. Partial successor creation refuses without cleanup or reset.

## [0.101.6] — GitHub checkpoint (unpublished)

### ➕ Added

- Complete a local capture through sequential authenticated payload reads, exact
  module/coverage verification, durable artifact assembly, distinct final status
  and create-only completion-manifest publication.
- Preserve original bounded signing/network/evidence authority and uncertain
  outcomes in one journal. Local completion reconciliation retains the exact
  timestamp and manifest; partial or altered files refuse without cleanup.
- Qualify the small actual-IC capture-to-inventory journey and cold reconciliation
  after server shutdown. Target succession, restore and Canic adapters remain
  unfinished; packages remain unpublished.

## [0.101.5] — GitHub checkpoint (unpublished)

### ✨ Features

- Authenticate certified payload replies against exact scheduled data requests.
  Reuse shared request/certificate validation and bounded payload decoding; refuse
  substituted ranges, mismatched chunks and over-budget complete evidence.
  Durable payload dispatch and persistence remain unimplemented.

- Keep local transfer binding independent of metadata signing: use retained
  admission and shared replay verification, removing unnecessary signer inputs.

- Bind authenticated metadata to a compact versioned transfer schedule in the
  original journal. Reauthenticate and compare on replay without new signing or
  HTTP; refuse conflicting records and partial writes. Qualify real metadata
  binding and offline replay. Payload reads remain unimplemented.

- Add fixed metadata-read argument construction and local authentication of signed
  requests and certified metadata replies. Bind the exact target, snapshot ID and
  take timestamp, preserve original timing and reuse bounded transfer validation.
  Payload download and local completion remain unimplemented.
- Add one durable metadata acquisition in the original capture journal, with
  original snapshot authentication, cumulative reservations, exact request/result
  retention and offline replay. Lost signing or HTTP outcomes remain spent.
  Qualify interruption boundaries and real metadata acquisition with PocketIC.
- Share private exact-call validation with the existing capture verifiers, without
  adding a public arbitrary-method interface. Existing event encodings remain
  unchanged; older readers refuse the new metadata events.

## [0.101.4] — GitHub checkpoint (unpublished)

### ✨ Features

- Add one-shot authenticated read-state lookup through the existing HTTP transport.
  Preserve exact signed lookup bytes, original trust/request binding and distinct
  unresolved outcomes; bound deadlines and responses without retries or polling.
  The integrating workflow must reserve and retain lookup authority separately.
- Qualify real take-reply lookup with PocketIC while preserving original journal,
  snapshots and stopped module.
- Add one durable lookup per capture operation in its original journal. Reserve
  signing and observation before signing, retain exact requests before dispatch,
  and retain authenticated certificates before reporting observations. Replay is
  offline; lost replies stay spent.
- Qualify all four new durability boundaries, cumulative budget admission and
  real pending-take observation with offline replay in PocketIC.
- Settle original calls from terminal retained lookup certificates without new
  remote work. Preserve observation timing separately, replay authentication and
  live admission before dependent effects. Qualify status/list recovery, rejection,
  conflicting results, uncertain settlement writes and real take recovery after
  server shutdown.
- Fix the private fixture default asset path so direct Cargo runs admit the same
  offline Testkit directory as Make.

### 📦 Dependencies

- Adopt Toko Tooling 0.44.3 through its verified exporter, including the revised
  LOC depth defaults and focused post-push review guidance.

## [0.101.3] — GitHub checkpoint (unpublished)

### 📝 Summary

- Recover a pending status, list or take from its surviving original certified
  reply without signing or submitting again. Preserve original request identity,
  trust, timing, spending and evidence; settled replay cannot replace the result.
  Missing replies still require a separately budgeted remote observation flow.

- Qualify interrupted snapshot reply recovery with PocketIC after server shutdown
  and application-guard revocation. Recording a reply grants no live application
  authority and does not complete a local backup.

- Authenticate original-request certificates under separately supplied observation
  windows without widening original submission authority. Preserve distinct
  pending, done, absent and pruned outcomes. Live lookup still requires Backup's
  durable observation workflow.

- Authenticate signed recovery lookups against the original call and sender,
  explicit expiry and sole original request-status path. Reuse direct-key
  signature verification; extra paths and ambiguous content refuse. This local
  helper grants no signing, network, settlement or retry authority.

### 📦 Dependencies

- Adopt Platform 0.33.7 in both workspaces, making the strict read-state content
  decoder available for recovery work. This supplies
  [Platform #167 — Add bounded strict read-state content decoding for signed recovery lookups](https://github.com/dragginzgame/ic-platform/issues/167).
  Existing capture/recovery and actual IC fixtures pass with the updated graph.
- Adopt Toko Tooling 0.44.1 through the authenticated exporter. `toko-cloc` now
  defaults to the same crate-level report as `-d 0`; both workspaces select the
  exact tooling version.

## [0.101.2] — GitHub checkpoint (unpublished)

### 📝 Summary

- Compose direct-host status and snapshot signing, live authority/time admission and authenticated
  HTTP through `DirectCaptureProvider`. Integrations retain bounded sign-only and
  continuous application-authority hooks. Preserve original spending on uncertain
  effects and verify settled replay without a live signer, guard or clock.

- Qualify real stopped-canister status, non-replacing snapshot creation and offline cold replay through a private
  Testkit/PocketIC workspace. Add explicit server setup, offline admission and
  qualification targets through the installed Tooling and Testkit owners.
  Product remains unpublished; download and complete backup/restore remain unfinished.

- Continue verified initial status with durable snapshot list/take reservations,
  original signed bytes and authenticated replies. Check cumulative budgets and
  capacity; preserve existing snapshots and installed code. Pending effects are
  never retried or inferred from inventory differences.

### 📦 Dependencies

- Use the already selected Tokio timer in production for bounded signing waits.
  Select Testkit 0.106.0/PocketIC 16.1 only in the private qualification graph.
  Admit PocketIC's `thiserror` 2.0.18 pin there while retaining 2.0.21 in the
  product lock; no supplier source or sibling override is added.

- Retain the incoming Platform 0.33.6 product lock selection and align the private
  qualification graph. Its new Cloud Engine module is outside selected features;
  canonical Types remain 0.20.4.

## [0.101.1] — GitHub checkpoint (unpublished)

This checkpoint adds authenticated direct-host status HTTP, bounded shared body
reading and direct-key request verification. Publication remains disabled.

### ✨ Features

- Authenticate externally signed status requests for direct Ed25519 and secp256k1
  identities through Platform. Bind the exact original sender, call and request
  ID; refuse invalid signatures and unsupported identity forms before HTTP.
  Historical verification changes no expiry, bytes or spending. Private keys and
  external signing remain integration-owned.

- Add direct-host HTTP submission of an already-retained status request, with
  original target/trust bindings, deadline and response-byte limits. Disable
  redirects, retries, proxies and polling; return only authenticated certificate
  evidence. Failed or cancelled submissions keep their original spending.
  Integrations still supply signing and application admission.

### 📦 Dependencies

- Enable Platform's existing Ed25519, secp256k1 and self-authenticating Principal
  capabilities. Add their registry-backed dependencies while preserving existing
  locked supplier versions; no local cryptographic implementation is added.

- Adopt Toko Tooling 0.43.2 through the authenticated exporter. The selected
  baseline and command payload remain unchanged; refresh the exact pin and
  snapshot provenance without changing product dependencies.

- Adopt restored `ic-host-http` 0.79.0 ([Tooling #360](https://github.com/dragginzgame/ic-tooling/issues/360))
  and use its Reqwest 0.13.5 re-export
  with opt-in Rustls. Replace the local response-reading loop with its bounded,
  fallible reader. Rustls now uses platform certificate verification; original
  IC trust and request/recovery policy stay Backup-owned. Preserve the incoming
  Platform 0.33.5 lock selection. All packages remain unpublished.

## [0.101.0] — GitHub checkpoint (unpublished)

This checkpoint adds certificate evidence, snapshot readback and held payload
sources. The minor version accounts for Host 0.79 public type identities.
Publication remains disabled.

### ⚠️ Breaking

- Adopt published `ic-host-artifacts` and `ic-host-fs` 0.79.0. Backup exposes
  their `Sha256Digest`, `ArtifactError` and `CapabilityError` types, so consumers
  exchanging these types must select the same Host 0.79 line. Rust type identity
  changes even though both source trees are unchanged from 0.78.0.
  Persisted formats, digest bytes and recovery authority are unchanged; no data
  migration or reset is required. See the
  [Host release notes](https://github.com/dragginzgame/ic-tooling/blob/1a7b1521c56531e25b7bf75e3b79e27fe6ee93a2/releases/foundation/CHANGELOG.md).

### ✨ Features

- Build and authenticate retained status replies and certified rejections through
  Platform 0.33.4. Bind original request IDs, exact replies, root commitments, delegated canister authority
  and time windows retained before submission. Refuse changed or unavailable proof
  without retry or renewed spending. Preserve exact rejection diagnostics and
  distinguish certified absence from pruned fields. Transient rejection settles
  without refunds. These local helpers supply no live transport.

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

- Require Platform 0.33.4 for bounded historical certificate verification and enable
  its `certification-verify` feature. Add six transitive cryptography packages;
  preserve all previously selected versions and keep Agent/HTTP runtimes absent.

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
