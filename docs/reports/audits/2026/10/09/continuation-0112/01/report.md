# Stage-owned predecessor checkpoint — 2026-10-09

Released 0.11.1 is confirmed at `7492b19d2e5db8c9952c39f045568166f9a1f1c7`.
Select compatible 0.11.2 for `ExecutionStageGuard::checkpoint`: re-admit the
exact persisted workflow, stage binding, child plan and complete ancestor
settlements; delegate all-Applied checkpoint derivation/publication to the existing
original-journal owner; recheck stage/ancestor admission before returning the
existing exact predecessor record. The caller's learned digest remains opaque
and independently qualified. A separate additive typed error retains both
existing stage and checkpoint errors without changing their variants.

The public learned-stage caller and real simulator capture/metadata/data stages
now use that method. Existing native setup helpers also delegate predecessor
assembly to it. No function, method or type is removed or renamed: manual record
assembly is replaced inside maintained callers. Standalone original-plan
checkpoint publication and exact identity-bound replay retain their independent
recovery contracts. No new record, digest, state flag, journal, counter or
accounting owner is introduced; existing IO/attempt bounds remain unchanged.

22 stage cases, 18 settlement cases, one public learned-stage case and three
actual Testkit planned-download cases pass (44 total). New negatives cover changed
workflow/binding, held/missing/pending journals, occupied checkpoint preservation,
and ancestor-history drift before/after publication. Post-publication failures
return no predecessor and preserve published checkpoint and original journals;
local replay cannot supply successor permission while workflow admission fails.
A focused interruption callback only observes the canonical publication boundary;
production publication always uses the existing checkpoint owner.

Warnings-denied core library/tests and Agent library Clippy pass. Both independent
Rust 1.88 consumers preserve the selected graph without simulator/dev-feature
unification; an additional standalone caller type-checks the new guard method and
public error/predecessor types on Rust 1.88. An initial unused-variable test warning
is retained in its first log and resolved by asserting changed-byte preservation;
final stage tests and core test Clippy pass. Root manifest/lock bytes are unchanged
(Host 0.9.2, Metrics 0.3.1, Testkit 0.27.0). Exact Testkit CLI/server admission remains
prepared from the prior batch; no installation or dependency reselection occurs.

Canonically adopt committed Shared 0.2.4 at
`ffbf665b8481c36b2d9f4d988abec557c3485fa6` from a clean isolated source. The same
92-file roster changes ten test-companion comments and consumer guidance, with
all selected bytes/modes matching committed blobs. Production helper behavior,
engineering rules and tool pins are unchanged. The exact owner exporter suite
qualifies incomplete selection refusals; actual consumer snapshot/pin/ShellCheck,
formatting and Markdown-link checks are recorded separately. No vendored in-place
edit or optional CI-suite expansion is introduced.

Released 0.11.1 Linux CI passes; both macOS jobs remain queued in
[exact-source CI](https://github.com/dragginzgame/ic-backup/actions/runs/37927689030).
Shared 0.2.4 native CI remains pending in
[its exact-source run](https://github.com/dragginzgame/shared-tooling/actions/runs/37927306875).
These runs do not qualify the uncommitted candidate. Issue #29 remains open for
full provider-driven orchestration, effect-boundary application/fence qualification,
complete product manifests/transfer and terminal/custody proof. Learned evidence,
local all-Applied journals and sequential checks grant no fresh effect or
fence/reference-release authority.

All source remains uncommitted; package/receipt stay at 0.11.1. No full CI/release
gate, root Git write, push, publication, live IC operation or sibling edit occurs.
[Qualification](qualification.json) binds exact sources, commands and retained logs.
