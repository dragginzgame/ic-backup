# Current handoff — 2026-10-30

## Platform 0.9.1 adoption

Baseline HEAD is `4a0c5522c88b1a294042e44e13584dbfa925e9ec` (Release 0.51.0).
The user requested the latest published Platform. The batch is complete with
focused checks and draft **0.51.1** notes. Workspace/package versions remain
**0.51.0**. Changes are unstaged/uncommitted; no release, publication, live effect,
sibling mutation or artifact cleanup was performed.

The manifest now requires Platform **0.9.1**. The incoming dirty Cargo.lock already
selected it and is preserved byte-for-byte at `/tmp/backup-platform091-incoming.lock`.
Unrelated dependency selections remain unchanged. Host/Testkit/release editor
**0.70.5**, development-only Platform **0.8.3**, Candid **0.10.39**, Agent **0.49.3**,
external Principal **0.1.5**, native management DTO **0.11.0**, PocketIC **16.1.0**
and Rust floor **1.91.0** retain their preceding selections.

The Cargo sparse index confirms 0.9.1 as latest published stable. Its archive
matches SHA-256
`4727a6baf3600d8784cb4b2a5f7970d25173f7c1f5f363006bba89fdddb0ded6`;
registry source identifies `8e21a5ea96eb6b2cabc92be8cffc7df3ac9745aa`.
Evidence is `/tmp/backup-platform091-index.jsonl` and `-registry-proof.log`.
Both published source directories and committed facade Rust source compare
unchanged against 0.9.0; `-published-source.diff` and `-source.diff` are empty.

## Compatibility and maintained behavior

Platform 0.9.1 moves its executor singleton to an optional runtime guard package
selected only by executor/runtime features. Backup selects neither feature.
A standalone consumer containing both Backup libraries and registry executor
2.0.0 now resolves successfully, with only Platform 0.9.1 and no Platform runtime
guard or Testkit/PocketIC/WAT. This is Cargo composition evidence, not execution
of an installed runtime. Its manifest, lock and metadata are retained under
`/tmp/backup-platform091-composition/`.

The current [Platform boundary](../ic-platform.md) describes the relaxed host
composition; applications enabling Platform's runtime still need one executor
owner. No product Rust source, public signature, native IC type, record/wire byte,
finite spending, single-submit, same-ID recovery or application-authority
contract changed. This patch adopts no experimental Candid codec.

Shared **0.27.11**, revision
`62edfbd43087a718bf0283e9040676a49211976d`, retains its reviewed 131 payloads.
Its [adoption review](../shared-tooling-review.json) preserves the original
0.9.0 qualification scope; that historical package-wide restriction is superseded
for current host consumers by the 0.9.1 feature selection above.

## Focused qualification

Strict Clippy passes for both packages, all targets/features, with warnings denied.
Actual Rust **1.91.0** independent Core and Agent consumers pass, retaining exact
normal-graph selection, simulator exclusions and public Host error probes.
Core remains Agent-free. Evidence: `target/msrv-consumers.zCg7j7`.

All **seven selected public request/snapshot codec cases** pass, covering exact
wire bytes, limits, original spending and reopen behavior. Formatting, dependency
inheritance/boundaries, local documentation links and diff checks pass.
The frozen manifest and lock hashes are in `/tmp/backup-platform091-graph.sha256`;
the incoming lock comparison also passes.

Logs use `/tmp/backup-platform091-*`: `-clippy.log`, `-consumers.log`,
`-codecs.log`, `-source-checks.log` and `-doc-links.log`. No new full gate,
PocketIC suite, native macOS matrix, registry-only release acceptance, installed
Canic migration or live application qualification was run. The unchanged facade
Rust source informed that focused selection; prior runtime passes retain their
original source/graph identities rather than becoming new 0.9.1 executions.

## Retained obligations and evidence

[Implementation contracts](../implementation-contracts.md) and the
[operator map](../canic-adoption.md#reusable-operator-composition) remain canonical.
Application authority, external obligations, pending effects, original evidence
custody and guarded terminal release remain integration-owned.

The [previous committed handoff](https://github.com/dragginzgame/ic-backup/blob/4a0c5522c88b1a294042e44e13584dbfa925e9ec/docs/status/current.md)
retains the complete Host/Shared/Platform 0.9.0 adoption, its 187 native/nine
authenticated simulator cases and earlier qualification history.
Preserve all `/tmp/backup-*`, `/tmp/ic-backup-agent-independent-build-*`,
`/tmp/ic-backup-agent-pocketic-*`, `/tmp/ic-backup-pocketic-*`, original Canic
journals, `target/msrv-consumers.*`, `target/shared-release-tests.*`, validation
failure logs and old/failed tool bundles. Historical passes keep their original
source/graph/host scope; this summary authorizes no disposal.
