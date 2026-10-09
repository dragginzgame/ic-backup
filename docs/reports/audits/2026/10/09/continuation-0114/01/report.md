# 0.11.4 capture coordination and Shared Tooling 0.2.6 review

Released base: `2939a41805ce5c2fa3913162b4028bc970d3d85f` (0.11.3).
The compatible undated 0.11.4 draft adds one provider-driven capture step and
adopts the latest committed Shared formatting include. Package/receipt remain
0.11.3; manifest and lock bytes retain the exact incoming selection: Host 0.9.3,
Testkit 0.27.1, Metrics 0.3.2, management SDK 0.11.0 and Agent 0.49.2.

## Implemented behavior and ownership

`workflow::ic_snapshot_capture::capture_snapshot` accepts only the original
`TakeCanisterSnapshot` payload. Existing canonical target/wire binding rejects
before spending. Open the existing original journal, never create it; complete
original-plan admission owns durable reservation. Keep the selected journal
locked through mandatory fresh integration admission, one existing mutation
provider invocation and bounded passive acknowledgement association. Retained
stage/ancestor checks bracket dispatch. Return any bounded acknowledgement in
typed association or post-reply rejection errors.

Success stays pending. Independently authenticated original attribution and the
existing explicit receipt owner alone establish outcomes. Failed admission,
provider errors and malformed/lost replies retain consumption, obligations and
references. Pending/Applied, missing/held or changed originals cannot authorize
another callback/provider invocation. No new schema, journal, decoder, spending
owner, default provider, permission, automatic uncertainty, retry or refund exists.
Fresh control, capture consistency, quiescence/fences and never-dispatched command
custody remain integration-owned; preflight has independent prior accounting.
Sequential checks do not fence noncooperating filesystem actors.

The isolated Testkit driver now calls the public capture step before explicitly
recording its qualified successful receipt. Its original checkpoint supplies the
predecessor for the existing metadata/data read steps and durable artifact writer.
Actual lost/malformed capture replies preserve pending originals/source references
across resume, with no recapture, checkpoint or successor stage. Fixture-specific
stopped/no-external-effects admission and exact ingress attribution do not qualify
arbitrary Canic applications or install a complete backup/restore runner.

## Shared adoption and retirement

Canonically export 93 files from clean isolated committed Shared 0.2.6
`ce13a5314916891fd239d9b199b4a91b04775054`. Add `make/rust-format.mk` and update
consumption/release/hook guidance; baseline and tool pins are unchanged. No
snapshot-listed file was patched locally. Replace the root formatting recipes
with the include, preserving exact pinned offline formatter admission and
portable checkout-local tool lookup. Export its companion in actual hook,
Testkit-routing and release-adapter fixtures. Both Shared owner suites and the
actual consumer fixtures pass independently; no root hook activation occurs.

Do not select optional `make/release.mk`: the local `release-resume` performs an
exact receipt/tag/saved-validation check after the shared runner. Prerequisites
alone cannot preserve that ordering. The new Shared guidance explicitly permits
specialized direct routing with post-run checks retained; keep the current owner
without another wrapper. The release adapter suite qualifies that retained route.

No Rust function, method or type was removed or renamed. The local Make recipes
`format-tools-check`, `fmt` and `fmt-check` are replaced by the shared include;
direct caller-side capture reservation/dispatch assembly is replaced by the new
coordinator. Existing model, codec, port, policy and receipt APIs retain their
distinct responsibilities.

## Focused qualification and limits

[qualification.json](qualification.json) binds exact source, graph and log hashes.
Retained evidence lives under `target/continuation-0114/`. Seven capture cases,
41 workflow/stage cases and three passive mutation association cases pass. Five
actual planned simulator cases cover successful capture-to-download plus lost/
malformed capture and data replies. Core library/tests and Agent library Clippy,
warning-denied core rustdoc, both independent Rust 1.88 consumers and a standalone
new capture API caller pass. Snapshot/dependency pins, formatting, ShellCheck,
selected Testkit offline admission and maintained local documentation links pass.
All runtime checks use the unchanged incoming graph and already prepared exact
Testkit 0.27.1/PocketIC 16.1.0 installation; older evidence/installations remain.

Released 0.11.3 Linux CI passes in its
[exact-source run](https://github.com/dragginzgame/ic-backup/actions/runs/37942396169);
both macOS jobs remain queued. That run does not qualify this uncommitted candidate.
The public GitHub description remains accurate about host libraries, direct Agent
transport and incomplete full runners. Keep [#29](https://github.com/dragginzgame/ic-backup/issues/29)
open for complete provider-driven transfer/upload/restore orchestration, application
admission and terminal/custody qualification; [#32](https://github.com/dragginzgame/ic-backup/issues/32)
and [#33](https://github.com/dragginzgame/ic-backup/issues/33) retain native acceptance.
No full workspace/CI/release gate, root Git write, release/version transaction,
publication, production IC effect, Canic/sibling edit or recovery cleanup occurred.
