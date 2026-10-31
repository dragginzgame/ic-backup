# Current handoff — 2026-10-31

## Current cleanup batch

The next ten approved simplifications are implemented on user commit
`2052263508cefb5a164937a3d3fdcf80c21b2e82` (title **0.52.3**).
Own workspace/package versions remain **0.52.2**; notes extend draft **0.52.3**.
This batch and the earlier Agent item-order fix remain unstaged/uncommitted.
No release, publication, live IC operation, sibling mutation or cleanup occurred.

Valid workflow fixtures now use typed plan, operation, graph and context
constructors. Core PocketIC declarations and dispatch name the original stages,
dependencies and ceilings. Retained capture reconstruction returns named evidence
instead of a tuple. Native fixtures share passive reply and explicit Applied-receipt
construction; complete-download faults distinguish lost and malformed replies.
Fault injection, cancellation points and independent effect/state assertions stay
with each scenario. Native substitutes qualify persistence, never IC effects.

The independent consumer shares one freshly opened source layout/plan scope across
destination lookup, upload planning and execution. Journals borrow that held scope;
all original metadata reads, admission, manifest verification and guard lifetimes
remain. This introduces no cross-call cache or source/dispatch authority.

Private learned-stage admission returns one typed failure instead of nested
boolean/optional success; callers preserve their existing public error variants.
Artifact member declarations own name/length/maximum admission through the existing
bounded no-follow reader. Existing digest helpers replace manual integer/checksum
appends with identical domains, widths and field order. Public APIs, schemas,
wire/hash bytes, original spending and recovery semantics are unchanged.

## Fresh focused qualification

Evidence and original incoming inputs are retained under
`/tmp/backup-cleanup-next.s8TMqt/`. The current source/configuration manifest is
`source-inputs.json`; `incoming.lock` and `incoming.patch` preserve the starting
lockfile and prior dirty work. Cargo manifests, lock, Shared snapshot and toolchain
are unchanged. Final formatting, links, repository-file/license, diff and frozen
input checks are recorded in `final-checks.log`.

Strict Clippy passes Core's test targets, including its library-test configuration,
and Agent's library/transport integration target: `core-clippy-recheck.log` and
`agent-clippy.log`. Initial missing/unused fixture-import diagnostics remain in
`core-clippy.log`; no lint suppression was added.

All **136 selected Core unit tests** and **seven public cases across six targets**
pass: `native.log`, `check-native.sh` and `public.log`. These cover the changed model
hashes/bindings, native workflow fixtures, cancellation and explicit receipts,
source drift, exact lengths/checksums, publication recovery and bounded reads.

All **13 selected real Core PocketIC cases** pass: `core-pocketic-local.log`.
They exercise original workflow execution, lost capture/data/allocation/upload/
load/start replies, cold retained-prefix reconstruction, same-ID round trip,
continuation at allocation/load/start and effect-free completed replay.
Original evidence is `/tmp/ic-backup-pocketic-3113451-*`.
`catalog-review.json` independently compares retained declarations against the
original exact IDs, dependencies, request commitments, inventory and 66/2 capture
and 67/0 restore budgets. The interrupted sandbox run is retained separately in
`core-pocketic.log` and `/tmp/ic-backup-pocketic-1271-*`; loopback binding was denied.
The passing rerun used local-server access, not a different backend or admission.

Agent's **three native consumer-input/evidence cases and three real PocketIC
journeys** pass: `agent-pocketic.log`. They cover cold capture/restore/terminal
replay, death after authenticated load without redispatch, and the operator's
three retention/partial-output failure modes. The independent candidate normal
graph/build is `/tmp/ic-backup-agent-independent-build-3142998-0`;
original journey evidence is `/tmp/ic-backup-agent-pocketic-3142998-*`.

No full CI gate, fresh MSRV/macOS matrix, registry-only release acceptance,
installed Canic migration or live qualification was run. These focused results do
not establish complete-product, application-fence or terminal-release authority.

## Retained dependency scope and prior evidence

The prior cleanup/adoption base remains release **0.52.2** at
`d8c3c1669b127b60d0bda68c58a3b343f015855b`. This batch changes no dependency selection:
Platform **0.13.0**, Testkit/Host/editor **0.72.1**, Candid **0.10.39**, Agent
**0.49.3**, external Principal **0.1.5**, native management DTO **0.11.0**, PocketIC
**16.1.0** and test-only Clap **4.6.7** remain. Testkit retains Platform **0.11.1**
in development; published library normal graphs exclude Clap/simulator packages.
Shared Tooling **0.27.12** remains revision
`69a8b92a1a892b17d355ec853ca63faa9599a928`, with all **132 payloads** intact.
[The adoption review](../shared-tooling-review.json) retains its original scope.

The complete preceding handoff is retained as `prior-status.md` in this batch's
evidence directory. It preserves the earlier 111 selected native/nine simulator
results, Rust 1.91 independent consumer proof, dependency review and Agent item-order
qualification. Those historical results retain their original source scope.
Earlier cleanup evidence remains `/tmp/backup-cleanup.KZGm6J/`; the original Clippy
failure remains `target/validation-failures/20261031T103413Z-2044126-8-clippy.log`.

[Implementation contracts](../implementation-contracts.md) and the
[operator map](../canic-adoption.md#reusable-operator-composition) remain canonical.
Preserve all prior `/tmp/backup-*`, simulator/independent-build roots, original Canic
journals, `target/msrv-consumers.*`, `target/shared-release-tests.*`, validation
failures and old/failed tool bundles. This handoff authorizes no disposal.
