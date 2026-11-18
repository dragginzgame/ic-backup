# Development setup

The virtual workspace has one library under `crates/ic-backup`, admitted for the
experimental direct-controller crates.io release. Qualification remains private.
Local inventory, payload checks and capture intent use Host filesystem/artifact
primitives and published Platform identity and management DTOs, with Serde/JSON
and typed errors. Platform's native protocol codec checks prepared status call
content; its historical certificate verifier authenticates retained replies and
certified rejections. Host HTTP 0.80.0 supplies bounded complete-body consumption. Backup directly
selects Reqwest 0.13.5 with opt-in Rustls and owns its client policy, without Agent. Its futures require a caller-owned Tokio runtime with IO
and timers; tests use the selected Tokio 1.54.1 runtime. External IC DTOs and
certificate codecs remain Platform-owned. No Canic framework dependency is added.

Host FS and Artifacts 0.80.2 resolve from verified registry archives, source
`828c2692523a7a906188c111c9b58f30d0f52468`. SHA-256 values are
FS `81c73a53b929355c2056a2783c0f61676251c0ddba2d6e0109c3ac53841b396b` and
Artifacts `f8645c876c9f26469c46e0ec764b639b192d2f2336d71815a2fd11d4c86b6f1e`.
No sibling override or shared target is used. Evidence is under
`target/evidence/host-tooling-recheck/`. HTTP 0.80.0 removes its old Artifacts
edge. Testkit 0.107.3 now selects FS/Artifacts/Process 0.80.2 in qualification.
Backup now owns `store::BackupDigest` and uses approved direct SHA-256 0.11.1;
its canonical commitment bytes and stored formats are unchanged. Publication
errors retain pre-visibility, post-visibility and explicit cleanup causes.
The direct capture provider composes status, list and take through the existing helpers.
Locked Platform 0.33.7 resolves canonical Types 0.20.4 through the existing dependency.
The Types patch leaves source and identity bytes unchanged; no direct Types
dependency, new feature or sibling override is needed. Its verified archive and
consumer graph evidence are under `target/evidence/types-0.20.4/`.

## Preparation

Use Bash, GNU Make, Git, Perl and a SHA-256 backend. Prepare the exact compiler
and shared tools explicitly:

```bash
rustup toolchain install 1.100.0 --profile minimal --component rustfmt --component clippy
make install-tools
make install-hooks
make fetch-dependencies
```

Validation is locked and offline; it does not install tools. Hook activation is
explicit through `make install-hooks`. Tooling 0.45.8 is adopted from its registry
archive with SHA-256
`adbc76769b2231de6704851c363550ddc486bb6204a0f89c2ad365eb7cf740a3`,
source `12af9e64f4a0ba03c8c94d3eee00987080ade98a`. Both workspaces declare
its exact pin and each member inherits the disabled development marker.
The canonical exporter installs twelve selected entrypoints, including the shared
pre-commit formatter hook, with derived companions, and owns [.toko-tooling.snapshot](../../.toko-tooling.snapshot).
The shared publication adapter is selected; no producer regression suites or
automatic version/tag release runner are installed. See the
[publication workflow](publication.md) for gates, eligibility and recovery.

Both CI jobs prepare Rust through the selected
`vendor/toko-tooling/scripts/dev/prepare-rust-toolchain.sh --toolchain-file rust-toolchain.toml`,
invoked by `scripts/ci/setup.pl rust-toolchain`. Each step adds the previously
prepared `.tools/host/bin` to PATH so the helper uses the installed `yq` and `jq`.
The TOML file owns channel/profile/components/targets; CI repeats no selection.
Canonical export retains this helper and its declared workspace-reader companion.
Setup diagnostics additionally selects the shared module (62 managed files total).

## Commands

```bash
make deps-list
make deps-update
toko-cloc -d 0
make check
```

`deps-list` lists local package versions, not the transitive graph or publication
status. `deps-update` explicitly updates Cargo dependencies and can use the
network. To update one dependency, use `DEPENDENCY_PACKAGE=name`; add
`DEPENDENCY_PRECISE=version` for a specific compatible version. The exact tooling
pin still controls its selection. `toko-cloc` and `toko-cloc -d 0` produce the same
compact totals; `-d 1` adds crate rows and `-d 2` adds top-level modules. Both run the installed PATH reporter; without the launcher,
use `bash vendor/toko-tooling/scripts/dev/toko-cloc -d 0` after host-tool setup.

Focused checks are `make toko-tooling-check`, `make doc-links-check`,
`make fmt-check`, `make clippy`, `make rustdoc` and `make test`.
`make check` / `make ci` aggregate those checks. `PACKAGE=ic-backup` selects Rust
checks; formatting and link checks remain repository-wide. For ordinary inventory
changes, select the owning tests and lint/docs checks:

```bash
cargo test --locked --offline -p ic-backup --lib store::
cargo clippy --locked --offline -p ic-backup --all-targets -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --locked --offline -p ic-backup --no-deps
cargo test --locked --offline -p ic-backup --doc
```

