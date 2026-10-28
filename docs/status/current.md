# Current handoff — 2026-10-28

## Current dependency batch

Backup requires published IC Platform **0.7.2**, source
`95b7caf960ec3af045923d85bad4801e9e463782`. The published crate's VCS identity
matches that committed release. The workspace minimum is updated; the lockfile
already selected 0.7.2 when this request began. Preserve its accompanying user
updates to Host Artifacts/FS/Process and Testkit **0.69.5**. Their declaration
minimums and the tooling-only release editor remain **0.69.4**.

Own package versions remain **0.48.0**; the compatible adoption note is open under
[0.48.1](../../CHANGELOG.md#0481). No staging, commit, version bump, release or
publication ran. Previous local Canic adoption guidance remains in the working
batch alongside these dependency notes.

Selected management DTOs **0.11.0**, external Principal **0.1.5**, Agent SDK
**0.49.3**, Candid **0.10.39** and Rust floor **1.91** remain unchanged. Review of
Platform's consumed Principal, management and Agent paths found no required API
migration; the Candid change documents an existing Wasm nesting limitation.
Backup remains host-side. No optional call, gzip, profiler or signature feature
was enabled. Original wire/hash bytes, schemas and spending owners are unchanged.
Lock SHA-256: `7868e6d070ea5d7b2cfcf43beaed873ee2415f50706888a194d7489d718ab6b8`.

## Focused adoption evidence

All 27 selected native request/lifecycle/metadata wire and inventory identity tests
pass, including independent goldens, Principal aliases, raw bounds, malformed
replies and exact hash commitments. Both independent candidate consumer journeys
pass on the selected published dependency graph: cold capture/transfer/upload,
same-ID restore and terminal replay; process death after authenticated load with
original pending spending and no redispatch. Actual wrong-trusted-root rejection
also passes without settling the original stop.

Strict Core/Agent all-target/all-feature Clippy passes. Dependency inheritance,
external IC boundaries, formatting and offline locked Testkit CLI/server admission
pass. Explicit setup prepared Testkit 0.69.5 with PocketIC **16.1.0**; old bundles
and evidence remain retained. No full local repository suite, new native host
matrix or registry-only Backup consumer qualification ran.

Logs: `/tmp/backup-platform072-*`. Independent graph/build evidence is
`/tmp/ic-backup-agent-independent-build-2391369-0`; application/journal evidence is
`/tmp/ic-backup-agent-pocketic-2391369-1` and
`/tmp/ic-backup-agent-pocketic-2391369-3`. Keep original journals and raw bytes.
The bounded adoption is complete; delivery and any selected release gate remain
user-owned.

## Previous delivery and Canic integration

Released Backup **0.48.0**, `df39f44aa8aa15eee3c79eb65a1496bbfcd52bf5`, contains
[#72](https://github.com/dragginzgame/ic-backup/issues/72) and
[#73](https://github.com/dragginzgame/ic-backup/issues/73). Their owning issues
retain acceptance status. [Delivered-source CI 40670367677](https://github.com/dragginzgame/ic-backup/actions/runs/40670367677)
was pending at the earlier issue review; this adoption does not re-poll or replace
that exact-source matrix. The [released handoff](https://github.com/dragginzgame/ic-backup/blob/df39f44aa8aa15eee3c79eb65a1496bbfcd52bf5/docs/status/current.md)
retains previous extraction/tooling evidence under `/tmp/backup-72*` and
`/tmp/backup-shared273-*`.

[Canic PR #702](https://github.com/dragginzgame/canic/pull/702), head
`78a57d5b1e83a357999bfd4afbe88a06b0809c64`, contributes complete Root checkpoint
preparation through Backup's original workflow/stage/attempt owner. Its isolated
checkout is `/tmp/canic-backup-adoption.lzm8Ut`, based on Canic 0.112.0,
`5c236ab130c0d63892ef281b45bdc3fa353e012f`. Prior evidence comprises 27 native
cases, two installed Root cases and strict lint on that contribution's own graph.
It is separate from this Platform adoption. The normal Canic checkout is untouched.
Its original/failed journals and raw evidence remain under
`target/original-root-checkpoint-*` in the isolated checkout and
`/tmp/canic-checkpoint-*` logs.

[Backup #29](https://github.com/dragginzgame/ic-backup/issues/29),
[Canic #513](https://github.com/dragginzgame/canic/issues/513) and
[#550](https://github.com/dragginzgame/canic/issues/550) retain production Root-paid
snapshot reconciliation, complete transfer/restore/start, external obligations,
application acceptance and terminal/source-reference composition. The
[integration map](../canic-adoption.md) retains the seams. Fixture management
calls do not grant production authority; `LiveCreateUnavailable` remains in place.

Shared **0.27.3**, `cc81a58a5b2dd287c577cbffd2c576c178b6785c`, and its
[source review](../shared-tooling-review.json) remain unchanged. Unresolved crypto
qualification stays with [Platform #50](https://github.com/dragginzgame/ic-platform/issues/50);
superseded Backup #68 supplies no correction or exemption. Preserve original
plans, pending/Applied spending, artifacts, references, obligations and failed
evidence, including `target/validation-failures/20261027T172540Z-3411640-8-clippy.log`.
