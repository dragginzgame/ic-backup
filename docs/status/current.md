# Current handoff — 2026-10-27

## Released source and ownership

Released Backup is **0.45.0**, `b11928151fb68a1b40115c0c3515b9198b9dd73b`.
The [released handoff](https://github.com/dragginzgame/ic-backup/blob/b11928151fb68a1b40115c0c3515b9198b9dd73b/docs/status/current.md)
retains earlier simplification, Tooling/Platform adoption and qualification evidence.
Core owns bounded records, guarded persistence and original-plan coordination;
Agent owns reserved signed transport. Applications retain authority, membership,
release binding, fencing and external obligations.
[Implementation contracts](../implementation-contracts.md) remain authoritative.

## Pending 0.45.1 shared formatting and release-fixture fix

Shared Tooling **0.26.23** is adopted from committed source
`48ec9794892da6f8e2812653ec16c5e8ea23df47` through the canonical impact review and
exporter. The 125-file snapshot updates seven payloads and adds the standard Rust
formatting driver. [The review](../shared-tooling-review.json) records provenance.
Backup keeps grouped root manifest sorting followed by Rustfmt, with no new
workspace or derive-sorting policy. The driver reports timings and reuses only
identical successful inputs; prepared tools remain checked and staged-hook
worktrees always run their formatters. Local fixtures carry the new helper and
qualify its progress, failure status, logs and custody. Gzip ownership guidance
needs no code change: Backup has no local application/golden-generator encoder.

[#70 — Diagnose the macOS older-release-patch adapter fixture failure](https://github.com/dragginzgame/ic-backup/issues/70)
is fixed locally. The older-release recovery fixture compared `wc -l` output to
string `2`. macOS pads that count; the exact numeric two-attempt ledger was rejected.
The assertion now compares numerically. No release runner, adapter, spending,
receipt, source-selection or recovery semantics change.

Diagnosis uses [CI run 40493923484](https://github.com/dragginzgame/ic-backup/actions/runs/40493923484),
at exact released source `b11928151fb68a1b40115c0c3515b9198b9dd73b`.
Both macOS jobs fail `older-release-patch`; Linux passes. Retained ARM64 artifact
`12729843978` has verified SHA-256
`0b3c998db11ff3b297d625aa701f8d41eeeb30dfc759509603c57b86ceeed321`.
Its fixture has exactly two push records, the expected 0.1.2/source/tag/complete
plan, and unchanged old plan and validation proof. A local padded-count regression
reproduces the failing string assertion after those preceding checks pass. The
optional Git-upstream warning is not the cause of this assertion failure.

The regression delegates counting to the real wc and pads only its presentation.
It exercises the same fresh-patch recovery and exact identities/evidence checks.
A single older-release case roster feeds both the normal consumer gate and
`--receipts-only`: explicit resume, unsorted selected-source resume, fresh patch,
fresh minor, failed fresh validation and padded counts. The focused selection now
covers the fresh branch that failed on macOS; no new command selector is added.
The fixture ownership map and release guide describe the maintained scope.

## Focused evidence

Under 0.26.23, actual root formatting passes with `--no-cache`; a normal invocation
records stable inputs and the next reports unchanged input reuse. All five named
hook cases pass, including partial staging, index refresh, unrelated edits,
dependency grouping, formatter exit 17 and checkout-local tool refusal. Evidence
remains at `target/hook-tests.wfbY23/` and `/tmp/backup-shared23-*`.
Testkit Make routing/refusal, offline tool admission and all five release metadata
cases pass; metadata evidence is at `target/shared-release-tests.G5zECg/`.

The following #70 receipts predate the snapshot refresh and retain that scope.
Before the fix, the padded-count regression fails at the exact string comparison
with two real push records. Trace and retained fixture are
`/tmp/backup-issue70-red.log` and `target/shared-release-tests.YI5o3w/`.
After the fix, all **13** selected consumer receipt/recovery cases pass, retaining
`/tmp/backup-issue70-green.log` and `target/shared-release-tests.nqoWsv/`.
Original plans, proof, old tag and source identity checks remain active.

Bash syntax, focused ShellCheck, non-mutating formatting, Shared snapshot
integrity, fixture ownership JSON admission and local Markdown links pass.
Check logs remain under `/tmp/backup-issue70-*`.
The native macOS ZIP, nested evidence archive and selected raw fixture files are
retained there too; no original CI or failed-regression evidence was deleted.

The selected Platform **0.7.0**, Host/Testkit/editor **0.69.0**, Rust **1.91** floor
and feature choices are unchanged. The incoming #70 release fixture is also
byte-identical after the snapshot refresh. Cargo.lock
remains byte-identical to HEAD, SHA-256
`8d380c16b76d2cd50250eb5e5aafefdc5c3def40c13a100b980eb9e9018c7771`.
No Rust compilation or simulator run was needed for these tooling-only changes;
prior product evidence retains its original source/graph scope.

## Limits and delivery

The padded-count regression runs on Linux; it is not a new native macOS pass.
The latest hosted macOS results still belong to the failing released source.
Deliver the patch and obtain current native CI acceptance before closing #70;
#71's delivered implementation also still awaits accepted hosted qualification.
No full local CI, complete release-adapter matrix, installed IC, publication or
application acceptance is claimed. Prior Agent crypto-bigint 0.5.5 qualification
remains unresolved. Notes select **0.45.1**; package versions remain **0.45.0**.
Changes are unstaged and uncommitted. No own package version, release, tag, push,
publication, deployment, live IC effect or sibling edit is performed.