Check for active commands using this workspace's target before compilation or
source/lock edits. Test fixtures remain under `target/local-store-tests/`, including
failed cases. The FIFO replacement regression uses the native Unix `mkfifo` utility (Linux/macOS).
Use `store::tests::payload` for native payload verification and held-source reads;
its `source_tests` selection covers block boundaries, retained custody and exact
original-source schedule reads. These synthetic bundles do not qualify capture evidence.
Select capture intent with `store::capture::intent`, common frame/reservation
checks with `store::operation::tests` and `store::operation::framing::tests`, and
custody/runtime with `store::operation::custody` / `store::operation::runtime`.
The acknowledged runtime child-process death case verifies original intent,
target claim and spent signing reservation with no repeat effect. It does not
qualify filesystem power loss.
Use `store::capture::transfer` for planning and bounded raw reply tests;
`store::capture::transfer::metadata` selects the metadata decoder alone.
Use `store::capture::transfer::readback` for uploaded-state comparison, including
exact float bits, lifecycle state, destination provenance and complete chunk sets.
Use `store::restore::procedure` for actual fixed restore coverage, routing and
byte comparison; the obsolete separate test-only readback cursor was removed.
`store::restore::tests::persistence_tests` selects cold restore reopening by ID /
approval, exact selected bytes and original target/source/operation lock refusal.
Use `store::capture::snapshot` for bounded list/take replies, frozen wire fixtures
and non-destructive take preflight. These are pure decoding/policy tests.
Use `store::capture::status` for decoded status preconditions and exact management
caller membership; `store::capture::status::wire` selects bounded raw status
projection and rejection tests. These exercise Platform DTOs/codecs but do not qualify IC effects. Native filesystem tests
establish no IC capture/load behavior.

CI configures Linux x86-64 and macOS ARM64/x86-64, plus formatting and a separate
pinned history secret scan. Local Linux validation does not qualify macOS.

Ordinary tests stay beside implemented code. The independent `qualification/`
workspace owns the actual-IC caller and its lock; product never depends on it.
`make fetch-dependencies` prepares both locked graphs and shared formatting
covers both. `make check` remains the core check; IC qualification is explicit.
Future dependency adoption uses the root exact tooling pin, member inheritance,
lockfile and `make toko-tooling-update`; never edit managed payloads manually.

Use `store::operation::runtime` for shared capture/restore one-shot reservation,
submission and recovery tests. Synthetic certificate/native HTTP fixtures check
ordering and authentication; actual IC effects belong to the private qualification
consumer. These fixtures do not qualify physical power loss or Canic adapters.

Use `store::capture::advance::tests::certificate_tests` for signed synthetic
certificate and durable replay tests. The common `store::operation` filter
owns reservation/recovery; `capture::advance` retains protocol primitives. Evidence for the minimum
Platform 0.33.4 certificate feature and six added crypto packages is retained under
`target/evidence/status-certificates/`. These are offline cryptographic checks,
not simulator or live-IC qualification. Select the nested `rejection_tests` module
for diagnostic projection and rejected replay; its evidence is under
`target/evidence/status-rejections/`.

Use `store::capture::advance::tests::http_tests` for real loopback HTTP, exact
request capture, response bounds, no-retry behavior and workflow replay. Request
bytes remain in the caller-owned `target/local-store-tests/status-http-*`
fixtures. TLS negotiation/plaintext refusal is also exercised; completed trusted
TLS handshakes, external signer process custody and hosted macOS remain unqualified
by these native tests. Original transport
evidence remains under `target/evidence/status-http/`. No separate transport crate
or runtime wrapper was introduced for this native host capability.

Use `store::capture::advance::tests::request_tests` for original direct-key
signature authentication and refused-signer recovery. Platform's existing `ecdsa`,
`ed25519` and `principal-self-authenticating` features supply the key/verification
and Principal owners. The reviewed disposable graph adds 26 registry packages
while preserving all previously locked suppliers. Graph, preparation and focused
check evidence is under `target/evidence/status-request/`; no Agent or signing
process dependency is introduced. The HTTP cases use real Ed25519/secp256k1 signed
envelopes; application and external signer qualification remain separate.

Host HTTP 0.79.0 adoption evidence is under `target/evidence/host-http-0.79.0/`:
verified registry archive, before/proposed/admitted graphs, compiler identities
and consumer checks. Reqwest is selected only through `ic-host-http::reqwest`;
its Rustls backend uses platform certificate verification and native trust
configuration instead of the earlier bundled WebPKI roots. No custom TLS roots,
verification bypass or IC trust discovery is added. That adoption preserved the incoming Platform 0.33.5
lock selection under the then-current 0.33.4 minimum. The graph adds
Host HTTP and Reqwest's native-verifier/AWS-LC support, including target-specific
packages; it removes the obsolete 0.12 Reqwest and unused old dependencies.

## Direct-provider and IC qualification

