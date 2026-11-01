# Current handoff — 2026-11-01

## Accepted fixture simplifications

The user accepted the ten recommendations from the latest Host/Testkit review.
All ten are implemented on top of the previous fixture batch. Own versions stay
**0.53.1** at commit `8d92938`; pending **0.53.2** covers compatible adoption and
both fixture batches. Changes remain unstaged/uncommitted.

Complete-download tests share scoped original metadata/workflow/stage setup and
writer construction. Guards remain borrowed from local owners; partial/durable
states, held journals, explicit successful cleanup and retained failed evidence
stay at each test site. Metadata/data fixture seeds reuse the existing explicit
Applied-journal helper, leaving checkpoint publication separate.

The independent-consumer runner returns status plus its exact stdout/stderr paths;
inspection reads that returned stdout without rescanning directories. Parent
scenarios reuse the consumer's existing Clap command enum and original spellings.
Native root bytes, 0600 create-only logs, process bounds, custody and call counts
are preserved. Application state assertions still make their original fresh reads.

Native fence fixtures select a named capture/restore purpose and own only the
active requirement declarations. Common source layout/record setup and source
reference checks remain explicit. Shared named receipt/context faults retain
case order, caller-specific replacement hashes and independent typed errors.
A narrow reply-write helper returns the synced descriptor so callers retain its
original qualification lifetime. Create/truncate policy, parent-directory sync
and cancellation points remain local. Valid
integrity-test plans use typed declarations; JSON remains for corruption inputs.
The remaining transfer-read golden loader delegates to neutral metadata support.

Only Rust tests/test support changed in this batch. Production APIs, record/wire/
hash formats, original spending, custody and recovery contracts remain intact.
Incoming Cargo.toml and Cargo.lock stayed byte-for-byte unchanged throughout.

## Fresh focused qualification

Evidence is `/tmp/backup-fixture-next-ten.kMU3t6/`: incoming source/handoff/Cargo
copies under `before/`, original test roster, implementation map, review diffs,
frozen inputs and command logs. All ten accepted audit findings have recorded
implementation owners. Earlier diagnostic lint logs remain retained.

Strict all-target/all-feature Clippy passes for both crates (`clippy.log`). All
**95 selected tests** pass: **85 Core unit cases** (`native.log`) and **10 Agent
independent-consumer suite cases** (`independent-pocketic.log`). Core cases cover
scoped download cancellation/failures, metadata/restore/transfer qualification,
capture/restore fencing, typed policy denials and upload fixture seeding.

The independently built normal consumer passes actual PocketIC application
update/replay, reply bounds, retention failures, process death, cold same-ID
capture/restore acceptance and operator partial-output failures. Its candidate
build is `/tmp/ic-backup-agent-independent-build-2558234-0`; original simulator
roots are `/tmp/ic-backup-agent-pocketic-2558234-*`. Private output, originals,
source bytes and failure evidence stay retained. Testkit admits the server assets
offline before bounded managed startup. The suite also runs its existing parser
and evidence unit cases; native fakes establish no IC behavior.

Formatting, inheritance/pins and direct IC dependency boundaries pass
(`source-checks.log`). All **661 test names in 151 files** remain unchanged
(`roster-check.log`). After final review preserved synced-descriptor lifetime,
all **23** affected metadata/download/restore workflow cases passed again
(`retained-reply-workflows.log`), along with strict Clippy and formatting.
Unchanged native/consumer proofs were reused. `validation-phases.json` binds both
phases; final checks bind **376** final Rust/manifest/lock inputs and confirm no
subsequent drift (`frozen.json`, `final-checks.log`).

No full CI/release gate, native macOS/MSRV matrix, package acceptance, installed-IC
or live Canic qualification was run. Focused evidence grants no generic fence or
terminal/reference-release authority. Existing pending/live journals are untouched.

## Preserved dependency and audit evidence

The preceding registry review confirms Host Artifacts/FS/Process **0.73.3** and
Testkit **0.75.2** were the latest non-yanked stable releases at that check. Its
four archives/checksums and **233** cached source files match; the exact Testkit
CLI admits authenticated **PocketIC 16.1.0**. Metrics **0.24.3** was already in
the incoming lock. Its consumed arithmetic source is unchanged from 0.24.1;
optional Wire/browser changes remain outside Backup's graph.

That review is retained at `/tmp/backup-host-testkit-refresh.Jqi85C/`, including
the accepted `audit.json`, strict Clippy, three real Core/Agent simulator cases,
matching normal graphs and original registry/archive evidence. Normal graphs
exclude simulator, Metrics/Wire, WAT, Clap and runtime guards; Core is Agent-free.
Earlier actual Rust **1.91.0** normal-consumer evidence retains that graph scope.

The first accepted fixture batch remains `/tmp/backup-fixture-cleanup.QTkXwS/`
with its **197** focused passes on Metrics 0.24.1 and retained original consumer/
simulator roots. Those results retain their original dependency scope. This
batch's `before/docs/status/current.md` preserves the preceding handoff.
Earlier adoptions remain `/tmp/backup-host-testkit-audit.RP2tAU/`,
`/tmp/backup-adopt-metrics-testkit.YY4bZ2/`,
`/tmp/backup-adopt-host-platform.ZA12oo/`, `/tmp/backup-cleanup-ten-next/` and
`/tmp/backup-adopt-upstream.sKsOIs/`.

Platform stays **0.15.3**, Types **0.15.1**, Candid **0.10.39**, Agent **0.49.3**,
external Principal **0.1.5**, native management DTO **0.11.0**, Build **0.73.1**,
and Shared `143a9ab` (**134** payloads; VERSION **0.27.21**, title **0.27.22**).
[Implementation contracts](../implementation-contracts.md), [the Platform boundary](../ic-platform.md)
and [the operator map](../canic-adoption.md#reusable-operator-composition) remain canonical.
Preserve all backup/simulator/independent-build roots, original Canic journals,
MSRV/release fixtures, validation failures and old/failed bundles. No staging,
commits, release, publication, sibling mutation, live IC effects or cleanup of
retained artifacts occurred. No disposal is authorized.
