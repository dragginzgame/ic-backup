# Minimal Backup replacement design

Status: capture and same-canister restore replacement implemented and qualified
locally, 2026-11-16. Canic process/manager/fence integration remains unqualified.
The [handoff](../status/current.md) owns current evidence and next work.

## Cleanup before feature expansion

Stop extending the current phase APIs. Design the complete selected product first,
then replace a coherent flow and remove its superseded owners in the same change.
The current implementation is a source of algorithms and regression cases, not a
required internal architecture. A facade over all its existing machinery is not
this replacement. The public capture now uses the replacement; the preserved checkpoint remains the reader for its older records.

The product remains a host library for one already-stopped canister: capture a
complete local snapshot bundle, inspect backup history, and restore that bundle
to the same existing canister ID/network/release, leaving the canister stopped.
Different targets may run independently. Canic owns membership, scheduling,
concurrency limits, application maintenance and settlement of external obligations.
No CLI, cloud storage, pruning, controller changes, canister creation, automatic
restart, cross-release restore or generalized workflow framework is added.

Half of legacy was a rough estimate, not a size requirement. Pre-replacement checkpoint code is
10,259 production / 29,474 total Rust LOC; frozen legacy was 15,646 / 44,877.
The design must remove responsibilities and duplicated decisions from the library,
not achieve a percentage by dropping tests, compressing code or moving it into a
supplier. No final LOC prediction is established by this proposal.

## The complete user-facing contract

The capture and restore operations below are implemented. Retain existing
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
or an extensible execution engine. The shared recorder now serves the complete selected restore.

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

### Consumer and direct connection

The public capture consumer follows this loop (pseudocode):

```text
connection = direct(transport, signer, application_guard, limits_and_timing)
plan = plan_capture(target, release, application_witness, connection.profile())
operation = store.create_capture(plan, approved_plan_digest)
loop:
    match operation.advance(connection):
        Progress(view): continue
        NeedsAttention(pending): retain pending; stop
        Complete(receipt): return receipt
```

Reopen takes the store and operation ID, not new authority. Inspection and local
completion verification need no signer object; the latter returns an existing exact
receipt after authenticating its original evidence and payload. Reconciliation is explicit: import an original reply,
or request one bounded read-state observation. It never calls `advance` recursively.
A consumer cannot select a data ordinal, sign a different method, or replenish an
allowance. A fleet runs this same loop independently for its selected targets;
Backup has no fleet scheduler. A typed refusal stops the loop; it is not `Progress`.

The approved intent retains the original public trust-root bytes and binds their
digest. Its direct profile contains transport/network/root commitments, signer
and application protocol commitments, per-invocation
signer/network/byte ceilings, timing durations and snapshot capacity. Existing
operation-wide limits remain cumulative. Backup derives absolute deadlines, exact
ingress expiry and historical certificate windows from a retained invocation time.
A later invocation can use a later time within the same finite approved policy;
resuming an already reserved invocation cannot replace its time or expiry. Profile
mismatch refuses before spending. The application guard receives the fixed action
and original witness; capture inspection, snapshot mutation and snapshot reading
are distinct permissions. Signer input contains exact derived method/arguments and
limits; there is one sign-only interface, not a provider trait per phase.

### Replacement journal facts

This is the current capture encoding, not a decoder for preserved older records.
Use a distinct store marker `ic-backup/host-store/1\n`, intent domain
`ic-backup/capture-intent/1` and journal seed domain
`ic-backup/operation-journal/1\0`. Retain strict canonical JSON and the existing
bounded length/payload/hash frame algorithm. Do not reuse `local-store/1`,
`local-capture-plan/1` or `local-ledger/1` for the changed layouts. Opening an old
store refuses before writing. There is one current reader, no dual-format dispatch.
The capture intent encoder commits the direct profile above; an
independently constructed golden vector remains part of the complete capture change.

A private closed step identity is `InitialStatus`, `ListSnapshots`, `TakeSnapshot`,
`Metadata`, `Data(ordinal)` or `FinalStatus`. It identifies evidence, not a public
command or configurable transition graph. The fixed capture procedure derives the
only legal next step, target, method and arguments from the intent and authenticated
prior results. One unresolved update blocks later steps. Ordinals are contiguous;
there is no retry-attempt counter or arbitrary management method in journal records.