The public capture uses one fixed operation loop. Select its native custody,
replay, intent, protocol and directly affected restore consumers with
`cargo test --locked --offline -p ic-backup --lib -- store:: --test-threads=4`.
The common lifecycle failure matrix lives under `operation/`;
exact request/certificate/HTTP and bounded codec checks remain with their owners.

The public-API integration test under `qualification/ic-backup-qualification`
drives actual PocketIC multi-extent capture/publication, authenticated succession,
older-source full restore, capture after restore and offline history/replay.
Separate cases lose original take/data/allocation/upload/load replies and recover
via an accounted observation or surviving certificate.
Its Backup/Platform/Testkit dependencies are test-only. The separate
`qualification/ic-backup-consumer` executable uses Backup as a normal dependency and is
built alone with `cargo build --locked --offline --manifest-path
qualification/Cargo.toml -p ic-backup-consumer`. The isolated fixture saves its
normal graph (excluding Testkit/PocketIC), runs separate capture/restore step and
continuation processes, then inspects completed records in new processes after
server shutdown using only store/ID/approval. No release or registry-consumer
qualification is implied. Fixture keys and application authority do not qualify
Canic adapters or power loss.

```bash
make install-testkit-server
make testkit-server-check
make test-qualification
```

Native CI lints the qualification workspace on Linux x86-64 and both macOS
hosts. Linux additionally prepares and admits the locked Testkit server, then
runs the focused `ic-backup-qualification --test capture` target, including
20 MiB transfer and separate-process recovery. Setup uses the existing bounded,
sanitized failure diagnostics. Actual-IC hosted macOS coverage remains unproved.
CI runs asynchronously; these checks add no requirement to await hosted results
before publication. The existing local publication gate is unchanged.

Actual-IC journeys use the optimized Cargo release profile under
`qualification/target/release/`; native product tests retain their ordinary
profile. This avoids measuring debug-mode crypto and payload replay as release
performance. The 20 MiB fixture explicitly bounds both server lifetime and idle
time at 30 minutes to accommodate long local source checks between IC calls.

Setup builds the private `server` example against the locked Testkit 0.110.0
library. `make install-testkit-server` explicitly downloads the supplier-selected
archive with a finite time/byte ceiling, then delegates checksum admission,
bounded decompression and create-only publication to Testkit. It is an ordered
`install-tools` extension after common tools and locked fetch. `tools-check`
and each fixture independently admit the prepared server offline. There is no
second PocketIC catalog, implicit download, Testkit CLI or copied executable bundle.
The gzip executable must be prepared on PATH; its absolute path is passed explicitly.
Prepared bytes live under `.tools/testkit-server/`; download attempts remain in
`qualification/target/testkit-setup/`. Preparation uses private permissions.
Original `.tools/testkit/` assets and failed attempts remain untouched.

Testkit's managed server has finite startup/lifetime/output limits. The fixture
owns instance deletion followed by server cancellation/join, retaining stdout,
stderr and the original process/cleanup report under each fixture directory.
Unwinding also attempts to retain stopped-process evidence. A deletion attempt is
not claimed as proof of remote deletion; successful runs require no abandoned
instance owners or cleanup errors. The 20 MiB profile keeps a 30-minute lifetime.
Keep all original journals, roots and byte evidence, including failed runs.
Product builds use root `target/`; qualification uses its own target directory.

Verified registry Testkit 0.107.3 archive SHA-256 is
`f1abdecdc13e767e8f5f7f83f2c19e48a414d6a7eea2e2ffdbfe95222c56d6c2`, source
`3c3fd63a6c16f4f1c38eef7984d211791022ba37`. Its native PocketIC 16.1 client still
pins thiserror 2.0.18; qualification selects that version and product retains
2.0.21. The managed-server feature does not enable unused Protocol/Types helpers.
That initial pass used Platform 0.33.7 / Types 0.20.4 in both graphs;
the current partial migration below supersedes that selection.
No direct PocketIC, Candid or CBOR dependency was added. Native DTOs stay private.
Host HTTP 0.80.0 archive SHA-256 is
`81c55b5294ff4c4bcab841ca6b72140796abd5c095651bf7e44e77ac2f5f843a`, source
`18198d6cfe6c097ef007a1a6a8b312c0a394e211`. This adoption's archives and checks are
under `target/evidence/supplier-refresh/`.

`target/evidence/direct-status/` retains the initial archive/index verification.
`target/evidence/snapshot-capture/` retains snapshot, custody, codec and consumer
validation logs; failed and successful IC fixture bundles remain retained. For fixture linting, run `cargo clippy --manifest-path
qualification/Cargo.toml --locked --offline --all-targets -- -D warnings`.
Local Linux acceptance does not establish hosted macOS or production HTTPS acceptance.

