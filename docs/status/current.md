# Current handoff — 2026-11-02

## Routing, fault and artifact cleanup, pending 0.55.2

Completed the three accepted follow-ups over the earlier four fixture cleanups.
This checkout still has both batches as local changes above release commit
`2b26bbe169237f5a4c82f3fbdfcebc8b23a6a3ed` (**0.55.1**). Own versions remain
0.55.1; pending notes remain **0.55.2** because public APIs, persisted schemas,
request/reply formats and accepted filesystem bounds are unchanged.

- Independent consumer modes map exhaustively to a derived execution action.
  Original Clap names, fault-specific modes and evidence filenames remain.
  Create/continue, inspection, restore, start and replay consume that action;
  lost-load still refuses an unexpected paused result. Application creation,
  observation and retained replay use the same carried classification.
- Upload/restore failure modes derive one phase, optional operation and lost or
  malformed reply outcome. Providers and pending-operation expectations share
  that description. Actual management ingress and discarded-oracle retention
  still precede fault injection; allowance refusal remains separate and no
  oracle supplies settlement or permission to repeat a call.
- Artifact-member opening derives exact length and maximum together, replacing
  two private parallel matches. Fixed files/regions retain exact lengths;
  metadata/chunks retain their original ceilings and non-exact lengths. The same
  no-follow regular-file owner performs every read and custody check.

Earlier work remains intact: read-failure position/outcome ownership, fresh Agent
source preparation, shared load/start payload construction and six direct
round-trip scenario expectations. All separate test registrations, original byte
checks, journals, recovery and effect-free replay assertions remain.

The incoming lockfile is preserved byte-for-byte. It selects Testkit **0.80.1**,
target-specific Runtime **0.18.1**, Platform/client **0.18.0**, Types **0.17.0**,
Host **0.74.1** and no Metrics package. Authenticated **PocketIC 16.1.0** and
pinned/minimum **Rust 1.99.1** remain. No dependency resolution, requirement or
compiler change belongs to this work. Shared Tooling stays at **0.27.27**, commit
`651f5f7acb2e3055abd51ba5f835a2112801666a`; snapshot payloads are unchanged.

## Focused qualification

Latest evidence is `/tmp/backup-three-simplifications.i_p7q3h0/`, including all
incoming files/diff, exact compiler/server admission, frozen inputs, case
selection, comparison and native/simulator logs.

- Strict Clippy passes Core's library, `pocketic_snapshot` and public
  `ic_snapshot_artifact` targets, plus Agent's `pocketic_transport` target
  (`core-clippy.log`, `agent-clippy.log`).
- All 31 selected artifact unit regressions and the public artifact case pass,
  including corruption, unsafe/missing/extra members, bounded reads, source
  preparation, custody drift and publication recovery (`artifact-unit.log`,
  `artifact-public.log`).
- All eight registered upload cases, four load/start lost/malformed cases and
  six connected round trips pass (`planned-upload.log`, `planned-restore.log`,
  `connected.log`). Selection derives from unchanged case registrations in
  `selected-core-cases.json`.
- All ten independent Agent consumer checks pass: command parsing, exact claims,
  opaque application capture/recovery/retention/replay, normal capture/restore,
  load process death and operator retention/partial-output failures
  (`agent-independent.log`). The separately copied normal consumer's original
  graph/build evidence is `/tmp/ic-backup-agent-independent-build-2314310-0`.
- Frozen source/lock inputs remain unchanged across qualification
  (`qualified-input-comparison.json`). Handoff edits afterward receive focused
  formatting, links, snapshot and repository/license checks (`final-checks.log`).

These are fresh focused Linux checks, not a new full CI/release gate, native
macOS, registry-consumer run, installed Canic or live IC qualification. No
staging, commit, own-version bump, release, publication, sibling edits or cleanup
occurred. Preserve partial/published artifacts, unfinished journals and references,
tool generations and original evidence.

## Requested release preparation

The user explicitly selected **0.55.2** Git/crates.io delivery for both crates.
Its single-group patch plan agrees with these notes; original remote main is
`2b26bbe169237f5a4c82f3fbdfcebc8b23a6a3ed`. Release preparation reconciles
current dependency notes without changing the incoming lockfile. Authenticated
Testkit 0.80.1 source differs only in `src/benchmark/analysis.rs`; its manifest
removes Metrics and every PocketIC source file matches 0.80.0. Proof and original
inputs are retained at `/tmp/backup-deploy-0552.0OuUpHJW/`. The complete release
gate and final exact Git/registry receipt remain separate evidence authorities.

## Earlier evidence and delivery

The preceding four-refactor evidence remains at
`/tmp/backup-four-fixture-simplifications.k78vvo9k/`; its incoming handoff is
preserved in this batch's `before/`. Current shared consumers are freshly checked
above; unchanged download-specific selections retain their earlier evidence.

The completed **0.55.1** Git/crates.io delivery is separate from this pending
batch. [Its receipt](../release.json) and
`/tmp/backup-deploy-0551.wZk0k93G/` own exact delivery/registry verification.
Earlier Testkit 0.80.0 adoption evidence remains at
`/tmp/backup-testkit-refresh.jXzVq3jM/` and does not qualify newer fixture source.

[Implementation contracts](../implementation-contracts.md),
[the Platform boundary](../ic-platform.md) and
[the operator map](../canic-adoption.md#reusable-operator-composition) remain canonical.
