# Development setup

The virtual workspace has one unpublished library under `crates/ic-backup`.
Local inventory, payload checks and capture intent use Host filesystem/artifact
primitives and published Platform identity and management DTOs, with Serde/JSON
and typed errors. Platform's native protocol codec checks prepared status call
content; its historical certificate verifier authenticates retained replies and
certified rejections. Host HTTP 0.79.0 supplies the bounded body reader and
Reqwest 0.13.5 re-export with opt-in Rustls, without a direct Reqwest dependency
or Agent. Its futures require a caller-owned Tokio runtime with IO
and timers; tests use the selected Tokio 1.54.1 runtime. External IC DTOs and
certificate codecs remain Platform-owned. No Canic framework dependency is added.

Host FS and Artifacts 0.79.0 resolve from verified registry archives. The
temporary sibling overrides were removed after native consumer qualification;
no sibling checkout is required. Registry admission and the affected persistence
suite for the earlier 0.77.3 adoption remain under
`target/evidence/registry-host-0.77.3/`; current archive/graph evidence is under
`target/evidence/host-0.79.0/`. Consumers exchanging Backup's public Host digest
and error types must use the 0.79 line. This changes Rust identities, not stored
formats or recovery authority; the 0.101.0 GitHub checkpoint contains the adoption.
Unpublished 0.101.5 development follows the 0.101.4 durable recovery checkpoint. The direct capture provider composes status, list and take through the existing helpers.
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
explicit through `make install-hooks`. Tooling 0.44.3 is adopted from its registry
archive with SHA-256
`8a9bc7b6827044baac8134cada7ee6bc31f86c70beec9608d1d39d911eba15e1`,
source `69ea17e551f33254d87de9824c30f2e74536eb10`. Both workspaces declare
its exact pin and each member inherits the disabled development marker.
The canonical exporter installs nine selected entrypoints, including the shared
pre-commit formatter hook, with derived companions, and owns [.toko-tooling.snapshot](../../.toko-tooling.snapshot).
No producer regression suites or release/publication entrypoints are selected.

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
its `source_tests` selection covers block boundaries, retained custody and local
source-to-readback composition. These synthetic bundles do not qualify capture evidence.
Select capture intent tests with `cargo test --locked --offline -p ic-backup
store::tests::capture`; select framed-ledger tests with the same command and
`store::capture::ledger::tests` as its filter. Its acknowledged child-process death case verifies original
intent and target-claim retention, not attempt spending or filesystem power loss.
Use `store::capture::transfer` for planning and bounded raw reply tests;
`store::capture::transfer::metadata` selects the metadata decoder alone.
Use `store::capture::transfer::readback` for uploaded-state comparison, including
exact float bits, lifecycle state, destination provenance, complete chunk sets and
ordered byte coverage. Its `data` submodule selects payload comparison cases alone.
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

Use `store::capture::advance` for first-status persistence and recovery tests.
Their injected provider checks ordering, not IC effects. The acknowledged child
fixtures retain evidence at reservation, invocation, prepared-request and settlement
boundaries; they do not qualify physical power loss or a concrete provider.

Use `store::capture::advance::tests::certificate_tests` for signed synthetic
certificate and durable replay tests. The full `store::capture::advance` filter
also covers the existing reservation/recovery cases. Evidence for the minimum
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

`store::capture::advance::tests::http_tests::direct_tests` selects concrete-provider
native tests; the full advance selection also covers retained historical checks.
The independent private package under `qualification/crates/ic-backup-qualification`
uses Testkit's native client boundary for real stopped-canister status, snapshot list/take and
cold replay after gateway shutdown. A second case retains a real take reply,
returns uncertainty before journal settlement, and settles after server shutdown
and guard revocation. It uses fixture signing/maintenance authority;
Canic process/fence qualification remains with its integration owner.

```bash
make install-testkit-server
make testkit-server-check
make test-qualification
```

Setup selects the unique Testkit version from `qualification/Cargo.lock` through
Tooling's Cargo installer and delegates server assets to Testkit. It is also an
ordered `install-tools` extension after common tools and locked fetch. The matching
`tools-check` extension and fixture library admission are offline and never install.
There is no second PocketIC catalog, auto-download or IC executable bundle.
Keep `.tools/testkit/` setup evidence and `qualification/target/fixtures/` original
journals, roots and raw server output, including failed runs. Production builds
use root `target/`; qualification uses its own `target/`, never sibling/shared output.

Testkit 0.106.0 archive SHA-256 is
`090cea21af2208dad7a359e99fe803b356fe8daa27cdc52d545df7de3b32c43e`, from source
`d12b0174cd04c556f065c39bc12dd366628cc6df`. Its native PocketIC 16.1 client pins
`thiserror = 2.0.18`; the separate qualification lock satisfies that requirement.
The product's compatible requirement admits it while its original lock retains
2.0.21. Both graphs select Platform 0.33.7/Types 0.20.4 and Host 0.79.0. Revisit
this concrete constraint when the client pin changes. No direct PocketIC, registry
management, Candid or CBOR dependency is added; native DTOs stay fixture-private.

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

`store::capture::advance::tests::http_tests::lookup_tests` selects one-shot lookup
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
notes above. `store::capture::advance::tests::http_tests::durable_lookup_tests`
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
`store::capture::advance::tests::http_tests::metadata_tests`. Eight native groups
cover the original journal's four metadata boundaries and retained authority.
`target/evidence/metadata-acquisition/` contains 132 advance, 10 ledger and 16
custody tests, 21 documentation examples, strict product/private Clippy and
warning-denying Rustdoc. `make test-qualification` passes all four IC fixtures;
`metadata_acquisition_and_offline_replay` reads real metadata and replays it after
server shutdown and signer revocation. Payload download remains unqualified.

For durable schedule binding, the native selections are
`store::capture::advance::tests::http_tests::metadata_tests::schedule_tests` and
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
`store::capture::advance::tests::http_tests::metadata_tests::data_tests`.
Evidence under `target/evidence/data-acquisition/` records 149 advance, 10 ledger
and 16 custody tests, 24 documentation examples, strict product/private Clippy
and warning-denying Rustdoc. All four IC fixtures pass; the fourth is now
`metadata_and_payload_acquisition_and_offline_replay` and reads real Wasm bytes
before exact offline replay. Its larger fixture evidence allowance admits the
retained data proof without changing product ceilings. Artifact assembly and
complete backup/restore remain unqualified.

Complete payload verification reuses that native selection: `cargo test -p
ic-backup --lib data_tests --locked --offline` selects 33 metadata/data tests.
`target/evidence/data-verification/` records those checks, 24 documentation
examples, strict product/private Clippy, warning-denying Rustdoc and four IC
fixtures. Coverage and combined module-hash refusal preserve accepted evidence;
the actual IC fixture also verifies complete bytes after shutdown. No assembled
files or completed backup are qualified by these checks.

Local assembly extends the same caller fixture and adds the native child
`data_tests::assembly_tests`. Latest `target/evidence/assembly/` records 159 advance,
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
