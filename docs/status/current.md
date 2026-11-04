# Current handoff — 2026-11-04

## Delivered 0.58.0

Both crates are published as **0.58.0**. Release commit
`dc1a0dbb301b50b42ac3ace013b13df14b46ae63` and annotated `v0.58.0` are verified on
`origin/main`; the [receipt](../release.json) binds source `1358f7a` and its complete
Linux release gate. All **554 test cases** pass, with no failures or ignored cases,
including both simulator suites. Clippy, tools, source/format admission, release/
hook fixtures, Rustdocs, independent consumers and archive compilation pass.
Publication separately passes Cargo verification and the mandatory lint gate.
Independent downloads match registry checksums, release provenance and all **341
non-generated source files**. Evidence is in `target/deploy-0580.b8b51qgs/` and
`.git/release-state/`. No native macOS, live IC or installed Canic acceptance is claimed.

`model::attempt_journal::MutationReservationError` now owns the four original
pending-mutation refusals. Application, management, transfer-read and upload request
errors carry it through `Reservation`, including a typed standard error source.
Former flat variants and manual conversions are removed. Checks and their order
are unchanged; observation-specific denials keep their current owners. Application
payload validation now lives with its sole ingress owner, preserving argument/method/
reply/receiver/target admission order and exact hashes. Immutable records use one
validator-based read path under the original root, lock, decode and byte bounds.
See [the API migration](../canic-adoption.md#058-api-migration).

The earlier cuts retire the independent capture-consistency and application-fence
framework and Agent adapter. Mandatory application admission, accounted application
ingress and fenced same-ID restore requirements remain. `ApplicationFenceState`
now belongs to `model::restore_safety` with unchanged meanings. Complete upload
planning/sessions replace public single-chunk preparation. Advancement carries one
checkpoint learned commitment to every successor, without per-successor overrides.

Guarded root checks refuse retained `consistency-requirement.json` or
`fence-obligation.json`, including malformed bytes/symlinks, before record use or
spending. Retained distinct-edge continuations refuse before callbacks without
rewriting bindings/journals. Recover these originals with **0.57.1 and the original
integration before upgrading**. Preserve bytes, pending attempts, references,
artifacts and external obligations. Never remove markers, rebuild plans, reinterpret
payloads or reset accounting to bypass admission. No migration or cleanup is provided.
Current v1 attempt/workflow/binding/artifact formats and recovery contracts remain.

Production LOC is **16,530**, down **99** in the latest batch and **2,119** from
18,649 before the cleanup. This comprises **15,344 library lines** and the unchanged
**1,186-line standalone example**. Separate tests comprise 39,177 LOC and 554 test
functions. Retired API fixtures are removed; new public error-source and retained-
state/fan-out refusal regressions cover the current contract. Standalone host-driven
backup/restore remains supported. Current Canic production inspection found no calls
to removed APIs or flat reservation-error matches; installed acceptance is separate.

## Current tooling and evidence

Host Artifacts/FS/Process **0.74.3**, Testkit **0.84.1** and Build release editor
**0.74.2** are explicitly selected. The incoming lockfile already selected Host
0.74.3; release preparation changes only Backup's package versions. Published
archives/VCS were verified; Artifacts
and consumed editor source are unchanged, while FS/Process relocate tests. Tool
and editor admission, 34 focused native cases, normal consumers and actual editor
preparation fixtures pass in `target/tooling-adoption-058.v843s4en/`.

Shared Tooling **0.29.4 at `628ccc7`** is adopted canonically: 149 payloads, eight
updates and one addition. Current retained-state admission, source/mode integrity,
changed-script checks and consumer receipt/recovery fixtures pass without cleanup.
See [the adoption review](../shared-tooling-review.json). Rust stays **1.99.1**;
Production Platform is now **0.22.0**; Testkit retains **0.21.0**. Types **0.18.0**
and authenticated PocketIC **16.1.0** remain.
Runtime **0.19.1** is adopted through Testkit's Wasm-only IC0 edge; the incoming
lockfile already selected it; release preparation preserves that selection. The published checksum/source
commit `74a5071` and unchanged selected IC0 source are verified. Six normal graphs
(both libraries on Linux and both macOS architectures) exclude Runtime; the Wasm
development graph enables only IC0. Rust 1.99.1 admission passes. Evidence is in
`target/runtime-adoption-058.vbdr3mto/`; no host compilation or IC-effect rerun is
needed for this target-only selection, and no Wasm product qualification is claimed.

Platform evidence is `target/platform-022-adoption.eutssg1n/`: verified published
checksum/source `b95c9a2`, unchanged selected codecs and reviewed Agent export
identities. The optional crypto/certification changes are not selected. Production
uses 0.22 while Testkit uses 0.21, sharing Types, Candid, management DTO and SDK
identities. Only Platform and required graph edges change; six unrelated Windows
edge rewrites from resolution are reverted to their incoming selections. Strict
library Clippy, 16 request/lifecycle codec and bounds cases, independent Core/Agent
consumers (`target/msrv-consumers.3G7EQ2/`) and the focused HTTP signed-retention,
single-dispatch/lost-reply case pass on Rust 1.99.1. No new simulator, macOS native,
live IC or full release qualification was claimed by that focused adoption.

Cleanup proof is `target/request-cleanup-058.m2dcuayv/`: pre-edit archive, untouched
index/Cargo hashes, retained first compile failure, strict both-package all-target/
all-feature Clippy, 112 affected unit cases, five public application cases, normal
Core/Agent consumers (`target/msrv-consumers.jiUQo7/`), warning-free Rustdocs and LOC.
Both focused HTTP application admission/retention checks pass. Earlier
runtime proof retains its exact source/dependency selection: 96 workflow cases,
retired-layout/artifact/restore checks and 22 selected simulator cases, including
cold standalone same-ID restore, process death, lost replies and terminal replay,
in `target/scope-cut-058.zh17kmds/`. Its failed simulator assertion and corrected
single-case retry remain retained. The request/error simplification changes no IC
effect, record bytes or protocol. These earlier focused receipts retain their
original input identities and scope; the final release gate freshly qualifies the
combined Linux source and dependency selection.

## Previous baseline and custody

Previous **0.57.1** is release commit `40d9e32` with annotated `v0.57.1`, source
`8edede0` and handoff `bba30fe`. Its original receipt remains in Git history;
publication/source evidence stays in `target/deploy-0571.ytcigfjv/` and
`.git/release-state/`. Preserve original/failed qualification, unfinished journals,
referenced/partial artifacts, obligations and tool generations.
