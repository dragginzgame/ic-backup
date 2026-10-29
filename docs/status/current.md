# Current handoff — 2026-10-29

## Shared and IC Tooling adoption

Delivered **0.49.1** is `920dbfb6a81d314406d947c98453dd54e7d2c265`.
[#74](https://github.com/dragginzgame/ic-backup/issues/74) and
[#75](https://github.com/dragginzgame/ic-backup/issues/75) are closed after delivery;
[CI run 40877518055](https://github.com/dragginzgame/ic-backup/actions/runs/40877518055)
passed the complete native matrix for that exact commit. That evidence belongs to
the delivered graph, not this dependency update.

This batch adopts published **IC Tooling 0.70.0** for Host Artifacts/FS/Process,
Testkit and the release metadata editor, preserving the preceding **IC Platform
0.8.1** adoption. Testkit now selects Platform 0.8.1 and explicit external Principal
conversions; development-only Platform 0.7.6 leaves the lock. No unrelated package
version changes. Published archives match crates.io checksums and source revision
`ecc8d61d19f0b711788f7e10e5dd728d0db6ff60`; exact registry metadata is retained at
`/tmp/backup-tooling070-registry.json`.

Host and Testkit Rust source is unchanged from the incoming 0.69.14 packages.
Host's public artifact/filesystem error identities nevertheless move to the 0.70
line; consumers exchanging them must align direct dependencies. All pending notes
therefore move to **0.50.0**. Workspace/package versions remain **0.49.1**; changes
are unstaged/uncommitted. No Rust source, persisted schema, wire/artifact bytes or
recovery authority changes. No release, publication, issue mutation, sibling edit
or live IC operation was performed.

Shared **0.27.7**, revision `11d519c604b1727a2bf6272557a97e770a63b29f`, is adopted
through its canonical exporter from an isolated clean checkout of independently
observed GitHub main. All **126 payloads** match that exact source and executable
modes; only four governance/audit files change. Selected installers, catalogs,
formatting and validation/release scripts are unchanged. The new observability
contract names `ic-metrics`/`ic-logging` as shared owners. Backup has no competing
engine or instrumentation to migrate, so no unused dependency is added. Producer
stack/dashboard changes are outside the selection. [The review](../shared-tooling-review.json)
and [consumer guide](../shared-tooling.md) retain scope and provenance.

Management DTO **0.11.0**, external Principal **0.1.5**, Agent SDK **0.49.3**,
Candid **0.10.39**, PocketIC **16.1.0** and Rust floor **1.91.0** remain unchanged.
Core/Agent retain their existing Platform feature selections; no ICRC, profiler or
owned self-authentication helper is selected. [Platform ownership](../ic-platform.md)
and [implementation contracts](../implementation-contracts.md) remain mandatory.
[Platform #50](https://github.com/dragginzgame/ic-platform/issues/50) retains
separate supplier crypto/TLS qualification.

## Focused qualification

Strict Core/Agent Clippy passes for all targets/features. All **21 selected
artifact/JSON persistence unit tests** and **six public checksum/artifact/recovery
tests** pass. Both independent normal consumers pass on actual Rust **1.91.0**,
including public Host error interoperability. Their graph excludes Testkit,
PocketIC and WAT; Core also excludes Agent. Evidence is
`target/msrv-consumers.hYmod9` and `/tmp/backup-tooling070-msrv.log`.

The independent normal-consumer PocketIC capture/transfer/manifest/upload/same-ID
load/start/terminal-replay case passes with Testkit 0.70.0. Production graph/build
evidence is `/tmp/ic-backup-agent-independent-build-536636-0`; authenticated fixture
`/tmp/ic-backup-agent-pocketic-536636-1` retains original plans, journals, exact
bytes, manifests, server/process output and replay evidence. This is candidate-path
Linux qualification, not new registry-only, macOS, installed Canic or live
application acceptance.

Common tool setup succeeds. The initial selected Testkit setup times out safely
behind an admitted installer owned by another repository. Supported bounded
waiting then admits the completed 0.70.0 selection; the release editor is prepared
through cache reuse. Offline **tools-check** passes. PocketIC assets remain 16.1.0;
old selections and failed logs are retained. All **12 release metadata preparation
and refusal cases** pass using the admitted 0.70.0 editor, with originals/traces in
`target/shared-release-tests.JfUFrx`.

Logs are `/tmp/backup-tooling070-*`; source-consistency results are retained in
`/tmp/backup-tooling070-source-checks.log`. The graph stayed unchanged through
validation. `/tmp/backup-tooling070-graph.sha256` binds manifest
`07e26a5b8d321b4a998f4ff3e027bc9729a405740abdf3f047e0ffbb549897a6`
and lock `68d8e6af070b6da4d2730ce6f7057d645067bfbe059c1df032ecf4c142ddd174`.
No full local gate or new complete native matrix was run.

## Earlier integration and retained evidence

The preceding Platform 0.8.1 batch passed 81 IC model/application-ingress tests,
16 HTTP tests, strict Clippy, independent MSRV consumers and a full PocketIC
round trip on its then-current Host/Testkit 0.69.14 graph. Preserve
`/tmp/backup-platform081-*`, `/tmp/ic-backup-agent-independent-build-3940488-0`,
`/tmp/ic-backup-agent-pocketic-3940488-1` and `target/msrv-consumers.g00fEa`.
The Platform archive SHA-256 remains
`b28e5352b4a6fa35217dd5df181a5555197936477570f034a53ffd7dda221c65`, with published
source `10c2610511d6a5525816185c999b25fd159c1d89`.

Delivered operator composition uses existing plan/journal/coordinator owners;
no second journal, scheduler or authority provider exists. The
[operator map](../canic-adoption.md#reusable-operator-composition) and
[Canic removal map](../canic-adoption.md#canic-engine-removal-map) retain integration
ownership. Earlier read-only Canic inspection used
`b9ed7f298bdaf8c9cceac142edf7e56b15d33208`, fingerprinted in
`/tmp/backup75-canic-source.json`. Fleet/Root policy, Root-paid effects, application
hooks and terminal acceptance remain Canic ports; multi-unit/fencing and
external-obligation acceptance remains under Canic #513/#550/#394.

Preserve `/tmp/backup75-*`, original build
`/tmp/ic-backup-agent-independent-build-2456269-0`, fixtures
`/tmp/ic-backup-agent-pocketic-2456269-*`, corrected death-case build
`/tmp/ic-backup-agent-independent-build-2635797-0` and fixture
`/tmp/ic-backup-agent-pocketic-2635797-1`. Preserve `/tmp/backup74-*`,
`/tmp/backup-049-guidance-*`, `/tmp/backup-simplify-*`, earlier independent/PocketIC
fixtures, Canic/release/consumer journals, `/tmp/backup-reference-release-*`,
`/tmp/backup-tooling696-*`, `/tmp/backup-platform07*`, `/tmp/backup-72*`,
`/tmp/backup-shared27*` and the 20261027/20261028 failure logs in
`target/validation-failures/`. Successful tests and closed issues authorize no cleanup.
