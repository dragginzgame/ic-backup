# Architectural sediment — second pass

## Scope and evidence

Requested repeat audit on 2026-10-18, against commit
`3317b4e6e49c0fe59e343069f68eb539324c0612` plus the incoming uncommitted first-pass
cleanup. That cleanup is preserved. Methods remain Shared Tooling's flow
convergence and module cleanup at `0b351cd2ba2bcd8927b783a5fec89afc8013f708`,
under the root AGENTS overlay. Verdict: **PASS WITH FINDINGS** for the inspected
ownership boundaries, not a storage or full-system qualification verdict.

The first pass's ownership map remains valid: Core owns product records,
bounds, custody and workflow over ports; Agent owns signed one-update transport;
Host owns byte mechanics, filesystem publication and process primitives;
Testkit owns bounded Candid decoder mechanics and simulator lifecycle; Shared
Tooling owns the exact vendored developer tooling. Applications own membership,
release identity, consistency and external-effect settlement. Public entry points
and package relationships are recorded in `docs/ownership.md`.

This pass inspected manifests/imports, artifact and JSON persistence adapters,
regression modules, fixture helpers, Agent package guidance, Make/CI consumers,
tooling fixtures and the prior ownership findings. History at `073b0b1` shows
that the public generic JSON API was removed in 0.28 while test-only wrappers
survived. Broader source searches found no remaining Canic-labelled Rust paths
or compatibility aliases. This is source inspection, not proof that every
public declaration is used by an external consumer.

## Findings and implementation

**LOW — stranded JSON test entry paths.** Two test-only wrappers exposed different
callback shapes for create and replace publication after their generic public
counterparts retired. Their three call sites now use the existing bounded encoder
and publication engine directly. Create/replace select Host's `PublicationMode`
once and share the barrier handling. Production behavior and public contracts are
unchanged. The four process-death combinations and held-parent substitution
assertions remain; no regression case was removed.

**LOW — duplicated FIFO setup.** The record-read regression carried a separate
absolute utility invocation. It now uses the existing local private-FIFO helper,
which verifies both native file type and mode. The bounded wait, missing writer
and device-refusal assertions remain. This is consumer fixture integration, not
a new generic Host capability.

**Retain — product adapters and custody checks.** JSON error projections, private
record permissions, original record limits and publication crash evidence still
exercise Backup's boundary. Generic implementation already delegates to Host.
The Make/CI fixtures exercise actual consumer wiring; snapshot payloads remain
unchanged. Distinct Core/Agent simulator fixtures remain intentional.

**Unresolved — canonical codec extraction.** A fresh GitHub read confirms Testkit
PR #123 is still open; the registry's latest Candid package is still 0.53.0.
Testkit and Host sibling HEADs remain `55ac3974` and `c8cd4345`; their working
sources were read-only evidence. No available replacement justifies deleting
Backup's bounded sequence/reserved visitors. Adopt and qualify a published owner
implementation after integration. Candid remains required for IC management.
Downstream Canic qualification remains outside this repository's edit authority.

## Validation

All 17 JSON persistence tests pass on the locked graph, including real process
termination, FIFO admission, exact byte limits and parent custody. Core library
and test-target Clippy passes with warnings denied. Formatting, local links,
snapshot integrity and diff checks pass. Searches find no surviving calls to the
removed wrappers. No dependency, lock, package version or persisted format changed.

Inputs before this pass and execution logs are retained in
`/tmp/backup-sediment-repeat-1xsydxjl/`. Earlier tests are prior evidence, not newly
executed results. No full local CI, simulator rerun, native macOS qualification,
external application build, release or sibling mutation ran in this pass.
