# Current handoff — 2026-10-28

## Current restore-reference correction

The working correction is based on `b52bebc08091b92a65c57109b922837ba45881ad`;
own versions remain **0.48.1** and the [0.48.2 notes](../../CHANGELOG.md#0482)
remain open. No staging, commit, version mutation or release ran for this fix.

The maintainer's full validation failed in duplicate-command reference-release
admission: the first command's quiescence probe could fail before the later
invalid identity was reached. A strengthened regression deliberately holds that
first command and reproduces `Custody(InFlight)` in place of `InvalidCommandSet`.
`release_restore` now validates the complete bounded catalogue before acquiring
any command guard. Valid catalogues still require every original inode-bound
quiescence guard held through durable reference publication and exact replay.
Duplicate and mixed-journal regressions preserve the original reference bytes.

All 14 focused native checks pass: six owning restore-release cases, six command
custody/lifecycle cases and the two public source/safety cases. Strict Core
library/test Clippy, shared formatting and documentation links pass. No new IC
effect or schema change is introduced. The original full validation has not been
rerun; the focused correction is complete and reviewable.

Preserve `target/validation-failures/20261028T131421Z-499630-13-test.log` and its
highlight/combined logs. `/tmp/backup-reference-release-before.log` retains the
deterministic pre-fix failure; matching `-after`, `-custody`, `-public`, `-clippy`
and `-source-checks` logs retain passing evidence. No retry, budget reset, inferred
receipt or reference cleanup is authorized by the correction.

## Dependency batch retained

The preceding batch selected published IC Tooling **0.69.6**,
`cba072ee8fe288eb92bb4a4ddf9753b8abc4b580`, for Host Artifacts/FS/Process,
Testkit and the tooling-only release editor. Platform **0.7.3** remains selected
from `6733f504b62b81456001a1735f2a25b3bbffe3b9`. Management DTOs **0.11.0**,
external Principal **0.1.5**, Agent SDK **0.49.3**, Candid **0.10.39** and Rust
floor **1.91** are unchanged. Lock SHA-256 remains
`f74e6ad555ff6fc2d3e04ad3ee8ad87a7d40a4395b769ec1f02f1940a2e33bd7`.

Earlier dependency qualification passed 33 artifact/persistence native cases,
both independent candidate consumer PocketIC journeys, strict Core/Agent lint,
offline tools/dependency admission, 12 release-preparation and 13 receipt/resume
cases. That evidence predates this ordering correction. Logs remain under
`/tmp/backup-tooling696-*`, with original consumer evidence under
`/tmp/ic-backup-agent-independent-build-57819-0` and
`/tmp/ic-backup-agent-pocketic-57819-{1,3}`. Release fixtures remain in
`target/shared-release-tests.Vrx3E8/` and `target/shared-release-tests.MIVV2H/`.
Older Platform runs remain under `/tmp/backup-platform073-*` and
`/tmp/backup-platform072-*`. Keep original journals, bytes and tool bundles;
no full native host matrix or registry-only Backup qualification is claimed.

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
