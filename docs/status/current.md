# Current handoff — 2026-10-28

## Accepted batch and delivery

Continue the requested #72/#73 fixes and adoption of new Shared/IC Tooling.
Released source remains **0.47.1**, `ca70617a3df142848700f980e459cc36e82da097`.
Its [successful native CI](https://github.com/dragginzgame/ic-backup/actions/runs/40656641186)
keeps its original released-source scope; it does not qualify this working batch.
Changes are local and uncommitted. Staging, commits, own versions and release
execution remain user-owned. No publication, deployment, live IC or cleanup ran.

The [pending changelog](../../CHANGELOG.md#0480) selects **0.48.0** because
`restore_snapshot` replaces its separate source-layout/plan arguments with an
explicit `IcRestoreSource`. Update consumers; own package versions remain 0.47.1.
Existing record schemas, request digests, original journals and spending remain
unchanged. No compatibility alias, migration or replacement authority is added.

## Restore and upload changes

[#72](https://github.com/dragginzgame/ic-backup/issues/72): existing snapshots bind
an independently qualified metadata request. Uploaded sources bind their exact
source-derived data plan, allocation ancestry, complete Applied histories and
retained checkpoint. Reopen/re-admit that provenance before spending and across
admission/settlement. Same-workflow loads require the exact upload predecessor.
Separate restore workflows retain explicit provenance; all-empty uploads require
the exact Applied allocation receipt without creating a placeholder data stage.

Negative native cases cover an independently valid load plan for another snapshot,
wrong canister, changed data plan, pending upload and substituted empty-allocation
receipt, with no new load spending. Core and independent Agent callers now use
uploaded provenance for actual upload-backed loads. Admission still independently
qualifies complete backend bytes, current permissions and application safety.

[#73](https://github.com/dragginzgame/ic-backup/issues/73): complete data uploads
use a guarded session. Full artifact checks bracket the batch; each bounded payload
must match its immutable original request digest. Closed-tree, Host file identity,
length and native change observations detect drift around reads. Those observations
are not content authority or a fence against noncooperating writers. Closing full
verification precedes checkpointing. Standalone preparation retains full per-call
verification; fresh-stage-only execution still refuses attempted partial reentry.

Deterministic reader accounting proves **3S + D** source bytes for complete upload
execution across three chunk sizes: S is total artifact bytes and D payload bytes.
This removes per-chunk full-content hashing, not every per-operation metadata or
journal check. In-place changes and identical-byte file replacement after the first
write stop successors while preserving original Applied/pending histories.

## Shared and dependency adoption

Shared **0.27.3**, `cc81a58a5b2dd287c577cbffd2c576c178b6785c`, is exported through
the canonical clean source checkout. Its trusted verifier compares all 126 payloads
and modes with that independently selected commit. [The review](../shared-tooling-review.json)
records scope and evidence. No vendored payload is patched locally. Optional
sccache and unattended schedules are not enabled.

Published IC Tooling **0.69.4**, `fa3786c86a00e5e9f98a0c729d86bf45a1bd1a82`,
selects Host Artifacts/FS/Process, Testkit and the tooling-only release editor.
Platform stays selected at **0.7.1**, qualified management DTOs at **0.11.0**,
external Principal at **0.1.5** and consumer Rust floor at **1.91**.
Working lock SHA-256:
`f16c46a2ed3c73f80c0011e8436730f9ad3bdf78c92d84a24fe1c92e239d4f53`.
Explicit `make install-tools` and offline `make tools-check` pass, preserving old
bundles and failures. The first release-fixture attempts correctly refused the
editor before setup completed; prepared reruns pass with separate retained logs.

## Focused evidence and limits

On the final 0.69.4 graph, 13 upload/source native cases and four restore workflow
cases pass. Strict Core test-target and affected Agent library/PocketIC-target
Clippy pass. Six connected real PocketIC cases pass: round trip, cold resume and
lost allocation/data/load/start containment. Both independent normal-consumer
cases pass, including process death after authenticated load before receipt.
These use the local candidate libraries; no new registry-only qualification ran.
Logs: `/tmp/backup-72*-tooling694-*.log`. Earlier 0.69.3 evidence remains separate.

Shared source proof, forced formatting, hooks, Testkit routing, dependency
inheritance/boundaries and documentation links pass. All 12 selected release
metadata cases and 13 receipt/recovery cases pass; fixture evidence remains at
`target/shared-release-tests.OwYAmF/` and `target/shared-release-tests.dOoUKx/`.
Tooling logs: `/tmp/backup-shared273-*`. Initial compile/lint diagnostics and
unprepared-editor refusals remain retained, not counted as passing evidence.

No full local repository gate, new native macOS/MSRV run or delivered-source CI
is claimed. #72/#73 remain open until delivery and applicable acceptance review.
[#29](https://github.com/dragginzgame/ic-backup/issues/29) retains full production
application/Canic, whole-unit/external-obligation and terminal acceptance. Canic
0.112.0 source still refuses live creation; no sibling was edited or tested here.

Unresolved crypto qualification stays with
[Platform #50](https://github.com/dragginzgame/ic-platform/issues/50); superseded
Backup #68 is not a fix or exemption. Preserve all original plans, journals,
bytes, references, obligations and failed evidence, including the earlier
`target/validation-failures/20261027T172540Z-3411640-8-clippy.log`.
