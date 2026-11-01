# Current handoff — 2026-11-01

## Accepted seven simplifications

All seven accepted recommendations are implemented over pushed **0.53.2**,
commit `4226eed`. Own versions remain **0.53.2**; the compatible **0.53.3** draft
covers this batch. Changes remain unstaged/uncommitted. No dependencies changed.

Artifact upload and verification fixtures share durable-original preparation.
Each caller retains its original plan, metadata, raw bytes and explicit cleanup.
Download and upload execution read operation sequences from their stored plans.
The immutable constructors pair each request/kind with one ordered operation;
checked ordinal admission remains in the child-plan builder. Existing exact
binding/plan admission, spending, source checks, receipt ordering and checkpoints
remain before/around the same effects.

Download admission uses named consumed/missing/held and metadata/intent/coverage/
read-free cases. Restore safety and fence reconciliation share four named fence
drifts while retaining independent policy errors and caller-specific hashes.
Membership and consistency reuse the existing named context faults. Separate
closed enums describe source corruption and forged-tree scenarios, preserving
exact filesystem mutations and rejection checks. Upload failure expectations use
named spending, applied-operation and provider-call fields instead of tuple slots.

No public API, wire/record/hash format, authority, custody or recovery contract
changed. Original assertions, case ordering, cancellation/drop points and retained
failed evidence remain. All **661 test names in 151 files** are unchanged.

## Fresh focused qualification

Evidence is `/tmp/backup-seven-cleanup.eM8zqj/`: incoming source/handoff/Cargo
copies under `before/`, original test roster, seven-outcome implementation map,
review diff, frozen inputs and logs. Cargo.toml and Cargo.lock are byte-for-byte
unchanged. `frozen.json` binds **377** Rust/manifest/lock inputs.

Strict both-crate all-target/all-feature Clippy passes (`clippy.log`). The initial
test formatting/length diagnostics are retained in `clippy-initial.log`; both
were fixed before the passing run, without lint suppression. Formatting,
inheritance/pins and direct IC dependency boundaries pass (`source-checks.log`).

All **95 selected Core unit cases** pass (`native.log`), covering affected policy,
model, download/upload workflows and real artifact filesystem behavior. All
**14 selected Core PocketIC cases** pass (`core-pocketic.log`): bounded download/
upload success and lost/malformed replies, connected same-ID restore, resume and
effect-free replay. Original simulator roots are
`/tmp/ic-backup-pocketic-4012022-*`.

The real Agent capture/transfer/upload/same-ID restore case passes. The independent
normal consumer built successfully and wrote its authenticated cold capture/
restore acceptance and terminal-replay evidence, but its test process has not yet
exited: owned simulator shutdown is waiting on kernel filesystem writeback.
`agent-pocketic.log` records the distinction; it is not a complete two-test pass.
The pending command is Cargo PID **4079518**, test PID **4110046**, managed server
PID **4154196**. Inspection found the server's remaining `TipSyncfs`/`TipThread`
threads in kernel writeback waits. Do not kill unrelated processes or remove
artifacts to unblock it. Check completion before compiling or changing its source
inputs; resume the tool session **26401** if available.

Independent build/graph evidence is
`/tmp/ic-backup-agent-independent-build-4110046-0`; original Agent simulator roots
are `/tmp/ic-backup-agent-pocketic-4110046-*`. The independent qualification file
is in the `-3` fixture. Offline Testkit admission precedes simulator startup.
Native fakes establish no IC effects. No full CI/release, macOS/MSRV matrix,
package acceptance, installed-IC or live Canic qualification was run.

## Preserved dependency and earlier evidence

Host Artifacts/FS/Process stay **0.73.3**, Testkit **0.75.2**, Metrics arithmetic-only
**0.24.3**, Platform **0.15.3**, Types **0.15.1**, Candid **0.10.39**, Agent
**0.49.3**, external Principal **0.1.5**, management DTO **0.11.0** and Build
**0.73.1**. Shared remains `143a9ab` (**134** payloads; VERSION **0.27.21**, title
**0.27.22**); authenticated PocketIC remains **16.1.0**.

The preceding handoff is preserved under this batch's `before/`. Earlier evidence
remains `/tmp/backup-fixture-next-ten.kMU3t6/`,
`/tmp/backup-host-testkit-refresh.Jqi85C/`, `/tmp/backup-fixture-cleanup.QTkXwS/`,
`/tmp/backup-host-testkit-audit.RP2tAU/`, `/tmp/backup-adopt-metrics-testkit.YY4bZ2/`,
`/tmp/backup-adopt-host-platform.ZA12oo/`, `/tmp/backup-cleanup-ten-next/` and
`/tmp/backup-adopt-upstream.sKsOIs/`. Historical checks retain their original
source/dependency scope; they are not fresh qualification of this batch.

[Implementation contracts](../implementation-contracts.md),
[the Platform boundary](../ic-platform.md) and
[the operator map](../canic-adoption.md#reusable-operator-composition) remain
canonical. Preserve original journals, referenced artifacts, simulator/consumer
roots, validation failures and old/failed bundles. No staging, commits, own-version
changes, release, publication, sibling mutation, live IC effects or cleanup of
retained artifacts occurred. No disposal is authorized.
