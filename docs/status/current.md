# Current handoff — 2026-11-05

## Delivered 0.58.2

Both crates **0.58.2** are published on crates.io. Release
`7ab98664c8de98faf5fc36f803ca5581b26be707` and annotated `v0.58.2` are on
`origin/main`; source is `49512f06f40e982227a673d0a44863067a0969c6`.
The user authorized Testkit adoption, Shared Tooling adoption/audit and deployment.
Contributor tooling/fixture changes preserve the published library API,
record/wire bytes and production behavior.

Shared Tooling **0.30.0 at `07dce12cbe4a798fd4a6d263095518678c4f6b45`** is
exported canonically: 155 payloads, 15 updates, six additions and 134 unchanged.
Linked reference docs/catalog/inactive unit templates complete the documentation
closure; no service or schedule is activated. Source/mode provenance and per-rule
review belong to [the adoption review](../shared-tooling-review.json).

All eight authored Rust `allow` attributes are removed. Core's shared fixtures use
checked dead-code expectations at their integration-target boundaries; Agent's
obsolete allowance/helper is retired. Unfulfilled expectations remain denied.
Agent HTTP/simulator qualification now lives under `examples/standalone/tests/`.
The example explicitly enables test discovery; all 41 Agent cases match the prior
registered cases after module-prefix normalization. Its independent consumer uses
explicit Cargo library/bin targets at maintained sources, without embedded Rust,
source copy lists or `include!`/`#[path]` shims. Fixture manifests declare test roles.
Default fixture scratch is `target/tmp`; explicit `TMPDIR` overrides and retained
artifacts remain untouched. Required staging scans fail on entry-read errors.

Testkit **0.86.0**, exact CLI and authenticated PocketIC **16.1.0** are selected.
All 56 Testkit Rust files match 0.85.1; only Testkit's lock identity changes in the
incoming adoption. Its optional Wire feature stays disabled. Original Testkit
proof is in `target/testkit-086-adoption._kp_7j1u/`, including two real simulator
cases and both simulator-target Clippy checks. This batch preserves that selection.

## Latest qualification and command ownership

The complete Linux `release-verify` gate passes: strict Clippy, **554 tests**,
simulator/independent consumer workflows, packaging and all tooling checks. The
[receipt](../release.json) binds original source and exact release inputs. Standard
delivery committed, tagged and atomically pushed; `make publish` then verified and
uploaded both crates. Independent registry checks confirm non-yanked versions,
archive checksums, clean release VCS identities and **341 source files** against
the release commit. Remote branch/tag identities match. Evidence is retained in
`target/deploy-0582.ft4gpnz7/` and `.git/release-state/`.

Audit evidence is `target/shared-030-audit.rwc459mo/`: incoming dirty inputs,
canonical export/source proof, all-target/all-feature strict Clippy for both
crates, 17 JSON persistence cases and all 37 relocated HTTP/simulator cases pass.
The latter include independent cold capture/restore, process death, lost-effect
recovery and effect-free terminal replay. Initial compile failures are retained.
Release metadata-only fixtures and both exact unpacked archives compile/verify;
latest archive evidence is `target/package-check.hzmiDc/`. Normal consumers exclude simulator
packages. Dependency, formatting, shell/Perl, local-link and LOC checks pass.

`make cloc` uses schema 2: Library/app, Tests, Examples, Benchmarks, Build/tooling,
Unclassified and total. It reports no unclassified Rust or legacy singular test
owner. Two intentional test-macro notes are retained. These categories do not
measure code removal; prior runtime/test totals use a different classification.
Agent transport checks now use `cargo test -p ic-backup-agent --example standalone`
with `tests::http_boundary::` or `tests::pocketic_transport::` filters when needed.
The ordinary Make test gate includes the enabled example without another runner.

Rust minimum and exact compiler stay **1.99.1**. Host Artifacts/FS/Process **0.74.3**,
Build release editor **0.74.2**, Platform **0.22.1**, Types **0.18.1** and target-only
Runtime **0.19.1** are selected. The incoming Platform patch arrived after the audit;
its checksum/source `fcf540b` and 63 unchanged consumed-source files are reviewed
in `target/deploy-0582.ft4gpnz7/`. Only unselected crypto modules change. No
executable catalog or compiler update is selected. Linux proof does not establish
native macOS, live IC or installed Canic acceptance. Hosted CI was not dispatched;
no release-stage reuse is enabled.

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

## Earlier evidence

The 0.58.1 release `cad45b3c327dfb3d3af0129ed23e3ed07e0b31c7`, original receipt
and registry proof remain in history and `target/deploy-0581.qy4t0oqc/`.
Earlier release/recovery proof stays at its original owner and Git revision;
no retained evidence is removed.
