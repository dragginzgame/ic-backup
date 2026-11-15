# Minimal Backup replacement design

Status: proposal for review, 2026-11-15. The maintainer requested a deletion-led
redesign after incremental cleanup failed to address the size of the rewrite.
This document replaces the previous helper-by-helper cleanup plan. It does not
change implemented record contracts, authorize artifact deletion, or claim a new
implementation. The [current handoff](../status/current.md) owns existing behavior;
the [audit](../reports/audits/2026/11/15/complexity/01/report.md) supplies findings.

## Cleanup before feature expansion

Stop extending the current phase APIs. Design the complete selected product first,
then replace a coherent flow and remove its superseded owners in the same change.
The current implementation is a source of algorithms and regression cases, not a
required internal architecture. A facade over all its existing machinery is not
this replacement. No new product code is part of this design review.

The product remains a host library for one already-stopped canister: capture a
complete local snapshot bundle, inspect backup history, and restore that bundle
to the same existing canister ID/network/release, leaving the canister stopped.
Different targets may run independently. Canic owns membership, scheduling,
concurrency limits, application maintenance and settlement of external obligations.
No CLI, cloud storage, pruning, controller changes, canister creation, automatic
restart, cross-release restore or generalized workflow framework is added.

Half of legacy was a rough estimate, not a size requirement. Current code is
10,259 production / 29,474 total Rust LOC; frozen legacy was 15,646 / 44,877.
The design must remove responsibilities and duplicated decisions from the library,
not achieve a percentage by dropping tests, compressing code or moving it into a
supplier. No final LOC prediction is established by this proposal.

## The complete user-facing contract

Names below are semantic sketches, not new Rust declarations. Retain existing
names where they already express these operations; expose no arbitrary journal
writes, management dispatch, phase selection or per-call budget constructors.

| Consumer action | Contract |
| --- | --- |
| Plan capture / restore | Review an immutable target, release, sender/route, trust, application witness and finite overall limits. Restore additionally binds an authenticated source. Pure over admitted inputs. |
| Create | Approve the exact plan once; retain intent and acquire target ownership before effects. An occupied or partial original refuses recreation. |
| Open / inspect | Admit the original operation; show progress and spent/pending authority without signing or network work. |
| Advance | Perform the next eligible work in the fixed capture or restore procedure, with at most one new management submission. No implicit polling or retry. |
| Reconcile | Recover the exact pending request using retained evidence or an explicitly budgeted observation; never resubmit it. |
| List / latest backup | Read completed local receipts. A summary does not establish source integrity. |

The consumer creates an operation and calls advance until it returns a completion
receipt or needs attention. It never calls separate metadata, data, transfer-bind,
assembly, final-status or publication entrypoints. Progress can name a stage for
diagnostics, but that stage is not a command or new authority. An uncertain call
returns its original identity and cause; continuing it cannot replenish spending.
Cold terminal replay is local, effect-free and independent of current time.

Configure the direct connection, original trust, timing policy and bounded external
signer once. The integration also supplies one application-admission interface
that checks the actual action (capture/read/restore allocation/load) against the
original continuous witness. Signing and application permission are distinct;
neither is a success Boolean that substitutes for authentication. Backup derives
exact call arguments, timing evidence and per-call reservations within approved
limits. The consumer does not manufacture certificate observation bytes or
implement one signer/provider trait for every phase.

## Two fixed procedures

Capture:

1. Retain intent and target custody. Authenticate stopped state, exact module and
   actual management caller; inspect snapshot capacity without replacing a snapshot.
2. Take one snapshot. Retain the authenticated original result and snapshot identity.
3. Read its metadata and derive a deterministic bounded transfer schedule. For each
   extent/chunk, retain the exact authenticated result and validate its coverage,
   length and hash. Preserve all supported snapshot metadata, including exact float bits.
4. Materialize the bundle from accepted evidence, verify complete coverage and the
   original module, then perform a distinct final stopped/module/controller check.
5. Close the original completion intent and publish its exact manifest create-only.
   Only verified publication yields a completed receipt. Partial files stay retained.

Restore:

1. Admit the approved source and current completed owner under the established
   target-before-source lock order. Retain a separate source-bound restore intent.
2. Check the target is stopped with the same release/module and admitted controller.
   Require current application rewind authority before allocating an upload snapshot.
3. Allocate one destination snapshot from the original metadata. Retain its exact
   authenticated ID before uploading any data. Upload the original ordered payload.
4. Read back destination metadata and every payload extent/chunk; compare with the
   held source, including empty chunks, complete coverage and exact metadata bits.
5. Recheck stopped target and live rewind authority before loading that exact
   destination. Authenticate load and stopped post-load state, then record a
   LoadedStopped receipt. Application acceptance, external settlement and start
   remain integration-owned; this receipt cannot rewind payments or obligations.

