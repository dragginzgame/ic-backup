# Current handoff — 2026-11-01

## Accepted four simplifications

All four accepted recommendations are implemented over released **0.53.4**,
commit `959b2d8`. Own versions and dependency selections remain unchanged. The
compatible **0.53.5** changelog draft covers this batch. Changes remain unstaged/
uncommitted; this implementation request does not start another release.

Agent's private observation path accepts only management, allocation-list and
data-readback requests. Its private application path accepts only mutation and
observation requests, with named reply binding fields. Public providers,
`ReservedUpdate`, `PreparedUpdate`, retention callbacks and port errors are
unchanged. The original request validator, signing/retention/submission owner and
reply-evidence framing remain byte-for-byte unchanged. Existing domains, payload/
authority commitments, chronological attempts and mutation's zero observation
slot are preserved. No unrelated request family can enter either private path.

Connected journey construction and retained evidence paths use the existing
`Stage` owner. Stage IDs remain **0/7/9/11/13/17/3**, with independent numeric order
assertions retained. Named failure expectations distinguish a stage from its
child operation (the second upload still names operation **1**) and carry an
optional successor. Start has no invented `u64::MAX` successor; real successor
absence, no-reissue, pending spending and terminal checks remain. JSON evidence
still emits the same numeric `pending_stage` values.

`AdmittedProgress` names the original progress view and borrowed journal map at
the existing progress/persistence/settlement handoff. Admission, coverage,
chronology, accounting, exact histories and error order are unchanged; no new
journal clone, filesystem pass, transition or lock order follows. Consistency
model/policy tests retain separate named fault cases in their original order,
with unchanged hashes, golden values and assertions.

No public API, wire/record/hash format, authority, custody or recovery contract
changed. All **661 original test names in 151 files** are preserved. Cargo.toml
and Cargo.lock remain byte-for-byte unchanged.

## Fresh focused qualification

Evidence is `/tmp/backup-four-cleanup.4kn9mu5z/`: incoming Rust/source/handoff/Cargo
copies under `before/`, original test roster, four-outcome implementation map,
Agent boundary comparison, review diffs, frozen inputs and command logs. Final
Rust/manifest/lock identities remained unchanged throughout passing validation.

Strict both-crate all-target/all-feature Clippy passes (`clippy.log`). Initial
semicolon and fixture evidence-projection diagnostics are retained separately;
they were fixed before the passing run, without new lint suppression. Formatting,
Cargo inheritance/pins and direct IC dependency boundaries pass
(`source-checks.log`).

All **74 selected tests** pass: **56 Core unit cases** (`core-native.log`), **two
Agent unit cases** (the successful library section of `agent-native.log`), **six
HTTP boundary cases** (`agent-http.log`), **six connected Core PocketIC journeys**
(`core-pocketic.log`) and **four Agent PocketIC cases** (`agent-pocketic.log`).
The initial combined Agent command's six HTTP fixtures could not bind sockets in
the sandbox; that failed output remains in `agent-native.log` and
`agent-native-sandbox.log`. The same six cases subsequently passed with local
socket access. Native HTTP substitutes establish transport behavior, not IC effects.

Fresh real IC evidence covers connected capture/restore/resume/replay and lost
allocation/upload/load/start replies, Agent status/list/data recovery, and an
independently built normal consumer's application update, completed replay and
process-death observation. Core roots remain `/tmp/ic-backup-pocketic-1122257-*`;
Agent roots remain `/tmp/ic-backup-agent-pocketic-1181318-*`, including application
consumer evidence under `-5` and `-6`. The normal graph/build is
`/tmp/ic-backup-agent-independent-build-1181318-0`. Offline Testkit admission
precedes managed startup; original plans, bytes, ingress and journals remain retained.

No new full CI/release gate, macOS/MSRV matrix, package acceptance, installed-IC
or live Canic qualification was run. Focused evidence grants no generic
application, fence, terminal or source-reference release authority.

## Released baseline and preserved evidence

**0.53.4** passed the complete local release gate and was atomically pushed/tagged
and published for both libraries. Registry archive checksums and VCS identities
were independently verified against commit `959b2d8`; the worktree was clean.
That delivery is recorded in `/tmp/backup-platform-testkit-deploy.44nom2x5/`,
including `deployment-verification.json`, full release/publication logs and the
[generated release receipt](../release.json). Its proof retains that source scope.

Platform stays **0.16.0**, Testkit **0.76.0**, Host Artifacts/FS/Process **0.73.4**,
arithmetic-only Metrics **0.24.4**, Types **0.15.1**, Candid **0.10.39**, Agent
**0.49.3**, external Principal **0.1.5**, native management DTO **0.11.0**, Build
editor **0.73.1** and authenticated PocketIC **16.1.0**. Shared remains `143a9ab`
(134 payloads; VERSION 0.27.21, commit title 0.27.22).

The preceding handoff is preserved under this batch's `before/`. Earlier evidence
remains `/tmp/backup-six-cleanup.jilrlpcv/`, `/tmp/backup-seven-cleanup.eM8zqj/`,
`/tmp/backup-fixture-next-ten.kMU3t6/`, `/tmp/backup-host-testkit-refresh.Jqi85C/`,
`/tmp/backup-fixture-cleanup.QTkXwS/`, `/tmp/backup-host-testkit-audit.RP2tAU/`,
`/tmp/backup-adopt-metrics-testkit.YY4bZ2/`, `/tmp/backup-adopt-host-platform.ZA12oo/`,
`/tmp/backup-cleanup-ten-next/` and `/tmp/backup-adopt-upstream.sKsOIs/`.
Historical qualification and failure observations retain their original scope.

[Implementation contracts](../implementation-contracts.md),
[the Platform boundary](../ic-platform.md) and
[the operator map](../canic-adoption.md#reusable-operator-composition) remain
canonical. Preserve original journals, referenced artifacts, simulator/consumer
roots, validation failures and old/failed bundles. No staging, commits, release,
publication, sibling mutation, live IC effects or disposal occurred in this batch.
