# Current handoff — 2026-10-23

## Ownership and source

Core owns bounded backup records, guarded persistence, reconciliation and workflow
coordination. Agent owns reserved signed transport. Platform owns external IC
identities/codecs/SDKs; Testkit owns simulator assets and managed processes.
Applications retain membership, permissions, consistency, fencing and external
obligations. Current contracts belong to [implementation contracts](../implementation-contracts.md)
and [the record/API boundary](../extraction-boundary.md).

Released source remains **0.40.0**, `70d4d49`; workspace/package versions remain
**0.40.0**. Compatible **0.40.1** notes cover the IC Tooling patch adoption.
No Rust source or public contract changes. No staging, commit, release, publication,
deployment, live IC effect or sibling source edit occurs here. Pending Shared
Tooling provenance and unrelated work are preserved.

## Published IC Tooling adoption

Host Artifacts/FS/Process and Testkit require published **0.66.2**; the contributor
Build release editor selects exactly **0.66.2**. Official registry metadata and
archive checksums identify source **`4bc810c84ba45baa191030eb625b8e793124581e`**.
All five packages' library/binary Rust source matches 0.66.0. Upstream changes
correct dependency documentation and Build executable examples, which Backup does
not consume. No adapter, protocol, decoder bound or cleanup change is needed.
The existing explicit PocketIC ownership transfer remains intact.

The incoming lock already selected these four library packages; it stays
byte-for-byte unchanged. Incoming Platform **0.6.11** is preserved and now passes
the focused native build/consumer checks below. Its authenticated archive has no
Rust-source difference from 0.6.10. Agent/Transport **0.49.3**, management DTO
**0.11.0**, external Principal **0.1.5** and PocketIC **16.1.0** remain selected.

Canonical setup waited 93 seconds for the admitted ic-timers owner of the same
Testkit slot, then rechecked inputs and reused its completed exact build. The
explicit wait budget was 300 seconds; no owner lock, process or failed generation
was removed. Both Testkit and Build pass offline admission; prior tools remain
retained. CLI tools keep explicit `cli`; product libraries do not enable it.

## Focused qualification

Linux Rust **1.99.1** passes strict all-target/all-feature Clippy, compilation and
warning-denied Rustdoc. Actual Rust **1.91.0** owning and independent normal-consumer
checks pass. Public Host checksum conversion/publication identities compile through
consumer-selected 0.66.2 types. Both unpacked package archives compile together
with original selections and verified archive checksums.

**9 public artifact/recovery/workflow tests pass**, including typed
checksum causes, non-UTF-8 paths, changed-byte publication/recovery refusal and
retained original spending/obligations. Normal graphs contain only Host 0.66.2
and Platform 0.6.11; they exclude Testkit/PocketIC/WAT/Wire/Clap and development-only
features. Core remains Agent-free. Manifest/lock fingerprints stay stable throughout
qualification. The refreshed editor renders independently checked manifest/lock
candidates for 0.40.1 in a disposable directory; real metadata stays at 0.40.0.
Read-only patch release preview agrees; no release operation runs.

Evidence: `/tmp/backup-tooling-0662-20261023-_3h2ofqx/`, including authenticated source comparisons,
setup and product logs, normal-consumer graphs, package archives, public test
fixtures and read-only editor checks. Preserved/validated lock SHA-256:
`310d3effeb82853c9850061f49baf90cd17e63a6275be2777903629748eec070`.

The [released-source handoff](https://github.com/dragginzgame/ic-backup/blob/70d4d49699fb0b14a6d2a293012a328321d2f3f0/docs/status/current.md)
retains the earlier 19 simulator/recovery cases on Host 0.66.0 and Platform 0.6.10.
Those are reused only as unchanged-implementation evidence, not new execution of
the 0.66.2 graph. The simulator suite, full local make ci, new hosted/native macOS,
registry-only installed-product, installed IC and Canic qualification are not run.

## Shared Tooling and application boundary

The pending **Shared Tooling 0.26.9** revision
**`3657c044c0e91321effe8835989491a4cd0b5cba`** and its reviewed provenance are
unchanged. All **122 payloads** still verify. See [the review](../shared-tooling-review.json)
and `/tmp/backup-shared-0269-20261023-wnhypkbf/` for the provenance-only refresh.
This IC Tooling adoption changes no shared payload, pin or build-policy helper.

[Canic adoption preparation](../canic-adoption.md) retains its source-specific
integration map and acceptance sequence. Production Root authority, whole-unit
fencing and external obligations remain application-owned. The live backup attempt
is deferred until Blob storage is complete; this patch adoption grants no live
effect authority. [Backup #29 — Complete original-plan execution and stage binding before Canic runner adoption](https://github.com/dragginzgame/ic-backup/issues/29)
retains complete application acceptance. Earlier registry-only 0.38 fixture
results keep their original source and graph scope.
