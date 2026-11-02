# Current handoff — 2026-11-02

## Upstream adoption, pending 0.55.0

Platform **0.18.0** and Shared Tooling **0.27.27** at
`651f5f7acb2e3055abd51ba5f835a2112801666a` remain latest at selection and are
reverified. Adopted published Testkit **0.79.0** and its arithmetic-only Metrics
**0.26.1** dependency. Testkit now shares Platform/client 0.18, canonical Types
**0.17.0** and Host Artifacts/FS/Process **0.74.1**. Remove the temporary simulator
alias and old Host 0.73, Platform/client 0.17, Types 0.16 and Metrics 0.25 stack.
Platform's `pocket-ic` feature remains development-only. Optional Wire and runtime
features remain disabled. Both suites retain authenticated **PocketIC 16.1.0**.

The new published simulator client uses canonical management DTO **0.11.0**;
its old 0.8 supplier leaves the lock. Production Candid, management DTOs, external
Principal **0.1.5** and Agent **0.49.3** remain unchanged. Scoped resolution adds
only the new Testkit/Metrics/client and target-specific Runtime generation;
existing package checksums remain unchanged. No sibling patch was added.

The earlier Host 0.74.1 and release editor Build **0.74.0** adoption remains.
Own versions stay **0.54.0**; pending notes remain **0.55.0** because public Host
error identities move from 0.73 to 0.74. Consumers exchanging those types must
align direct Host dependencies. The minimum and pinned compiler remain
**Rust 1.99.1**, with actual rustc/Cargo identities recorded before qualification.

## Shared Tooling

All **144** canonical payloads are rechecked byte-for-byte and by executable
mode against the independently selected latest commit. The previous canonical
refresh added three and updated six files, including an explicit exporter root
for the linked validation-runner guide. No vendored payload is patched or
re-exported by this Testkit update. Common executable catalogs remain unchanged.

Backup retains its complete positional CI/release gate, coordinated release
group and mandatory exact resume receipt check. The optional group-plan mode
remains disabled. [The review](../shared-tooling-review.json) and
[consumer guide](../shared-tooling.md) record provenance and decisions.

## Focused qualification

Latest evidence is `/tmp/backup-platform-testkit-refresh.lw0kvf7w/`: incoming
sources/diff, registry selections, authenticated archives/VCS identities, source
comparisons, frozen inputs and successful qualification logs are retained.

- Explicit setup and offline compiler/tool/inheritance/boundary admission pass.
  Locked Testkit 0.79 CLI preparation retains the original authenticated server
  and existing Build editor (`setup.log`, `admission.log`).
- Strict both-library all-target/all-feature Clippy passes
  (`clippy-corrected.log`). The initial stale-alias import failure is retained;
  successful qualification uses refrozen corrected inputs.
- Both independent normal consumers pass on Rust 1.99.1, including exact Host
  error probes, Core's Agent exclusion and simulator/WAT exclusion. Evidence is
  `target/msrv-consumers.qt1xpQ` and `production-graph-proof.json`.
- Both exact normalized archives compile together with graph/license/digest
  admission (`consumers-package.log`, `target/package-check.jPV73u`).
- All six connected Core PocketIC cases and the real Agent capture/complete
  transfer/upload/same-ID restore pass (`core-pocketic.log`, `agent-pocketic.log`).
  Actual Testkit setup/check routing and refusal fixtures pass separately.
- All 854 captured source/lock inputs remain unchanged across qualification.
  Review/handoff edits afterward receive focused formatting/link/snapshot checks.

Earlier Host/filesystem, exact codec, sixteen HTTP, consumer runner and twelve
Build editor preparation checks remain at `/tmp/backup-upstream-refresh.ti730kq7/`;
those unchanged boundaries are not presented as freshly rerun here. This is
focused Linux qualification, not a full CI/release gate, native macOS, installed
Canic or live IC acceptance. No staging, commit, own-version bump, release,
publication, sibling edits or cleanup occurred. Preserve journals, referenced
artifacts, unfinished recovery, tool generations and failed evidence.

## Preserved earlier work

The four accepted simplifications remain intact: common journey driver
construction, fresh restore-source contexts, one Agent routing/reply-binding
decision and six named round-trip scenarios with separate registrations.
Incoming evidence is `/tmp/backup-four-simplifications.tm9_9hks/`. Original digest
domains, attempt identities, bytes, recovery and effect-free replay remain.

The completed **0.54.0** delivery and issues #76/#77 closure remain owned by
[the release receipt](../release.json), GitHub and
`/tmp/backup-deploy-054.qme75qrm/`. That delivery does not authorize releasing this
pending batch. The incoming handoff is retained in the latest evidence `before/`.

[Implementation contracts](../implementation-contracts.md),
[the Platform boundary](../ic-platform.md) and
[the operator map](../canic-adoption.md#reusable-operator-composition) remain canonical.
