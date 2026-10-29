# Current handoff — 2026-10-29

## Local simplification batch

All ten reviewed simplifications are implemented without public API, persisted
schema, wire-byte or hash changes. Capture and restore share original mutation
reservation/dispatch; metadata, allocation and restore share reply-preserving
checkpoint completion after releasing attempt exclusion. Stage persistence now
separates guards from bounded binding/ancestor admission, retaining every recheck.

Attempt projection uses explicit idle, pending-mutation, pending-observation and
Applied states. Snapshot regions have named identities. Observation requests carry
one owned reservation component. Download/upload plans share learned-predecessor
binding; capture/allocation settlement share expected inventory identity assembly.
Digest owners reuse existing framing methods. Shared plan/workflow/metadata fixture
builders live in `test_support`; unit assertions and independent goldens remain
beside their owners. The original 531-case Core registry remains intact.

Focused validation passes: **494 Core unit tests**, **17 public integration tests**,
six connected real PocketIC workflows and two independent Agent consumer journeys.
Coverage includes cancellation/lock custody, malformed and lost replies, changed
ancestors, exact histories/hashes, same-ID round-trip, retained continuation and
process death after authenticated load without redispatch. Strict Core/Agent
Clippy passes for all targets/features with warnings denied. Formatting, dependency
inheritance/boundaries, repository inventory and documentation links pass.
No full local gate, new native macOS/MSRV matrix, registry-only package acceptance
or live IC operation ran. Earlier delivered CI does not qualify this working tree.

Evidence is `/tmp/backup-simplify-*`. Connected fixtures remain under
`/tmp/ic-backup-pocketic-822447-{0,2,4,6,8,10}`. Independent build/graph evidence is
`/tmp/ic-backup-agent-independent-build-886517-0`, with authenticated fixtures in
`/tmp/ic-backup-agent-pocketic-886517-1` and `-3`. Keep their original journals,
requests/replies, artifact bytes, commands, source references and failed evidence.

## Dependency graph and prior tooling adoption

Incoming lock updates now select Host/Testkit **0.69.11** and Platform **0.7.5**,
with AWS-LC **1.19.0**/sys **0.47.0**. The user paused further dependency updates
for final validation; manifest and lock fingerprints stayed unchanged across the
focused run. The qualified lock SHA-256 is
`7a827b98a485c169729f1c45122dace05e992cfe56184053a3f7161efa943fbb`;
`/tmp/backup-simplify-graph.sha256` records both inputs. Exact Testkit CLI/server
setup and offline admission pass, retaining PocketIC **16.1.0**.

Declared Host/Testkit minimums and release editor remain **0.69.9**, Platform's
minimum **0.7.3**. Management DTO **0.11.0**, external Principal **0.1.5**, Agent
SDK **0.49.3**, Candid **0.10.39** and Rust floor **1.91** remain unchanged. This
refactor preserves incoming dependencies; its tests supply no supplier crypto or
production TLS acceptance. Retain [Platform #50](https://github.com/dragginzgame/ic-platform/issues/50)
and the prior supplier qualification boundaries.

Shared **0.27.6**, revision `5a72359b5be19d5221334cf2c9b3204621d3f3f6`, remains
exporter-owned. All 126 payloads/modes matched the independently selected commit;
[the review](../shared-tooling-review.json) retains that adoption and its older
graph qualification. No vendored payload was edited in this simplification batch.
Backup keeps its regular-file gate, three-host CI, fresh managed servers and raw
management ingress. No cached-baseline API, schedule or compiler cache is enabled.

Repository versions remain **0.48.2**, with pending **0.48.3** changelog notes.
Prior tooling/ownership work and these refactors remain unstaged and uncommitted.
Staging, commits, release, delivery and destructive cleanup remain user-owned.

## Delivered source and issue ownership

Delivered **0.48.2** is `ba7f5d4ae9aa3c1e0d8e7b0ae7c08b75fee214a0`.
[CI run 40698974873](https://github.com/dragginzgame/ic-backup/actions/runs/40698974873),
attempt 1, passed Linux, Apple Silicon and Intel macOS, including all 531 Core
cases and independent PocketIC consumers. Earlier failures retain their status.
[#72](https://github.com/dragginzgame/ic-backup/issues/72) and
[#73](https://github.com/dragginzgame/ic-backup/issues/73) were closed as completed
using that exact evidence. [#29](https://github.com/dragginzgame/ic-backup/issues/29)
was closed as not planned as redundant coordination, with unresolved application
acceptance transferred to Canic. GitHub remains the issue tracker.

[Canic #513](https://github.com/dragginzgame/canic/issues/513) owns engine/Root-paid
composition, complete transfer, exact custody and terminal/reference authority;
[#550](https://github.com/dragginzgame/canic/issues/550) owns application freeze,
external obligations, post-load acceptance and safe release;
[#394](https://github.com/dragginzgame/canic/issues/394) owns fresh live admission.
See [the integration map](../canic-adoption.md). At the preceding review,
[Canic PR #702](https://github.com/dragginzgame/canic/pull/702) was open at
`78a57d5b1e83a357999bfd4afbe88a06b0809c64`, isolated in
`/tmp/canic-backup-adoption.lzm8Ut`. Its 27 native/two Root cases retain their
limited scope; no sibling source was changed or production Root-paid dispatch
qualified here. Reviewed Canic main was 0.112.0 at
`5c236ab130c0d63892ef281b45bdc3fa353e012f`.

## Retained historical evidence

Keep `target/validation-failures/20261028T131421Z-499630-13-test.log`, its highlight
and combined logs, and `target/validation-failures/20261027T172540Z-3411640-8-clippy.log`.
Retain `/tmp/backup-reference-release-*`, `/tmp/backup-tooling696-*`,
`/tmp/backup-platform073-*`, `/tmp/backup-platform072-*`, `/tmp/backup-72*`,
`/tmp/backup-shared273-*`, `/tmp/backup-shared276-*`, original consumer/release/Canic
journals and byte evidence. Closed issues and successful tests authorize no cleanup.
