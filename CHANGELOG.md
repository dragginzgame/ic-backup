# Changelog

## [0.44.1] - 2026-10-24

- Adopt Shared Tooling 0.26.14 for Cargo-tool setup diagnostics, literal formatter
  catalog admission, aggregate offline tool reports and release recovery fixes.
  Compose the shared external-IC dependency guard with Backup's remaining local
  exclusions. Distinguish busy Testkit admission from missing installation and
  retain earlier uncertain release evidence when a committed candidate advances.
  IC Tooling remains at the latest published release, 0.68.0.

## [0.44.0] - 2026-10-24

- **Breaking Rust dependency identity:** adopt published Host 0.68.0. Consumers
  exchanging Host checksum/publication errors with Backup must align their Host
  dependencies to 0.68. Product wire/storage and recovery contracts are unchanged.
- Adopt Testkit and the contributor release editor at 0.68.0. Backup does not
  enable Testkit's optional Wire adapter; simulator lifecycle and asset ownership
  remain with Testkit. Prepare tools with `make install-testkit-server` and
  `make install-release-editor`.

## [0.43.0] - 2026-10-24

- **Breaking Rust API:** remove `ExecutionStageGuard::create`. Use `prepare` to
  create a stage with its complete original attempt journals; use `open` for
  record inspection or `resume` for complete retained-accounting admission.
  Partial preparations and existing journals remain preserved without repair.

- Simplify test maintenance by removing self-testing provider fakes and sharing
  private, create-only Agent fixture evidence writes. Production workflow failure
  coverage and independent-consumer isolation remain intact.
- Refresh Canic adoption guidance and separate historical extraction qualification
  from current implementation contracts.

## [0.42.0] - 2026-10-24

- **Breaking Rust dependency identity:** adopt published Host 0.67.0. Consumers
  exchanging Host checksum/publication errors with Backup must align their Host
  dependencies to 0.67. Product wire/storage formats and recovery contracts remain
  unchanged.
- Adopt Testkit and the contributor release editor at 0.67.0. Keep simulator
  lifecycle/asset ownership with Testkit; prepare the updated selections with
  `make install-testkit-server` and `make install-release-editor`.