The preceding Platform 0.33.6 adoption aligned the qualification lock.
Its cached registry archive matches SHA-256
`24586654cc055365afd1c4bfb8dcd8f26f389bedba5cbee3dc722d99fce4a00a`, source
`95b61fb4bcdeb9f8e70ddcb65e83929213ff99a5`. Comparison with 0.33.5 found the new
Cloud Engine module and feature exports outside Backup's selected features;
canonical Types remain 0.20.4. The earlier snapshot checkpoint was qualified with that selection.

`store::capture::advance::tests::observation_tests` selects the local observation
certificate checks; `store::capture::advance::tests::certificate_tests` also covers
the shared existing terminal projection. Evidence is retained under
`target/evidence/capture-observation/`. These use real signatures over synthetic
certificates and do not qualify remote read-state requests or journal settlement
of newly fetched evidence. The original retained-result recovery checks remain
under `target/evidence/capture-settlement/`.

## Platform 0.33.7 and Tooling 0.44.1 adoption

Platform's verified archive SHA-256 is
`3e07e0565c70df9222016e93d6f488312176ef5e7036ded60db8b4bc3f61c05f`, source
`663cf913997490f448553119a6caf66b3cdecc81`. Both manifest minimums now select
0.33.7; the incoming lock updates are retained. Types 0.20.4 and all other
supplier selections remain unchanged. The new strict read-state decoder and its
supplier regressions were inspected; Backup's live lookup remains unimplemented.
The existing certificate byte reader only gains internal visibility for that
shared decoder; Cloud Engine changes remain outside selected features.

Tooling's preview retained all nine entrypoints and 51 managed files, changing
only the baseline and two LOC reporter files. Exporter receipt updates are
canonical. The 0.44 migration removes a Python evidence-verifier API that Backup
does not call or select. Tool pins, compiler, hooks, CI and Testkit wiring remain
unchanged; the existing tools pass offline admission without reinstallation.
The updated snapshot adds no release entrypoint or product API change.

`target/evidence/platform-tooling-refresh/` retains registry indexes, archive
verification, exporter preview/update, compiler identities and consumer checks:
93 capture/recovery tests, 17 documentation examples, strict product/private
Clippy, warning-denying Rustdoc, both actual IC fixtures, tools/snapshot admission,
formatting, documentation links, dependency listing and equal default/`-d 0`
LOC output. Native qualification covers existing status/list/take and interrupted
reply settlement, not new remote observation. The broader product suite and
hosted macOS were not selected. Original fixture bundles remain retained.

`store::capture::advance::tests::lookup_tests` selects strict signed read-state
request admission. The helper shares direct-key verification with existing call
requests; use the full `store::capture::advance` filter for that shared boundary.
`target/evidence/capture-lookup/` retains focused and full capture checks, lints,
documentation and existing actual IC fixture runs. Synthetic signed lookups qualify
local admission only; actual read-state and durable observation remain pending.

The 0.44.2 refresh changes only the selected baseline and ownership receipt;
all executable bytes remain identical to 0.44.1. Both workspace pins/locks are
updated without changing other suppliers. Registry checksum, exporter preview,
update and focused checks are retained under `target/evidence/tooling-refresh-next/`.
Existing product/IC qualification remains applicable; this documentation-only
supplier change does not justify rerunning those suites.

`store::capture::advance::tests::http_lookup_tests` selects one-shot lookup
transport checks. Evidence is under `target/evidence/lookup-http/`; the full advance
suite also checks the shared HTTP reader's original call behavior. Both maintained
IC fixtures now perform one additional externally reserved read-state request and
retain its policy, exact request, certificate and projected reply. Their production
ledger remains unchanged by lookup; native durable observation is still pending.

Tooling 0.44.3 preserves the baseline and updates the two selected LOC reporter
files. Default/zero depth now shows totals, depth one shows crates and depth two
shows top-level modules; JSON counting is unchanged. The canonical preview/update
and local depth checks are under `target/evidence/tooling-refresh-latest/`.
That directory also retains six lookup HTTP regression groups and strict Clippy:
new cases acknowledge server receipt before dropping a lookup, verify connection
closure with the listener/runtime still active, and independently bound stalled
reads by lookup expiry and deadline. No runtime dependency or production code
changed in this follow-up; earlier actual IC qualification is reused.

The durable single-lookup continuation supersedes the earlier pending-accounting
notes above. `store::capture::advance::tests::durable_lookup_tests`
selects eight native groups covering conserved spending, exact replay, live
admission and uncertain writes at all four new transitions. Evidence under
`target/evidence/durable-lookup/` records 112 advance tests, 10 ledger tests,
16 custody tests, 18 documentation examples, strict product/private Clippy and
warning-denying Rustdoc. All three actual IC fixtures pass; the new fixture uses
product journal reservations for pending-take lookup and replays retained proof
without a signer or server. The subsequent settlement batch below qualifies original-call recovery from
this new proof.