| Fact | Retained fields beyond the common sequence/kind | Admission and recovery |
| --- | --- | --- |
| Created | None | Exact immutable intent seeds the chain. |
| CallReserved | Step, invocation time | Before signing, reserve one signer invocation, its network ceiling, one update dispatch, request/result byte ceilings and all four call frames. Timing and costs derive from the committed profile; overflow refuses. |
| CallPrepared | Step, request length/hash | The exact authenticated signed envelope is durable before this reference. Derive request ID from its bytes; do not persist a second authoritative ID. |
| CallSubmitted | Step | Sync before HTTP entry. Presence means the original dispatch allowance is consumed, even if no packet was sent. Never submit again. |
| CallResult | Step, result length/hash | Retain authenticated original terminal reply/rejection before the reference. Decode and apply phase policy afterwards; invalid policy does not undo settlement. |
| ObservationReserved | Step, invocation time | Explicit reconciliation only, after CallSubmitted with no terminal outcome. Reserve a separate signer invocation, its network ceiling, one read-state dispatch, byte ceilings and four observation frames. At most one observation per call and within the original operation-wide observation ceiling. |
| ObservationPrepared | Step, request length/hash | Exact signed read-state bytes bind the original sender and sole request-status path. Its expiry/windows are distinct from the update's. |
| ObservationSubmitted | Step | Sync before the one read-state HTTP entry; uncertainty never reissues it. |
| ObservationResult | Step, certificate length/hash | Retain the exact authenticated observation, including unknown/nonterminal outcomes. A terminal outcome resolves the original call by derivation, without copying reply bytes or another settlement event. |
| MaterializationStarted | None | After every payload result passes policy, reserve local completion frames and manifest allowance before creating bundle members. |
| PublishReady | Completion time | After exact payload verification and the distinct final status, freeze completion time and close the journal. Reconstruct one exact manifest from this closed prefix and payload. |

Each evidence reference determines one private canonical filename from step and
role. No caller-supplied paths or independent persisted evidence catalog. Bounds
remain 16 KiB/frame, 65,536 frames and 128 MiB journal, additionally constrained by
original evidence limits. Reserved evidence includes framing and proof bytes, not
only decoded payload. No successful result refunds the reserved maximum. Before
snapshot take, admit the fixed remaining capture overhead; after metadata, admit
the full derived data schedule before its first request. An unexpectedly large
snapshot may therefore stop after metadata; it never grants more authority or
permission to delete that snapshot. A real-IC full 1 MiB whole-chunk round trip now qualifies an explicitly
approved 2 MiB request / 4 MiB result profile with a 2 MiB certificate ceiling.
Old intents retain their original smaller limits; no authority is enlarged.

A call has one terminal result source: its original reply or a terminal observation.
Once resolved, a second import cannot replace it. Unknown/nonterminal observations
leave the update pending; an original authenticated terminal reply may still settle
it. An interrupted observation reservation stays spent, including after a later
reply settles the update. There is no requirement to invent a successful lookup
or a new signature before using that independent original reply.

### Crash boundaries and removed states

A durable signing reservation without retained signer output is not permission to
sign again. A complete prepared request with no submission reservation can submit
once after current admission and original deadline checks. A submitted call without
a terminal result returns `NeedsAttention`; `advance` never polls or retransmits.
Read-state uses the same custody discipline but its own proof/time semantics.

Retain each evidence file create-only, sync it and its directory, then append/sync
its reference. An uncertain write invalidates the live handle. Missing referenced
files, partial frames, changed bytes and unreferenced files refuse on reopen; no
automatic orphan adoption, truncation, cleanup or refunded allowance. A surviving
reply supplied for reconciliation must bind the original request and authority;
it does not authorize overwriting an orphan file or repairing the journal.

Delete transfer-binding, data-accepted and assembly-verified events. Schedule,
coverage and semantic validity derive from original authenticated results under
the intent's algorithm domain. A retained malformed/invalid result blocks the
next step even though its call is settled. Materialization starts only after those
checks; partial files stay retained, exact complete files may be reused, and missing
members may be created under the original intent. Reverify the assembled payload
before final status and publication. No separate persisted assembly digest is needed.

`PublishReady` freezes the manifest inputs before publication; the manifest may
commit the closed journal hash, but the journal cannot commit that manifest hash.
Exact create-only publication can reconcile without a new timestamp, signature,
guard or network request. Read-only completed replay also performs no publication
or synchronization. Target succession requires the exact published completion,
not just PublishReady. Old unfinished operations retain their original owner.

Keep independent full admission on reopen and at source/custody boundaries. Any
live-state reuse must still detect changed bytes/authority after awaits; held locks
and descriptors do not make files immutable. No persistent cache or throughput
claim. The replacement must measure multi-extent IO before claiming improvement.

