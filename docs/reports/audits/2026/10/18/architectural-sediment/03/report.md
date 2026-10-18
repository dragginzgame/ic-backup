# Architectural sediment — repeat review

Date: 2026-10-18. Scope: current working tree over released 0.31.0 at
`3317b4e6e49c0fe59e343069f68eb539324c0612`, including incoming uncommitted Platform,
Host, Testkit and Shared Tooling adoption. Before-images and command receipts:
`/tmp/backup-audit-49lj1dvx/`. This is a structural ownership review, not a new
storage-correctness, security, performance or application qualification verdict.

Methods: the snapshot's [flow convergence](../../../../../../../../audits/flow-convergence-and-duplication.md)
and [module cleanup](../../../../../../../../audits/module-cleanup.md), under the
local overlay and product contracts. The snapshot source is Shared Tooling
0.24.0, `cab12cd3dfa4fdb34846b25397f0d72881bb9ae0`.

## Boundary and evidence

[The ownership map](../../../../../../../ownership.md) describes public entrypoints
and delegated capabilities. Two host libraries are published: Core owns bounded
records, accounting, snapshot layout, guarded persistence, reconciliation and
coordination; Agent depends on Core and owns one reserved signed update after
mandatory durable retention. No CLI, canister or complete application runner is
implemented. Integrations retain fresh authority, membership, release identity,
quiescence and external obligations.

Manifest/import traces confirm Platform 0.1.8 owns the Candid, Principal,
management DTO and Agent dependency boundary. Host owns generic hashing,
descriptor IO/publication and process custody. Testkit owns simulator assets and
managed startup; it remains development-only. Shared Tooling owns common tooling
and its implementation regressions. Inspected manifests, source, tests, fixtures,
golden generators, Make/CI/release wiring and active documents agree with these
boundaries after the corrections below. Historical source/release documents
remain evidence of their named versions, not current dependency instructions.

Published registry sources determine usable APIs. Read-only sibling evidence:
Host at `70b553e0`, Platform's published source `57fcf8d4`, Testkit at `55ac3974`,
Wire at `ce56d5eb`, Canic at `5583496b` and clean committed Shared source at
`cab12cd3`. These observations include working-tree content and do not qualify
unpublished sibling changes. Canic's actual `crates/canic-backup/src/artifacts`
consumer already calls Host FS and projects into Backup records/errors; its
application runner and integration acceptance remain Canic-owned.

## Findings and implemented disposition

1. **Producer qualification retained downstream.** Thirteen exported generic
   test roots have no executable local caller. A fourteenth, the dependency-checker
   matrix, runs under `tooling-check` but tests the Shared implementation rather
   than Backup's policy or Make wiring. Retire these roots with the canonical
   exporter, which also retires two unused companions. All 114 remaining payloads
   retain identical bytes and modes at the same revision. The
   [selection receipt](../../../../../../../shared-tooling-review.json) lists every
   retirement; no vendored file was hand-edited. This removes unused distribution
   obligations and duplicate producer qualification, with no runtime or public
   contract change. No speed-up is claimed.

   Backup still runs the canonical checker on its actual policy in CI, plus
   independent production-consumer admission. Retain generic fixtures that its
   local failure-retention test invokes dynamically, and the release runner fixture
   used with its own selected invocation. Actual Make/Testkit routing, hooks,
   release/resume and evidence contracts remain consumer qualifications. The
   focused tooling gate proves the retained caller closure still executes.

2. **Superseded active ownership guidance.** The current ownership review still
   claimed a clean starting tree and blocked codec extraction. The implementation
   reference described a removed raw-stream adapter, old pending dependency lines
   and superseded Host publication methods. Correct these descriptions against
   current source and manifests, retaining published changelog history. Update
   the Canic note to its observed direct Host implementation. No API is restored,
   dependency changed or migration fallback introduced.

## Deliberate retention and limits

- Product bounded Candid visitors and the strict reserved marker enforce Backup
  limits. Platform owns generic decoding; these are necessary projections.
- Local descriptor/extent/closed-tree checks and journal/reference admission
  defend different corruption and custody boundaries. Generic Host traversal
  does not replace those product obligations. Retained-file sync and streamed
  create-only effects keep their current error and recovery behavior.
- Agent's explicit Reqwest client enforces no proxy, redirects or HTTP retries;
  single dispatch and mandatory ingress/reply custody stay local.
- Core/Agent PocketIC suites qualify different integration boundaries and WAT
  fixtures. Independent Perl goldens intentionally avoid the production encoder;
  their shared primitives already live in `SnapshotGolden.pm`. No generated Rust
  dependency path or independent maintained Cargo workspace requires migration.
- Shared snapshot copies are intentional distribution. Only roots without a
  distinct consumer proof were retired; owner implementations remain upstream.

No further product deletion or ownership move was justified by the inspected
callers. Application acceptance and deployment artifact admission stay with the
consumer; introducing a deployment gate into these host libraries would assign
it to the wrong owner. Native macOS, installed IC and Canic builds were not run.

## Verification

Fresh focused tooling, formatting, direct-dependency boundary and pin checks,
locked Cargo metadata, snapshot integrity, local links and diff whitespace pass.
Before/after checks confirm Rust, manifests, golden/WAT fixtures and the Git index
are unchanged. A concurrent lockfile edit changes Host Artifacts/FS/Process from
0.27.2 to 0.27.3; it is preserved and excluded from this cleanup. Existing 634 native
tests, strict Clippy/build/Rustdoc, Rust 1.91 independent consumers and package/identity
checks apply to their original Host 0.27.2 graph, not the concurrent update. Host
0.27.3 product qualification remains separate. No full local CI, package
version change, commit, release, publication, deployment or artifact cleanup ran.