The retained-observation settlement batch uses the same native filters plus all
three actual IC fixtures. `target/evidence/lookup-settlement/` retains 118 advance,
10 ledger and 16 custody tests, 19 documentation examples, strict product/private
Clippy, warning-denying Rustdoc and format/link checks. The pending-take fixture now
settles from the lookup proof after shutdown and reopens again for offline replay.
Direct Cargo invocation uses a physical default Testkit asset path without `..`;
Make and an explicit `IC_BACKUP_TESTKIT_DIRECTORY` keep their existing selection.

For the 0.101.5 local metadata authentication boundary, select
`store::capture::advance::tests::metadata_tests`. Six native groups exercise exact
signed request/certificate identity, metadata profile and bounds. Evidence under
`target/evidence/metadata-auth/` includes the full advance and transfer selections,
strict lints and Rustdoc. The existing actual-IC suite covers unchanged callers of
the shared verifier. The subsequent metadata acquisition fixture is described below.

For durable metadata acquisition, select
`store::capture::advance::tests::metadata_acquisition_tests`. Eight native groups
cover the original journal's four metadata boundaries and retained authority.
`target/evidence/metadata-acquisition/` contains 132 advance, 10 ledger and 16
custody tests, 21 documentation examples, strict product/private Clippy and
warning-denying Rustdoc. `make test-qualification` passes all four IC fixtures;
`metadata_acquisition_and_offline_replay` reads real metadata and replays it after
server shutdown and signer revocation. Payload download remains unqualified.

For durable schedule binding, the native selections are
`store::capture::advance::tests::schedule_tests` and
`store::capture::transfer::binding`. Evidence under
`target/evidence/transfer-binding/` contains 135 advance, 48 transfer and 10 ledger
tests, strict product/private Clippy and all four PocketIC fixtures. The real
metadata fixture now verifies schedule binding and byte-preserving offline replay.
The existing deterministic geometry tests remain the version-one schedule owner;
future ordering changes require explicit compatibility handling.

The signer-free local binding cleanup is qualified under
`target/evidence/transfer-replay-cleanup/`: 135 capture-flow tests, 22 documentation
examples, strict product/private Clippy, warning-denying Rustdoc and all four IC
fixtures. The binding callers compile without a metadata signer or supplied
metadata admission; shared replay checks retain original capture authentication.

Scheduled data authentication is selected by
`store::capture::advance::tests::data_tests`. Evidence in
`target/evidence/data-auth/` records 141 advance and 48 transfer tests, 23
documentation examples, strict product/private Clippy and warning-denying Rustdoc.
All four existing PocketIC fixtures pass after sharing certificate comparison;
these regress status/snapshot/metadata callers, not remote payload acquisition.

Durable data acquisition uses the focused native selection
`store::capture::advance::tests::data_acquisition_tests`.
Evidence under `target/evidence/data-acquisition/` records 149 advance, 10 ledger
and 16 custody tests, 24 documentation examples, strict product/private Clippy
and warning-denying Rustdoc. All four IC fixtures pass; the fourth is now
`metadata_and_payload_acquisition_and_offline_replay` and reads real Wasm bytes
before exact offline replay. Its larger fixture evidence allowance admits the
retained data proof without changing product ceilings. Artifact assembly and
complete backup/restore remain unqualified.

Complete payload verification uses `cargo test -p ic-backup --lib
data_acquisition_tests --locked --offline`. Historical evidence below records the
then-selected metadata/data cases.
`target/evidence/data-verification/` records those checks, 24 documentation
examples, strict product/private Clippy, warning-denying Rustdoc and four IC
fixtures. Coverage and combined module-hash refusal preserve accepted evidence;
the actual IC fixture also verifies complete bytes after shutdown. No assembled
files or completed backup are qualified by these checks.

Local assembly uses the shared capture fixture and the flat native module
`store::capture::advance::tests::assembly_tests`. Latest `target/evidence/assembly/` records 159 advance,
10 ledger, 16 custody and 23 shared payload tests; 24 examples; strict product/private
Clippy; warning-denying Rustdoc; and all four IC fixtures. Eight assembly groups
cover exact files, empty members, incomplete coverage/hash refusal, interrupted
members, unsafe/changed custody, both uncertain journal transitions, original
frame budgets and event ordering. The real IC fixture writes and replays the
unpublished bundle after server shutdown. These checks do not qualify completion
publication, automatic partial-member recovery or physical power loss.

Final status and completion use the existing native advance selection and the
renamed actual IC fixture `complete_capture_and_offline_replay`.
`target/evidence/completion/` records 169 advance tests (four worker threads),
10 ledger and 54 store tests, 24 examples, strict product/private Clippy and
warning-denying Rustdoc. All four IC fixtures pass. The complete fixture validates
capture-to-manifest, inventory and payload verification before exact local
reconciliation after shutdown. Its original evidence allowance is 6 MiB, including
all six calls and the 1 MiB manifest ceiling; product bounds did not increase.
Four final-status groups and six completion groups cover distinct request identity,
original budgets, refused policy, uncertain frames, exact receipt/hash/timestamp,
full/partial staging and existing consumers. Physical power loss, target succession,
large-transfer performance and restore are not qualified by this batch.

