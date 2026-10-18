# Current handoff — 2026-10-18

Released baseline: `ic-backup` and `ic-backup-agent` 0.31.1 at
`8a768650dd89c1cdb1ea6132dfe7c82fd55a224c` (`v0.31.1`). Package versions remain
0.31.1; compatible pending notes select 0.31.2. No public API or persisted schema
change is selected. Earlier receipts remain in the
[October 18 archive](archive-2026-10-18.md),
[October 17 archive](archive-2026-10-17.md) and
[repeat ownership review](../reports/audits/2026/10/18/architectural-sediment/03/report.md).
Release metadata retains authority.

## Current boundary

Core owns backup records, bounds, guarded persistence and coordination. Agent owns
one reserved update's signing, retention and transport. Platform owns the Candid,
Principal, management DTO and SDK dependency boundary. Core selects `candid` and
`management-canister-types`; only Agent adds `agent`. Direct upstream declarations,
compatibility reexports and pin exceptions are removed. The
[Platform contract](../ic-platform.md) retains the original 0.1.8 adoption evidence.

Host owns hashing, descriptor IO and process custody. Testkit owns simulator assets
and managed processes. Backup retains fixture evidence, bounded DTO visitors,
Reqwest client policy and original journals. Applications own membership, routing,
release identity, consistency and external-effect settlement. A standalone CLI and
complete application runner remain outside the implemented scope.

The incoming manifest/lock update is preserved: Testkit 0.54.0, Host 0.27.4 and
Platform 0.1.9 are locked. Host requirements remain 0.27.1 and Platform's minimum
remains 0.1.8. This maintenance pass changes no dependency selection. Shared Tooling
0.24.0 remains at `cab12cd3dfa4fdb34846b25397f0d72881bb9ae0` with 114 exact payloads.

## Prepared maintenance

Rust 1.99.1 is selected for development; consumer MSRV remains 1.91.0. README and
development guidance agree. Native CI compares the already-admitted common Rust
tool directory, preserving per-tool physical identity instead of a second roster
that included the retired extractor. Changelog notes link the owning issues.
Canonical explicit setup has prepared the common tools and locked Testkit 0.54.0
CLI for Rust 1.99.1. Prior bundles and failed attempts remain retained. Development
guidance describes compiler-bound tool receipts and the current common catalog.

The latest delivered [CI run](https://github.com/dragginzgame/ic-backup/actions/runs/39123427210)
failed on all three hosts in tool reuse before product validation. The retained
Linux artifact confirms successful installer checks and the obsolete comparison.
[The CI repair](https://github.com/dragginzgame/ic-backup/issues/60) is local only;
no native hosted result exists for the dirty batch. Testkit consumer acceptance
still requires a successful containing native matrix. Earlier 0.31.0 green CI is
historical evidence, not qualification of this delivery.

## Focused qualification

On the current pending graph, Rust 1.99.1 all-target/all-feature checking and strict
Clippy pass. All 78 IC model/codec, 14 Agent HTTP, 34 Core PocketIC and 12 Agent
PocketIC cases pass. The sandboxed HTTP attempt refused socket binding; the
loopback-enabled rerun passes, with both attempts retained. Rust 1.91 all-target
checking and both independent production consumers pass; the latter retain
`target/msrv-consumers.dGrOin/`. Hooks and actionlint pass. The actual CI comparison
block reproduces the old failure, accepts current reuse and rejects missing or
mismatched targets. Formatting, dependency/pin checks, snapshot integrity and
local links pass. Before-images and receipts: `/tmp/backup-issue-pass-mlttf_tl/`.

The subsequent tooling qualification passes `make tools-check` and the complete
native CI reuse step on Linux, with only its fixture directory changed to a
unique caller-owned path. Offline host/IC/common/selected Cargo reuse, hostile
application flag/wrapper isolation and exact physical identities pass together.
The initial sandbox refusal to acquire a shared-store lock is retained separately
from the successful host-enabled run. Both current package archives compile
together in an independent consumer on the original normal graph; archive and
unpacked-file checksums pass. Evidence: `/tmp/backup-native-followup-wafn4qmr/`,
`target/ci-fixtures/native-reuse.y_641iwl/` and `target/package-check.CN2Dxh/`.

These are focused Linux checks, not full local CI, native macOS, installed-IC,
package publication or complete application acceptance. The earlier 634-test and
package/identity receipts retain their original source and dependency identities.
No own version change, staging, commit, push, merge, CI dispatch, release,
publication, deployment or artifact cleanup was performed.
