# Changelog

## [0.5.3] - 2026-10-07

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