Authenticated target succession uses the existing Host atomic writer and shared
initial-intent persistence. `target/evidence/succession/` retains its focused native
and actual-IC results, including failed checks and their corrections. Select native
handoff cases with `cargo test -p ic-backup --lib succession_tests --locked --offline
-- --test-threads=4`; retain `store::tests::capture` for initial-budget and custody
regressions. The private `complete_capture_and_offline_replay` fixture now creates
a successor after shutdown and replays the old completed receipt with no additional
signatures. It does not perform another capture or qualify power loss.


Capture lifecycle cases live under `store/operation/`;
protocol primitives retain their tests under `store/capture/advance/tests/`;
restore cases live under `store/restore/tests/`. Select the restore suite with
`cargo test -p ic-backup --lib store::restore::tests --locked --offline`.
Shared native helpers belong in named fixture modules, with explicit imports.
Keep single-consumer helpers beside their tests; do not nest test groups to inherit
another group's fixtures. Synthetic native evidence does not qualify IC effects.

## Pre-replacement reference

The local Git reference `preserve/backup-0.101.13-before-replacement` preserves
the full pre-replacement source and both dependency locks, including the tested
rand 0.10.4 selection. It is a preservation checkpoint, not a published release.
Build outputs and successful validation logs are retained separately under
`.git/recovery/backup-0.101.13-before-replacement/`; `manifest.json` records
compiler identities, original paths and byte digests. Keep the original journals,
fixtures, failed evidence and store artifacts in their existing locations.

This version is a library: the preserved executables are unit/IC test runners,
not a general operator recovery CLI. An existing consumer must use the preserved
library API and original application authority. Rebuild from that source with
its root and private qualification locks and pinned compiler; do not use current
replacement readers on old stores. Inspect or restore the reference in the normal
checkout only after preserving current work. This does not authorize a second
checkout, evidence cleanup, automatic migration or external effects.

### Host-store/1 reader before complete restore

The later exact uncommitted capture/status-only-restore reader is preserved under
`.git/recovery/host-store-1-before-restore-20261116/`.
`manifest.json` records all 185 source file hashes, both original locks and six
rebuilt library/test artifacts, selected from Cargo's artifact records.
The source archive SHA-256 is
`6658a8963fe3d56cb5fe13ec210c9170f8d6b7b50f0de33583e627287a371b06`.
The directory also retains toolchain output, build records and preservation inputs.
No index, ref, commit, tag or delivery was changed.

This reader understands the newer host-store/1 capture intents and its original
local-restore-plan/1 status journal. The earlier b7c2c6a reader does not understand
those newer sources. Keep retained operations under their matching reader and
original authority; no migration/reset is selected. Extract only into an explicitly
authorized location, never over retained records. This is reproducible source/build
preservation, not a new recovery CLI or generalized bundle import.

The rebuilt preserved `ic-capture-6996afc6891e5900` independently passed all three
preceding IC journeys; evidence is `target/evidence/restore-replacement/preserved-ic.log`.
Current replacement evidence is in `target/evidence/restore-replacement/`, with
original actual-IC fixtures under `qualification/target/fixtures/`.

Final Backup-owned cold recovery acceptance is retained under
`target/evidence/backup-completion/`: pending and completed IC restores reopen from
ID/approval after their in-memory plan is dropped, without a signer/connection.
The current public example on `BackupStore::open_restore` compiles independently.
Canic integration owners are linked from the sibling review and current handoff.

The subsequent legacy-review corrections are qualified by
`target/evidence/review-fixes-*.log`: 232 native tests, ten actual-IC journeys,
five public examples, strict Clippy for both workspaces and warning-denying Rustdoc.
The shared recorder changes justify the selected `store::` suite and all ten
`--test capture` journeys. New cases are
`full_mib_chunk_captures_and_restores_through_complete_envelopes_and_proofs`,
`load_certificate_survives_local_write_failure_and_settles_offline`, and
`isolated_normal_consumer_continues_capture_restore_and_replays_in_new_processes`.
Native runtime/HTTP/certificate cases cover redacted received-proof errors,
closing deadlines, offline submitted-lookup import and unchanged old allowances.
The independent executable's graph/build and each child output are retained with
its IC fixture. It shares the existing qualification lock, which adds only that
local package; supplier versions and the root lock are unchanged. Final comment,
error-message and import cleanup is checked by Clippy/Rustdoc without repeating
unchanged runtime journeys. No release/support matrix or legacy test rerun is claimed.

### CI setup diagnostics

