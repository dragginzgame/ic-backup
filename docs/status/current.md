# Current handoff — 2026-11-02

## Testkit adoption, pending 0.55.1

Adopted published Testkit **0.80.0** over the incoming **0.55.0** release source
`3944f3d9825a103027717e3040d4a6fc2c55fcb4`. Own package versions remain 0.55.0.
Pending notes select **0.55.1**: this development dependency update changes no
public Backup contract and needs no consumer migration. Preserve the incoming
`async-lock` **3.5.0** lockfile update.

All **30** Testkit Rust source files match authenticated 0.79.0 byte-for-byte.
The normalized manifest changes optional Wire 0.14 to 0.15; Backup enables only
the empty default feature and resolves no Wire package. Scoped lock resolution
replaces only Testkit; every other incoming dependency record is unchanged.
Platform/client **0.18.0**, canonical Types **0.17.0**, management DTO **0.11.0**,
Host **0.74.1**, arithmetic-only Metrics **0.26.1**, Build editor **0.74.0** and
authenticated **PocketIC 16.1.0** remain selected. Shared Tooling stays at
**0.27.27**, commit `651f5f7acb2e3055abd51ba5f835a2112801666a`; no snapshot refresh
or payload edits are part of this task. PocketIC and Testkit remain development-only.

## Focused qualification

Evidence is `/tmp/backup-testkit-refresh.jXzVq3jM/`, including incoming files/diff,
registry selection, checksummed archive/VCS identity, source/manifest comparison,
lock/feature proofs, frozen inputs and original logs.

- Exact Testkit CLI setup and offline server admission pass, retaining the
  authenticated server and previous/failed tool generations. An initial check
  refused while setup still held its lock; validation continued after setup
  completed (`testkit-setup.log`, `server-admission-complete.log`).
- Compiler admission records actual Rust/Cargo **1.99.1**. Cargo inheritance and
  IC boundary checks pass. Strict both-library all-target/all-feature Clippy
  passes (`admission.log`, `clippy.log`).
- All six connected Core PocketIC capture/restore/recovery cases and the real
  Agent capture/complete transfer/upload/same-ID restore pass
  (`core-pocketic.log`, `agent-pocketic.log`). Actual Testkit setup/check routing
  and refusal fixtures pass (`testkit-routing.log`).
- All **854** captured source/lock inputs remain unchanged across qualification.
  Review/handoff edits afterward receive focused formatting/link/snapshot checks.

This is focused Linux evidence. No new full CI/release gate, independent
package/MSRV consumer rerun, native macOS, installed Canic or live IC acceptance
is claimed. Earlier unified-graph consumer/package and product qualification
remains at `/tmp/backup-platform-testkit-refresh.lw0kvf7w/`; its incoming review
is preserved in this batch's `before/`. No staging, commit, own-version bump,
release, publication, sibling edits or cleanup occurred. Preserve original
journals, referenced artifacts, unfinished recovery and failed evidence.

## Preserved baseline

The 0.55.0 source includes the unified Host/Platform/Types simulator graph and
four accepted simplifications: journey driver construction, fresh restore-source
contexts, one Agent routing/reply-binding decision and six named round-trip
scenarios. Original digest domains, attempt identities, bytes, recovery and
effect-free replay remain unchanged. The earlier Host 0.73 to 0.74 public error
identity migration still requires consumers sharing those types to align Host.

[Implementation contracts](../implementation-contracts.md),
[the Platform boundary](../ic-platform.md),
[the tooling review](../shared-tooling-review.json) and
[the operator map](../canic-adoption.md#reusable-operator-composition) remain canonical.