These are two ordinary private procedures. Sharing the call mechanics must not
turn their semantic ordering into configurable stage graphs, callback registries
or an extensible execution engine. Restore upload/load remain unimplemented today.

## Small internal ownership model

| Owner | Necessary responsibility | Existing machinery it replaces |
| --- | --- | --- |
| Store | Private files, locks, immutable intent, one journal per operation, evidence admission and exact completion publication | Per-phase file inventories and publicly composable persistence operations |
| Call recorder | One bounded prepare/submit/result lifecycle, original spending and explicit pending reconciliation | Status/snapshot/metadata/data/restore copies of preparation and submission state handling |
| Capture / restore | Fixed phase prerequisites, canonical arguments and interpretation of results | Separate caller-driven phase APIs, phase-specific providers and nested workflow error families |
| Snapshot / bundle | Platform-backed bounded decoding, deterministic geometry, exact comparisons, held source access | Public transfer-binding and assembly mini-workflows; duplicate derived schedules |

These are semantic owners, not a requirement for four new crates, directories or
traits. Keep one product crate and the existing private qualification workspace.
Platform owns IC DTOs/codecs/crypto; Host owns filesystem/artifact/HTTP primitives.
Do not relocate Backup protocol/persistence complexity into those suppliers.

The journal records facts that cannot be safely reconstructed: original authority,
reserved invocations, exact signed requests, submission reservations, authenticated
results, and local publication intent. Use one private call-state representation
for update calls. A closed step identity distinguishes initial/final status and
transfer ordinals; fixed procedures derive the permitted method and arguments.
Retained evidence binds that identity. No method strings supplied by ordinary
callers and no plugin dispatch registry.

Use the same finite call lifecycle for capture and restore, but keep separate
operation authority and journals. Read-state observation retains its different
request/proof semantics; it must not be disguised as another update submission.
Every pending call gets one original-request reconciliation path, subject to its
original bounded allowance. Do not infer take/upload/load success from plausible
state or invent an unqualified manager receipt. Missing signer output remains
spent and may remain unrecoverable; no second signing attempt is implied.

Eliminate independently persisted facts that can be deterministically derived
from the plan and authenticated results: transfer-binding stage, accepted-data
stage and separate externally visible assembly-verification stage. Derive schedule,
coverage, next step and budgets on bounded reopen; retain those derived facts
within the admitted live operation. An invalid retained result blocks progress;
its presence alone never establishes policy success. A format/algorithm version
binds these derivations. This is a proposed new encoding, not reinterpretation of
old events under changed semantics.

Materialization is a private part of capture completion, not a consumer API. Keep
explicit local write intent before creating payload files, retained partial files,
exact existing-file comparison and closing custody. Freeze original completion
facts/timestamp before manifest publication so a lost return can reconcile exactly.
Derive the manifest from those frozen facts without circular manifest/journal
hashes. Do not delete raw proof merely because payload bytes have been assembled.
The exact replacement event encoding requires review before implementation.

Full evidence admission belongs at reopen and independent trust/custody boundaries.
A live operation may retain already validated state under its held exclusive
custody; this is not permission to trust mtime or assume descriptors make bytes
immutable. Post-await checks and source reads must still detect changed authority
and substituted/corrupt files. Measure growing multi-extent fixtures before
claiming an amortized or throughput improvement. No persistent validation cache.

## Concrete retirement map

Paths below are relative to `crates/ic-backup/src/store/`. Replace and remove means
the current orchestration/API owner goes away after its replacement is qualified;
it does not mean every decoder or safety predicate in that directory is discarded.