- Add an explicit registry-only mode to the independent authenticated consumer
  harness. Require exact published Core/Agent versions and the original external
  dependency graph, with no path fallback or simulator packages in the consumer.
  This qualifies the generic installed-consumer portion of
  [#29 — Complete original-plan execution and stage binding before Canic runner adoption](https://github.com/dragginzgame/ic-backup/issues/29);
  production application acceptance remains outstanding.

- Add public consumer coverage and integration guidance for application-owned Root
  ingress using the existing workflow and attempt guards. Qualify exact retained
  envelope binding, pending lost replies and cold replay while preserving separately
  owned paid-effect evidence. This native fixture does not enable Canic live capture.

## [0.41.0] - 2026-10-24

- **Breaking Rust API:** remove standalone inventory/graph persistence and
  caller-declared completion readiness. Persist full operation plans and derive
  progress from their original journals. Inventory/graph models and their wire
  formats remain unchanged; preserve existing files and recovery evidence.
- **Breaking Rust API:** remove the five unused synchronous membership, control,
  snapshot-read, consistency and restore-safety provider traits and errors. Use
  mandatory async workflow admission callbacks with the existing evidence models
  and pure policies; integrations retain all freshness, authority and custody duties.
- **Breaking Rust API:** remove redundant `Request`/`Association` variants and
  their `From` conversions from metadata/download/allocation/data settlement errors.
  Handle request/association failures through the existing single-call `Read` or
  `Upload` error path. Reply retention, spending and lifecycle semantics are unchanged.

- Consolidate contributor documentation around current contract owners and focused
  checks. Remove stale extraction-era implementation claims and describe the current
  async admission boundary in the crate overview.

## [0.40.1] - 2026-10-24

- Adopt published Host, Testkit and the contributor release editor at 0.66.2,
  aligning dependency minimums and tool selection with the maintained patch.
  Public APIs, wire/storage formats and lifecycle behavior remain unchanged.

## [0.40.0] - 2026-10-23

- **Breaking Rust dependency identity:** adopt published Host 0.66.0. Consumers
  exchanging Host checksum/publication errors with Backup must align their Host
  dependencies to 0.66. Wire/storage formats and recovery contracts are unchanged.
- Adopt Testkit and the contributor release editor at 0.66.0. Simulator fixtures
  explicitly transfer client/server ownership through Testkit's maintained API,
  retaining their existing cleanup custody and raw output. Prepare the selected
  tools with `make install-testkit-server` and `make install-release-editor`.

- Adopt Shared Tooling 0.26.8, fixing setup lock withdrawal races and removing the
  global Cargo source-cache initialization lock. Run `make install-tools` once to
  refresh Cargo-tool selections under the changed build policy.

## [0.39.0] - 2026-10-23

- **Breaking Rust dependency identity:** adopt published Host 0.65.0. Consumers
  exchanging Host checksum/publication errors with Backup must align their Host
  dependencies to 0.65. Wire/storage formats and recovery contracts are unchanged.
- Adopt Testkit and the contributor release editor at 0.65.0. Backup does not use
  Testkit's optional Wire helpers changed in this release. Prepare the selected
  tools with `make install-testkit-server` and `make install-release-editor`.
- Adopt Shared Tooling 0.26.7: explicit Cargo tool setup waits for an occupied
  tool slot with a bounded budget and rechecks inputs before reuse. Offline
  admission remains immediate and never installs.

- Document Canic's adopted dependency boundary, the remaining application-runner
  integration and acceptance sequence ahead of the post-Blob backup attempt.

## [0.38.0] - 2026-10-22

- **Breaking Rust dependency identity:** adopt published Host 0.64.0. Consumers
  naming or exchanging Host types exposed by Backup must align their Host
  dependencies to 0.64. Preserve public checksum-error conversion and typed
  declaration causes ([#66 — Align artifact error conversion with published Host 0.64](https://github.com/dragginzgame/ic-backup/issues/66)).
  Wire/storage formats and recovery behavior are unchanged.
- Adopt Testkit and the contributor release editor at 0.64.0. Simulator clients
  and DTOs now come directly from Platform's development-only PocketIC feature;
  Testkit retains server admission and managed process ownership. Prepare the
  updated tools with `make install-testkit-server` and `make install-release-editor`.
- Adopt Shared Tooling 0.26.6, including caller-relative validation-log paths,
  declared fixture companions, CI/release integrity guidance and clarified
  asynchronous CI/runtime approval boundaries.

## [0.37.4] - 2026-10-22

- Qualify locked Host/Testkit 0.63.3, Platform 0.6.7 and Tokio 1.54.1 on Linux
  through Core/Agent tests, authenticated simulator journeys and independent
  Rust 1.91 and package consumers.
- Focus lost-reply simulator cases on their recovery boundaries and reuse the
  production workflow driver for successful source preparation. Retain distinct
  recovery assertions and connected same-ID round-trip coverage.
- Reject incomplete expected-byte coverage before simulator readback verification,
  preserving complete stopped-state restoration checks.
- Bind simulator evidence to Testkit's server selection and the retained dependency
  lock instead of hardcoded client/server version labels.
- Keep implemented contracts at their canonical documentation owners; retain
  extraction rationale, future-product proposals and linked historical evidence.

## [0.37.3] - 2026-10-21

- Qualify Host/Testkit 0.63.2 and Platform 0.6.5 with the selected authenticated
  simulator and independent consumer capture/restore and interruption checks.

- Adopt Shared Tooling 0.26.4, including asynchronous hosted CI for ordinary
  development and authorized merges. Focused checks and enforced GitHub checks
  and reviews still apply; release qualification remains separate.
- Preserve primary validation failures and complete bounded diagnostics with the
  refreshed shared runner; reject NUL-corrupted checksum output. Shell checks
  follow sourced helpers, and CI help describes prepared-cache validation.
- Manual IC tool setup and checks now require the same explicit selection, such
  as `IC_TOOLS=wasm-opt`. Required host/Cargo/Testkit setup is unchanged; existing
  IC bundles and failed evidence stay retained.
- Keep dependency fetching in explicit setup; ordinary CI validation consumes
  the prepared cache without dispatching another fetch
  ([#64 — Keep dependency fetching in setup rather than ordinary CI validation](https://github.com/dragginzgame/ic-backup/issues/64)).
- Deliver terminal settlement replay under the held original journal lock and
  guarded source-reference release. These implementation changes were described
  in the 0.37.1 and 0.37.2 notes but were absent from those releases' source.
- Qualify independent Core/Agent consumer processes through authenticated capture,
  durable publication, same-ID restore and effect-free terminal replay. Process
  death after authenticated load preserves pending spending and refuses redispatch.
  Production application qualification remains open under
  [#29 — Complete original-plan execution and stage binding before Canic runner adoption](https://github.com/dragginzgame/ic-backup/issues/29).

## [0.37.2] - 2026-10-21

- Deliver terminal settlement replay under the held original journal lock and
  guarded source-reference release. These implementation changes were described
  in the 0.37.1 notes but were not included in that release's source.
- Qualify independent Core/Agent consumer processes through authenticated capture,
  durable publication, same-ID restore and effect-free terminal replay. Process
  death after authenticated load preserves pending spending and refuses redispatch.
  Production application qualification remains open under
  [#29 — Complete original-plan execution and stage binding before Canic runner adoption](https://github.com/dragginzgame/ic-backup/issues/29).

## [0.37.1] - 2026-10-21

- Allow terminal providers to replay complete workflow settlement under the
  already-held original journal lock. Preserve strict record, digest and bounds
  checks without releasing custody or attempting to reacquire that lock.
- Extend the connected PocketIC journey through guarded source-reference release
  and effect-free replay. Missing or mismatched application acceptance and lost
  replies retain the original references. Production application qualification
  remains open under [#29 — Complete original-plan execution and stage binding
  before Canic runner adoption](https://github.com/dragginzgame/ic-backup/issues/29).
- Qualify an independent Core/Agent consumer process over the complete capture,
  durable artifact, upload, same-ID restore and guarded reference-release journey.
  Cold replay makes no new calls; process death after authenticated load retains
  the pending original and refuses redispatch. Testkit stays outside its normal
  dependency graph. This is isolated application evidence, not Canic acceptance.
- Consumers sharing Host public types can use published Backup 0.37.0 with Host
  0.63; [#62 — Align published Backup public types with Host](https://github.com/dragginzgame/ic-backup/issues/62)
  is resolved after its native qualification completed.

## [0.37.0] - 2026-10-21

- **Breaking:** Adopt published IC Tooling Host Artifacts, FS and Process 0.63.0.
  Consumers sharing `PersistenceError::Publication` or the `DirectoryChecksumError`
  conversion into `ArtifactError` must align their direct Host dependencies to
  0.63. Publication phases, stored records and recovery behavior are unchanged.
- Align Testkit and the release editor with IC Tooling 0.63.0. Their installers
  explicitly select the required `cli` feature; rerun `make install-tools`.
  Library dependencies keep CLI features disabled. Testkit no longer introduces
  IC Metrics or a second Platform version.
- Adopt published Platform 0.6.0 with existing features. Principal admission,
  management wire formats, decoder limits and Agent submission behavior remain
  unchanged.

## [0.36.1] - 2026-10-21

- Qualify published IC Tooling Host/Testkit 0.62.1 and Platform 0.5.1 with the
  existing features and Rust 1.91 floor. Align the explicit release editor with
  IC Build 0.62.1; refresh prepared tools with `make install-tools`.
- Adopt Shared Tooling 0.25.9's guard against a second increment after undelivered
  committed preparation, plus numbered pending-note support
  ([Shared Tooling #292 — Prevent committed release preparation from silently incrementing twice](https://github.com/dragginzgame/shared-tooling/issues/292)).
  Keep the reviewed snapshot while the optional IC installer's 0.26 macOS fix
  awaits a committed release.
- Remove unused IC executables from required setup, CI and release preflight.
  Manual `make install-ic-tools` and `make ic-tools-check` remain available;
  existing bundles and failed evidence are retained. Testkit admission is unchanged.
- Separate Shared Tooling and Platform adoption history from current guidance;
  manifests, the snapshot and the handoff remain the selection/evidence owners.

## [0.36.0] - 2026-10-20

- **Breaking:** Adopt Host Artifacts, FS and Process 0.32.0. Consumers sharing
  `PersistenceError::Publication` or the `DirectoryChecksumError` conversion into
  `ArtifactError` must align their direct Host dependencies to 0.32. Publication
  phases, retained evidence and persisted records are unchanged.

- Adopt published Platform 0.5.0 without expanding features or changing public
  types, management wire formats, decoder limits or Agent submission behavior.

- Align simulator qualification with published Testkit 0.60.0, sharing Platform
  0.5 decoder identities. Prepare its exact CLI with `make install-testkit-server`
  before running offline checks.
- Adopt Shared Tooling 0.25.7's early Clippy gate: release validation runs the
  unchanged lint scope before expensive checks, and publication/dry runs stop
  before Cargo packaging or upload when lint fails, including under parallel Make.
- Adopt Shared Tooling 0.25.6: failed release metadata checks retain Cargo output,
  root inputs and separate Cargo/logger statuses, preserving the original failure
  without retries. See [Shared Tooling #291 — Retain failed Cargo metadata diagnostics and root inputs](https://github.com/dragginzgame/shared-tooling/issues/291).

## [0.35.1] - 2026-10-20

- Align simulator qualification with published Testkit 0.59.0, sharing the
  production Host 0.31 and Platform 0.4 dependency identities. Prepare its selected
  CLI with `make install-testkit-server` before running offline checks.
- Use complete manifest-and-lockfile preparation for release metadata edits,
  removing the unused manifest-only path while retaining comments, original
  inputs and rollback checks.
- Compile the public Host checksum-error conversion in the independent Rust 1.91
  consumer, covering the identity mismatch reported in
  [#62 — Align published Backup public types with Host 0.29](https://github.com/dragginzgame/ic-backup/issues/62).

## [0.35.0] - 2026-10-20

### Breaking

- Adopt Host Artifacts, FS and Process 0.31.1. Consumers sharing
  `PersistenceError::Publication` must align their Host FS dependency to 0.31
  for the carried `NamedWriteError<std::io::Error>` identity.

### Changed

- Adopt published Platform 0.4.0, preserving principal admission, management wire
  DTOs, bounded decoding and Agent submission behavior.
- Adopt committed Shared Tooling 0.25.4 and its grouped formatting defaults.
  Release preparation uses the explicitly prepared IC Build Cargo editor; retire
  the superseded Perl renderers and retain original inputs and rollback checks.
- Enforce regular source files and complete package-license copies, including
  exact license bytes in both prepared Cargo archives.

### Fixed

- Collect failed CI evidence through the shared orchestrator. Failed selector
  output remains diagnostic; redirected parents are retained as links without
  traversing their contents or losing independent validation logs.

## [0.34.1] - 2026-10-20

### Changed

- Adopt published Platform 0.3.0's owned principal for internal admission, converting
  explicitly at management and Agent boundaries. Preserve external DTO/SDK
  identities, canonical text, wire bytes, decoder limits and persisted records.
- Share immutable workflow and checkpoint read admission through the existing
  persistence owner, preserving record limits, lock scope and typed errors.
- Describe implemented workflow coordination separately from integration-owned
  application admission and terminal evidence.

### Testing

- Prepare ordinary workflow fixtures through the production stage owner, while
  retaining explicit record-only, missing-journal and interrupted-preparation cases.
- Qualify the pending Host 0.30.1 lock through the complete Linux validation gate,
  including Agent transport, independent Rust 1.91 consumers and package archives.

## [0.34.0] - 2026-10-19

### Breaking

- Align the pending Host Artifacts, FS and Process update to 0.30.0. Consumers
  matching or constructing `PersistenceError::Publication` with Host FS types
  must align to Host FS 0.30 for the carried error's Rust identity.

### Changed

- Adopt published Platform 0.2.2 for Candid, Principal, management DTOs and Agent.
  Preserve existing host-only features, bounded management codecs and upstream
  identities; no CDK runtime or application codec migration is required.

### Fixed

- Restore simulator fixture compilation with Testkit 0.56.0 and require Platform
  0.2.2's published client exports, preserving exact management routing and
  upstream identities. Resolves the consumer blocker tracked in
  [Platform #27](https://github.com/dragginzgame/ic-platform/issues/27).
- Render release manifest edits through the reviewed Shared Tooling owner, accepting
  retained comments, dependency aliases and table forms while preserving requirement
  operators and precision. Keep original receipts, lock selection and rollback.

### Testing

- Use one simulator workflow driver and capture provider for single-stage and
  connected journeys, preserving lost-reply spending and no-redispatch assertions.

## [0.33.0] - 2026-10-19

### Breaking

- Adopt Host Artifacts, FS and Process 0.29.0. Consumers matching or constructing
  `PersistenceError::Publication` with Host FS types must align to Host FS 0.29;
  its carried `NamedWriteError<std::io::Error>` has that release's Rust identity.

### Changed

- Keep compound upload submission and independently qualified settlement under one
  original journal guard, reusing the admitted reply. Preserve public APIs, exact
  evidence, pending spending and checkpointing after guard release.
- Organize current implementation obligations by capability and remove superseded
  restore-copy and dependency-adoption instructions.

### Testing

- Compile retained WAT fixtures through Host Artifacts with explicit source/output
  bounds. Remove direct WAT dependencies and keep text compilation test-only.
- Consolidate overlapping simulator journeys around production coordinators,
  retaining first-read and retained-prefix loss checks, source custody and all
  distinct successful-reconciliation scenarios.
- Qualify connected capture-to-restore workflows with frozen source/restore
  allocations, cold continuation, lost replies and effect-free completed replay.
  Keep application authority and terminal-release acceptance integration-owned.

## [0.32.0] - 2026-10-19

### Breaking

- Adopt Host Artifacts, FS and Process 0.28.0 as direct production dependencies.
  `PersistenceError::Publication` now carries Host FS 0.28's
  `NamedWriteError<std::io::Error>` Rust identity. Consumers matching or constructing
  that error with direct Host FS types must align their dependency to 0.28.

## [0.31.4] - 2026-10-19

### Changed

- Share post-admission stage scheduling between fresh and retained workflow runs.
  Preserve create-only fresh admission, complete retained-evidence checks, exact
  successor commitments, callback ordering and original spending/replay behavior.
- Reconcile implementation contracts with the current Host publication error and
  guarded restore-reference release; retain migration history in this changelog.

## [0.31.3] - 2026-10-18

### Changed

- Qualify the selected published Platform 0.1.14 and Testkit 0.54.1 graph.
  Preserve Candid/Principal/management identities, decoder limits and one-dispatch
  Agent behavior. Focused Linux checks include Rust 1.91 independent consumers and
  actual Core/Agent PocketIC journeys; native macOS acceptance remains separate.

## [0.31.2] - 2026-10-18

### Changed

- Select Rust 1.99.1 for development, retaining the Rust 1.91.0 consumer minimum.
  Prepare its tool receipts with `make install-rust-tools` and
  `make install-testkit-server` before offline qualification.
  Addresses [#58 — Qualify Rust 1.99.1 as the development compiler](https://github.com/dragginzgame/ic-backup/issues/58).

### Fixed

- Align the README compiler floor with the existing manifests and qualification.
  Addresses [#59 — Reconcile the advisory root README compiler-floor statement](https://github.com/dragginzgame/ic-backup/issues/59).
- Compare the admitted common Rust tool set during native CI reuse instead of
  requiring the retired Candid extractor. Preserve exact resolved-path checks.
  Align local setup guidance with the same reviewed catalog.
  Addresses [#60 — Compare admitted Rust tools in native reuse qualification](https://github.com/dragginzgame/ic-backup/issues/60).

## [0.31.1] - 2026-10-18

### Changed

- Adopt published `ic-platform` 0.1.8 as the Candid boundary for management
  encoders, bounded decoders, derives and test oracles. Remove direct Candid and
  standalone Testkit codec dependencies; retain exact wire bytes, decoder limits,
  public upstream identities and redacted errors. Enforce direct dependency
  ownership with Shared Tooling’s canonical boundary check.
- Source every principal through Platform’s root export, preserving upstream
  0.1.5 identity, normalization and hashes. Remove the direct principal dependency
  and exact-pin exception, and enforce the ownership boundary.
- Adopt Platform’s native Agent feature for supported transport/signing identities.
  Keep Reqwest client policy explicit. Published `CallResponse` and
  `AnonymousIdentity` exports complete the SDK migration; remove its direct
  dependency and policy exception. Migrate all 24 snapshot/status DTOs through
  Platform too, removing the direct management dependency and final pin exception
  while preserving upstream 0.11.0 identities. Completes
  [Platform #13](https://github.com/dragginzgame/ic-platform/issues/13).
  Preserve single dispatch, durable retention, bounds and pending spending.

- Adopt reviewed Shared Tooling 0.24.0 with dependency-role groups and matching
  grouped formatting in hooks, CI and selected-source release checks. Keep
  current package versions and dependency selections unchanged by this adoption.
- Extend independent consumer admission to reject WAT tooling alongside simulator
  packages, including transitive and renamed identities. Document the separate
  downstream deployment artifact/evidence obligation.

- Adopt published Testkit and standalone Candid codec 0.53.0, aligning their
  transitive Host dependencies with the already-selected 0.27.1 packages.
  Keep simulator qualification in development dependencies and management
  decoder limits and product contracts unchanged.

- Call published Testkit library server admission directly in both simulator
  fixtures and remove the shell/Make/CLI bridge. Keep explicit setup, offline
  admission, private server digests and managed logs. Resolves #55 after native
  consumer qualification and delivery.

- Remove stranded test-only JSON publication wrappers; crash and parent-custody
  regressions use the bounded encoder and publication engine directly. Reuse the
  verified private FIFO fixture helper for record-admission qualification.

- Use Host Artifacts hashing writers for snapshot region streams, removing the
  parallel local hash-state array and moving direct `sha2` use to test dependencies.
  Preserve coverage, descriptor/extent checks, exact retained bytes and publication.
- Retire the producer-only Host checksum regression and duplicate record checks;
  keep record vectors with the model and publication/recovery and projection
  checks with their adapters. Use local names for test roots.
- Retire unused exported producer fixtures through the canonical Shared Tooling
  exporter. Keep real dependency-policy admission and consumer, hook, release and
  evidence qualification; correct stale active ownership and Host API guidance.
- Correct active ownership and transport guidance, and archive the pre-release
  handoff while retaining all historical receipts and source attribution.

### Fixed

- Restore hook sorting qualification after Platform adoption by perturbing current
  inherited dependencies. Retain fixture-setup diagnostics in the case log.

## [0.31.0] - 2026-10-17

### Breaking

- Adopt Host Artifacts, FS and Process 0.27.1. Consumers sharing exposed Host FS
  error types must align direct Host dependencies with 0.27. Native command file
  identity now comes from Host FS; persisted identities and custody stay exact.

- Replace AgentMutationProvider and AgentSnapshotUploadProvider with
  AgentUpdateProvider, implementing both existing write ports. Retention callbacks
  accept a borrowed ReservedUpdate and handle its Mutation or Upload variant.
  Exact signed retention, request admission, distinct reply domains and pending
  spending remain unchanged. No compatibility aliases are retained.

### Changed

- Refresh the reviewed Shared Tooling snapshot to 0.23.0, retaining consumer
  setup/check ordering and explicit Testkit ownership. Prepare ICP CLI 1.7.0 with
  `make install-ic-tools` before offline tool admission; other tool pins stay fixed.
- Qualify the selected Testkit/Candid 0.52 graph; simulator startup uses explicit
  Spawn configuration with the original timeout, output files and TTL settings.

- Directory publication delegates ordered sync/checksum traversal to Host FS,
  removing the local visitor
  and unused raw stream adapter. Preserve exact checksum framing, native error
  projections, crash boundaries and original publication/recovery custody.

## [0.30.0] - 2026-10-17

### Breaking

- Remove Agent `TransportError::{Reservation, EnvelopeTooLarge, Indeterminate,
  ReplyTooLarge}` without aliases. Construction retains configuration/context/
  identity errors; provider errors own dispatch outcomes. Private signing/admission
  now returns the existing provider refusal directly before ingress retention.
- Remove public `ops::artifacts::checksum_directory`, `checksum_relative_files`
  and the local `DirectoryChecksumError` without compatibility reexports. Use
  Host FS `read::tree::checksum` and Host Artifacts `artifact::tree`; convert raw
  SHA-256 identities into product records at the caller boundary. Exact retained
  tree framing, snapshot layout admission and publication/recovery checks remain.

### Changed

- Carry IC snapshot member names, length bounds, expected digests and payload roles
  from one product owner into verification and upload preparation. Remove dummy
  region hashes, positional classification and filename-prefix interpretation;
  retain exact v1 bytes, closed-tree admission and descriptor identity checks.
- Adopt published Host Artifacts, FS and Process 0.26.1. Host owns pure directory
  checksum composition and no-follow directory hashing; Backup retains private
  record/error projections and journal-bound snapshot custody.

## [0.29.0] - 2026-10-17

### Breaking

- Remove `checkpoint_execution_settlement`, public `create_execution_settlement`
  and `ExecutionStageGuard::checkpoint` without aliases. Use
  `reconcile_execution_settlement` or `ExecutionStageGuard::reconcile_checkpoint`
  to create an absent checkpoint or replay an exact occupied one. Read-only replay
  remains separate and refuses absent evidence; original journals are never recreated.

- Replace `IcMutationAcknowledgementInput` and `IcSnapshotTransferReadResponseInput`
  with `model::ic_update_response::IcUpdateResponseInput`, without compatibility
  aliases. Update imports and input construction; each response wrapper retains its
  existing byte limit, typed errors and method-specific decoding.

### Changed

- Re-admit returned evidence directly in capture, transfer-read, upload and fence
  workflows, preserving journal/association/stage order and exact replies in errors.
- Handle Agent responses directly in the retained single-submission owner, removing
  the private intermediate outcome and repeat validation of the immutable journal.
  Signed ingress retention, bounded replies and indeterminate pending outcomes remain.

- Adopt committed Shared Tooling 0.20.0 through the canonical snapshot exporter.
  Preserve Backup's IC tools through explicit ordered setup/check selection and
  PATH publication after the common host/Cargo steps. Update consumer qualification
  and release fixtures; reviewed executable pins and product dependencies stay unchanged.

- Use the published standalone `ic-testkit-candid` codec for bounded exact-result
  decoding, preserving method quotas, bounded DTOs, wire bytes and redacted errors.
  The existing locked 0.51.0 package becomes a direct production dependency;
  simulator dependencies remain confined to qualification.
- Map Host filesystem locking failures directly into journal lock errors, removing
  the intermediate private error layer while preserving custody and contention.

- Prepare data-upload plans with a bounded batch reader. Payload bindings and the
  canonical source checksum use the same bytes, with full source checks before and
  after the batch. Source passes no longer grow with the number of planned extents;
  actual writes retain fresh per-payload source verification and original spending.
- Make admission, retained-evidence rechecks and single provider invocation explicit
  at capture, transfer-read, upload and fence acquisition workflow boundaries.


## [0.28.0] - 2026-10-17

### Breaking

- Remove `ops::persistence::PublicationError` without an alias.
  `PersistenceError::Publication` carries
  `ic_host_fs::durable::NamedWriteError<std::io::Error>` directly. Update imports
  and exhaustive matches to Host's enum and align a direct `ic-host-fs` dependency
  with Backup's selected 0.24 line. Producer, cleanup and publication-phase evidence
  remain unchanged; visible output after failed completion still needs reconciliation.

- Remove the generic public `ops::persistence::write_json_durable` and
  `create_json_durable` APIs. Use Backup's guarded record operations for product
  persistence. For independent documents, encode with an explicit size limit and
  publish the admitted bytes through `ic_host_fs::durable::write_bytes`.

- Remove generic `ops::artifacts::stage_relative_path` copying and the unused
  `checksum_file`, `checksum_path` and `checksum_relative_path` selectors. IC
  snapshot staging remains with `DownloadJournalGuard::stage_ic_snapshot_artifact`.
  Independent file hashing/copying belongs to Host FS and Host Artifacts with
  caller-selected custody, bounds and publication; retain existing partial trees.
- Make `ops::artifacts::checksum_reader` and `ops::persistence::read_json` internal.
  Use guarded Backup record readers for product evidence. Independent JSON callers
  can use Host FS bounded regular-file reads followed by their own decoding.

- Remove `IcSnapshotLocalMetrics`, `DownloadJournalGuard::ic_snapshot_metrics` and
  Metrics type reexports, along with Core's direct Metrics dependency. Measure work
  in caller instrumentation or performance fixtures; choose `ic-metrics` directly
  for report arithmetic. Retained records and recovery are unchanged.
- Remove public `AgentTransport::prepare`, `PreparedUpdate::submit` and
  `UpdateOutcome`. Use the configured Agent providers and their mandatory durable
  ingress retainer. `PreparedUpdate` still exposes signed bytes/request IDs to the
  retainer; accepted, lost and failed calls remain pending and grant no retry.

### Changed

- Update the independent MSRV consumer to verify the canonical publication-error
  type against the selected Host FS dependency, replacing the removed diagnostics
  imports without changing the locked dependency graph.

- Share journal-parent resolution between command custody and restore retention,
  preserving absent-leaf admission, alias rejection and caller-specific errors.
- Share ordered update response identity checks across mutation, transfer read and
  upload policy. Original reservation checks run first, followed by authority,
  attempt, context and target checks, then the original bounded method decoder.

- Share directory checksum framing with the existing Host-backed digest owner,
  preserving exact component ordering and original path/digest/NUL/newline bytes.
- Stream remaining canonical record, upload-stage and Agent reply digest framing
  through Host Artifacts instead of allocating whole framing buffers. Original domains, field
  order, integer encoding and v1 identities remain unchanged.
- Use Host Artifacts' typed SHA-256 digest for chunk comparisons and filenames,
  removing the private generic hex encoder without changing retained artifact names.
- Bound manifest and restore-reference release encoding before publication using
  their existing record limits, retaining guarded verification and crash recovery.
- Expose `ArtifactChecksumRecord::from_digest` for already-computed raw SHA-256
  bytes; this wraps a digest without hashing it again or granting byte custody.

## [0.27.1] - 2026-10-16

### Changed

- Delegate inherited command custody and exact existing-sidecar quiescence to
  published Host Process, retaining original records, bounded grace and spending
  ([#53 — Adopt published Host Process inherited command custody](https://github.com/dragginzgame/ic-backup/issues/53)).
- Delegate sidecar acquisition and verified directory publication to Host FS,
  preserving ordered crash barriers, checksum verification and retained recovery
  ([#52 — Adopt published Host lock and directory commit primitives](https://github.com/dragginzgame/ic-backup/issues/52)).
- Decode snapshot read replies once through qualification and append/receipt
  handling under the same original journal guard, retaining interruption recovery.
- Store only the checksum hash in memory; JSON still emits and validates the
  required SHA-256 algorithm with unchanged canonical bytes.

- Encode bounded journal and declaration publication bytes once before effects,
  preserving exact pretty-JSON limits, public value writers and crash recovery.
- Reuse the admitted upload extent list during final plan assembly, retaining
  original allowance checks before source IO and fresh source verification.

### Testing

- Run handwritten simulator fixtures directly from WAT-generated Wasm, removing
  the fixture-only optimizer step. Generic optimizer qualification stays upstream
  with Shared Tooling; existing fixture and optimizer evidence remains retained.

## [0.27.0] - 2026-10-16

### Breaking

- Remove generic restore-artifact staging, publication and copy-verification APIs,
  `LocalRestoreArtifactView` and their error types. Use IC snapshot upload preparation
  from verified original artifacts. Retain old copies and settle unfinished copy-based
  restores with the original released tool before adopting this cut; no automatic
  import or cleanup is provided.

### Changed

- Share ordered passive settlement identity checks across lifecycle, capture,
  allocation and upload-data recovery, preserving typed denials and pending spending.
- Share exact qualified Applied receipt handling across metadata, download, upload
  and restore while preserving append-before-receipt and source-retention ordering.
- Use published Host FS for no-follow layout and snapshot-artifact directory
  selection, retiring the duplicate local opener while preserving custody checks.
- Use Host FS for no-follow root-relative artifact selection, retiring Backup's
  duplicate component walker while preserving path and artifact admission.
- Use published Host Process for bounded Agent test-server HTTP capture, retiring
  duplicate framing and read-loop code while retaining raw request diagnostics.
- Use Host Artifacts' exact-length hash and read APIs for IC snapshot verification
  and upload source slices, preserving Backup's file-shape and I/O error mapping.

## [0.26.0] - 2026-10-16

### Breaking

- Directory artifact staging now refuses carriage returns or newlines in the
  destination or any nested directory path. Choose directory paths without
  those bytes; an already-created partial staging tree is retained for inspection
  rather than removed or retried.

### Changed

- Share aggregate-bound sequence decoding for effect graphs and ordered
  observation admission across membership, controller and snapshot-read policy.
  The existing bounds and denial order remain intact.
- Reuse bounded sequence decoding for the remaining controller, snapshot-ID and
  restore-reference boundaries while retaining their byte and count limits.
- Parse principal text once when constructing IC requests, snapshot metadata
  reads and application-fence payloads; canonical target and wire bytes still
  bind to the same principal.
- Use Host FS directory identity admission for layout and snapshot-artifact
  custody checks, preserving their typed replacement errors. The local
  no-follow opener remains until its Host replacement is published.
- Use Host FS private-directory creation for artifact staging and workflow
  stages, including durable parent-link synchronization and explicit
  interrupted-creation errors.
- Share exact hash and big-endian integer framing across consistency,
  restore-safety and fence-reconciliation request digests without changing
  their domain tags or bytes. Use Host FS for descriptor unlock; existing-only
  sidecar admission remains local pending the upstream lock primitive.
- Stream framed evidence into Host Artifacts' SHA-256 writer without retaining
  a second copy of the framing bytes; checksum records and digests are unchanged.
- Use Host FS's post-sync admission callback for JSON crash qualification,
  removing Backup's duplicate pre-publication file sync.

### Testing

- Wait a bounded interval for a reaped command's inherited sidecar to become
  quiescent in the restore-reference regression, while checking that in-flight
  attempts leave the original reference bytes intact.
- Share simulator call setup and lost/settled reply bookkeeping across mutation
  and transfer-read journeys while retaining their distinct reply validation.

## [0.25.1] - 2026-10-15

### Changed

- Use one bounded sequence decoder for persisted records and IC replies while
  retaining the IC wire decoder's size-hint and typed excess-value checks.
- Share capture and upload inventory-settlement admission and metadata-derived
  data-stage plan construction while retaining each method's original authority,
  attempt limits and outcome requirements.
- Frame admitted IC reply evidence through one local digest helper without
  changing the existing domain-separated hashes.
- Share pending observation-attempt admission and Agent reply projection across
  the three recovery readback lanes, retaining their method-specific checks and
  evidence domains.

### Fixed

- Include the repository's MIT license text in the published Agent package,
  matching the Core package's distribution.
- Create the package-check fixture root in fresh checkouts before validating
  both archives and their independent consumer.

## [0.25.0] - 2026-10-15

### Breaking

- Raise the minimum supported Rust version to 1.91.0. Library consumers must
  build with Rust 1.91.0 or newer.
- Replace the Host error inside `PersistenceError::Publication` with Backup's
  `PublicationError`. Consumers matching that variant must switch to the new
  type; producer, cleanup and before/after-publication evidence remain available.
- Select Host FS, Process and Artifacts 0.19.0. JSON publication now rejects
  symlink ancestors and `..` in output parents before creating directories;
  intentionally selected aliases must be resolved at the owning input boundary
  ([#50 — Qualify Host private-parent refusal](https://github.com/dragginzgame/ic-backup/issues/50)).

### Changed

- Use the published Host descriptor walker and test-only crash barrier in place
  of local copies, retaining Backup's artifact and recovery policy
  ([#48 — Adopt published Host descriptor walks and test-only crash barriers](https://github.com/dragginzgame/ic-backup/issues/48)).
- Adopt reviewed Shared Tooling 0.15.0, including its release and fixture helpers
  and focused local validation cadence.

### Testing

- Qualify the published Testkit 0.45.0 dev dependency and exact managed-server
  CLI across the Agent HTTP and both PocketIC suites. Testkit's pending typed
  HTTP capture remains upstream; Agent evidence retention stays local.
- Keep consumer CI, snapshot, evidence and Testkit command fixtures while moving
  generic Shared Tooling implementation regressions back to their upstream gate
  ([#51 — Trim repeated upstream tooling regression](https://github.com/dragginzgame/ic-backup/issues/51)).
- Share offline lockfile preparation and graph admission between independent
  MSRV consumers and the paired package archive consumer.

## [0.24.7] - 2026-10-15

### Testing

- Use Host FS's published fixture allocator for Core unit-test roots as well as
  integration roots, retiring the local timestamp/counter naming and redundant
  root creation. Tests that need an absent artifact use a child of an allocated
  fixture directory.
- Qualify local diagnostic assertions and the independent MSRV consumer against
  Metrics 0.10's summary aggregate projection.

## [0.24.6] - 2026-10-14

### Changed

- Use one stage preparation, execution and checkpoint sequence for new and retained
  workflows, preserving their original evidence and spending rules.
- Reuse the bounded persisted-sequence decoder for effect dependencies and
  settlement journals, preserving their limits and error labels.

### Testing

- Use Host FS's published private fixture allocator for Core and Agent integration
  roots. Retire the duplicate collision loops and their redundant allocation tests;
  each journey still owns its retained evidence and cleanup.

## [0.24.5] - 2026-10-14

### Changed

- Share ordered observed network, caller and release matching across policy
  gates, and one v1 snapshot member mapping across artifact write, read and
  verification paths. Typed mismatches and on-disk names stay unchanged.

## [0.24.4] - 2026-10-14

### Changed

- Share bounded persisted-sequence decoding and local directory custody checks
  while preserving each record's limits, errors and no-follow behavior.
- Reuse the bounded sequence reader for execution predecessors and download
  artifacts, and one Candid decoder setup for IC replies while preserving each
  method's wire limits and rejection behavior.
- Share the bounded Candid sequence reader for snapshot inventory, metadata and
  data replies, retaining each field's limit, error and typed excess probe.
- Isolate Agent endpoint admission as a pure written-origin check while keeping
  trust, signer and timeout policy in the transport constructor.

### Testing

- Share the independent Candid golden generators' encoding primitives without
  changing the checked-in metadata, data or upload vectors.
- Build both PocketIC fixtures through Testkit's managed sync/async constructors,
  retaining their separate simulator state, server output and application setup.

## [0.24.3] - 2026-10-14

### Changed

- Share inventory-backed current-target admission between consistency and restore
  safety, and original plan/journal reads between progress and settlement, while
  preserving their distinct errors and selected-journal custody.
- Use Host FS 0.17.11 for durable private parent preparation, retiring Backup's
  duplicate directory walk while preserving owner-only modes and crash barriers.
- Share bounded canonical principal-set admission, the reserved single-submit
  workflow sequence, and exact Applied-receipt recording across snapshot stages.
  Their existing typed errors, authority checks and reply retention remain local.
- Share the Agent adapters' retained single-update sequence while preserving each
  port's reply evidence and fence reply-retention requirement.
- Use one no-follow artifact-tree walk for checksum, staging and commit; commit
  still owns file/directory sync order, no-replace publication and recovery.

### Testing

- Share offline Testkit admission, server checksum and Wasm optimization setup
  between the core and Agent PocketIC fixtures, retaining each fixture's own
  server output, canister state and evidence.

## [0.24.2] - 2026-10-14

### Fixed

- Reject written dot-segment and backslash paths, whitespace and control
  characters in Agent endpoints before URL normalization can admit them.

### Changed

- Adopt the reviewed Shared Tooling 0.11.2 snapshot, including its shared release-tag
  check and fixture failure retention. Contributor ShellCheck setup continues through
  `install-ci-tool.sh`; shared tool selections are unchanged.
- Qualify the Host Artifacts, FS and Process 0.17.7 lock selection. Backup retains
  its existing publication and recovery contracts; the new Host retained-file APIs
  now create the private IC snapshot files, synchronizing their directory as each
  file is named while retaining incomplete bytes for recovery.
- Qualify the Metrics 0.9.2 and Testkit 0.38.0 lock selections. Backup's
  diagnostic arithmetic and retained record contracts are unchanged.
- Share bounded immutable requirement/obligation persistence and Agent observation
  response assembly. Original plan checks, typed mismatches and the three distinct
  reply-evidence domains remain unchanged.

### Testing

- Keep the isolated release adapter fixture aligned with Shared Tooling's
  exact annotated-tag check.

## [0.24.1] - 2026-10-13

### Testing

- Run the complete native macOS gate once under system Bash 3.2, including
  Testkit GNU Make routing, with a 90-minute cold-runner budget
  ([#37](https://github.com/dragginzgame/ic-backup/issues/37)).
- Share independent consumer dependency-graph admission between the MSRV and
  package gates; CI invokes the common ShellCheck installer directly.

### Changed

- Share immutable record publication/admission, mutation and upload single-submit
  acknowledgement handling, and exact inventory-settlement identity checks.
- Qualify the Host Artifacts, FS and Process 0.17.3 lock selection. Their
  published Rust source is unchanged from 0.17.2; Backup's public contracts,
  retained records and effect authority are unchanged.
- Consolidate no-follow directory traversal for local artifact checksums and
  staging while retaining the existing durable publication checks.

## [0.24.0] - 2026-10-13

### Breaking

- Adopt Metrics 0.9.0 for Backup's public `MeasurementSummary`,
  `MeasurementAggregate` and `MeasurementHistogram` reexports. Consumers with
  a direct Metrics dependency must align to 0.9 and use the renamed
  `*_overflowed` accessors and current typed errors. Local diagnostics remain
  transient; backup records and spending authority are unchanged.

## [0.23.0] - 2026-10-13

### Breaking

- Select IC Host Artifacts and FS 0.17.1. `PersistenceError::Publication`
  contains Host FS 0.17's `NamedWriteError` Rust type; consumers with direct
  Host dependencies or typed error matches must align to 0.17. Persisted
  backup records are unchanged.

### Fixed

- Recheck the exact terminal journal bytes before guarded restore-reference
  release. Changed bytes keep the original reference and command evidence
  retained ([#29](https://github.com/dragginzgame/ic-backup/issues/29)).

### Changed

- Select Testkit 0.37.0, Metrics 0.8.4 and `rustls-native-certs` 0.8.5.
  Backup's public metrics types and transport configuration are unchanged.
- Adopt Shared Tooling 0.10.0's snapshot-owned common tool catalogs. Existing
  executable versions and digests remain selected; contributor setup now reads
  those catalogs from the reviewed snapshot.

## [0.22.0] - 2026-10-13

### Breaking

- Select IC Host Artifacts and FS 0.16.0. `PersistenceError::Publication`
  now contains Host FS 0.16's `NamedWriteError` Rust type; consumers with
  direct Host dependencies or typed error matches must align to 0.16.
  The owning Host source and Backup's persisted records are unchanged.

### Added

- Expose the exact Applied attempt receipt as a read-only direct-mutation or
  reconciliation-observation view, so integrations can match retained evidence
  without treating observation settlement as the original reply. Original
  journals, attempt spending and effect authority are unchanged
  ([#29](https://github.com/dragginzgame/ic-backup/issues/29)).

### Changed

- Adopt Shared Tooling 0.9.13, including locked Cargo-source preparation
  guidance, stricter tool setup failure retention and the updated change-reporting
  rules. Reprepare contributor Cargo tools with `make install-rust-tools` before
  offline admission; existing tool generations remain retained.
- Qualify the incoming Metrics 0.8.3 lock selection. Backup's existing diagnostic
  summary, aggregate and histogram APIs are unchanged.

### Fixed

- Reject a replaced command-custody sidecar by its recorded identity even when
  another process briefly holds a lock on the replacement. Quiescence still
  checks the opened sidecar after locking it.

## [0.21.3] - 2026-10-13

### Added

- Release an exact restore reference under held layout and journal guards after
  integration-owned terminal qualification and existing command-custody checks.
  Active or replaced command sidecars refuse; exact replay synchronizes the
  current reference state without dispatching effects
  ([#29](https://github.com/dragginzgame/ic-backup/issues/29)).

### Testing

- Qualify original and retained workflow edge commitments against a real
  PocketIC snapshot ID. Recovery refuses a missing retained edge before further
  IC effects, while preserving the original capture and metadata journals
  ([#29](https://github.com/dragginzgame/ic-backup/issues/29)).
- Qualify the incoming IC Host 0.15.2 lock selection; its published Rust sources
  match 0.15.1 across the three selected Host packages.

## [0.21.2] - 2026-10-12

- Allow original-workflow drivers to qualify a distinct learned commitment for
  each successor edge. Retained continuation checks supplied edge commitments
  against occupied bindings before callbacks, while existing drivers keep their
  stage checkpoint digest by default. Original journals, spending and terminal
  authority are unchanged
  ([#29](https://github.com/dragginzgame/ic-backup/issues/29)).

## [0.21.1] - 2026-10-12

- Qualify the Metrics 0.8.1 selection for local snapshot diagnostics and public
  summary, aggregate and histogram reexports. Checked scaled means preserve
  fractional report precision; diagnostic sampling, retained records and effect
  authority are unchanged
  ([#41](https://github.com/dragginzgame/ic-backup/issues/41)).

## [0.21.0] - 2026-10-12

- **Breaking:** Retained workflow continuation now rejects a supplied learned
  stage digest that conflicts with an occupied successor's original binding,
  before invoking the driver. Integrations using distinct learned commitments
  for different edges must use the lower-level stage owners; this coordinator
  accepts one learned digest per stage. The PocketIC recovery fixture now rebuilds
  its driver from retained capture and metadata bytes after interruption
  ([#29](https://github.com/dragginzgame/ic-backup/issues/29)).

## [0.20.5] - 2026-10-12

- Add a one-shot original-workflow coordinator that prepares each fixed stage in
  dependency order, delegates fresh admission and effects to an integration
  driver, and reconciles the complete local checkpoints. A retained-run entry
  point preflights the exact occupied stage set and replays its settled originals
  before dispatching absent successors. Pending or omitted stages refuse without
  redispatch. An isolated PocketIC capture-to-artifact journey exercises the new
  coordinator, including a discarded data reply; product terminal and reference
  release remain separate
  ([#29](https://github.com/dragginzgame/ic-backup/issues/29)).
- Qualify the incoming Host 0.14.10 and Testkit 0.34.6 lock selections together
  with Syn 3.0.9 and AWS-LC 1.18.2/0.46.0. The exact Testkit CLI is prepared and
  the native, simulator, MSRV consumer and package gates pass.

## [0.20.4] - 2026-10-12

- Replay the complete original workflow's local stage checkpoints in one
  read-only view. Every allocated stage must retain its exact bound plan,
  complete Applied journal history and matching predecessor identities;
  missing or changed evidence refuses without publication, spending changes or
  terminal/reference-release authority
  ([#29](https://github.com/dragginzgame/ic-backup/issues/29)).

## [0.20.3] - 2026-10-12

- Adopt Shared Tooling 0.9.0. Snapshot refresh and integrity checks now share
  one manifest reader that rejects NUL bytes and exposes a syntax-only
  `--read-manifest` mode; the local LOC helper can emit complete JSON reports.
  Existing snapshots and human-readable reports remain supported
  ([Shared #133](https://github.com/dragginzgame/shared-tooling/issues/133)).
- Reconcile an uncertain local execution-checkpoint publication using the
  complete original journals. Replay an exact occupied checkpoint without
  replacement, create only when the checkpoint is absent, and read back new
  publication before returning a stage predecessor. Changed or incomplete
  evidence refuses without provider calls, spending changes or release authority
  ([#29](https://github.com/dragginzgame/ic-backup/issues/29)).

## [0.20.2] - 2026-10-12

- Reconcile a lost execution-stage checkpoint response from the complete retained
  original journals and existing immutable checkpoint. The read-only replay
  reconstructs the exact predecessor commitment without republishing, resetting
  attempts or granting terminal/reference release
  ([#29](https://github.com/dragginzgame/ic-backup/issues/29)).
- Adopt Shared Tooling 0.8.3 and add `make check-cargo-tool-versions` for
  explicit online freshness checks of contributor Cargo tools. It reports
  missing or unavailable registry evidence without installing tools or changing
  pins; offline `make rust-tools-check` remains the admission check. Formatting
  now reads the selected Rust tool catalog, and Cargo installation refuses
  redirected build-observation files before activation
  ([Shared #135](https://github.com/dragginzgame/shared-tooling/issues/135),
  [Shared #136](https://github.com/dragginzgame/shared-tooling/issues/136)).

## [0.20.1] - 2026-10-12

- Adopt Shared Tooling 0.8.1. The snapshot records its intended entrypoints and
  derives their companion payloads, retaining the reviewed 102-file selection.
  Snapshot refresh, verification and tooling inventory now reject malformed
  tab-separated fields while preserving literal spaces in paths. Failed Cargo
  tool installation retains both build-environment observations and names changed
  fields without activating a changed selection
  ([Shared #133](https://github.com/dragginzgame/shared-tooling/issues/133),
  [Shared #134](https://github.com/dragginzgame/shared-tooling/issues/134)).
  Its `cargo-sort-derives` 0.14.1 selection preserves literal `$crate` paths in
  direct and conditional derives. Backup has no affected product macros;
  contributors should rerun `make install-rust-tools` and `make tools-check`
  ([#36](https://github.com/dragginzgame/ic-backup/issues/36),
  [Shared #113](https://github.com/dragginzgame/shared-tooling/issues/113)).
- Qualify the selected Host 0.14.5 packages. Their published Rust sources are
  unchanged from 0.14.4; backup behavior and public Host interfaces remain
  unchanged. Qualify the incoming Testkit 0.34.3 and Metrics 0.7.1 lock graph
  without changing backup records or effect authority.

## [0.20.0] - 2026-10-12

- Adopt Shared Tooling 0.6.5 and stop CI when the declared manifest or lockfile
  changes between stages, retaining input identities and restart guidance.
  Qualify Host 0.14.3 with unchanged library behavior
  ([#36](https://github.com/dragginzgame/ic-backup/issues/36),
  [Shared #129](https://github.com/dragginzgame/shared-tooling/issues/129)).

- **Breaking:** Adopt Metrics 0.7 for public local diagnostic summaries and
  histogram aggregates. Totals and integer means now use `u128`; callers sharing
  these types must align their Metrics dependency and report fields. Individual
  samples, counts and buckets remain `u64`. Diagnostics remain volatile and
  existing backup records need no conversion
  ([#40](https://github.com/dragginzgame/ic-backup/issues/40)).

## [0.19.2] - 2026-10-12

- Adopt Shared Tooling 0.6.4's common Make entrypoint for setup, formatting,
  hooks, snapshot verification and releases, preserving ordered Testkit setup
  and Backup's exact resume receipt check
  ([Shared #130](https://github.com/dragginzgame/shared-tooling/issues/130)).

- Qualify original fence acquisition, observation and explicit settlement against
  an isolated capture-only application fixture. Discarded replies retain pending
  spending; retained late observations settle without another read or release.
  Cancelled qualification and changed retained reply bytes preserve the exact
  pending journal; explicit local recovery uses the original observation
  ([#29](https://github.com/dragginzgame/ic-backup/issues/29),
  [#25](https://github.com/dragginzgame/ic-backup/issues/25)).

## [0.19.1] - 2026-10-11

- Add explicit guarded fence-observation settlement with independent application
  qualification. Recheck original plans and complete journals before recording
  an exact receipt; cancellation, rejection and settled uncertainty preserve
  spending, obligations and source references
  ([#29](https://github.com/dragginzgame/ic-backup/issues/29)).

## [0.19.0] - 2026-10-11

- **Breaking:** Select Host Artifact/FS 0.14. Consumers sharing the public
  publication-error type must align with Host 0.14; record formats and local
  custody remain unchanged.
- Adopt Shared Tooling 0.6.3. A normal release retry can archive failed local
  preparation after a clean descendant commit and repeat validation against
  current metadata. Original plans, backups and failed evidence stay retained.
  Reconcile delivered adoption against exact-source CI
  ([#36](https://github.com/dragginzgame/ic-backup/issues/36)).

## [0.18.4] - 2026-10-11

- Adopt Shared Tooling 0.6.2. Cargo setup now stops on lock, validation and
  activation failures without reporting success or changing the prior common
  selection; failed candidates and foreign locks remain retained. Interrupted
  release source conflicts report exact plan, phase and commit identities
  ([shared #120](https://github.com/dragginzgame/shared-tooling/issues/120),
  [shared #126](https://github.com/dragginzgame/shared-tooling/issues/126),
  [#36](https://github.com/dragginzgame/ic-backup/issues/36)).
- Add a guarded fence-acquisition observation step with fresh application
  admission and one durable original-plan reservation. Cancellation and failures
  preserve pending attempts, source references and returned evidence; matching
  replies never automatically settle acquisition or grant fence release
  ([#25](https://github.com/dragginzgame/ic-backup/issues/25),
  [#29](https://github.com/dragginzgame/ic-backup/issues/29)).

## [0.18.3] - 2026-10-11

- Adopt Shared Tooling 0.6.0. Common and selected Cargo tools now reuse shared
  bundles with exact build identities; application flags and wrappers stay local.
  Contributors must rerun `make install-tools` and `make tools-check`. Preserve
  previous selections and failed builds
  ([#36](https://github.com/dragginzgame/ic-backup/issues/36)).
- Use canonical evidence selection for shared Cargo bundles and safe local
  records when store admission fails, preserving uploadable diagnostics and
  explicit incomplete-collection failures
  ([shared #121](https://github.com/dragginzgame/shared-tooling/issues/121)).

## [0.18.2] - 2026-10-11

- Adopt Shared Tooling 0.5.0. Host and IC tools share verified bundles across
  checkouts while retaining each repository's pins and existing command paths.
  Contributors must rerun host/IC setup and offline checks. Common Cargo tools
  use `.tools/rust/common/bin`; earlier installations and failed attempts remain
  retained. CI checks native reuse with downloads disabled and archives only
  this checkout's referenced shared bundles and local Rust evidence
  ([#36](https://github.com/dragginzgame/ic-backup/issues/36)).
- Retain uploadable local CI logs before shared evidence collection. Rejected
  shared-store or tool paths preserve diagnostics and the original failure status
  while leaving local archives available
  ([shared #121](https://github.com/dragginzgame/shared-tooling/issues/121)).

## [0.18.1] - 2026-10-11

- Add an async application-fence acquisition step that checks retained stage,
  requirement and obligation evidence, durably reserves through the original
  journal and holds exclusion across fresh admission and one provider call.
  Replies and failures retain pending spending; changed post-reply evidence
  retains the acknowledgement
  ([#25](https://github.com/dragginzgame/ic-backup/issues/25),
  [#29](https://github.com/dragginzgame/ic-backup/issues/29)).

## [0.18.0] - 2026-10-11

- **Breaking:** Adopt Metrics 0.6.1. `prepared_chunk_bytes()` now returns the shared
  `MeasurementAggregate` without latest; duration summaries remain ordered.
  Exclude unrepresentable unit conversions instead of clipping samples, preserving
  exact-cap versus actual-overflow reporting
  ([#38](https://github.com/dragginzgame/ic-backup/issues/38)).

- **Breaking:** Select Host Artifact/FS 0.13. Consumers sharing
  `PersistenceError::Publication` must align with the new Host Rust type identity.
  Testkit 0.33.2 retains its separate dev-only Host 0.12 graph; persisted records
  and Backup's publication/custody contracts remain unchanged.
- **Breaking:** Await application-fence acquisition and reconciliation providers
  with the currently guarded original journal. Add `AgentFenceAcquisitionProvider`
  with mandatory durable signed-ingress and raw-reply retention. Cancellation or
  reply-retention failure preserves pending spending and obligations; replies
  never establish Active fence custody or settle automatically
  ([#25](https://github.com/dragginzgame/ic-backup/issues/25),
  [#29](https://github.com/dragginzgame/ic-backup/issues/29)).
- Adopt committed Shared Tooling 0.3.11: use the selected consumer directory for
  Cargo tool operations, normalize host-tool temporary roots and keep validation
  error classification consistent
  ([shared #101](https://github.com/dragginzgame/shared-tooling/issues/101),
  [shared #110](https://github.com/dragginzgame/shared-tooling/issues/110),
  [shared #111](https://github.com/dragginzgame/shared-tooling/issues/111)).

## [0.17.0] - 2026-10-11

- **Breaking:** Await snapshot upload providers and upload/allocation/data-upload
  coordinators with async admission and independent qualification. Providers
  recheck the currently guarded original journal; cancellation preserves spending,
  earlier receipts and source evidence and denies reentry
  ([#25](https://github.com/dragginzgame/ic-backup/issues/25),
  [#29](https://github.com/dragginzgame/ic-backup/issues/29)).
- **Breaking:** Await lifecycle/capture, metadata-allocation and data-write recovery
  observation providers and supply their currently guarded original journal.
  Add `AgentObservationProvider` with mandatory signed-ingress retention before
  one separately reserved status/list/readback update. Cancellation preserves
  both pending attempts; replies produce no automatic settlement
  ([#25](https://github.com/dragginzgame/ic-backup/issues/25),
  [#29](https://github.com/dragginzgame/ic-backup/issues/29)).
- Add `AgentSnapshotUploadProvider` with mandatory durable signed-ingress retention
  before one source-bound metadata/data update. Replies stay passive; accepted/lost
  updates produce no automatic receipt, observation, retry or allowance reset.
- Adopt committed Shared Tooling 0.3.8's coordinated-archive qualification rules.
  The existing paired-package check already follows this ownership; registry
  publication keeps ordinary Cargo admission
  ([shared #109](https://github.com/dragginzgame/shared-tooling/issues/109)).
- Qualify incoming Host 0.12.10 and Metrics 0.5.6 with unchanged Rust source;
  Testkit remains 0.33.1. Consolidate passive Agent reply-evidence framing while
  preserving exact existing domain and byte identities.

## [0.16.0] - 2026-10-10

- **Breaking:** Await snapshot transfer providers and read/metadata/download
  coordinators, with async fresh admission and independent qualification under
  original journal exclusion. Providers receive the currently guarded original
  record; cancellation retains consumption and denies reentry
  ([#25](https://github.com/dragginzgame/ic-backup/issues/25),
  [#29](https://github.com/dragginzgame/ic-backup/issues/29)).
- Add `AgentSnapshotTransferReadProvider` with mandatory durable signed-ingress
  retention before one metadata/data update. Replies remain passive; lost or
  accepted reads grant no automatic receipt, retry or allowance reset.
- Accept both numeric-descriptor and FIFO GNU Make jobservers in the Testkit
  routing fixture. Reject missing/non-FIFO paths before compiler effects, fixing
  macOS Homebrew Make admission
  ([#37](https://github.com/dragginzgame/ic-backup/issues/37)).
- Qualify incoming Host 0.12.8 and Metrics 0.5.5, and released Testkit 0.33.1;
  production Rust behavior and persisted product records remain unchanged.

## [0.15.0] - 2026-10-10

- **Breaking:** Make `IcMutationProvider` and the capture/load/start coordinators
  async. Await the coordinators, supply async admission/qualification callbacks,
  and pass the currently guarded original journal to mutation providers. Cancelled
  operations retain spending and deny reentry
  ([#25](https://github.com/dragginzgame/ic-backup/issues/25),
  [#29](https://github.com/dragginzgame/ic-backup/issues/29)).
- Add `AgentMutationProvider` with mandatory durable signed-ingress retention
  before one submission. Replies remain passive; accepted/lost responses never
  retry or settle journals automatically.
- Verify both current package archives together in a retained independent consumer,
  avoiding the published same-version core during unreleased API changes. Keep
  original dependency selections, archive bytes and root versions unchanged
  ([shared #109](https://github.com/dragginzgame/shared-tooling/issues/109)).
- Adopt Shared Tooling 0.3.7: reject failed Git index observations, isolate
  validation fixtures, and select the Testkit CLI directly from the lockfile
  through the shared installer instead of local Cargo metadata parsing
  ([shared #106](https://github.com/dragginzgame/shared-tooling/issues/106),
  [shared #105](https://github.com/dragginzgame/shared-tooling/issues/105),
  [shared #96](https://github.com/dragginzgame/shared-tooling/issues/96)).
  Fail mandatory Bash 3.2 assertions explicitly in shared and local fixtures,
  retaining failed inputs instead of reporting success
  ([shared #107](https://github.com/dragginzgame/shared-tooling/issues/107)).
- Retain failed Testkit routing and package fixtures under CI's temporary evidence
  root so the failure archive includes their original command logs.
- Select Binaryen 133 and exercise optimized Wasm in both real simulator suites.
  Prepare the toolset with `make install-ic-tools`; retain earlier bundles
  ([shared #102](https://github.com/dragginzgame/shared-tooling/issues/102)).
- Qualify Host 0.12.6's pre-effect NUL-path rejection, Testkit 0.33's scoped
  connection-reset classification and Metrics 0.5.4's unchanged arithmetic.
  Backup adds no simulator recovery/retry policy or persisted-format change.
- Keep only the newest CI run per workflow and branch or PR, cancelling older
  queued and running checks while retaining the existing host matrix and gates
  ([Shared #108](https://github.com/dragginzgame/shared-tooling/issues/108)).

## [0.14.2] - 2026-10-10

- Coordinate original same-ID load/start stages with fresh application safety,
  independently qualified receipts and retained source requirements. Lost or
  malformed replies stop without restart/reissue or reference release
  ([#29](https://github.com/dragginzgame/ic-backup/issues/29)).
- Adopt Shared Tooling 0.3.3: reject malformed validation nesting depth before
  dispatch and fail premature runner exits while retaining available evidence
  ([shared #104](https://github.com/dragginzgame/shared-tooling/issues/104)).
  Isolate consumer fixture logging from outer validation and require completed
  fixture execution before reporting success or removing evidence
  ([shared #105](https://github.com/dragginzgame/shared-tooling/issues/105)).
- Qualify Host 0.12.4, Testkit 0.32.2 and Metrics 0.5.3, prepare the exact selected
  Testkit CLI, and preserve Cargo jobserver access in local dependency discovery.
  Published Rust behavior and persisted product records are unchanged.

## [0.14.1] - 2026-10-10

- Plan and coordinate a complete original snapshot data-upload stage, buffering
  one exact source payload at a time. Require qualified receipts before dependent
  writes; preserve source bytes, pending spending and replies on failure
  ([#29](https://github.com/dragginzgame/ic-backup/issues/29)).
- Adopt Shared Tooling 0.3.2: refuse missing setup prerequisites before downloads,
  preserve Cargo jobserver descriptors and retain prematurely exited fixtures.
  Align local Cargo/Testkit recipes and fixture completion with these owners.
- Qualify Host Artifact/FS 0.12.2; published Rust behavior and persisted records
  match 0.12.0.
- Qualify the incoming Metrics 0.5.1 selection; arithmetic remains unchanged.
- Qualify incoming Testkit 0.32 and its exact CLI; its test-only Host graph now
  aligns with direct Host 0.12 without changing product records or spending.

## [0.14.0] - 2026-10-10

- **Breaking:** Adopt Shared Tooling 0.3.0. `make install-tools` now prepares
  the complete host, IC and Cargo toolsets, then locked dependencies and Testkit.
  `make tools-check` checks those sets offline in the same order. Prepare the
  declared Rust toolchain first; optional ripgrep/cloc flags are retired
  ([#36](https://github.com/dragginzgame/ic-backup/issues/36),
  [shared #98](https://github.com/dragginzgame/shared-tooling/issues/98)).
- Use the ordered aggregate for release preflight and CI, removing duplicate
  formatter/Testkit setup. Preserve earlier tools and recovery evidence.
- **Breaking:** Select direct Host Artifact/FS 0.12.0. Consumers sharing
  `PersistenceError::Publication` must align to the Host 0.12 Rust error type.
  Library behavior and persisted records are unchanged.
- Qualify incoming Testkit 0.31.0 and its exact CLI, retaining bounded build
  diagnostics and complete metadata/tool probes. Its independently selected
  test-only Host graph remains 0.11.

## [0.13.0] - 2026-10-10

- **Breaking:** Select direct Host Artifact/FS 0.11.0. Consumers sharing
  `PersistenceError::Publication` must align to the Host 0.11 Rust error type.
  Publication errors now display retained secondary cleanup failures. Select
  Testkit 0.29.0 and explicitly prepare its CLI; its test-only Host graph also
  uses 0.11.
- Adopt Shared Tooling 0.2.14. Original release preflight prepares and checks the
  locked Testkit CLI; standalone native qualification checks it before builds,
  including parallel Make. Missing or invalid selections fail without implicit
  validation installation ([shared #96](https://github.com/dragginzgame/shared-tooling/issues/96)).

- Coordinate an original metadata allocation through an independently qualified
  retained reply, explicit receipt and exact checkpoint for learned data binding.
  Preserve pending/Applied spending and reply evidence after failures; adopt the
  coordinator in the real Testkit upload caller
  ([#29](https://github.com/dragginzgame/ic-backup/issues/29)).

- Adopt committed Shared Tooling's stronger Make-mode admission and LF/CR
  directory-path rejection, preserving ordinary commands and retained evidence
  ([shared #30](https://github.com/dragginzgame/shared-tooling/issues/30),
  [shared #95](https://github.com/dragginzgame/shared-tooling/issues/95)).

## [0.12.1] - 2026-10-10

- Coordinate one original metadata stage through a single read, independently
  qualified receipt and exact learned-evidence checkpoint. Retain pending spending
  and returned bytes on failure; adopt it in the real Testkit download caller
  ([#29](https://github.com/dragginzgame/ic-backup/issues/29)).
- Coordinate one original metadata/data upload under retained stages and fresh
  admission. Keep acknowledgements pending and preserve replies/spending on failure;
  qualify original allocation-to-data binding and lost/malformed replies in Testkit
  ([#29](https://github.com/dragginzgame/ic-backup/issues/29)).
- Adopt committed Shared Tooling 0.2.10's concise formatting output, retaining
  full failure logs in CI and preserving formatter order, hooks and release checks
  ([#34](https://github.com/dragginzgame/ic-backup/issues/34)).

## [0.12.0] - 2026-10-09

- **Breaking:** Upgrade direct Artifact/FS dependencies to Host 0.10.1.
  `PersistenceError::Publication` now carries the Host 0.10 Rust error identity;
  align direct Host dependencies when sharing that value. Preserve v1 records,
  private publication, barriers and original spending. Testkit 0.28 uses the
  same Host line; explicitly prepare its CLI with `make install-testkit-server`.

- Adopt Shared Tooling 0.2.8's Make execution safeguards: reject modes that skip
  commands or hide failures, keep admission bound to the selected snapshot and
  support recursive Make arguments. Preserve formatting and release-resume checks
  ([shared #30](https://github.com/dragginzgame/shared-tooling/issues/30),
  [shared #90](https://github.com/dragginzgame/shared-tooling/issues/90)).

## [0.11.5] - 2026-10-09

- Stream a complete original snapshot data plan through the library, requiring
  explicit qualified receipts before dependent reads and durable publication.
  Retain pending spending, partial artifacts and returned replies on failure
  ([#29](https://github.com/dragginzgame/ic-backup/issues/29)).
- Finish shared formatting ownership by removing redundant local target
  declarations while preserving release-resume checks
  ([#34](https://github.com/dragginzgame/ic-backup/issues/34)).
- Qualify the selected Host 0.9.4 / Metrics 0.3.3 graph through local streaming,
  original-spending and direct Agent simulator journeys
  ([#25](https://github.com/dragginzgame/ic-backup/issues/25)).

## [0.11.4] - 2026-10-09

- Add a provider-driven snapshot capture step with durable original spending,
  mandatory fresh admission and one update. Connect it to the isolated download
  journey; lost or malformed replies retain pending attempts and block recapture
  ([#29](https://github.com/dragginzgame/ic-backup/issues/29)).
- Adopt Shared Tooling 0.2.6's reusable Rust formatting include, preserving pinned
  offline formatter admission and checkout-local tool lookup. Keep the specialized
  release-resume receipt check after the shared runner.

## [0.11.3] - 2026-10-09

- Add one provider-driven snapshot metadata/data read step with durable original
  spending and explicit fresh admission. Retain pending attempts and returned
  reply evidence on failure, without retry or automatic outcome recording
  ([#29](https://github.com/dragginzgame/ic-backup/issues/29)).
- Adopt Shared Tooling 0.2.5's literal hook-path handling and failed Git-read
  refusal ([upstream #89](https://github.com/dragginzgame/shared-tooling/issues/89)).
- Run automatic native CI once for each main release commit instead of repeating
  the same matrix for its tag. Preserve PR checks and explicit exact-ref
  qualification through manual dispatch
  ([#33](https://github.com/dragginzgame/ic-backup/issues/33)).

## [0.11.2] - 2026-10-09

- Publish stage-owned predecessor checkpoints only after admitting the exact workflow, binding and complete ancestor histories; recheck those originals after publication and retain checkpoint/journal evidence on failure ([#29](https://github.com/dragginzgame/ic-backup/issues/29)).
- Adopt Shared Tooling 0.2.4 companion declarations for reusable test suites, retaining the existing snapshot selection ([shared #73](https://github.com/dragginzgame/shared-tooling/issues/73)).

## [0.11.1] - 2026-10-09

- Qualify the selected Host 0.9.2/Testkit 0.27.0 graph and explicit CLI/server preparation through both libraries’ managed simulator cases, retaining pending spending and original recovery evidence ([#32](https://github.com/dragginzgame/ic-backup/issues/32)).

- Adopt Shared Tooling 0.2.3 guidance for complete optional installer-test selections and evidence-based native CI queue review; preserve our existing tool selection ([shared #73](https://github.com/dragginzgame/shared-tooling/issues/73)).

- Derive and durably publish an exact all-Applied execution checkpoint from the retained original plan and complete journals. Reject pending, missing, held or changed evidence without replacing checkpoints or resetting spending; use the canonical entrypoint in learned-stage callers ([#29](https://github.com/dragginzgame/ic-backup/issues/29)).
- Adopt the committed Shared Tooling 0.2.2 installer fixes: process the final IC pin row without a newline and refuse late destination directories and replace symlink entries without following their targets during CI tool publication ([shared #87](https://github.com/dragginzgame/shared-tooling/issues/87), [shared #88](https://github.com/dragginzgame/shared-tooling/issues/88)).

## [0.11.0] - 2026-10-09

- Name the locked Testkit version and explicit setup command when CLI admission fails; document Testkit preparation before release validation.

- Use the canonical JSON size checker directly throughout local persistence, retaining original record bounds, typed failures and interruption recovery.

- **Breaking:** Adopt the incoming Host 0.9 artifact/FS selection. `PersistenceError::Publication` carries the Host 0.9 Rust error identity; align direct Host dependencies when exchanging that value. Preserve record formats, publication barriers and original spending.

- **Breaking:** Move PocketIC preparation and offline admission to the locked Testkit CLI under Shared Tooling 0.2.0. Run `make install-testkit-server` and reinstall the shared five-tool IC bundle; `make testkit-server-check` replaces the retired alignment command. Keep old bundles and failed evidence ([#32](https://github.com/dragginzgame/ic-backup/issues/32), [shared #76](https://github.com/dragginzgame/shared-tooling/issues/76)).

- Resume an exact learned execution stage together with its complete original journal progress. Reject missing, changed or held evidence without creating journals, reading artifacts or resetting spending ([#29](https://github.com/dragginzgame/ic-backup/issues/29)).
- Adopt the committed Shared Tooling 0.1.38 fixes: reject multi-document dependency exceptions and conflicting Cargo-tool receipts, recheck installation paths and retain original build failures. Formatting hooks and their adoption checks find checkout-local tools without a PATH export while preserving staged inputs and pinned prerequisites ([shared #86](https://github.com/dragginzgame/shared-tooling/issues/86), [shared #65](https://github.com/dragginzgame/shared-tooling/issues/65), [shared #85](https://github.com/dragginzgame/shared-tooling/issues/85)).
- Qualify the incoming published Metrics 0.2.20 and Testkit 0.25.5 selection with original spending, stage recovery and isolated simulator contracts intact.

## [0.10.1] - 2026-10-09

- Prepare a learned execution stage with its complete durable original attempt-journal set before returning. Preserve partial preparation after failure or interruption; reopen creates no missing journals or new allowance ([#29](https://github.com/dragginzgame/ic-backup/issues/29)).
- Reject later-stage preparation and reopen when any ancestor's retained binding, plan, settlement or original journals are missing or changed. Check shared ancestors once without retaining multiple stage locks ([#29](https://github.com/dragginzgame/ic-backup/issues/29)).
- Adopt Shared Tooling 0.1.35 with optional exact Cargo binary/example installation and explicit profile/receipt checks. Clarify authorized locked dependency-cache preparation while preserving explicit offline settings ([shared #65](https://github.com/dragginzgame/shared-tooling/issues/65), [shared #84](https://github.com/dragginzgame/shared-tooling/issues/84)).

## [0.10.0] - 2026-10-09

- **Breaking:** Retire this repository's fleet tooling report and its regression suite. Run fleet reports from Shared Tooling; local `make cloc`, setup and offline checks remain supported ([#30](https://github.com/dragginzgame/ic-backup/issues/30)).
- Adopt Shared Tooling 0.1.34 with optional fleet selections, concise maintenance handoffs and literal offline PocketIC manifest-path selection under `CDPATH` ([shared #83](https://github.com/dragginzgame/shared-tooling/issues/83), [shared #81](https://github.com/dragginzgame/shared-tooling/issues/81), [shared #82](https://github.com/dragginzgame/shared-tooling/issues/82)).
- Bound HTTP fixture request capture, retain partial bytes and original read failures, and preserve the primary test failure during server cleanup. Explicitly select blocking accepted sockets while retaining the original deadline and transport/accounting assertions ([#31](https://github.com/dragginzgame/ic-backup/issues/31)).

## [0.9.2] - 2026-10-09

- Adopt Shared Tooling 0.1.32 so unchanged IC tool selections reuse verified installations after pin comments or row order change, preserving original receipts and bundles ([shared #79](https://github.com/dragginzgame/shared-tooling/issues/79)).
- Bind learned execution stages to immutable original target and attempt allocations, fixed child plans and exact predecessor journal settlements. Preserve pending attempts on reopen and refuse occupied or incomplete preparation ([#29](https://github.com/dragginzgame/ic-backup/issues/29)).
- Plan complete snapshot data reads from exact retained metadata within the original stage allowance. Qualify the isolated capture/download path through Testkit, including durable publication and safe stopping after lost or malformed replies ([#29](https://github.com/dragginzgame/ic-backup/issues/29)).
- Qualify the incoming published Host 0.8.5 selection within the existing compatible requirements, retaining private publication and existing-only command custody.

## [0.9.1] - 2026-10-08

- Adopt Shared Tooling 0.1.29 and report every staged, unstaged and untracked release-source violation, preserving files and the index. Initial preflight refusals explain that validation and version preparation have not started ([shared #74](https://github.com/dragginzgame/shared-tooling/issues/74)).
- Use the reviewed ShellCheck 0.11 installer in Linux CI and prefer a prepared checkout-local ShellCheck, avoiding Ubuntu's older-checker failure on the shared dependency checker.

## [0.9.0] - 2026-10-08

- Adopt Shared Tooling 0.1.28: preserve literal installed tool-link targets, use simulation-only shared release fixtures and add the maintenance task catalog without activating a schedule.
- Lower the qualified MSRV to Rust 1.88.0. Check both libraries and independent consumers explicitly in CI, preserving the selected lockfile and separate Rust 1.99 development toolchain.

- Run simulator qualification through published `ic-testkit`, using its complete PocketIC API and managed server startup. Retain caller-owned server logs and original lost-reply/recovery coverage; remove the direct PocketIC dependency.
- Select PocketIC 16.1 through one consumer-owned IC tool matrix; run `make install-ic-tools` to prepare the matching server. Shared installer and admission behavior remain unchanged ([shared #76](https://github.com/dragginzgame/shared-tooling/issues/76)).
- **Breaking:** Adopt the incoming Host 0.8 graph. `PersistenceError::Publication` carries the Host 0.8 error identity; align direct Host dependencies when sharing that value. Preserve record formats, publication barriers and original spending. Metrics 0.2.14 remains within the existing compatible requirement.

## [0.8.1] - 2026-10-08

- Adopt Shared Tooling 0.1.27. Keep release and validation fixtures independent of inherited Make includes and flags; resolve local script roots safely under `CDPATH`. Tooling counts recognize dotted snapshot names ([shared #7](https://github.com/dragginzgame/shared-tooling/issues/7), [shared #64](https://github.com/dragginzgame/shared-tooling/issues/64), [shared #69](https://github.com/dragginzgame/shared-tooling/issues/69)).
- Include the installer evidence fixtures and their declared companions so `tooling-check` resolves every required helper; canonical refresh refuses incomplete selections ([shared #73](https://github.com/dragginzgame/shared-tooling/issues/73)).
- Qualify Host artifacts/FS 0.7.1 and Metrics 0.2.13 with their existing compatible requirements, retaining publication, recovery and diagnostic contracts.

## [0.8.0] - 2026-10-08

- **Breaking:** Select direct Rust `ic-backup-agent` transport and retire the ICP backend/probe and its unused process dependency. Submit exact original reserved updates with explicit endpoint/caller/root trust, no retries or automatic polling, and retained signed ingress identity. Pending/lost replies preserve original spending; integrations still own fresh permissions and application safety ([#25](https://github.com/dragginzgame/ic-backup/issues/25)).
- Include both libraries in native CI, packaging and explicit publication; release preparation binds both member manifests and advances internal requirements together.

- Add original-plan-bound execution progress and durable attempt reservations. Require every retained journal and Applied prerequisites before consuming allowance; preserve pending recovery after lost replies ([#29](https://github.com/dragginzgame/ic-backup/issues/29)).
- Adopt Shared Tooling 0.1.25 to refresh matching local tracking refs after confirmed direct release or completed resume, reducing stale ahead-of-origin reports ([shared #62](https://github.com/dragginzgame/shared-tooling/issues/62)).
- Archive retained CI fixtures before upload, preserving legal Unix filenames, modes and links with the existing evidence roots and retention policy ([#28](https://github.com/dragginzgame/ic-backup/issues/28)).
- Select Metrics 0.2.12 within the current compatible requirement; retain empty host features and exact diagnostic/accounting separation.
- **Breaking:** Preserve the externally selected Host 0.7 libraries; the exposed publication-error Rust identity changes. Artifact/FS source retains its contracts; qualify the exact graph. The process package retires with its sole ICP caller.

## [0.7.0] - 2026-10-08

- **Breaking:** Adopt published Host 0.5 libraries. `PersistenceError::Publication` now carries the 0.5 `NamedWriteError` Rust type; align direct Host dependencies when matching or exchanging that value. Preserve JSON publication, checksum framing, record formats and original spending contracts.
- Adopt Shared Tooling 0.1.23 to recheck release payloads and exact tags after final hooks and verify published identity on completed resume ([shared #58](https://github.com/dragginzgame/shared-tooling/issues/58)).
- Remove duplicate Unix error-conversion helpers and use Rustix's native IO conversion directly, preserving artifact publication, lock failures and interruption recovery.

## [0.6.0] - 2026-10-08

- **Breaking:** JSON publication now returns `PersistenceError::Publication` with shared typed producer, cleanup and before/after-publication evidence. Update exhaustive error matches; reconcile visible output after completion failures. Delegate allocation and atomic publication to `ic-host-fs`, retaining private parents, serialize-once preflight and crash recovery ([#24](https://github.com/dragginzgame/ic-backup/issues/24)).

- Expose checked directory-checksum composition for already-admitted file identities, preserving canonical framing without filesystem IO. Verify composed identities through durable publication/recovery and reject changed bytes ([#26](https://github.com/dragginzgame/ic-backup/issues/26)).
- Adopt Shared Tooling 0.1.21; retain the qualified direct release policy and reject unsupported delivery selections before effects.

## [0.5.4] - 2026-10-07

- Require the macOS filesystem compile repair and qualify published ic-host 0.4.6, including its native filename checks. Preserve artifact, journal and spending contracts ([host #18](https://github.com/dragginzgame/ic-host-tooling/issues/18), [host #19](https://github.com/dragginzgame/ic-host-tooling/issues/19)).

- Adopt Shared Tooling 0.1.20, including scoped contribution authority, temporary-path and changelog fixes, guarded Rust setup and a locked PocketIC alignment check ([#23](https://github.com/dragginzgame/ic-backup/issues/23)).

- Qualify the pinned ICP CLI against a real local gateway. Retain pending spending after its management routing refusal; reuse published `ic-host-process` for bounded test execution ([#25](https://github.com/dragginzgame/ic-backup/issues/25)).

- Bind snapshot metadata/data reads to original reserved attempts through a provider contract and bounded passive response checks. Real PocketIC loss cases retain pending spending and stop safely ([#22](https://github.com/dragginzgame/ic-backup/issues/22)).

- Stop snapshot artifact appends when an original region file is replaced or changes length, retaining partial bytes and unchanged journal state ([#21](https://github.com/dragginzgame/ic-backup/issues/21)).

## [0.5.3] - 2026-10-07

- Accidental release containing release metadata and handoff updates; no product implementation changes from 0.5.2.

## [0.5.2] - 2026-10-07

- Reuse published ic-host 0.4 error conversion for artifact and record IO.
  Preserve native failures, exact byte limits, private publication and command
  custody; qualify the maintainer-selected dependency graph
  ([#20](https://github.com/dragginzgame/ic-backup/issues/20)).

- Expose bounded prepared chunk-size distributions through shared `ic-metrics`
  histograms. Keep the existing summary getter derived from the same samples;
  failures do not contribute bytes, and reopening starts empty
  ([#15](https://github.com/dragginzgame/ic-backup/issues/15)).

- Use the reviewed shared setup and LOC commands with pinned ripgrep/cloc.
  Reject Make modes that skip execution or hide failures before release,
  validation and formatting-hook dispatch. Adopt the corrected consumer LOC
  fixtures, retain complete validation logs when requested, and keep draft notes
  attached to headings with trailing whitespace
  ([#19](https://github.com/dragginzgame/ic-backup/issues/19)).

- Run generic release increment and retry coverage once through the shared
  suite. Retain consumer receipt, selected-commit, index and publication checks,
  with exact receipt and validation evidence across Make recovery
  ([#18](https://github.com/dragginzgame/ic-backup/issues/18)).

## [0.5.1] - 2026-10-07

- Release non-spawning command-quiescence exclusion when its guard drops, even
  while a descriptor copy remains open. Preserve dispatched child and descendant
  custody and retained lock evidence
  ([#13](https://github.com/dragginzgame/ic-backup/issues/13)).

- Bind independently qualified stop/start/load settlement to the original spent
  status observation and exact retained evidence. Keep lost replies pending and
  preserve consumed allowances through explicit receipt recording and reopen
  ([#14](https://github.com/dragginzgame/ic-backup/issues/14)).

- Bind independently qualified snapshot-capture settlement to the exact original
  baseline and spent inventory observation. Require an explicitly attributed new
  ID; candidate counts never imply success, retry or refunded spending
  ([#16](https://github.com/dragginzgame/ic-backup/issues/16)).

- Qualify real single-canister capture, durable streamed download and same-ID
  upload/load recovery on PocketIC. Reconcile deliberately lost capture,
  allocation, data and stop/load/start replies using original reserved observations.
  Verify complete restored state while stopped before starting; retain pending
  load/status reservations without retry when the recovery reply is also lost.
  Preserve consumed allowances and source references
  ([#17](https://github.com/dragginzgame/ic-backup/issues/17)).

## [0.5.0] - 2026-10-07

- **Breaking:** `checksum_reader` now retries interrupted reads internally.
  Callers must own blocking/timeouts and cannot use the first `Interrupted` error
  as a termination signal. Invalid reader byte counts return `InvalidData`.
  Delegate raw hashing, digest parsing, copies and output limits to
  `ic-host-artifacts`, and no-follow record reads to `ic-host-fs`. Preserve checksum
  records, exact layout lock names, bounded source verification and secure publication
  ([#11](https://github.com/dragginzgame/ic-backup/issues/11)).

- Reserve exclusive integration fixture directories without relying on the host
  clock. Skip occupied paths without changing retained evidence, with bounded
  collision handling and fixture-path diagnostics
  ([#10](https://github.com/dragginzgame/ic-backup/issues/10)).

- Verify shared snapshots without executing the inspected checksum helper. Bind
  release pushes to the captured destination URL through validation and recovery
  ([#12](https://github.com/dragginzgame/ic-backup/issues/12)).

## [0.4.2] - 2026-10-06

- Refresh shared validation and release checks. Keep successful Rust test names
  and failure context out of error highlights while preserving real diagnostics
  ([shared-tooling #22](https://github.com/dragginzgame/shared-tooling/issues/22)).
- Remove duplicate hook fixtures after native consumer qualification; retain the
  independent formatter exit-status check
  ([#5](https://github.com/dragginzgame/ic-backup/issues/5)).

## [0.4.1] - 2026-10-06

- Enforce workspace version and dependency inheritance with the committed shared
  checker. Read versions through its offline Cargo/TOML parser, preserving exact
  committed-source checks and rejection before metadata writes
  ([#6](https://github.com/dragginzgame/ic-backup/issues/6)).
- Preserve temporary CI fixtures and upload failed validation evidence for native
  installer and recovery diagnosis. Restore exact authenticated archives in
  installer refusal fixtures and retain installer diagnostics on failure
  ([#9](https://github.com/dragginzgame/ic-backup/issues/9)).
  Keep failed dependency, validation and release regression inputs available
  for those artifacts while removing successful temporary fixtures
  ([shared-tooling #21](https://github.com/dragginzgame/shared-tooling/issues/21)).
- Share the reviewed formatter pin across setup, formatting and checks; reject
  unavailable or failed formatter probes before changing files
  ([#7](https://github.com/dragginzgame/ic-backup/issues/7)).
- Finalize release notes through the shared selector using the saved original
  version. Preserve historical notes and metadata rollback; missing note content
  no longer blocks release preparation
  ([#8](https://github.com/dragginzgame/ic-backup/issues/8)).

## [0.4.0] - 2026-10-06

- Bind data-upload recovery reads to the original destination, exact extent and
  already spent observation. Compare retained chunk evidence without turning
  matching bytes into a write receipt, retry or source-release permission.

- Admit separately qualified data-upload settlement against exact reserved
  readback, attempt identities and a fresh challenge. Keep byte comparison
  independent from write attribution and preserve exhausted spending through reopen.
- Admit independently qualified metadata-allocation settlement against unchanged
  original inventory, exact reserved list evidence and a current challenge.
  Require explicit attribution to a new non-source ID; candidate counts grant no outcome.
- **Breaking:** use `ic-metrics` 0.2 for public local summary return types.
  Name returned summaries through `ic_backup::ops::persistence::MeasurementSummary`;
  consumers also using Metrics directly must use its 0.2 types.
  Use `ic-host-tools` 0.2 while preserving artifact and persistence behavior.
- Adopt Shared Tooling’s exact lockfile transformer and actual formatting-hook
  checker, preserving dependency selections, release recovery and working edits
  ([#5](https://github.com/dragginzgame/ic-backup/issues/5)).
  Refresh committed installer failure handling and retain existing hook cases
  until replacement coverage passes on native macOS.

## [0.3.9] - 2026-10-06

- Bind metadata-upload recovery inventory to exact original payloads and already
  reserved observations. Retain lost replies and spending across reopen without
  inferring upload success, retry permission or source release from inventory counts.
- Use `ic-host-tools` 0.1.13 for private artifact copying, preserving exact
  checksums and partial failure evidence while retrying interrupted reads.
- Check record output budgets with its shared bounded writer, avoiding full
  JSON allocations during size admission while retaining existing limits and errors.
- Read exact upload chunks through its bounded stream collector, retaining local
  descriptor confinement, range checks and original source verification.
  Reject invalid shared stream byte counts with IO errors instead of panicking
  ([ic-host-tools #5](https://github.com/dragginzgame/ic-host-tools/issues/5)).
- Refresh Shared Tooling's portable checksums and installer receipt handling.
  Add `make check-doc-links` and check maintained local Markdown targets in CI.

## [0.3.8] - 2026-10-06

- Delegate bounded regular-file record reads to `ic-host-tools`, retaining exact
  record limits, no-follow/FIFO rejection and existing typed persistence errors.
  Require its corrected archive dependency minimum
  ([ic-host-tools #3](https://github.com/dragginzgame/ic-host-tools/issues/3)). Durable publication, confinement,
  journals and command custody keep their owners.

- Define a single-update upload provider contract for exact source-bound metadata
  and data requests under their original pending reservations. Reuse passive reply
  admission and shared failure types, retaining spent attempts and source references
  through failures and reopen. Live transport and lost-effect settlement remain pending.

## [0.3.7] - 2026-10-06

- Prepare snapshot upload metadata and bounded region/chunk bytes from verified
  retained IC artifacts. Preserve globals and optional fields, reject unavailable
  values and bind each new snapshot/data request to its original source evidence.
- Associate bounded upload replies with independently accounted pending attempts.
  Keep lost replies and spending retained without automatic receipts, retries or
  source-reference release. Live upload transport remains unimplemented.
- Use `ic-metrics` for per-guard host verification/preparation durations and prepared
  chunk sizes. Keep successful and rejected work separate; ordinary reopen starts
  empty measurements without changing retained progress or spending.
- Adopt Shared Tooling's pinned local jq/yq and IC executable setup, offline
  integrity checks and scoped dependency-pinning validation in CI and releases.
  [#3](https://github.com/dragginzgame/ic-backup/issues/3)
- Make non-UTF-8 artifact-name coverage portable when macOS rejects the raw
  filename during fixture setup, preserving exact production name rejection.
  [#4](https://github.com/dragginzgame/ic-backup/issues/4)

## [0.3.6] - 2026-10-06

- Stream admitted snapshot data into private local artifacts, retaining exact
  metadata and verifying complete region/chunk bytes before durable publication.
  Recover publication through the original download journal; interrupted transfers
  retain partial evidence without repeating reads or changing effect spending.
- Explicitly verify published IC artifacts against the retained original plan,
  metadata, region lengths and chunk hashes. Reject changed or unsafe files without
  rewriting evidence or adding artifact reads to ordinary resume.
- Adopt Shared Tooling's failure-log fallback and stronger release index checks.
  Reject staged changes hidden by restored working files, and make missing-tag
  fixtures fail explicitly under conditional shell evaluation.
  [#2](https://github.com/dragginzgame/ic-backup/issues/2)
- Make FIFO safety fixtures portable to macOS while preserving real filesystem
  rejection checks for artifacts and restore-reference records.

## [0.3.5] - 2026-10-06

- Bind snapshot data reads to retained metadata with checked memory ranges and
  exact chunk identities. Decode bounded replies with exact lengths and chunk
  hashes, retaining evidence without changing spending or claiming complete transfer.
- Check full declared snapshot data coverage incrementally, rejecting gaps,
  overlaps, duplicate chunks and mixed metadata while keeping memory bounded.
  Coverage retains no bytes and grants no durable-transfer or execution authority.
- Adopt the latest Shared Tooling release recovery: normal release commands finish
  the exact interrupted release before validating newer fixes. Check retained
  receipts against the selected release commit, preserving original evidence.
  [#2](https://github.com/dragginzgame/ic-backup/issues/2)
- Preserve formatter failures and support hook setup through symlinked checkout
  paths. Make dependency-failure fixtures reject explicitly when shell conditional
  evaluation disables automatic error handling.

## [0.3.4] - 2026-10-06

- Encode exact snapshot metadata reads and decode bounded replies while preserving
  global values, optional fields and chunk identities. Retain request/raw-byte
  evidence without changing journals or treating metadata as complete transfer,
  fresh permission or mutation settlement.

## [0.3.3] - 2026-10-05

- Durably publish exact staged restore artifacts and recover canonical copies after
  interruption without rereading source trees or resetting journals, obligations
  or source references. Add explicit published-copy verification; conflicts and
  changed bytes remain retained for review.

## [0.3.2] - 2026-10-05

- Bind exact IC status and snapshot-list requests to already reserved recovery
  observations. Associate bounded replies with both original attempts while keeping
  lost replies pending, without automatic mutation settlement or repeat calls.

## [0.3.1] - 2026-10-05

- Bind exact IC capture, load, start and stop requests to already reserved original
  mutation attempts. Add a single-update provider contract and bounded passive
  reply association, preserving spent journals and restore references through
  failures and recovery without automatic settlement, retries or release.

## [0.3.0] - 2026-10-05

- **Breaking:** Use the shared release workflow for patch, minor and major releases,
  with exact saved-version recovery through `make release-resume VERSION=X.Y.Z`.
  Replace standalone preparation/stage/commit/push and `release-x` commands with
  the selected one-shot release command. Preserve build and recovery evidence,
  and atomically push only the selected branch and tag, disabling implicit tag pushes.
- **Breaking:** Update developer setup to `make install-hooks` and install
  cargo-sort 2.1.4. The shared hook formats and refreshes only selected files,
  rejects partial staging, and preserves unrelated edits. CI and releases check
  both Cargo manifest ordering and Rust formatting independently.
- Retry preflight or validation-only release failures through the normal target
  against corrected source, retaining earlier evidence. Once preparation starts,
  resume the exact saved version instead of selecting another increment.
- Isolate release-check fixtures from enclosing release identities and Make
  overrides so their dependency checks also succeed inside the release gate,
  while preserving rejection of an explicitly changed validated source.

- Bind fresh local restore-source verification to the exact original requirement,
  source manifest and selected existing IDs. Retain both layout/journal guards and
  all obligations without changing spending or authorizing upload, load or release.
- Privately stage exact checksum-bound local restore artifacts for original operation
  identities. Verify retained copies explicitly after interruption, preserving partial
  evidence, spent attempts and source references without upload or automatic cleanup.

## [0.2.3] - 2026-10-05

- Retain immutable original execution settlement checkpoints with exact chronological
  journal fingerprints. Local replay requires every original operation Applied and
  unchanged evidence, without remote calls, new spending or fence/reference release.
- Publish an immutable, freshly byte-checked local download manifest under the
  original plan. Replay exact retained snapshot/checksum evidence locally without
  reading artifact trees, changing progress or releasing obligations.

## [0.2.2] - 2026-10-05

- Bind exact bounded application acquisition receiver/method/arguments to the
  original reserved mutation. Add a single-update provider and passive reply
  association contract; failures and acknowledgements retain pending spending
  and obligations without automatic settlement, retries or release.
- Bind fence-acquisition reconciliation to exact original pending mutation and
  observation reservations. Add passive application attribution checks and a
  read-only provider contract, preserving spent attempts, obligations and source
  references through failures and late replies without automatic settlement or release.

## [0.2.1] - 2026-10-05

- Let an explicit release command relabel the single current numbered draft to
  its requested version, preserving notes and history. Keep competing/released
  version rejection and restore the original draft on preparation failure.
- Retain exact original capture/restore fence obligations before acquisition under
  bounded immutable publication. Reuse original attempt journals for recovery and
  retain obligations and source references through lost replies and settled outcomes;
  no current fence custody, dispatch or release is inferred.
- Retain immutable same-network/same-release restore safety requirements under exact
  original source and restore plans. Add fresh load/start application safety evidence
  checks, requiring stopped targets, outside-snapshot fence custody and restored-state
  acceptance without settling lost effects, replenishing attempts or releasing references.

## [0.2.0] - 2026-10-05

- Refresh the checksum-bound Shared Tooling baseline and helpers from one reviewed
  committed revision. Add macOS 15 CI on Apple Silicon and Intel, system Bash 3.2
  checks, and portable checksum/edit helpers; native macOS qualification is pending.
- Upgrade the exact `ic-management-canister-types` dependency from 0.8.0 to 0.11.0.
  Preserve the six-method wire bytes, hashes and v1 records; exercise the new status
  settings through the bounded projection while retaining historical provenance.
- Let `make shell-check` use an existing `~/.local/bin/shellcheck` when ShellCheck
  is absent from PATH, preserving PATH precedence and mandatory lint failures.
- Restore the seven vendored Shared Tooling documents to their pinned bytes after
  local branding and prose edits caused snapshot verification to fail. Keep local
  navigation and host guidance outside the verified snapshot.
- Extract explicit fresh verification of durable downloaded artifacts under the
  retained original plan and exact selected targets. Reject changed declarations,
  unsafe paths and checksum mismatches without rewriting journals, replenishing
  attempts or releasing restore references; ordinary resume remains distinct.
- Add bounded status/controller reply projections and exact lifecycle
  acknowledgements for the existing IC request methods. Preserve raw evidence and
  original load/status attempts through local replay; decoded state and acknowledgements
  require authenticated integration evidence before effect settlement or fresh authority.
- Extract pure snapshot inventory comparison from Canic's capture recovery path.
  Retain exact request/reply evidence, reject lost or changed baseline entries and
  expose canonical new candidates without attributing them to the capture or
  settling pending attempts. Native recovery preserves original evidence and limits.
- Add bounded IC snapshot capture/inventory reply decoding with exact raw IDs,
  required timestamp/size fields, canonical unique inventories and request/reply
  evidence hashes. Reject malformed or ambiguous wire data and preserve spent
  pending journals through local replay; transport and effect settlement remain
  integration-owned.

## [0.1.8] - 2026-10-05

- Adopt the Dragginzgame engineering baseline with explicit source identity and
  local backup rules. Vendor reviewed shared principles and tooling with offline
  checksum/mode verification, fail-fast CI summaries and retained failure logs.
- Keep one top-level changelog draft until release selection; prepare and date
  unnumbered or selected numbered notes without an Unreleased queue. Registry
  publication continues to delegate admission to Cargo.
- Isolate release-test repository, failure-log and summary paths from the calling
  CI runner so nested checks use their own fixtures and retain failure evidence.

## [0.1.7] - 2026-10-04

- Add an ephemeral snapshot-read provider contract bound to original mutation
  intent and independently declared exact snapshot-list bytes. Validate current
  context, target and controller/public/allowed-viewer access without granting
  mutation control; preserve spent pending observations through local recovery.
- Retain immutable original-plan consistency requirements without replacement or
  downgrade. Add current stopped/drained target and application-fence observation
  contracts bound to exact selection, challenge, capture boundary, retained fence
  identity and original membership revision. Native recovery preserves spent
  journals and retained obligations; fence effects and backend qualification remain pending.

## [0.1.6] - 2026-10-04

- Add a closed host-ingress IC management request codec for status, snapshot
  inventory, stop/start, new snapshot capture and exact snapshot load. Bind fixed
  receiver, routing target, replicated call mode, method and exact Candid bytes;
  validate original mutation and independently reserved observation payloads with
  bounded records, independent wire goldens and local journal recovery qualification.
- Add a fallible membership-provider contract with ephemeral requests bound to
  original plan/operation, challenge, effect boundary and bounded observation calls.
  Pure validation rejects changed context/full inventory and replayed results;
  native recovery preserves spent allowances without claiming authority or continuity.
- Add direct controller-observation contracts for exact IC mutation payloads.
  Validate canonical current target/context/controller sets; reject revoked callers,
  other-controller routing assumptions and stale results while retaining spent
  journal authority. Provider qualification and full effect admission remain pending.

## [0.1.5] - 2026-10-04

- Bind canonical inventory, explicit physical selection, dependency graph and exact
  network/caller/release/request declarations into immutable operation plans. Check
  graph/table identity and aggregate assigned limits, derive exact original journal
  authority and qualify bounded no-overwrite persistence and consumed-attempt replay.
- Derive local execution progress from the complete original plan and exact retained
  attempt journals. Reject missing/mismatched evidence and attempted unmet dependencies;
  project unresolved observations and exhausted allowances without resetting budgets,
  scheduling calls or claiming terminal completion.

## [0.1.4] - 2026-10-04

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
