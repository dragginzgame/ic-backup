# Current handoff — 2026-10-04

The maintainer requested continued extraction after the layout/reference batch.
This batch adds owned command descriptor custody and exact existing-sidecar
quiescence admission. Both batches remain uncommitted for review on `8fc3a77`;
the pre-existing dirty work and recovery evidence were preserved. Canic and
sibling repositories remain read-only. No live IC effects, release transactions,
commits, pushes or package uploads ran.

`CommandLifetimeLock` acquires a private operation sidecar and exposes a
validated v1 `CommandCustodyRecord` binding the canonical journal path, exact
operation sequence and filesystem device/inode. Callers must durably retain
this evidence and exact intent before dispatch. `spawn` consumes one local
allowance before attempting creation, preserves stdio and consumes the command
so no parent-side descriptor copy survives spawn. The owned child duplicate
inherits custody; the owner remains close-on-exec. Durable paid-call authority
and bounded backend execution remain separate contracts.

`finish` closes owner custody without explicitly unlocking inherited holders
and probes within a 250 ms grace period. Fresh `CommandQuiescenceGuard`
admission opens only an existing sidecar and checks the retained device/inode;
missing or replaced evidence never causes blind recreation. Success retains an
exclusive non-spawning guard. The selected backend must preserve custody in
all relevant descendants; these mechanisms do not establish remote success,
terminal completion or permission to retry an unresolved effect. Read
[the maintained boundary](../extraction-boundary.md) and
[the custody schema](../contracts/command-custody.schema.json).

The preceding layout/reference implementation remains intact:
`BackupLayoutGuard` locks a stable parent sidecar and retains an open root
identity. Model-owned v1 restore references bind exact journal/immutable-intent
identities, bounded to 1 MiB of encoded JSON, 1,024 entries and 4,096 UTF-8 bytes
per journal location. Retention publishes before a recoverable journal;
exact-repeat adoption completes synchronization after a lost response. Missing
or moved journals retain source artifacts. No reference release or prune API is
exposed; exact terminal evidence and qualified backend custody must govern it.

Fresh targeted Linux qualification passed: 48 unit tests and one public-API
integration journey, warning-denied Clippy/docs, formatting, Rust 1.91.0
all-target/all-feature compilation and standalone Cargo package verification.
Actual exec/descriptor checks cover child/owner close-on-exec flags, exact file
identity, one-spawn admission, failed spawn, unsafe/replaced/missing evidence
and bounded finish. Acknowledged process-death cases prove that a descendant
blocks admission after both owner and direct child exit; only final descendant
exit permits fresh exclusive admission. The public recovery journey now retains
custody evidence before dispatch and separately admits quiescence. Existing
artifact, publication, layout and reference regressions also passed. This is
native filesystem/process evidence; no PocketIC or real IC backend was used.
No broad validation ran.

Retained logs: `target/command-custody-tests.log`,
`target/command-custody-clippy.log`, `target/command-custody-msrv.log`,
`target/command-custody-docs.log` and `target/command-custody-package.log`.
The registry fetch log is `target/command-custody-fetch.log`. Packaged source
and builds remain under this repository's `target/package/`. Earlier extraction,
layout/publication logs and evidence remain retained.

[Command custody provenance](../command-custody-source.json) records exact
inspected Canic source hashes, adapted destinations, consumer references,
dependency source inspection and retained ownership.
[Layout provenance](../layout-source.json), the original
[planning baseline](../source-baseline.json) and
[first extraction provenance](../extraction-source.json) remain retained.
Canic HEAD remains `3978e02d28fea9022c7a6e84a7ec6d6e4a0d4af3`; its worktree
was dirty during inspection. Provenance records do not constitute qualification.

The workspace remains at `0.1.1`, Rust 2024, development Rust 1.99.0 and MSRV
1.91.0. Unix builds inherit pinned published `command-fds` 0.3.3 for safe owned
descriptor setup; no Rust unsafe hook or borrowed/clonable raw handle was copied.
The lockfile adds only `command-fds`, `nix` and `cfg_aliases`; previously locked
versions remain unchanged. No Canic dependencies, sibling patches or shared
build target exist. At the maintainer's request, completed changes now sit in
the undated `0.1.2` changelog draft beneath an empty Unreleased entry. Cargo
metadata remains at `0.1.1`; no release preparation ran. Existing release receipt
and historical changelog notes remain unchanged. Registry publication continues
to delegate to Cargo; strict receipts/tags govern repository release transactions. See
[development](../development.md) and [releasing](../releasing.md).
This continuation does not authorize publication.

Full B1/B2 completion and independently usable canister backup/restore are not
established. Generic authority/consistency, journal/transition, spending-budget
and executor contracts still precede runner extraction. Backend custody
qualification, terminal reference release, prune, transport and CLI remain
pending. Canic's current backup executor topology preflight still rejects.
Read [the design](../extraction-design.md) for the maintained implementation
sequence. The standing no-commit rule remains in force; Canic adoption and live
IC effects require their own instructions.
