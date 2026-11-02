# Current handoff — 2026-11-02

## Selected delivery: 0.55.3

The user requested deployment of the completed cleanup and audit fixes. The
selected destination is `dragginzgame/ic-backup` main plus annotated `v0.55.3`,
followed by crates.io publication of both **ic-backup** and **ic-backup-agent**.
The standard direct patch runner owns version preparation, complete validation,
receipt, commit/tag and atomic push. Publication is a separate recorded command.

This handoff records the selected inputs before release preparation. The
[generated release receipt](../release.json) and retained release/publication
results establish the actual completed version and source. Exact evidence is
`/tmp/backup-deploy-0553.ZSoQWjKL/`: `release.log`, `publication.log`,
`registry-verification.json` and `delivery-result.json`. No successful delivery
is inferred from this preparation note. The complete local gate must qualify
this committed source; hosted native results remain separate evidence.

## Changes and reviewed inputs

- Remove the bounded-record decoder forwarding layer, preserving count,
  aggregate-check ordering and malformed-input rejection.
- Reuse the independent consumer's stage catalog and inspect progress by original
  stage/operation identity. Preserve original byte, spending, refusal and replay
  assertions; reordered stages and gapped operation IDs have a regression.
- Check all three independent golden generators in normal CI. The consumer
  fixture rejects drift in all six vector/contract outputs.
- Adopt Shared Tooling **0.27.28**, exact merged commit
  `03b5d9a47beac9767ced96008a9e7b450b61a82a`, delivered through
  [PR #348](https://github.com/dragginzgame/shared-tooling/pull/348).
  Inventory rejects traversal errors; compiler-cache operations refuse incomplete
  scans, and store maintenance protects unreadable generations. The canonical
  exporter retains 144 payloads with seven updates and no additions/retirements.
- Preserve the incoming locked Host Artifacts/FS/Process **0.74.2** update.
  All six old/new archives match their locked checksums. Artifacts production
  sources are identical; FS moves existing no-op directory entry work to its
  visitor default and uses existing hex-byte formatting in its CLI; Process
  changes one test's bounded accept readiness. Archive/VCS/source proof and diffs
  are retained in the delivery evidence. No public contract or persisted/wire
  format changes were found.

Platform/client remains **0.18.0**, Types **0.17.0** through Platform, Testkit
**0.80.1**, target-specific Runtime **0.18.1**, authenticated PocketIC **16.1.0**
and release editor Build **0.74.0**. Minimum and pinned compiler remain Rust
**1.99.1**. No dependency refresh is requested during release qualification.

## Earlier focused qualification and custody

The audit/adoption evidence is `/tmp/backup-audit-fixes.PfeuvfwT/`: original
source/index, supplier patch/PR/protection records, export/source proof and Linux
logs. Both supplier maintenance fixtures, installed inventory rejection, golden
parity, tool/snapshot/dependency admission, consumer tooling, hooks and release
adapter checks pass. The supplier PR merged without a protection bypass; hosted
native jobs were still running when admitted. The supplier sibling working
checkout was not modified.

The earlier decoder/stage cleanup evidence is
`/tmp/backup-decoder-stage-cleanup.4qhi2h01/`: strict focused Clippy, 47 model
cases and all 11 independent-consumer checks passed on its original graph.
That graph used Host **0.74.1**; it is historical proof rather than fresh
qualification of this incoming Host update. The authorized complete release gate
supplies the fresh source/graph qualification. The original handoff is retained
as `status-before-delivery.md` in the new delivery evidence.

The prior delivered baseline was **0.55.2**, release commit
`5f94655f779431349393274e98f616fe580b0867`, with both registry archives verified.
Its original delivery evidence remains `/tmp/backup-deploy-0552.0OuUpHJW/`.
Preserve unfinished journals, referenced and partial artifacts, source references,
original/failed qualification evidence and retained tool generations. Neither
this release nor local Linux qualification establishes installed Canic, live IC
or native macOS acceptance, or authority for product/store cleanup.

[Implementation contracts](../implementation-contracts.md),
[the Platform boundary](../ic-platform.md) and
[the operator map](../canic-adoption.md#reusable-operator-composition) remain canonical.