Restore uses these same facts with source-bound intent and its fixed allocation,
upload/readback/load/status steps. Readback and load policy remain separate from
the shared recorder's spending/custody mechanics. LoadedStopped is the sole restore
local completion footer; no separate receipt or restored payload tree is persisted.

### Capture replacement qualification

The [common recorder](../../crates/ic-backup/src/store/operation/mod.rs),
[fixed procedure](../../crates/ic-backup/src/store/capture/procedure/mod.rs) and
[immutable intent](../../crates/ic-backup/src/store/capture/intent/mod.rs) are now
the selected public capture. The [capture contract](../architecture/capture-intent.md)
owns exact formats, original evidence/spending, custody, observation and completion.

The public store facade creates/reopens the approved intent. One advance loop
derives all calls and local work. Original reply import takes raw certificates;
explicit observation has separate original timing and finite allowance.
Inspection/completed replay need no signer, guard, current time or transport.
Completed receipts survive authenticated target succession; missing/staged
receipts cannot excuse a changed owner. Older held sources and local restore
create/open now consume this public boundary synchronously.

The superseded phase orchestration, phase event families, transfer-binding and
data-acceptance records, assembly workflow, settlement routers and old consumer
fixtures are deleted. Existing Platform codecs, signatures/certificates, Host
one-shot HTTP, closed payload hashing and manifest publication remain owners.
Test-only call wrappers retain exact existing protocol regressions, not a second
production lifecycle. Restore status-only persistence and its signer/adapter/event
families are deleted; the actual restore consumer uses the same recorder.

Native coverage includes complete/pending/prepared recovery, partial/orphan/changed
files, exact materialization/publication, finite limits and certified policy
failure/rejection. An acknowledged process is killed after a durable signing
reservation; reopening preserves original intent/claim/spending without effects.
Shared framing boundaries, strict request/certificate/HTTP tests and bounded
metadata/data/readback algorithms remain tested with their owning code.

The public-API IC fixture drives multi-extent module, heap, stable memory
and empty/nonempty chunks through publication, succession, older-source restore
through full upload/readback/load, capture after restore, local history and offline replay. Separate actual-IC cases recover an
interrupted take via one observation and a payload call via surviving original
certificate after shutdown. Fixture keys/permissions qualify no Canic process,
production maintenance or physical power loss. The current Linux run establishes
no hosted macOS or throughput improvement. A separate normal-dependency executable
is built alone without Testkit/PocketIC and continues capture/restore in new
processes, then replays completion after backend shutdown.

Focused legacy anchors remain frozen at
`4ae087fad8c23de712aa60bf569bde4b2b932651`: submission/HTTP recovery,
attempt journal, snapshot coverage and artifact completion. History
`d245b0f03a505ba5215d65c10bdebf1890c7dc37` preserves original-request binding,
`a860d14421b9ff1c61c1dce079cdc6a19316bfd0` separates observation/effect
admission, and `9f7b886a824eec67dda8f81792a560436419bb2a` preserves original
verification-pass ownership. Provenance, changed assumptions and exclusions are
recorded in the capture/transfer contracts. No legacy implementation was copied.

### Size, maintained surface and repeated IO

The preserved pre-replacement checkpoint measured 10,259 production / 29,474 total
Rust LOC. At the restore-reduction checkpoint before ID-only cold reopening, the tree measured 7,795 production /
19,891 total: 2,464 fewer production lines (24.0%) and 9,583 fewer total (32.5%).
Toko's classifier includes product and qualification workspaces, excludes comments/
blanks and reports no unclassified code. The reduction comes from retiring phase
owners and repeated fixtures. Compared with frozen legacy's 15,646 / 44,877, the
current counts are 49.8% / 44.3%; functionality differs because Canic manager/fence
integration remains unfinished. These are not equivalent-feature costs.
Public stage families are retired. Capture and restore share one connection,
sign-only adapter, action guard, progress and typed view; original proof/admission
records and unused status-stage progress are private or removed. No compatibility
engine or public phase commands remain. The restore cut adds 154 net production
lines while removing 1,711 total lines from the immediately preserved 7,641 /
21,602 tree, and implements the complete actual-IC restore consumer.

