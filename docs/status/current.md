# Current handoff — 2026-11-01

## Platform/Testkit adoption and authorized delivery

The prepared **0.53.4** batch extends the six accepted simplifications over
**0.53.3** (`75161ea`) with published Platform **0.16.0** and Testkit **0.76.0**.
The user authorized adoption followed by deployment. The established destination
is `dragginzgame/ic-backup`, `origin/main`, and both workspace libraries on
crates.io. The shared direct release runner owns version preparation, complete
release validation, commit/tag and atomic push. Registry upload is a separate
command. Actual release identity belongs to [the generated receipt](../release.json),
not this handoff; publication additionally requires registry readback.

No Backup Rust migration or new product feature is required. Platform's selected
Candid, native management DTO, Principal and Agent identities remain unchanged.
Platform 0.16's owned certification/cryptography implementations are unselected;
its Agent module adds explicit upstream external certificate/hash-tree exports.
Testkit 0.76 source bytes match 0.75.2 and align the dependency declarations.

The incoming lockfile already selected Host Artifacts/FS/Process **0.73.4** and
arithmetic-only Metrics **0.24.4**. Those source archives match their previous
0.73.3/0.24.3 source bytes. The adoption preserves these selections; Host's existing
compatible workspace requirements remain. Types stays **0.15.1**, Candid
**0.10.39**, Agent **0.49.3**, external Principal **0.1.5**, native management DTO
**0.11.0**, Build editor **0.73.1** and authenticated PocketIC **16.1.0**.

## Source review and preparation

Evidence is `/tmp/backup-platform-testkit-deploy.44nom2x5/`. It retains incoming
manifests/lock/handoff/receipt and diff, official registry responses, checksum-
verified old/new Platform and Testkit archives, Host/Metrics archive comparison,
locked metadata, normal production graphs and setup/admission logs. Registry
selection found the non-yanked stable releases above; no broad Cargo update,
sibling source patch or Shared Tooling refresh was performed.

Locked root metadata admits. The Core/Agent normal/build graphs exclude Testkit,
PocketIC, WAT, Metrics, Wire, Clap and Platform runtime guards; Core remains
Agent-free. No independent maintained workspace or second checked-in lockfile
needs adoption. Generated consumer fixtures derive and admit their graph from
the root selection through their existing preparation owner.

The exact locked Testkit CLI/server and release editor are prepared. Offline
`tools-check`, formatting, dependency inheritance/pins, direct IC boundaries and
local documentation links pass (`admission.log`). The complete native release gate
runs against committed original source through `make release-patch`; its retained
validation sidecar and release logs own that outcome. Source edits and dependency
updates must stay paused while validation runs. Publication retains its separate
Cargo admission and confirmed registry observation; never blindly retry an
uncertain push or upload.

## Six completed simplifications included in this batch

Restore safety and fence reconciliation share a private pure comparison in the
original identity/membership/external-obligation/active order. Callers retain
their typed errors, purpose/lane checks and start-only controlled execution.
The upload fixture passes an owned checkpoint between callbacks after dropping
the stage guard. Three upload fixtures share scoped metadata/payload setup with
unchanged caller-specific checksum seeds. Named receipt/plan faults preserve case
order, bytes, spending and provider counts. Connected restore reuses its already-
derived destination while retaining binding-time reconstruction and fresh source
verification. Public APIs, wire/record/hash formats and custody/recovery contracts
are unchanged. All 661 original test names in 151 files are preserved.

That implementation's focused evidence remains
`/tmp/backup-six-cleanup.jilrlpcv/`: strict Clippy, source checks, 54 native cases
and six connected Core PocketIC journeys passed on its then-selected dependency
graph. Original simulator roots are `/tmp/ic-backup-pocketic-1376189-*`.
Those historical checks do not claim fresh qualification of the new selections.

## Preserved earlier evidence and obligations

The preceding seven-change batch completed all **111 tests**: its previously
pending independent-consumer shutdown eventually exited successfully. Evidence
remains `/tmp/backup-seven-cleanup.eM8zqj/`, including the final two-test pass in
`agent-pocketic.log`. Preserve `/tmp/ic-backup-agent-independent-build-4110046-0`,
`/tmp/ic-backup-agent-pocketic-4110046-*` and `/tmp/ic-backup-pocketic-4012022-*`.
Historical observations and dependency scopes are not rewritten.

Earlier evidence remains `/tmp/backup-fixture-next-ten.kMU3t6/`,
`/tmp/backup-host-testkit-refresh.Jqi85C/`, `/tmp/backup-fixture-cleanup.QTkXwS/`,
`/tmp/backup-host-testkit-audit.RP2tAU/`, `/tmp/backup-adopt-metrics-testkit.YY4bZ2/`,
`/tmp/backup-adopt-host-platform.ZA12oo/`, `/tmp/backup-cleanup-ten-next/` and
`/tmp/backup-adopt-upstream.sKsOIs/`. Shared remains `143a9ab` (134 payloads;
VERSION 0.27.21, commit title 0.27.22).

[Implementation contracts](../implementation-contracts.md),
[the Platform boundary](../ic-platform.md) and
[the operator map](../canic-adoption.md#reusable-operator-composition) remain
canonical. Keep original journals, referenced artifacts, simulator/consumer roots,
validation failures and old/failed bundles. Local simulator qualification grants
no generic application/fence/terminal authority. Delivery authorization covers
this repository and its registry packages, not live canister effects, sibling
changes or disposal of retained artifacts.
