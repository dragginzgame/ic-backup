# Current handoff — 2026-10-24

## Ownership and source

Core owns bounded records, guarded persistence, reconciliation and original-plan
coordination. Agent owns reserved signed transport. Host owns generic IO/process/
hash capabilities; Platform owns external IC identities and SDKs; Testkit owns
simulator assets and managed processes. Applications own current membership,
permissions, release binding, whole-unit fencing and external obligations.
[Implementation contracts](../implementation-contracts.md) and the
[record/API boundary](../extraction-boundary.md) retain the exact safety contracts.

Released source remains **0.41.0**, `9bb769b`. Pending **0.42.0** combines the
existing ingress/registry-consumer work with Host's breaking dependency identity
change. Consumers exchanging Host types must align to **0.67**. Package versions
remain 0.41.0; no staging, commit, push, release, publication, live IC effect or
sibling edit occurs. The [released handoff](https://github.com/dragginzgame/ic-backup/blob/9bb769b/docs/status/current.md)
retains earlier extraction/documentation evidence at its original source scope.

## Published IC Tooling adoption

Host Artifacts/FS/Process, Testkit and the contributor Build release editor select
published **0.67.0**, authenticated source **248df96b27b44366095d9d8e195cca8a1023bde1**.
Archive/source identities are retained in `/tmp/backup-tooling-067-published.json`.
All three Host Rust source trees match 0.66.3 byte-for-byte. Testkit's used server
CLI/asset/startup contracts preserve behavior; unused benchmark comparison changes
need no adapter. Build's changed archive/configuration APIs are not used here;
its release editor passes the isolated metadata/retention fixtures.

The lock changes exactly four packages to 0.67.0. Incoming Platform **0.6.13**
and all other selections are preserved; Agent **0.49.3**, management DTO **0.11.0**,
external Principal **0.1.5** and PocketIC **16.1.0** remain selected. Testkit and
Build CLIs are prepared through the canonical installer and pass offline admission.
The existing server asset is reused; old/failed tools and fixture evidence remain.
Shared Tooling stays **0.26.9**, with no vendored payload edits. No storage, wire,
decoder, custody or lifecycle change is introduced.

## Current qualification

Linux strict all-target/all-feature Clippy, warning-denied Rustdoc, dependency
inheritance/boundary checks, formatting and documentation links pass. All **13**
selected public artifact/recovery/ingress cases pass. Actual Rust **1.91.0**
all-target checks and both independent MSRV consumers pass, including public
Host checksum/publication error identities. Their normal graphs select only Host
0.67.0 and exclude Testkit/PocketIC/WAT/Clap and development Host features; Core's
independent graph remains Agent-free.

Both exact unpacked package archives compile together with frozen original
selections and verified archive/file checksums. Evidence:
`target/package-check.SEVcLy/` and `target/msrv-consumers.lK9fzS/`.
Both authenticated candidate-consumer simulator cases pass: cold round trip with
accepted terminal replay, and process death after load with no redispatch. Evidence:
`/tmp/ic-backup-agent-independent-build-3420265-0/` and
`/tmp/ic-backup-agent-pocketic-3420265-{1,2}/`. The read-only minor release preview
agrees with 0.42.0; no release effects ran. Logs are
`/tmp/backup-tooling-067-{setup,clippy,native,msrv,package,editor,boundaries,rustdoc,simulator}.log`.
Current lock SHA-256: `51303f850f90f171e817a494b5b986d6e8c4c273698bf2710153e1cb09b79b51`.

## Retained Canic and issue work

The public application-ingress fixture and adoption guide retain the existing
Core seam for exact Root ingress, original pending spending and cold settled
replay. Synthetic native receipts do not qualify Wire, current authority, real
Root-paid effects, whole-unit fencing or terminal release. Canic's working Root
storage/Wire/catalogue integration remains application-owned; its pending restore
caller uses the removed effect_order API and must be adapted before adopting 0.41+.
See [Canic adoption](../canic-adoption.md); preserve its active engine meanwhile.

The two registry-only consumer cases passed on **released 0.41.0 with Host/Testkit
0.66.3**, before this adoption. Archive/graph evidence remains at
`/tmp/ic-backup-agent-independent-build-2976459-0/`; fixtures at
`/tmp/ic-backup-agent-pocketic-2976459-{1,2}/`. Default candidate-mode evidence is
at `/tmp/ic-backup-agent-independent-build-3123817-0/`. Those results are not new
registry qualification of pending 0.42.0. The explicit registry mode rejects
source/version drift and never falls back to candidate paths.

[Backup #29](https://github.com/dragginzgame/ic-backup/issues/29) still requires
Canic production acceptance. [Backup #68](https://github.com/dragginzgame/ic-backup/issues/68)
still depends on Platform's qualified crypto correction; this tooling adoption
changes no Agent signing dependency. No native macOS, installed registry-only
Canic, installed IC or full local make ci run is claimed. Blob readiness and
separately selected application/network/effect authority precede a live backup.