Toko 0.45.8's `SetupDiagnostics.pm` owns command capture/status and redaction;
`scripts/ci/setup.pl` owns the per-phase directory and exclusive artifact writer
for host tools, Rust toolchain/tools, dependency fetches and the secret scanner.
Only a failed command's bounded, sanitized excerpt is retained; successful phases
retain a status-only receipt. No cache,
home, environment dump or backup store is uploaded. Hidden attempt directories
under `target/ci-setup/` hold at most 100 lines / 32 KiB of diagnostics plus a small
status header per phase. Sensitive URLs, known secret environment values,
credential lines and private keys are omitted; oversized lines/truncation are
explicit, and workflow commands are neutralized before printing or upload.
Failures print an 8 KiB excerpt and retain their original exit status. Failed
collection is reported separately and cannot turn a failed command green.
CI uploads only these diagnostic files, including hidden directories, for seven
days and reports the artifact URL or missing/failed upload. This does not assert
hosted retrieval until observed. Run `perl scripts/ci/test-setup.pl` after changing
this wiring; the focused offline fixture covers early failure, hidden evidence,
redaction/bounds, empty diagnostics, collector failure and symlink refusal.

### Earlier partial Platform migration

Protocol 0.2.8 owns product data/allocation/empty-response decoding for 0.104.0.
The earlier qualification below used 0.2.7; 0.2.8 leaves this management codec unchanged. The
qualification signer and proxy use Platform 0.34.9's offline envelope encoder and
strict packet decoders. Complete envelopes are decoded before selecting content;
request-ID/signable failures refuse, and only new signer outputs are encoded.
Retained original requests are never re-encoded during recovery.
The shared fixture authority is exposed through the private consumer library;
qualification uses normal module/package imports instead of a source-path override.

Product Platform 0.33.7 remains for status/list/take/metadata, request DTOs,
certificate verification and retained-envelope authentication. Qualification's
`ic-platform-legacy` dev dependency supplies the product's current typed source-read
API and its matching Principal, not an alternate transport or fallback reader.
Both cohorts are explicit and this remains partial adoption. The new Protocol
uses Types 0.21.2; old public product identities remain Types 0.20.4.

The qualification graph selects Testkit 0.107.4. A separate resolution probe
confirmed 0.107.3 also remains compatible with Protocol 0.2.7 while its optional
codec feature is disabled; the patch is not a prerequisite for this migration.
The managed-server source is byte-identical between the two Testkit archives.
The selected 0.107.4 archive checksum is
`8629db43fa7a3e3655184c6173a5ccdecf67a87d29f0c929228bec7f8a1c392b`, source
`67ac61f89a7311e278027bee3078de7e24a0a241`. This upgrade changes no server asset
or lifecycle limits. Current migration logs are under
`target/evidence/platform-recheck-current/`; the previous Host/Testkit qualification
remains historical evidence under its original directory.

### Platform 0.34.10 qualification continuation

The private qualification manifest now requires Platform 0.34.10, matching the
pre-existing lock update. Its archive SHA-256 is
`7798e7684a2e857355408f2c4252d1187589380609b5f2ef259482e6e1a34571`, source
`efdc02936dde0319ef32825a499fa28db20cb1ff`. Selected ingress and Ed25519 source
are byte-identical to 0.34.9. Product Platform 0.33.7 / Protocol 0.2.8 remain
unchanged because the full migration blockers are recorded in the handoff.

The preserved qualification lock also selects Process 0.80.3 and related
transitives. Strict qualification Clippy and the actual caller target
`cargo test --release --manifest-path qualification/Cargo.toml --locked --offline
-p ic-backup-qualification --test capture` pass all 11 journeys (105.24 seconds)
with `IC_BACKUP_TESTKIT_DIRECTORY` selecting the existing prepared server.
This includes 20 MiB, original-reply/observation recovery, process cleanup and
the isolated normal consumer. Evidence: `target/evidence/platform-qualification-03410/`;
original fixture bytes and server output remain under qualification target.
Linux results do not qualify macOS-specific Process changes or the blocked
product certificate/status migration. No full product suite or release gate
was repeated for this qualification-only manifest change.

### Current Protocol 0.3.3 / Platform 0.34.11 adoption

All production management request encoding and reply decoding selects Protocol 0.3.3, source
`64897c9c522ab6730620985b6d58607d9c593903`, archive SHA-256
`4676f2fabfff6a9aab9d6cccdf2a5870059e88b21345bc1913818be621dc1f34`.
Production ingress, direct-key and certificate verification, plus fixture signing use
Platform 0.34.11, source `80a5b488666629dd28152f6bcb6d489091d55994`, archive SHA-256
`6269645258c22c1cccbaaf62ec710b7a30fbbcfe879aff6a53ba53b8526c9c84`.

Normal graphs select only Platform 0.34.11 and Types 0.21.2; neither lock retains
Platform 0.33 or Types 0.20. Product test dependencies alone enable
`management-snapshot-fixtures` and `certificate-fixture-signing`. Independent
malformed schemas and request-codec comparisons use test-only Candid 0.10.40 and
Management Canister Types 0.8.0, already selected by the actual-IC graph. They
are not alternate production decoders. No sibling patches or copied codec source.

Latest evidence is under `target/evidence/platform-full-port/`; earlier failed
ports remain under `target/evidence/platform-next/`. The handoff owns results.