Linux strace of the successful full IC consumer (two 24-call captures, succession,
older source, restore status and offline replay) measured host-store regular-file
IO only: 346,761,555 certified-result bytes read from 1,107,724 retained bytes;
36,771,333 payload bytes read from 524,368 retained bytes; 11,070,177 journal
bytes read from 36,147 final capture-journal bytes. This includes all admission,
source, completion and replay checks in that consumer, not one isolated advance.
Raw trace and per-file summary remain in `target/evidence/capture-replacement-runtime/`.
No baseline trace or throughput comparison exists; strace adds timing overhead.

The follow-through removes immediate duplicate append scans and full historical
projection after retaining a new live result. Each append re-admits its exact
original prefix before writing, verifies expected new length/head on closing and
returns that checked journal. A live invocation extends its already-authenticated
procedure using owned evidence authenticated at its network/import boundary.
Certified rejection has one procedure owner. No validation survives an invocation
as a cache; original bytes/authority remain rechecked across awaits and cold
open/source boundaries authenticate independently.

The preserved equivalent two-capture/status-only-restore IO trace after that reduction reads 227,756,038 certified-result
bytes, 7,429,093 capture-journal bytes and 34,936,045 payload bytes: approximately
34%, 33% and 5% fewer respectively. Sync counts remain unchanged. Same-length
prior-evidence/payload mutations during HTTP and stale/false-write prefix tests
refuse. Evidence is under `target/evidence/reduction-followthrough/`.
Source/payload scans remain substantial; this is IO reduction, not a throughput
claim. Further reuse must preserve independent source admission and closing custody.

After Host 0.80.2 adoption, `operation/custody::Recorder::read` now uses the
identity returned by Host's bounded read instead of hashing its returned bytes a
second time. Exact declared length and expected digest are still compared before
the independent closing file-identity check. No reads, custody checks or journal
admissions are removed, and no cache is added; replay remains quadratic.
An optimized Linux component probe uses deterministic 4 KiB, 1 MiB and 4 MiB files,
warm page cache and seven alternating-order paired rounds. Median read/admit/hash
time decreases by 22.7%, 43.5% and 44.1%, respectively. This excludes custody/stat
checks, certified replay and IC effects; it is not an end-to-end backup speedup.
Probe source, lock, raw timings and focused validation are retained under
`target/evidence/replay-hash-review/`.

Focused legacy review at `4ae087fad8c23de712aa60bf569bde4b2b932651` revisited
`ops/persistence/download_journal/ic_snapshot_artifact/verification/mod.rs`
(`checksum_child_with`, `checksum_exact_reader`) and its `tests/mod.rs` cases
`exact_stream_identity_and_shape_errors_keep_product_contract`,
`exact_stream_preserves_original_io_error` and
`callback_reads_reject_same_length_replacement_and_retain_both_files`.
History `9f7b886a824eec67dda8f81792a560436419bb2a` consolidates verification's
read/hash pass while preserving closing custody. Backup applies the same ownership
principle to Host's complete bounded identity; it retains its own expected journal
reference and held descriptor rather than legacy's callback/download lifecycle.
The owning custody tests cover changed/replaced/aliased evidence and now include
truncation, growth, wrong declared length and wrong digest. No legacy code was
copied; shared-reader IO faults remain the existing payload/Host tests' concern.

