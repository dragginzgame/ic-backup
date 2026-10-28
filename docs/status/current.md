# Current handoff — 2026-10-28

## Current dependency batch

Backup now requires published IC Tooling **0.69.6** for Host Artifacts/FS/Process
and Testkit; the tooling-only release editor also selects **0.69.6**. Published
library VCS identities match `cba072ee8fe288eb92bb4a4ddf9753b8abc4b580`.
The incoming lockfile already selected those library versions; their minimums and
`RELEASE_EDITOR_VERSION` now match. Platform **0.7.3**, source
`6733f504b62b81456001a1735f2a25b3bbffe3b9`, remains selected from the preceding
adoption. No sibling checkout or snapshot-owned tooling was edited.

The base remains `7036715c6e7ab6438c912a53fe834fb297cd74fb`, with own versions
**0.48.1**. Both compatible dependency adoptions share the open
[0.48.2 notes](../../CHANGELOG.md#0482). No staging, commit, own version bump,
release or publication ran. Management DTOs **0.11.0**, external Principal
**0.1.5**, Agent SDK **0.49.3**, Candid **0.10.39** and Rust floor **1.91** remain
unchanged. Review found no Backup API migration or new feature requirement.
Original wire/hash bytes, schemas, host transport and spending owners remain intact.
Lock SHA-256: `f74e6ad555ff6fc2d3e04ad3ee8ad87a7d40a4395b769ec1f02f1940a2e33bd7`.

## Focused adoption evidence

All 33 selected native artifact, durable JSON, publication/recovery and public
checksum/source-reference cases pass. Both independent candidate consumer journeys
pass on the 0.69.6 graph: cold capture/transfer/upload, same-ID restore and terminal
replay; process death after authenticated load with original pending spending and
no redispatch. Original journals and raw byte evidence remain retained.

Explicit setup prepared Testkit 0.69.6 with PocketIC **16.1.0** and the 0.69.6
release editor. Aggregate offline tools admission, dependency inheritance,
external IC boundaries, formatting, documentation links and strict Core/Agent
all-target/all-feature Clippy pass. All 12 release-preparation and 13 receipt/resume
cases pass with the actual selected editor in isolated substitute repositories.
No real release/Git effects occurred. No full local repository suite, new native
macOS/MSRV matrix or registry-only Backup consumer qualification ran; the producer's
macOS build fix is not fresh consumer macOS evidence.

Logs: `/tmp/backup-tooling696-*`. Independent graph/build evidence is
`/tmp/ic-backup-agent-independent-build-57819-0`; application/journal evidence is
`/tmp/ic-backup-agent-pocketic-57819-1` and
`/tmp/ic-backup-agent-pocketic-57819-3`. Release fixture evidence remains in
`target/shared-release-tests.Vrx3E8/` and `target/shared-release-tests.MIVV2H/`.
Keep all original files and previous tool bundles. Earlier Platform qualification
remains separately under `/tmp/backup-platform073-*` and
`/tmp/backup-platform072-*`; its 27 wire/identity regressions and three PocketIC
cases qualified their earlier dependency graphs. The current bounded dependency
batch is complete; delivery and any selected release gate remain user-owned.

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