### Testkit 0.108.0 qualification adoption

The private workspace selects Testkit 0.108.0 from verified registry source
`78f1d2fbfbda6cac3b39f49dfe8aaa3c26472135`, archive SHA-256
`6fa2e6157d27d15ba1bc93923adc3221e1c09f9f28ec2f8bdde09b04b528cf80`.
The selected server sources are byte-identical to 0.107.4; held-reply modules
move to directory modules without content changes. The release raises its Process
floor to 0.80.3 and optional Protocol floor to 0.3.2 and relaxes exact WAT/parser
pins. Backup selects only `managed-server`; its qualification lock changes only
the Testkit version/checksum. PocketIC remains 16.1.0 with thiserror 2.0.18.

The pre-existing Platform 0.34.12 and Tokio 1.54.1 lock selections are retained.
Both `make install-testkit-server` and `make testkit-server-check` admit the
existing prepared server without replacing original assets. Qualification Clippy
passes; the handoff owns actual-IC results. Evidence, archive and pre-update lock:
`target/evidence/testkit-next/`. No product dependency or server setup API changes.

### Platform package identity recheck

The latest registry review selects Platform 0.34.13 / Protocol 0.3.5 / Types
0.21.2 in both locks, already present when the recheck began. The two newer
archives identify source `379789a1b9db1e9348a5673ed847896e5011f81c` and match
registry SHA-256 checksums:

- Platform: `d890bfa81018e07fe2fb9f2621ee89c181bd061a2e98f4f979c599501f239dc1`.
- Protocol: `a2ae9adc784809f210d491f301a9b4549bf619dbbeac835dc7c8d49d1eb152b0`.

Source diffs from Platform 0.34.12 / Protocol 0.3.3 affect only unselected
features. Both locked metadata graphs contain one registry identity per Platform
package; both all-target Clippy checks pass. Compatible minimum requirements stay
unchanged because Backup needs none of the new APIs. Prior actual-IC evidence
remains applicable; no fresh IC or hosted run is claimed. Archives, source diffs,
saved locks, resolved graphs and logs: `target/evidence/platform-park-refresh/`.
The handoff records the separate direct test-dependency cleanup on Platform #11.

### Testkit 0.109.0 qualification adoption

The qualification workspace now selects Testkit 0.109.0, source
`1fdd03705377c1a74211ec1f4ca9a15440adc0a0`, verified registry archive SHA-256
`6405078e5332b34ab898d4c32c41f39b358655bb3d4aae508d3ff17f781cf66c`.
The lock already selected 0.108.1 before this adoption; only Testkit's version and
checksum changed. PocketIC 16.1, thiserror 2.0.18 and other selections remain.

The new bounded held-reply clock API is outside Backup's selected features.
Server sources and manifest dependencies are unchanged from 0.108.1; the selected
library root differs only in documentation. Existing actual-IC evidence remains
applicable. The handoff records fresh focused qualification, with archive/source
diffs, saved lock and logs under `target/evidence/testkit-refresh/`.

### Toko Tooling 0.45 / Testkit 0.110 adoption

Tooling 0.45.0 uses source `53c0eba3b1b207e0cd9e747eece438e21b693261` and
registry archive SHA-256
`3789a925c1165317eff84cc80436195b66c0478b0c2b441bc2f1a755caf6a23a`.
Both workspaces inherit the exact new pin. The canonical preview/update retains
all ten selected entrypoints and 59 managed files; only the baseline bytes change.
No managed receipt/file was edited by hand. Installed executable bytes and tool
selectors are unchanged. The new consistent-flow policy matches the delivered
Backup #3 command/guarantee/trigger map; no new release runner or proof reuse is
activated by updating this snapshot.

Testkit 0.110.0 uses source `23f80e236343d7ff8ec070536863daecb89c77ec` and
registry archive SHA-256
`16709c74c5c169ea786224ea8e10a8b45446219000c18c441e8aa73bd1409252`.
Its source tree is byte-identical to 0.109.0; the manifest raises Process to 0.81.
Qualification consumes `Stopped::process.failure`, so its direct Process catalog
also selects 0.81.0. Keeping 0.80 would create incompatible public report types.

Process 0.81.0 uses source `0908efd0a94e1284f8f79c81902c1ee75db9d497`, archive
SHA-256 `1b81431d49e906feee6812a4fa8539feea57803f9937b926472ae947fa9bf6c5`.
It adds finite stdin bytes and removes Copy from input/policy. Testkit retains its
null-input server path; Backup adds no process or credential handling. PocketIC
16.1 and qualification thiserror 2.0.18 remain selected. Relative to saved starting
locks, root changes only Tooling; qualification changes Tooling, Testkit and Process.
The starting Process selection was 0.80.5. Pre-existing Platform/bytes selections
are preserved. Provenance, preview/export, lock snapshots, source diffs and caller
checks are under `target/evidence/tooling-testkit-adoption/`; the handoff owns
current execution results and qualification limits.
