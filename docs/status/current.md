# Current handoff — 2026-11-06

## Pending test consolidation

The cleanup is prepared for the user-authorized **0.58.5 deployment** from `main`
to `origin/main` and crates.io. The original cleanup baseline is `b70084f`; full
release validation and delivery remain pending. Native suites share workflow providers,
identity-fault assertions and artifact fixtures. Common immutable IO tests live
at their owner; two-layout restore checks remain local. Eight model suites share
explicit schema-field checks. Duplicate-version tests retain their original
first-serialized-version case and also explicitly reject duplicate root versions,
including when an embedded record is serialized first.

The independent standalone consumer owns the complete upload/restore journey,
including a known stored Wasm chunk, a cleared live store, changed live state,
complete byte/state verification, acceptance and terminal replay. First and second
lost uploads have separate cold-process checks; successors follow stage order,
not numeric IDs. Twelve superseded Core cases and the overlapping direct Agent
round trip are retired. The direct trip's sole-use upload helper is removed.
Core capture/download, retained-prefix and explicit reconciliation probes remain;
native matrices cover malformed second-upload and start replies. Direct Agent
probes retain signed ingress, trust rejection and passive upload/readback proof.
[The coverage map](../development.md#simulator-coverage-ownership) distinguishes
native validation/order proof from actual IC effects.

Shared's count is **38,134 test lines**, down **1,417** across the cleanup from
39,551; the latest pass removes **167** from 38,301. Production remains **15,187**.
Public APIs, product logic, stored formats and standalone behavior stay.
The incoming lockfile update selects Platform **0.23.2**: published source
`81b3f05f53ed113d02a671d47d13f1a36222e845` and archive checksum
`8e18706c6bac24e9d0bc0afa3568cafa14c1cf20901780c8d27d40cd4b38a0af`
are verified. Only unused additive create-canister DTO/protocol exports change;
management 0.11.0, Types 0.18.1, external Principal 0.1.5 and SDK suppliers stay.
The release gate will qualify this current lockfile. Earlier source snapshots, counts and qualification remain in
`target/evidence/test-cleanup-0585/` and `target/evidence/test-convergence-0585/`,
including all **22 retained Core simulator-target cases**, **13 selected consumer
cases**, persistence checks and transferred native fault matrices.

The latest source/case mapping and logs are in `target/evidence/test-trim-0585-03/`:
**47 affected native tests**, all **five retained direct Agent simulator probes**
and the complete consumer round trip pass. Strict Core library/test and Agent
standalone/test Clippy pass on Rust/Cargo **1.99.1**. Full release and other-host
gates were not rerun. Earlier sandbox loopback and mechanical failures remain in
their original evidence; real simulator qualification used loopback-enabled runs.

## Delivered 0.58.4

Both crates **0.58.4** are published on crates.io. Release
`c43c196582ccc8249263c6a01dfee8b8ac70c748` and annotated `v0.58.4` are on
`origin/main`; original source is `6f27877da34e6d54ea6ca06fb9abdeb730c2db3f`.
The user authorized new Testkit adoption and deployment, including the committed
Shared Tooling cleanup at `42ffa824362227ce9fa5ce61eec5efa9c8c12de9`.
No history rewriting or additional artifact cleanup occurred during delivery.

Testkit **0.87.0** is checksum/source-verified at published commit
`e3b8a88539c54cb1da3ae471954fa234beabdecf`; its exact CLI and authenticated
PocketIC **16.1.0** are installed/offline-admitted. Core fixture coordinators now
await client and provider operations on caller-owned Tokio test runtimes. Async
restore providers share a Tokio mutex instead of RefCell borrows across simulator IO.
Agent uses the single async client path. Successful fixture owners explicitly
await instance deletion while retaining their servers, preserve deletion failures
and never retry cleanup. Panic Drop still stops/reaps the owned server. Immediate
polling remains only for in-memory fakes and pre-dispatch refusal checks.

Testkit now uses Platform **0.23.0** protocol/CBOR codecs in development and shares
that facade with production; Platform 0.22.3 and unused Tokio support packages
leave the lock. Normal production features, Types **0.18.1**, external Principal
**0.1.5**, management DTO **0.11.0**, Agent **0.49.3**, Candid **0.10.40** and
Wasm-only Runtime **0.19.2** remain. Host/Build **0.74.4** and Rust minimum/exact
compiler **1.99.1** are unchanged. Core's new Tokio declaration is test-only;
public APIs, v1 records, original accounting and backup/restore behavior stay.

## Delivered 0.58.4 qualification

Evidence is `target/evidence/testkit-deploy-0584.yt3b8nrm/`: official archive/source
comparison, strict all-target/all-feature Clippy, complete upload/restore,
lost-load refusal and independent cold-process restore/terminal replay pass.
Actual registries preserve all 34 Core and 37 Agent example cases from 0.58.3.
Initial mechanical migration failures are retained. Candid/dependency/format/link
checks pass. The complete Linux release gate also passes, as recorded below. Native macOS,
installed IC and Canic acceptance remain separate; hosted CI was not manually
dispatched.

Shared **0.30.4** at `2aa41c7394300302fe802b69b31b855bdc89b671` remains selected
with 157 exact payloads. Candid checks cover all 324 Rust files and 36 named
external declarations, including the explicitly selected inventory target module.
Cleanup removed 183 historical reports/handoffs (3,407,885 bytes), retaining
[compact conclusions and original Git sources](../reports/README.md). Product
source/license provenance, vectors, release/recovery records and ignored artifacts
remain in custody. Prior focused tooling/hook proof and limits belong to
[the adoption review](../shared-tooling-review.json) and
`target/evidence/shared-0304.x_mz38e6/`. No service, schedule, candidate projection
or release-stage reuse is selected.

## Release and registry qualification

The [receipt](../release.json) binds the complete Linux `release-verify` gate:
strict Clippy, **554 tests**, independent normal consumers, docs, tooling and
packaging. Standard delivery committed, tagged and atomically pushed; `make publish`
then verified and uploaded both crates. Independent checks confirm non-yanked
registry versions, archive checksums, clean VCS identities and **341 source files**
against the release commit. Remote branch/tag identities match. Proof stays in
`target/evidence/testkit-deploy-0584.yt3b8nrm/` and `.git/release-state/`.
The 0.58.3 receipt and proof remain at their original Git revision and in
`target/deploy-0583.cl8y8lzv/`; no historical evidence is relabelled or removed.

## Recovery and retained contracts

Standalone host-driven backup/restore, mandatory application admission and fenced
same-ID restore safety remain. Current v1 records/hashes and bounded attempt
spending are unchanged; persist intent before effects, reconcile lost replies,
and preserve unfinished journals, artifacts, references and external obligations.

Layouts containing `consistency-requirement.json` or `fence-obligation.json` refuse
before record use/spending, including malformed bytes and symlinks. Distinct-edge
learned continuations refuse without rewriting bindings. Recover those originals
with **0.57.1 and the original qualified integration before upgrading**. Never
remove markers, rebuild plans, reinterpret payloads, re-sign requests or reset
accounting to bypass admission. No migration, evidence cleanup or refund is provided.
See [the migration](../canic-adoption.md#058-api-migration) and
[implementation contracts](../implementation-contracts.md).