| Current code / surface | Decision | Replacement and evidence to retain |
| --- | --- | --- |
| `capture/advance/mod.rs`, `snapshots/`, `metadata/workflow/`, `data/`, `final_status/` phase orchestration | Replace and remove duplicated lifecycle bodies and public advance/inspect families | Fixed capture procedure plus call recorder; pending, prepared continuation, policy rejection and cold replay cases |
| `capture/advance/direct/`, `call/` and `restore/direct/` integration composition | Replace phase-specific provider/signer contracts | One direct signing boundary and action-aware application admission; exact key identity, timing and revoked-permission cases |
| `restore/persistence/status/` status-only lifecycle | Replace, not extend for each upload phase | Restore uses the same private call recorder; separate restore authority and source custody remain |
| `capture/ledger/{metadata,data,final_status}/` phase-specific call states and repeated event families | Replace their call-state encodings | Common call facts plus fixed step identity; finite reservations, no refund and full/partial frame cases |
| `capture/transfer/binding/` and public bind/inspect transfer surface | Remove independent binding stage | Schedule derives from approved plan and retained authenticated metadata; preserve geometry and source-association tests |
| `capture/assembly/` public orchestration and assembly progress states | Remove public workflow; retain materialization checks internally | Completion owns bounded writing, exact comparison and interrupted-file refusal |
| `capture/advance/settlement/`, `lookup/settlement/` call-kind-specific routing | Replace duplicated settlement routing | One pending-call owner; keep update versus observation proof distinction and wrong-request/time-window cases |
| `capture/advance/request/`, `certificate/`, `http/` | Retain necessary primitives, make implementation details private | Exact envelope/certificate/trust/time/body bounds and one-shot HTTP; remove exports used only for caller composition |
| `capture/status/`, `snapshot/`, `transfer/{metadata,data,readback}/` | Retain policy/codec algorithms; narrow public exposure | Complete metadata/coverage/hash/float-bit regressions; remove synthetic full DTOs or wrappers if no consumer needs them |
| `manifest/`, `payload/`, `capture/source/`, `completion/`, `succession/`, `restore/persistence/` | Retain custody/publication algorithms; recombine under store and operations | Original files/locks/source binding, exact receipts, target exclusion, same-store checks and history |
| `capture/plan/` and `restore/mod.rs` planning | Retain immutable approvals; remove unsupported configuration-only execution promises from the replacement API | Bind actual qualified route, source and finite limits; no placeholder grants Canic execution |
| `store/mod.rs` broad re-export list and phase-specific error/progress families | Replace public facade and remove obsolete exports | Operation-level progress/error context with original step/cause; preserve typed failures, no string classification |

Do not keep compatibility aliases, forwarding wrappers or both old/new production
engines merely to avoid deleting callers. Update the real consumer fixture in the
same replacement. Test helpers that only construct removed phase APIs go with them;
retain their distinct behavioral cases at the new owning boundary. One shared
failure matrix should cover shared call mechanics, with phase-specific policy and
actual-IC cases. Tests are not deleted to meet a LOC number.

## Canic boundary and unresolved proof

Canic remains a required integration. The reviewed sibling topology has a manager
controlling workers; an operator signing on the host is not automatically a worker
controller. A smaller API must not quietly change that authority or claim direct
host qualification covers a manager route. See the [sibling review](sibling-integration.md)
and [Canic issue](https://github.com/dragginzgame/canic/issues/1).

A Canic-owned executor must supply a bounded, authenticated join between original
host intent and its retained management result. Its keys, maintenance, two-hop
spending and recovery remain explicit. Do not add that execution variant merely
as a plan enum or opaque provider success flag. The exact manager protocol and
consumer proof remain a design/qualification dependency before claiming the whole
Canic-integrated product is finished; no sibling implementation is authorized here.

## Compatibility and replacement boundary

The current 0.101.13 tree is unpublished and contains local changes beyond pushed
0.101.12. Before replacing its production readers, preserve the exact current
source/locks and reproducible qualified recovery build for any retained operations.
The older pushed version alone is not sufficient evidence for the newer records.
Preparing that preservation does not authorize a commit, push or publication now.

A replacement may use a new explicit format. It must refuse old/unknown stores
intact, not migrate or reset them on open. Existing unfinished operations stay
with their original implementation, authority and artifacts. Use an explicitly
new store for replacement qualification; do not delete old/failed evidence. A
cross-format import is separate work, not a hidden adapter inside the replacement.

Provenance: review frozen legacy `4ae087fad8c23de712aa60bf569bde4b2b932651`, especially
standalone capture/restore composition, shared submission and attempt-journal
projection. History `94b94db89f77968eb4b480ef6d0576d41f6472e2` demonstrates converging
allocation/data-plan/reply ownership. Keep its one-owner/no-blind-retry lessons and
relevant regression cases; leave its catalogs, broad callback API and start/release
workflow behind. No source was copied for this proposal.

## Review and implementation exit criteria

Review this complete shape before writing replacement code. Resolve the public
consumer sketch, exact new journal facts, original-operation preservation and
Canic route dependency explicitly. Do not resume incremental phase additions.

The first implementation unit must replace a complete capture path and switch its
consumer, with the corresponding retirement-map deletions. Do not land a new
abstraction underneath all the old paths and call that the replacement. Keep
necessary shared safety primitives rather than rewriting them for aesthetic purity.

Then complete same-ID upload/readback/load through that same call recorder. Accept
only with an independent consumer demonstrating capture, cold pending/terminal
handling, restore and local history, plus real-IC multi-extent and failure cases.
Compare maintained public surface, production and total LOC, duplicate lifecycle
owners and measured repeated IO. Report any required obligation that remains
unproved. A smaller capture-only tree is not a finished smaller backup product.