A subsequent replay pass removes duplicate request-signature verification for
calls with original replies; result certificates still bind the same owned,
already-authenticated request. Both before/after optimized normal consumers
include the hash reuse above. Nine alternating-order paired runs of offline
open-plus-inspect on the same retained multi-extent IC fixture measured median
capture 184.02 to 180.54 ms and restore 614.34 to 605.98 ms. These small local
differences (1.9% / 1.4%) are not a general throughput guarantee. Request
verification count falls from two to one for that path; retained reads and
independent custody checks remain. Raw timings, binaries and fixture selection
are in `target/evidence/replay-auth-review/`; original-request provenance and
rejection coverage are in [the capture contract](../architecture/capture-intent.md#focused-legacy-review--2026-11-15).

Evidence lookup now takes references from the selected call instead of walking
the journal and allocating every preceding filename. One name-indexed map owns
the existing held descriptors and closed-directory membership, replacing the
linear held-file list and temporary name list. Each reference selection is
constant-time and held-file lookup is logarithmic; all byte reads, hashes and
independent admission boundaries remain. Overall replay is still quadratic.
Nine alternating-order offline open-plus-inspect pairs, after native tests had
finished, measured capture 179.93 to 180.59 ms and restore 608.33 to 605.16 ms on
the same fixture: no clear end-to-end speed improvement. Keep the simpler direct
ownership and reduced lookup/allocation work without a throughput claim. Evidence
is in `target/evidence/replay-index-review/`; the preliminary timings collected
alongside native tests are retained separately and excluded from those medians.

The larger-payload follow-up removes the early byte comparison during read-only
materialization; the full closing tree hash still compares every member against
commitments derived from original evidence. Creation retains its admission pass
before any missing file is written. On the retained 20 MiB capacity fixture,
optimized offline open-plus-inspect reads 220 to 140 MiB of payload for capture
and 500 to 380 MiB for restore, measured with `strace`. Original evidence, journal
and other store-byte totals are identical. This saves 80 / 120 MiB, or 13.8% / 8.1%
of all store bytes read by those commands. These are logical reads, not physical
disk traffic. Three alternating-order paired runs without tracing measured median
capture 963.43 to 959.29 ms and restore 2570.78 to 2562.18 ms: no meaningful speedup
is established. Evidence replay remains the larger unchanged read cost; overall
history replay is still quadratic. Sources, before/after consumers, exact original
fixture selection, timings and traces remain in `target/evidence/replay-payload-review/`.

### Completed restore cut and remaining work

The new [restore contract](local-store.md#immutable-restore-approval-and-original-custody)
owns exact implemented intent, fixed ordering, source custody, finite allowance,
readback/load, LoadedStopped and capture-after-restore behavior. The real consumer
switched in the same batch that deleted old restore status signing/submission,
adapters and event encodings. `store/operation/` physically owns the shared
recorder; there is no forwarding recorder under capture. One private sealed intent
view serves two concrete procedures, without a public provider registry or graph.

The exact source/lock/build reader immediately before this cut is preserved under
`.git/recovery/host-store-1-before-restore-20261116/`, with independent actual-IC
rerun, before its restore encoding changed. The earlier b7c2c6a reader alone cannot
read these newer capture sources. Preservation grants no commit, push, migration
or cleanup authority.

The completed Backup-owned public boundary also reopens restore from retained ID
and approval, re-admitting exact intent/source and held target lock without new
credentials. Fresh native/IC evidence and remaining qualification limits are in the handoff.
Further cuts must preserve original source authentication and closing custody.
The remaining product dependency is Canic's original-result executor join and
continuous application/rewind fence, then its actual consumer qualification.
Root's two-hop effect journal protects a separate boundary and cannot be removed
as a duplicate of Backup's host recorder. No sibling changes were made here.

## Concrete retirement map

Paths below are relative to `crates/ic-backup/src/store/`. Replace and remove means
the superseded orchestration/API owner has been removed with its qualified replacement;
it does not mean every decoder or safety predicate in that directory is discarded.

| Current code / surface | Decision | Replacement and evidence to retain |
| --- | --- | --- |
| `capture/advance/mod.rs`, `snapshots/`, `metadata/workflow/`, `data/`, `final_status/` phase orchestration | Replace and remove duplicated lifecycle bodies and public advance/inspect families | Fixed capture procedure plus call recorder; pending, prepared continuation, policy rejection and cold replay cases |
| `capture/advance/direct/`, `call/` and `restore/direct/` integration composition | Replace phase-specific provider/signer contracts | One direct signing boundary and action-aware application admission; exact key identity, timing and revoked-permission cases |
| `restore/persistence/status/` status-only lifecycle | Replaced and removed | Restore uses the same private call recorder; separate restore authority and source custody remain |
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

The maintainer accepted this direction and authorized preservation and replacement.
Preservation is complete at local reference `preserve/backup-0.101.13-before-replacement`
(commit `b7c2c6ae754a5ae2452e65b9be537b517614ebd2`); see
[the retained build reference](../development/setup.md#pre-replacement-reference).
The consumer loop and replacement journal facts above now specify the capture
cut, now implemented with the complete restore consumer. The Canic route dependency remains explicit. Do not resume
incremental phase additions or request the same design approval again.

The first implementation unit must replace a complete capture path and switch its
consumer, with the corresponding retirement-map deletions. Do not land a new
abstraction underneath all the old paths and call that the replacement. Keep
necessary shared safety primitives rather than rewriting them for aesthetic purity.

Same-ID upload/readback/load now uses that recorder, with an independent actual-IC
consumer demonstrating capture, cold pending/terminal handling, restored bytes and
local history. Canic must still demonstrate its actual manager/fence consumer.
Compare maintained public surface, production and total LOC, duplicate lifecycle
owners and measured repeated IO. Report any required obligation that remains
unproved. A smaller capture-only tree is not a finished smaller backup product.
