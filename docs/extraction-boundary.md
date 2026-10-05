<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-backup/ic-backup-readme-header.svg" alt="IC Backup — Verified backups and safe recovery for Internet Computer apps" width="100%">
</p>

<!-- helper-navigation:start -->
<p align="center">
  <a href="https://github.com/dragginzgame/canic"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/canic.svg" width="18" height="18" alt=""> <strong>canic</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/icydb"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/icydb.svg" width="18" height="18" alt=""> <strong>icydb</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-timers"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-timers.svg" width="18" height="18" alt=""> <strong>ic-timers</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-memory"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-memory.svg" width="18" height="18" alt=""> <strong>ic-memory</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-query"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-query.svg" width="18" height="18" alt=""> <strong>ic-query</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-backup"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-backup.svg" width="18" height="18" alt=""> <strong>ic-backup</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-blob-storage"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-blob-storage.svg" width="18" height="18" alt=""> <strong>ic-blob-storage</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-testkit"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-testkit.svg" width="18" height="18" alt=""> <strong>ic-testkit</strong></a>
</p>
<!-- helper-navigation:end -->


# Implemented extraction boundary

This is the detailed reference for behavior already implemented in `ic-backup`.
It distinguishes locally qualified mechanisms from integration responsibilities
and planned product work. Start with the [project overview](../README.md) for the
plain-language product concept or [current status](status/current.md) for the
active implementation baseline.

## At a glance

| Area | Implemented here | Does not yet establish |
| --- | --- | --- |
| Local artifacts | Checksums, secure staging and durable verified-directory publication | Complete transfer from an IC snapshot backend |
| Durable records | Bounded JSON, locks, journals and interruption-safe local transitions | Permission to repeat or settle a remote operation |
| Inventory and plans | Exact targets, dependency graphs, request bindings and finite attempt limits | Current membership, control or application consistency |
| IC request/reply codecs | Typed bytes and bounded decoding for selected snapshot and lifecycle calls | Authenticated transport or trustworthy reply association |
| Integration ports | Explicit contracts for membership, control, read access and consistency evidence | Installed live providers or application-specific safety |
| End-to-end product | Local foundations only | Capture/restore runners, transport, CLI or a qualified backup journey |

<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-backup/ic-backup-implementation-boundary.svg" alt="Implementation boundary showing locally tested models, policy and persistence, application-supplied integration contracts, and the planned IC transport, runners and CLI" width="800">
</p>

## Terms used in this reference

| Term | Meaning here |
| --- | --- |
| Record | Bounded persisted data with a maintained schema and validation rules |
| Journal | Durable progress and evidence for one exact operation |
| Plan | An immutable declaration of targets, requests, ordering and attempt limits |
| Policy | A pure decision or validation step that performs no IO |
| Port | A contract an integration implements to supply current external evidence |
| Qualified | Demonstrated within the named evidence boundary; not a universal product claim |
| Reconciliation | Inspecting retained evidence to determine an uncertain operation's outcome without blindly repeating it |

## Extraction history and provenance

The first extraction copies and reshapes Canic's local artifact and persistence
mechanisms into `ic-backup`. It performs no IC calls and introduces no Canic
dependency. [Fresh provenance](extraction-source.json) records inspected
working-tree hashes, copied source/destinations and consumer references. The
original [planning baseline](source-baseline.json) remains retained evidence.
MIT contributor notices remain in the root license.
The next local batch adapts layout lifetime exclusion and conservative restore
reference retention; [its source provenance](layout-source.json) identifies the
inspected inputs without replacing the first batch's evidence.
Command custody follows with [separate provenance](command-custody-source.json),
owned child descriptor setup and exact local quiescence admission. Local download
journal lifecycle adaptation has [its own provenance](download-journal-source.json).
Pending claims and exact receipts underpin the local attempt ledger, with
[separate source provenance](attempt-journal-source.json).
Canonical inventory and pure target expansion follow with
[their own provenance](inventory-source.json).
Explicit operation ordering follows with
[separate graph provenance](effect-graph-source.json).
Plan-to-journal declaration binding has
[its own provenance](operation-plan-source.json).
Retained-journal execution progress has
[separate source provenance](execution-progress-source.json).
Closed IC request encoding has
[its own source and upstream codec provenance](ic-request-source.json).
Typed ephemeral membership observations have
[separate source provenance](membership-source.json).
Direct caller-controller observations have
[separate control provenance](control-authority-source.json).
Snapshot-list permission observations have
[separate read provenance](snapshot-read-source.json).
Original consistency requirements and current capture/fence observations have
[separate consistency provenance](consistency-source.json).
Snapshot capture/inventory reply admission has
[separate reply provenance](ic-snapshot-reply-source.json).

## Maintained local contracts

`model::artifacts::ArtifactChecksumRecord` is the maintained v1 checksum record.
Its JSON fields are exactly `algorithm` and `hash`; missing/unknown fields reject.
The algorithm is exactly `sha256`. Digests have 64 hexadecimal digits and are
normalized to lowercase during construction/deserialization. Record equality
uses canonical values. Validation returns typed errors. No predecessor readers
or alternative schema generation is maintained.

`ops::artifacts` streams file bytes through a 64 KiB buffer. Unix traversal opens
descriptors without following artifact symlinks, checks actual entry types and
rejects traversal components and special entries. Directory digests preserve
the source recipe: sort relative UTF-8 paths, then hash each path, NUL, its
lowercase file digest and LF. Empty directories and filesystem attributes do
not participate. Non-UTF-8 entry names reject rather than collapse into the
same lossy path representation. The list of file digests is collected in memory;
this primitive does not implement an operation-wide resource budget.

Staging copies exact descriptor-read bytes into a new destination, using 0700
directories and 0600 files. The caller owns the trusted destination parent.
It does not overwrite occupied destinations. A failure may leave partial staging
evidence. Staging is separate from durable publication.

`ops::persistence::commit_artifact_directory` accepts distinct staging/canonical
siblings and an exact expected digest. It synchronizes files and directories,
verifies their bytes, uses atomic no-replace publication and synchronizes the
parent. If only the canonical tree exists, it verifies and synchronizes that
tree before reporting recovery. Both-present, both-absent, changed-byte and
unsafe-entry cases reject without overwriting evidence. Publication uses the
source's Linux/Android/Apple implementation; the fresh evidence here is Linux.
The caller excludes other writers and owns the sibling parent.

`create_json_durable` publishes a new record using a synchronized private sibling
temporary and a create-only hard link. `write_json_durable` replaces a record
using a synchronized temporary and rename. Both synchronize the parent; newly
created private parent directories and their links are also synchronized.
Serialization failure leaves the previous document intact. A failure after
publication may leave the canonical record present and requires reconciliation.
Neither helper supplies schema admission or operation transitions.

`read_json` requires an explicit byte limit, reads at most that limit plus one,
and rejects excess bytes before decoding. Unix reads reject symlinks and
non-regular final entries. Record parents are operator-owned trusted directories.
`JournalLock::acquire` uses a private regular no-follow sidecar, returns immediately
with a typed contention error, and keeps its descriptor close-on-exec. Dropping
the guard or owner process releases the lock while retaining the sidecar.
It does not prove an external command tree is quiescent.

`BackupLayoutGuard::acquire` resolves an existing operator-selected directory
once, including an explicitly selected root link. It owns a stable parent-side
`.ic-backup-layout-<SHA-256-of-basename>.lock` sidecar and an open directory
descriptor. The lock survives removal of its contents/root. Cooperating users
cannot acquire the same root name after deletion/recreation while its guard is
held. Reference operations compare the current root's device/inode with the held
descriptor and reject a replaced root. Missing roots are never recreated.
The operator owns the trusted parent and all cooperating layout writers/removers
must acquire this guard. It does not exclude arbitrary filesystem writers.

The maintained `restore-references.json` document contains exactly `version: 1`
and `restores`, a list of `RestoreReferenceRecord` values with exactly `journal`
and `authority`. Journal locations are absolute, normalized UTF-8 paths;
authority is a canonical lowercase 64-digit SHA-256 digest of immutable intent
supplied by the integration. This records a dependency, not fresh effect
authority. Model transitions reject conflicting intent at an existing journal
location, order entries by path and adopt exact repeats without duplicating them.
Decoding rejects other generations, unknown/missing fields, invalid hashes,
duplicate journal identities and traversal/encoding aliases.
The [JSON schema](contracts/restore-references.schema.json) records the boundary;
runtime checks enforce journal-key uniqueness, UTF-8 byte bounds and canonical
normalization in addition to its structural constraints.

Reads and writes admit at most 1 MiB of encoded JSON, 1,024 entries and 4,096 UTF-8
bytes per journal location. Decoding bounds collection growth; IO bounds total
input before decoding. Human operators select an existing journal parent;
`retain_restore` resolves that parent once without creating directories and
rejects an unsafe existing journal leaf. The journal itself may be absent or
external. A guard durably publishes the dependency before a caller publishes its
recoverable restore journal. Exact-repeat adoption completes file/directory
synchronization after a lost publication response. Malformed or unsafe retained
evidence returns an error and is not interpreted as an empty dependency set.

No reference release or local prune API is exposed yet. Unfinished, missing or
moved journals continue to retain source artifacts. Adding release requires
the terminal-completion and subprocess-custody contracts; these local primitives
cannot establish them. The layout/reference process-death tests qualify layout
exclusion and reference publication; command custody has separate evidence below.

`CommandLifetimeLock::acquire` resolves an existing journal parent and acquires a
private regular `<journal>.command-<u64-sequence>.lock` sidecar. Its validated v1
`CommandCustodyRecord` contains exactly `version`, `journal`,
`operation_sequence`, `device` and `inode`. The record binds an absolute canonical
UTF-8 journal location (at most 4,096 bytes), an exact sequence and the observed
regular-file identity; inode zero rejects. Standalone reads use the explicit
32 KiB record bound. [The schema](contracts/command-custody.schema.json) describes
the machine boundary. It records local custody, not immutable plan identity,
paid-call budget, fresh network/controller authority or terminal completion.

Before dispatch, callers durably retain this identity and the operation's exact
intent using the journal owner's transitions. `spawn` checks the current sidecar
identity and consumes its guard's one-spawn allowance before attempting process
creation. Spawn failure cannot reuse that same guard's allowance. Durable
attempt accounting remains a future journal/workflow contract; acquiring a new
local guard is not permission to repeat an unresolved paid effect.

Child setup uses pinned `command-fds` 0.3.3's
[owned descriptor API](https://docs.rs/command-fds/0.3.3/command_fds/trait.CommandFdExt.html).
Only a child duplicate loses close-on-exec; the owner remains close-on-exec.
The consumed `Command` drops its duplicate immediately after spawn, so it cannot
retain false custody in the parent. Stdio is preserved. The child receives its
descriptor number in `IC_BACKUP_COMMAND_CUSTODY_FD`; raw descriptor numbers are
not persisted and no clonable borrowed lifetime handle is exported. Caller code
owns the admitted program/argv, output/deadline bounds and reaping the direct child.
This developer API does not introduce arbitrary program hooks into product plans.

`finish` closes owner custody without explicitly unlocking a shared inherited
file description and probes for at most the 250 ms grace period.
`CommandQuiescenceGuard::acquire` makes a fresh nonblocking probe using retained
evidence. Both open an existing regular no-follow sidecar and require its exact
device/inode identity; missing paths are never recreated for reconciliation.
Success keeps an exclusive non-spawning guard until dropped. A replaced sidecar,
contention or an unsafe/missing entry rejects without removing local evidence.
Lock admission excludes cooperating holders of that exact sidecar.

The selected backend must preserve custody in all relevant descendants. A program
that closes inherited descriptors is outside this local contract. Fresh Linux
tests use real exec/descendants, descriptor flags and acknowledged parent death;
the descendant remains a blocker after both owner and direct child have exited.
This does not qualify a real ICP backend or prove remote success/failure.
Terminal reference release still requires qualified backend custody plus exact
terminal journal/completion evidence; it remains unimplemented.

## Source ownership and remaining contract work

### Local download journal lifecycle

<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-backup/ic-backup-download-journal.svg" alt="Local download journal states from created through downloaded and checksum verified to durable, including explicit inspection and reconciliation after interruption" width="800">
</p>

The [download journal provenance](download-journal-source.json) records adapted
Canic journal states, validation, projections, artifact operations and consumer
references. `DownloadJournalRecord` is the maintained local v1 record with
exactly `version`, `intent` and `artifacts`. The caller supplies a canonical
SHA-256 digest of immutable intent; this record does not authenticate that digest
or establish current network/caller/controller/release authority.

Each `DownloadArtifactRecord` retains exactly `canister_id`, `snapshot_id`,
`snapshot_taken_at_timestamp`, `snapshot_total_size_bytes`, `staging_path`,
`artifact_path`, `state` and required nullable `checksum`. Principal text is
validated by published `ic_principal` conversion and emitted in canonical lower
case. Hash input normalizes similarly; snapshot tokens remain exact bounded
graphic ASCII strings, with no inferred identity or case folding. Timestamp and
size are required u64 observations, not optional provenance. Encoded directory
size need not equal the observed IC snapshot size. The backend must qualify its
token codec and metadata/extent coverage before integrating a remote transfer.

One snapshot is selected per canonical physical principal. The nonempty set is
bounded to 1,024 entries, sorted by principal text and rejects duplicate aliases.
Staging and canonical directories are fixed as `artifacts/<principal>.tmp` and
`artifacts/<principal>`. Decoding requires those exact identity-derived paths,
rejects traversal/label aliases, unknown/missing/duplicate fields and other
generations. [The machine schema](contracts/download-journal.schema.json) records
structural bounds; runtime additionally verifies principal checksums, derived
paths, canonical identity and property uniqueness. Persistence admits at most
1 MiB of input and canonical pretty-encoded output; snapshot tokens admit at
most 256 ASCII bytes. Count growth is bounded during decoding.

Model transitions own the immediate `Created` → `Downloaded` →
`ChecksumVerified` → `Durable` sequence. They atomically update state/checksum,
reject repeats, skips, regressions and snapshot rebinding, and preserve immutable
metadata and paths. Created/Downloaded require null checksum; later states
require the validated SHA-256 checksum record. No independent progress counters,
timestamp markers or mutable topology declarations were copied. Read-only views
derive next actions and pending counts from retained artifact entries. A
`Download` resume action reports incomplete local progress; it grants no remote
retry or paid-call authority.

`DownloadJournalGuard` borrows `BackupLayoutGuard` and exclusively locks the fixed
`download-journal.json`. Creation never replaces evidence; opening validates
bounded retained records under the expected exact intent digest. Staging belongs
to the caller. `record_downloaded` requires a safe directory and records the
caller's completed-transfer attestation; it cannot prove complete IC metadata or
that a command tree ended. The integration must supply both before calling it.
`verify_artifact` computes actual staged bytes through the extracted secure
directory checksum implementation. `finalize_artifact` publishes or adopts only
the matching verified tree, then durably advances the journal. The fixed artifact
parent must be an existing nonsymlink directory; all cooperating writers use
layout exclusion. No staging deletion/redownload or remote command occurs here.

Any attempted persistence failure or artifact-commit failure disables that guard
until it is dropped and retained evidence reopened. A publication may have
completed despite its error; in-memory progress cannot authorize continuation.
Model validation and encoded output admission precede persistence, including
before artifact publication. Reopening a ChecksumVerified journal after its
directory was published reconciles those exact canonical bytes without resetting
identity or transferring again. Durable resume views read local journal evidence
only, even if artifact bytes later become unavailable; they establish local
publication progress, not full backup terminal completion or fresh verification.

Fresh native regressions rerun the adapted state/validation cases and cover exact
metadata, identity aliases, schema fields, count/token/byte bounds, failed writes
on both publication sides, changed bytes and unsafe/replaced paths. A real owner
dies at an acknowledged barrier after artifact publication and before journal
advancement; recovery adopts the original tree. An external public-API journey
also exercises this interruption boundary. These are filesystem/process tests,
not PocketIC/real IC transfer or snapshot-effect qualification.

### Fresh local download integrity

`policy::download_integrity::validate` borrows the existing original operation plan
and download journal. It requires the full plan digest as journal intent, exact
canonical selected-target coverage and Durable state with checksums for every
entry. Missing, extra and substituted targets reject before byte IO. Views retain
the exact snapshot token, metadata, derived paths and checksum owned by the journal;
they are structural projections, not verified-byte permits or serialized receipts.
The pre-capture plan cannot independently prove the later captured snapshot ID.

`DownloadJournalGuard::verify_durable_artifacts` explicitly reads current published
bytes. Under borrowed layout and journal exclusion, it re-admits the retained plan
by its full digest and requires the retained journal to equal the held record before
and after traversal. Existing bounded record owners enforce 1 MiB input/output;
selection and journals remain capped at 1,024 targets and snapshot tokens at 256
ASCII bytes. Every identity-derived artifact must be a directory; the existing
no-follow traversal streams its checksum and compares it to retained evidence.
Typed failures retain all original records and artifact evidence. No persisted
schema, state transition, checksum algorithm or freshness flag is added.

Ordinary open/resume remains a local progress projection, even after published
bytes change. This explicit operation writes no journal, creates no receipt,
replenishes no allowance and releases no restore reference. Checks are sequential
observations, not an atomic tree/set snapshot; integrations maintain stable byte
custody and separately qualify authentic capture identity, metadata/extent transfer,
application consistency, completion and release admission.

Fresh native production tests cover exact coverage, original-intent changes, every
non-durable state, maximum target/token bounds, changed bytes, retained-plan/journal
changes, absent/file/symlink substitutions and unusable custody. Public recovery
checks preserve exhausted pending reservations and unfinished restore references.
These qualify local persistence and traversal, not IC management or backup completion.
See [the contract](contracts/download-integrity.json) and
[fresh source/consumer provenance](download-integrity-source.json).

### Per-operation local attempt accounting

`model::attempt_journal` adapts Canic's pending-claim and receipt discipline into
`AttemptJournalRecord`. Its immutable `AttemptAuthorityRecord` contains an exact
`OperationBindingRecord` and original `AttemptBudgetRecord`. The binding retains
intent digest, operation sequence, network fingerprint, caller and physical target
principals, release digest and exact mutating-request digest. Principal and hash
aliases normalize at admission; unknown/missing/duplicate fields and generations
other than v1 reject. The record is a declaration, not authenticated intent or
fresh network/caller/controller/read authority. This generic owner validates digest
shape; the separate IC request model qualifies supported payload bytes. Network,
release and authoritative backend meaning remain integration-owned.

Mutation and reconciliation-observation ceilings are separate, immutable u32
limits with a checked combined maximum of 1,024; zero is allowed. No transition
refunds or replaces budgets. Each reservation consumes one allowance and receives
the next global attempt number starting at one. A pending mutation blocks another
mutation. A separately reserved observation binds that exact pending mutation and
its own request digest; only one observation may remain unsettled. Direct mutation
receipts match the exact pending attempt and immutable mutating request. Observation
receipts match their exact pending observation/request, so stale replies cannot
settle a later mutation. History is append-only; counters and progress are derived
by chronological validation, never serialized authority flags.

Qualified Applied or NotApplied receipts settle their attempt without refunding
allowance. Applied stops further reservations for that operation. An observation
may resolve an exhausted mutation without permitting another mutation. Uncertain
means a **qualified settled observation** cannot determine mutation outcome: the
observation allowance stays spent and the mutation stays pending. A lost observation
response stays pending until qualified settlement; invocation failure alone cannot
produce Uncertain or NotApplied. Integrations own retained evidence, current
authority, command quiescence and external-effect settlement. Receipt hashes do
not prove those facts. A new paid observation always consumes separate allowance.

`ops::persistence::AttemptJournalGuard` borrows stable layout exclusion, exclusively
locks `attempt-<operation_sequence>.json` and creates without replacing evidence.
Opening requires the full exact original binding and budget, not just a digest.
Reservations are durably published before returning their number. Any write
failure disables the guard until drop/reopen; retained chronology decides whether
consumption happened. Reopen neither resets allowance nor blindly retries. Reads
and canonical pretty-encoded writes admit at most 1 MiB; decoding bounds history
to 2,048 events. [The machine schema](contracts/attempt-journal.schema.json) specifies
the closed record and canonical, domain-separated authority hash independently
of mutable history or JSON formatting.

Local replay validates retained accounting and performs no remote observations or
effects. Applied is retained operation evidence, not a full terminal run receipt,
verified manifest or permission to release restore references. Fresh verification
is a separate operation. This primitive accounts for caller-owned explicit
reservations; it performs no dispatch and does not bound opaque backend retries,
cycles or remote calls inside one command. Those require qualified executor ports
and runner integration. No completed backup/restore runner is exposed.

Fresh native regressions cover exact bindings and golden hash bytes, exhausted
budgets, stale/mismatched receipts, closed schemas, bounded chronology, locking and
unsafe/replaced paths. Lost writes before/after publication stop the guard; real
acknowledged owner death before writing, after writing and after reservation return
retains the correct consumed allowance. A public-API journey resolves a retained
exhausted mutation using separate observation allowance. No fake management
backend or live IC call participates in this qualification.

### Declared physical inventory and pure selection

`model::inventory::InventoryRecord` adapts Canic topology rows and registry target
projection into a canonical v1 declaration. Each private `InventoryTargetRecord`
retains exact canonical `canister_id` and required nullable `parent_canister_id`,
`role` and `module_hash`. Principal text and SHA-256 module digests normalize at
admission. Role is opaque UTF-8 text, bounded to 256 bytes: null, empty and literal
"null" are distinct. Roles grant no control, selection or root privileges. Module
digests are declarations, not fresh observations. No credentials, Fleet/Root types,
framework runtime dependencies or application membership claims enter the record.

Inventories contain 1–1,024 unique physical targets, sorted by canonical principal
text. A declared parent must be present in the full inventory; missing parents,
self-links and longer cycles reject rather than overwrite rows or truncate walks.
Multiple disconnected parentless entries are valid. None means no declared parent,
not proof of a physical or authoritative application root. Traversal is iterative;
the target count bounds parent depth to 1,023. Integrations retain a complete declared
forest for expansion, or explicitly describe standalone targets without parent edges.
The library never infers missing parents or silently changes their identity.

The inventory digest hashes a specified domain-separated binary encoding over the
canonical full target list, with u32 big-endian UTF-8 lengths and explicit nullable
tags. It includes all identity, parent, role and module fields, so input ordering
and equivalent principal/hash case normalize without delimiter ambiguity. This is
a new generic v1 contract, not Canic's existing sorted text digest; no old digest
reader, converter or compatibility alias is supplied. Matching hashes prove declared
equality, not fresh membership, controller custody, revision continuity, application
fencing or consistency. [The machine schema](contracts/inventory.schema.json) specifies
exact fields, bounds, binary encoding and independently encoded golden vectors.

`policy::selection::select` takes a nonempty bounded list of exact principal
selectors and explicit `SelectionExpansion`: Exact, DirectChildren or Descendants.
Exact includes only named targets. DirectChildren adds immediate children of each
named target. Descendants includes the full declared subtree of each named target.
Unknown/malformed principals and equivalent explicit duplicates reject; overlapping
expansions union physical targets. There is no role lookup, implicit exclusion of
a privileged root or non-neutral request default. Views retain references to the
original records in canonical order and bind the **full original inventory** digest,
including unselected targets. Parent links may point outside a selected subset.
Canonical presentation order is not a lifecycle/effect order. Policy performs no
IO, serialized record writes, scheduling or persisted transitions.

`ops::persistence::create_inventory` holds layout/journal exclusion and durably
creates fixed `inventory.json` without replacing prior evidence. `read_inventory`
validates the retained forest and original expected digest, observing only local
records. Lost creation responses reconcile by reading that exact declaration;
existing records cannot be replaced through this API. Symlinks/unsafe entries,
replaced roots and malformed records reject. Encoded reads and canonical pretty
writes each admit at most 1 MiB. Canonical output is checked independently because
JSON escaping of admitted role text can expand beyond input/field byte bounds.

Fresh native regressions rerun Canic's order-independent hash and direct/recursive
child cases against these owners. New cases cover two golden encodings, nullable
distinctions, exact field changes, normalized duplicates, missing parents/cycles,
maximum count/depth, UTF-8 and escaped-output bounds, selector errors, overlapping
expansions, unchanged prior bytes, locks and unsafe/replaced paths. An external
public-API journey persists/reopens an explicit declaration and selects a subtree
without framework or backend dependencies. Live authoritative discovery and IC
effects remain outside this qualification.

### Explicit operation dependency graphs and causal readiness

`model::effect_graph::EffectGraphRecord` adapts Canic backup phase and restore
ordering mechanics into a generic v1 dependency declaration. Its private
`EffectNodeRecord` retains exactly `operation_sequence` and `depends_on`. Sequences
are opaque unsigned u64 identities: zero, nonconsecutive values and u64::MAX are
admitted. They are not array indices or a numeric dispatch order. Canonical records
sort nodes and each prerequisite set by sequence, rejecting duplicates instead
of overwriting identities or silently collapsing repeated edges.

Graphs admit 1–8,192 operations, at most 1,024 direct prerequisites per operation
and at most 65,536 total edges. The decoder bounds both node lists and aggregate
edge retention during parsing. Every prerequisite must name an existing exact
operation. Self-loops and longer cycles reject. Iterative topological validation
admits a maximum chain depth of 8,191 without recursion. It emits the smallest
currently ready sequence at each step, so reordered equivalent declarations yield
one deterministic planning order. This computed order is not serialized or editable.
The graph digest uses domain-separated binary encoding with u32 counts and u64
identities; it binds canonical explicit edges, not JSON formatting or cached order.
[The machine schema](contracts/effect-graph.schema.json) specifies fields, limits,
canonical hashing and an independently encoded golden vector.

Application order must be declared and qualified explicitly. The graph does not
infer parent-before-child, reverse start order, control routing, privileged roots
or physical inventory edges. Canic can supply its parent-first restoration and
child-first restart dependencies through its own adapter. Other applications may
have different valid dependencies. This primitive is one component of a future
complete plan: it contains no operation payload, target/request/network/release
binding, consistency guarantee, spending limit, receipt or execution permission.
Complete plans must bind graph identities to their exact reviewed operations and
the existing immutable attempt authority; no such runner wiring is implemented.

`policy::effect_order::readiness` takes a passive `EffectProgressRequest` with
caller-declared completed sequence identities. Excessive, duplicate or unknown
completed identities reject. A declared completed operation must include all its
prerequisites in that same completed set; inconsistent causal progress rejects.
The read-only view binds the original graph digest, counts admitted declared
completion and separates incomplete ready/blocked nodes in planning order.
Policy performs no IO, record writes, scheduling or mutations. Ready means only
that declared prerequisites are present. A completed identity is not a qualified
receipt: callers own actual evidence, exact binding and remote-effect settlement.
Neither readiness nor an empty incomplete set grants fresh authority, paid-call
allowance, command quiescence, terminal completion or reference release.

`ops::persistence::create_effect_graph` durably creates fixed `effect-graph.json`
under layout/journal exclusion without replacing retained evidence.
`read_effect_graph` admits only a validated graph under its original exact expected
digest. Lost creation replies reconcile through that exact local read; no graph
update/replacement API is exposed. Encoded input and canonical pretty-output each
admit at most 1 MiB; dense valid graphs can exceed the output bound and reject
before publication. Missing/unsafe entries, replaced roots and malformed graphs
reject without repair, progress reset or remote observations.

Fresh native regressions adapt relevant Canic parent/child ordering into explicit
dependencies, without importing relocation, Root scopes or fake management effects.
They cover a golden binary hash, changed identities/edges, deterministic ready ties,
unknown/duplicate/missing dependencies, cycles, closed schemas, cached-order injection,
maximum chain/count, exact direct/aggregate edge bounds, causal progress, immutable
original bytes, contention and unsafe/replaced paths. A public-API journey reopens
the original graph and projects declared progress locally. These checks qualify
graph/policy/persistence behavior, not full plans, live effects or actual completion.

### Immutable operation plan and derived original attempt authority

`model::operation_plan::OperationPlanRecord` adapts Canic's plan validation,
operation projection and original-plan resume discipline into a generic local
binding. Private v1 fields retain canonical `PlanContextRecord`, the full original
`InventoryRecord`, nonempty exact `selected_targets`, original `EffectGraphRecord`,
one `PlannedOperationRecord` per node and original aggregate `PlanBudgetRecord`.
The context binds declared network fingerprint, caller principal and opaque release
evidence digest. Each operation binds exact sequence, canonical physical target,
integration-owned mutating-request digest and original `AttemptBudgetRecord`.
Hashes/principals normalize at their owning boundaries. Unknown/missing/duplicate
fields, other generations and editable derived intent/authority/views reject.

The explicit selection admits at most 1,024 unique exact inventory targets.
Operations admit at most 8,192 unique sequences and match graph identities one for
one; missing/extra/mismatched operation bindings reject. Every operation targets
the selected physical set, and every selected target must have at least one bound
operation. Original inventory parents can remain unselected. Canonical selection
and operation ordering is presentation/hash order, not dispatch order. Plan
construction does not infer application consistency or lifecycle requirements.

Per-operation allowances retain the existing combined maximum of 1,024 attempts.
Separate aggregate mutation/observation ceilings have a checked combined maximum
of 65,536, including zero. Checked sums of assigned original operation allowances
must fit each original aggregate ceiling. Unassigned headroom cannot be spent by
derivation: there is no budget replacement/reallocation API. Allocated totals are
read-only projections, not consumed allowance, charged cycles or current balances.
Backend internal retries/calls/costs still need their own qualified executor bounds.

The full canonical plan digest binds context, original inventory/graph digests,
explicit selection, original aggregate ceilings and every exact operation target,
request and allowance. Binary encoding uses a separate v1 domain and exact byte
lengths/widths; JSON formatting, allocated views, journal progress and derived
authority are excluded. This avoids hashing an intent into itself. Changing even
an unselected inventory row, another operation's request or unused aggregate ceiling
changes the original intent and every derived authority. See
[the machine schema](contracts/operation-plan.schema.json) for fields, invariants,
embedded inventory/graph contracts and independently encoded golden bytes.

`attempt_authority(sequence)` derives existing `AttemptAuthorityRecord` with the
full original plan digest as intent, shared exact context, exact target/request and
that operation's original limits. Repeated derivation returns the same declaration;
it consumes no allowance and creates/resets no journal. Journal creation remains
no-overwrite and reopening uses retained consumption. An exhausted pending mutation
therefore stays spent/pending when its authority is derived again. Derivation does
not check graph progress, fresh permissions, paid-effect settlement or actual bytes;
future workflow admission must bind qualified receipts and request codecs before
calling the existing approved ops boundaries.

`ops::persistence::create_operation_plan` durably creates fixed `operation-plan.json`
under layout/journal exclusion without replacing evidence. `read_operation_plan`
admits only the bounded validated declaration under its exact original expected
digest. Lost creation replies reconcile by that exact local read. Embedded inventory
and graph are the plan's original bindings; separate retained files are not adopted,
rewritten or assumed equal. Encoded input and canonical pretty-output each admit at
most 1 MiB; structurally valid maximum-count plans can exceed persistence bounds
and reject before publication. Unsafe entries and replaced roots reject without
repair or remote observations.

This record is a declaration binding primitive, not a complete authenticated
backup/restore plan or an executable request. Typed payload meaning, release/network
qualification, fresh membership/controller/read checks, capacity, lifecycle policy,
application consistency/fencing and external-effect settlement remain their owners'
contracts. No serialized accepted-preflight/Proven flag grants dispatch. No runner,
snapshot effect, terminal proof, relocation/migration or reference release is added.

Fresh native regressions cover canonical plan golden bytes, case/order aliases,
all declaration owners, full intent/derived authority, wrong graph/table/selection,
assigned/aggregate limits, closed schemas and rejected view/preflight injection.
They also qualify count/output bounds, original no-overwrite bytes, lock contention,
unsafe files and replaced roots. An external public-API journey persists/reopens
the original plan, derives its exact journal authority and recovers spent pending
allowance without resetting it. This proves local binding/persistence/accounting,
not IC authority, typed management requests or canister effects.

### Pure execution progress from exact original journals

`policy::execution_progress::progress` adapts Canic's plan/journal integrity and
resume-report responsibilities into a pure local evidence join. Its passive request
borrows the original `OperationPlanRecord` and a complete set of validated
`AttemptJournalRecord` references. At most 8,192 journals are admitted. Every
original graph operation requires exactly one journal; duplicate/unknown identities,
missing evidence and changed full intent/context/target/request/original limits
reject. Missing journals are never implicitly created or treated as unused allowance.
The original full plan digest is computed once, rather than once per journal.

Applied identities come from model-replayed retained receipts, not a supplied
completion list or editable progress counter. Every operation with consumed
mutation allowance must have retained Applied evidence for each declared
prerequisite, including pending, settled NotApplied and Applied operations. This
checks causal retained evidence. Per-operation journals lack a shared dispatch
sequence; a set of current Applied receipts cannot prove that a dependent call
actually began after its prerequisite. Trusted workflow admission must enforce
that chronology and retain appropriate evidence before future effects.

Views bind the original plan/graph digests and project exact operation identities
in deterministic graph planning order. Conditions distinguish awaiting dependencies,
available/exhausted mutation allowance, unresolved mutation, unresolved observation,
exhausted reconciliation and retained Applied evidence. A lost observation reply
stays observation-unresolved even when its allowance is exhausted. A qualified
settled Uncertain observation leaves the mutation unresolved; exhaustion never
refunds or grants a fresh retry. Zero mutation allowance yields exhaustion, not
completion. Applied operations may retain unspent allowances that cannot be used.

Checked totals sum original per-operation used/remaining allowances. Unassigned
aggregate headroom stays excluded and no original ceiling changes. Read-only views
serialize for output but have no deserialization/record-admission or persistence
API. The policy performs no IO, record serialization, authority probes, state
mutation, journal creation or scheduling. Callers own coherent retained evidence
under appropriate layout/journal custody. Existing persisted v1 schemas are unchanged.

Retained receipts remain integration-qualified evidence, not self-authenticating
signatures or current live authority. Even all operations Applied establishes no
full backup/restore terminal proof, complete artifact/manifest verification,
application fence disposition or reference release. Remaining transfer/response
codecs, fresh authority, lifecycle safety and backend reconciliation still precede
runners.

Fresh native cases cover every original binding, complete coverage/counts, exact
65,536 assigned allowance, maximum journal count, reverse numeric dependency order,
attempted unmet prerequisites and all projected conditions. Public recovery
reopens exact persisted plan/journals, preserves bytes/consumption through a lost
observation response and rejects omitted or discarded prerequisite evidence.
These qualify local model/policy/persistence behavior only, without IC effects.

### Closed IC host-ingress request codec

`model::ic_request::IcManagementRequestRecord` retains v1 private `method`, canonical
effective `target` and required nullable `snapshot_id`. The closed methods are
`canister_status`, `list_canister_snapshots`, `load_canister_snapshot`,
`start_canister`, `stop_canister` and `take_canister_snapshot`. Load requires
1–256 exact raw snapshot bytes; every other method requires null. Byte ordering,
zero bytes and repeated bytes remain exact. Generic download snapshot strings are
backend tokens and are not implicitly converted, folded or adopted here.
Unknown/missing/duplicate fields, unsupported methods/generations and injected
argument/digest/receiver/authority fields reject. Decode bounds raw byte retention.

The model derives cached Candid arguments with the existing lockfile-selected
`candid` (currently 0.10.37) and `ic-management-canister-types = 0.11.0`. Original
0.10.35 Candid and 0.8.0 SDK source evidence remain historical; the request/reply
wire goldens have also been rerun against the current selection. See
[SDK upgrade evidence](management-types-upgrade.json). Receiver is fixed to the management principal
`aaaaa-aa`; target is the effective routing principal and encoded `canister_id`.
All supported methods use replicated update ingress, including the two semantic
observations. Capture fixes `replace_snapshot = None`, `uninstall_code = Some(false)`
and `sender_canister_version = None`. Load fixes sender version to None and names
exact raw snapshot bytes. Sender version concerns the calling canister, not a
target-version compare-and-swap guard. These field/mode definitions follow the
[IC management interface](https://docs.internetcomputer.org/references/ic-interface-spec/management-canister/).
The operator host still needs separately qualified current lifecycle/permission
checks. Target existence, snapshot ownership and same-release restore safety are
not inferred by the encoder.

Derived Candid arguments admit at most 4 KiB. Callers retaining records use the
existing bounded JSON ops with independent 8 KiB input/canonical-output bounds and
their own custody. The model owns field/count admission; no runner request layout,
transport or dispatch persistence API is added. Cached argument/raw-principal bytes
are private and excluded from JSON; decoding reconstructs them from exact fields.
This creates one maintained product v1 shape, without alternate codec readers.

The domain-separated request digest binds fixed raw receiver, exact raw effective
principal, the replicated-update byte, exact method name and exact derived Candid
bytes with specified u8/u32 lengths. It distinguishes stop/start and status/inventory
even when argument bytes match. Text principal aliases canonicalize; raw snapshot
bytes never normalize. Network/caller/intent/release/budgets remain bound by the
outer operation plan and authority, keeping payload hashing nonrecursive. See
[the wire schema](contracts/ic-request.schema.json) for exact encoding, closed
method policy and registered independently constructed Candid/hash golden vectors.

`validate_mutation_binding` requires mutation class and exact original target/payload
digest. `validate_observation_binding` requires observation class, the same original
target and a separately reserved observation digest; the original binding still
names its mutation. These checks return byte-admission results, not credentials,
fresh preflight, custody, spending or dispatch permits. Observations can still cost
resources and consume their original independent allowance. No reply, settlement,
Applied receipt, lifecycle effect, transfer or terminal proof is fabricated.

Fresh native regressions compare every registered independent wire vector, decode
the official upstream types, check fixed host-ingress/capture fields, principal
aliases, method/routing/snapshot sensitivity, exact binding classes and raw/JSON
bounds. A public journey retains a typed payload and original plan, reopens spent
mutation authority and records a separate exact observation reservation without
reset or replacing evidence. It qualifies local encoding/accounting only; actual
snapshot/lifecycle behavior still requires a selected IC backend. Live application
providers, remaining transfer/response codecs and full uncertain-effect reconciliation
remain pending.

### Bounded IC snapshot capture/inventory replies

`model::ic_snapshot_reply::IcSnapshotReply::decode` accepts only the existing
`take_canister_snapshot` and `list_canister_snapshots` requests. It borrows the
immutable request owner and decodes exactly one upstream Candid value: a snapshot
record for capture or a vector of snapshot records for inventory. Every descriptor
requires `id : blob`, `taken_at_timestamp : nat64` and `total_size : nat64`, matching
the [primary management interface](https://docs.internetcomputer.org/references/ic-interface-spec/management-canister/)
and the existing pinned SDK. No optional-metadata fallback or ICP JSON reader is added.

Raw input admits at most 1 MiB before parsing. Decoding has a 2 MiB work quota,
zero skipped work and at most 16 type-table entries. Bounded Serde sequence visitors
retain at most 1,024 inventory descriptors and 1–256 exact ID bytes per descriptor
without preallocating from untrusted lengths. Missing/wrong fields, unknown/skipped
fields, oversized sequences, extra arguments, trailing bytes and invalid Candid
reject as typed `InvalidReply`; raw overflow and unsupported methods have separate
typed errors. This local bound is not a claim about IC capacity or universal replies.
Timestamp/size accept the full nat64 range, including zero; neither is a local
artifact-byte length or evidence of a completed download.

Inventory views are sorted by exact raw IDs and reject duplicates even when metadata
differs. Zero and repeated bytes remain distinct; no token renderer/converter or
downstream string identity is inferred. `IcSnapshotInfo` exposes read-only fields.
Capture has exactly one descriptor; inventory may be empty. These result types have
no Serde admission, persisted authority flags or receipt transition.

The payload checksum hashes exact raw bytes, so reordering equal inventory entries
does not collapse evidence. A separate v1 domain hashes the existing request digest
and raw checksum as two fixed 64-byte lowercase SHA-256 strings. This binds method,
routing/target/argument bytes through their existing canonical owner. The response
itself contains no target, network, caller, challenge or application operation ID.
An integration can associate the same bytes with another declared request; the hash
then differs, but decoding cannot identify which transport association is authentic.
Actual authenticated target/network/caller, freshness, permissions and original
per-call spending remain with the integration.

No inventory delta, timestamp or parsed capture descriptor settles a lost mutation,
completes a transfer, fabricates an Applied receipt, refunds spending or permits a
new paid call. Local replay decodes retained bytes without any remote observation.
It is not a product terminal receipt or terminal/reference-release API.

Native qualification runs every registered independently assembled Candid/hash
golden through production decoding and official SDK types. It covers all combined
entry/ID bounds, full nat64 values, raw ordering/target hash sensitivity, malformed
headers/types/lengths, truncation, extra/skipped/trailing data and duplicate identity.
A public recovery journey reopens retained original intent and exhausted pending
mutation/observation reservations, checks exact retained reply bytes and rejects
changed request association while keeping journal bytes and allowances unchanged.
This proves local wire/persistence behavior, not authenticated IC snapshot effects.
See [the machine contract](contracts/ic-snapshot-reply.json) and
[fresh source and consumer provenance](ic-snapshot-reply-source.json).

Snapshot metadata/data transfer codecs, full status metadata, live transport,
fresh effect admission, safe capture/load reconciliation and runner wiring remain pending.

### Pure snapshot inventory comparison

`policy::snapshot_inventory_delta::compare` borrows an immutable new-snapshot
capture request and two admitted inventory replies. Both replies must belong to
the existing list method and the exact canonical capture target. The reply owner
already bounds counts/raw IDs and rejects duplicates; comparison uses a linear
merge over those canonical views. Every original baseline ID must still exist with
the exact timestamp and size. Missing IDs and metadata drift are typed failures,
including when one apparent new candidate remains. Method/target errors disclose
neither raw IDs nor payloads.

`SnapshotInventoryDeltaView` exposes borrowed original capture, baseline and
observed reply owners plus read-only candidate references in exact raw-ID order.
Empty, singleton and multiple candidate sets remain descriptive. Raw ordering and
exact request/reply hashes stay available through their original owners; no new
wire record, hash encoding, provider, journal transition or persisted baseline
layout is introduced. At most 1,024 candidate references can be retained, following
the existing inventory bound; comparison performs no IO or serialization.

Canic's baseline preservation and set-difference responsibilities are adapted.
Its singleton-to-completed-receipt inference, ambiguity cleanup/restart, generic
token IDs and upload reconciliation are not copied. Another controller could have
created a singleton; zero candidates do not prove that capture failed. Integrations
must qualify original pre-effect baseline retention, actual authenticated context,
observation chronology, current permissions and exclusive effect attribution.
Neither a timestamp nor this projection supplies those facts or permits a retry.
Uncertain capture and lost observation replies retain their existing obligations
and consumed allowances. No receipt or reconciliation provider is implemented.

Native cases cover zero/single/multiple new IDs, byte-prefix ordering and raw-order
hash preservation, baseline loss at every merge position, timestamp/size drift,
each request role/target mismatch and the full 1,024-entry/256-ID-byte combination.
A public recovery journey durably retains original intent and integration-owned
baseline/observation fixture bytes, reopens exhausted pending journals, compares
all candidate cardinalities and rejects baseline loss. Exact journal/evidence bytes
and original allowances remain unchanged, including after a denied fresh mutation.
These qualify pure comparison and local custody only. See
[the machine contract](contracts/snapshot-inventory-delta.json) and
[fresh source/consumer provenance](snapshot-inventory-delta-source.json).

### Bounded IC lifecycle reply evidence

`model::ic_lifecycle_reply::IcLifecycleReply::decode` borrows the existing immutable
status, stop, start or snapshot-load request. Stop/start/load admit only the canonical
six-byte empty Candid argument tuple, `DIDL` followed by two zero bytes. A Candid
null value, unused type definitions, extra arguments and trailing bytes reject.
The [management interface](https://docs.internetcomputer.org/references/ic-interface-spec/management-canister/)
specifies those empty results; strict canonical admission is a local boundary.

Status decoding is a projection: exactly one Candid value with required `status`
and `settings.controllers`. It preserves the upstream Running/Stopping/Stopped
enum and passes the complete declared controller vector through the existing
`ControllerSet` owner for canonical sorting and duplicate rejection. A bounded
sequence visitor retains at most 10 principals without allocation from the declared
length. Missing settings/controllers never default into an empty set; an explicitly
empty set remains empty. No other status field is retained or qualified. This
projection may admit records lacking fields outside its scope and skips unfamiliar
extensions under finite work limits; it is not full SDK result validation.

Raw replies are capped at 1 MiB before parsing. Status decoding has 2 MiB work,
64 KiB skipped-work and 64 type-table-entry bounds. Malformed types, unsupported
status variants, required-field/count failures, quota exhaustion, extra arguments
and trailing bytes reject as typed InvalidReply. Raw overflow, unsupported methods
and canonical controller failures remain separately typed. No raw identifiers,
payloads or Candid diagnostics enter public error messages.

Read-only kinds expose a status/controller projection or local acknowledgement.
Exact raw SHA-256 retains skipped fields and original controller ordering. A v1
NUL-terminated domain plus the existing request digest and raw checksum as two
fixed 64-byte lowercase SHA-256 strings binds declared association. It authenticates
no network/caller/target/challenge, origin or timing; identical empty replies can
be associated with different methods or raw load IDs, producing different hashes.

The integration owns fresh authenticated observations, original per-call spending,
controller custody, continuous application fencing, stopped/drained evidence and
same-ID/same-release load safety. A decoded Stopped value is not drain or actual
load-history evidence. Status visibility alone grants no mutation control. A wire
acknowledgement is not a local completion receipt, terminal proof, fresh dispatch
permit or fence/reference release. Pending attempts and consumed limits remain
unchanged; no provider or workflow dispatch is installed.

Canic's typed lifecycle states, status/controller projection and required pending
observation responsibilities are adapted. Agent/CLI calls, optional status defaults,
Root/Fleet routing, command success-to-completed receipts and status-equality
reconciliation are not copied. The same existing SDK/Candid dependencies remain.
Native evidence includes all registered hand-assembled Candid/hash fixtures, a full
SDK status fixture, required-field/variant/controller admission, count/raw/work/type
bounds, malformed tuples/truncation and exact method/target/load-ID hash sensitivity.
A public journey reopens original pending load/status reservations and retained raw
fixture bytes, admits Stopped/controller and acknowledgement views, rejects rebound
load evidence and malformed replies, and preserves exact journals/evidence/allowances.
It performs no IC effects or settlement. See
[the contract](contracts/ic-lifecycle-reply.json) and
[fresh source/consumer provenance](ic-lifecycle-reply-source.json).

### Ephemeral membership port and pure result admission

`model::membership::MembershipObservationRequest` borrows the immutable original
operation plan and derives its exact operation binding. Its private fields bind
full plan intent, sequence, canonical network/caller/target/release/mutation request,
full original inventory/selection, an integration-owned challenge, explicit
BeforeEffect/AfterEffect boundary and a 0–1,024 descriptive remote-observation
ceiling. Unknown operations and excessive ceilings reject before provider use.
Zero admits a qualified local observation path, never a paid probe.

The canonical request digest uses a NUL-terminated v1 domain, 64 ASCII intent
bytes, u64 big-endian sequence, 64 ASCII challenge bytes, a before/after byte and
u32 big-endian call ceiling. Full original intent binds context, inventory,
selection, graph, payloads and original allowances; none is reissued or replaced.
The challenge's unpredictability, uniqueness and current applicability stay
integration-owned. A digest proves binding, not freshness or authentic observations.

`ports::membership::MembershipProvider` returns a passive `MembershipObservation`:
exact current request digest, actually observed canonical context, complete current
inventory, optional opaque current revision, required opaque evidence identifier
and actual remote-observation count. A provider must qualify actual observations
and locked authenticated context; echoed declarations or decoded old receipts are
not observations. No provider/default implementation or live backend is installed.
Unavailable/Unsupported fail before remote effects; Indeterminate retains consumed
allowance/evidence and stops without implying retry. Diagnostics carry typed redacted
errors rather than provider credentials or raw output.

The descriptive ceiling bounds one invocation's reported calls; it is not an
original spending allowance, current remaining reconciliation allowance, reservation
or dispatch permit. Integrations still need separately approved prior per-call
accounting and retained uncertainty. No preflight budget owner or paid-call workflow
is implemented by this port; repeated requests do not replenish any journal.

Pure `policy::membership::validate` admits exact current request, observed network,
caller and release, full original inventory equality and calls within the descriptive
ceiling. Changed unselected parents/metadata reject too. Its private-field view
borrows current inventory, original selected targets, optional revision and evidence;
it performs no IO, provider call, record serialization, journal mutation or scheduling.
Request, result and view have no Serde/persisted fresh-authority admission. This adds
one typed v1 port contract without changing existing persisted schemas. See
[the machine-readable typed contract](contracts/membership-port.json).

Matching hashes or current revisions before/after do not exclude intermediate
membership changes or prove a consistency fence, fresh controller/read permissions,
same-release restoration safety or effect completion. Required qualified application
and authority ports remain separate. Terminal replay must never call this provider;
a new live verification is a distinct operation.

Fresh native cases cover exact independent request hash, canonical aliases, original
intent/operation/challenge/boundary/ceiling sensitivity, observed context mismatch,
full inventory drift, zero/max call ceilings and the 1,024-target bound. A public
integration journey reopens original intent and spent pending journal authority,
rejects an old result under a new challenge without a provider call, and preserves
consumption through all typed provider failures. These qualify local contracts and
recovery only, not actual current membership, authority or IC effects.

### Direct caller-controller observations for exact IC mutation payloads

`model::control_authority::ControlObservationRequest` derives the exact original
operation binding and checks the supplied `IcManagementRequestRecord` with the
existing mutation-payload validator. Only encoded capture/load/stop/start methods
admit this request; status/inventory observation methods and changed target/digest
reject before provider use. The request retains exact wire bytes, integration-owned
challenge and a 0–1,024 descriptive remote-call ceiling. Its domain-separated digest
binds full original intent, sequence, exact wire digest, challenge and ceiling using
documented fixed ASCII/u64/u32 framing. Creation changes no plan or original allowance.

`ControllerSet` admits at most 10 known principals, normalizes/sorts canonical
identities and rejects equivalent duplicates. An explicitly known empty set is
representable and always denies caller control. Unknown/missing controllers require
a provider failure; no status decoder, permissive defaults or old flags are copied.
`ControlObservationInput` is passive provider data. Model admission canonicalizes
the actual target and retains actual canonical context, known controllers, current
request digest, opaque qualified evidence and reported actual calls in an immutable
`ControlObservation`. All these types have no Serde or persisted authority lane.

`ports::control_authority::ControlAuthorityProvider` has no installed/default
implementation. It owns fresh authenticated observations, actual context/target,
complete controller evidence, challenge timing/uniqueness, coherent custody and
separately approved prior per-call accounting. Unavailable/Unsupported fail before
remote effects; Indeterminate retains consumed allowance/evidence and stops without
retry admission. Public/read success, echoed declarations, old receipts and
unqualified query responses cannot supply current controller authority.

Pure `policy::control_authority::validate` checks exact current request,
actually observed network/caller/release and target, reported calls within the
descriptive ceiling, then original caller membership in the controller set. Its
private-field view borrows observed evidence and grants no signing, dispatch or
spending permit. Another controller, a controlling Root/parent or status/snapshot
read access never satisfies this lane. Subnet-admin exceptions are also excluded.
The controller bound and distinction between control and read visibility follow
the [IC management interface](https://docs.internetcomputer.org/references/ic-interface-spec/management-canister/).
No provider call, record serialization, plan/journal mutation or scheduling occurs
inside policy. The descriptive ceiling is no fresh budget or reservation.

Target-controller membership is one component of preflight. Load additionally needs
qualified snapshot source/origin permissions, ownership and same-ID/same-release
recovery safety. Current lifecycle, capacity, application fence/settlement, fresh
effect-boundary permission and actual backend effects remain separately qualified.
Terminal replay performs no provider calls. See
[the typed machine contract](contracts/control-authority-port.json).

Fresh native tests cover every registered mutation, independent canonical request
hash, wrong payload/class/target/context/challenge/ceiling, canonical controller
aliases/duplicates/bounds, empty/Root-only/revoked-caller denial and zero/max call
reporting. A public journey reopens original plan and spent pending journal, rejects
stale results without calling the provider, denies revoked caller control and
preserves bytes/allowances across every typed provider failure. These qualify local
contracts/recovery only; no actual controller custody or IC backend is modeled.

### Snapshot-list permission observations and pure caller read paths

`model::snapshot_read::SnapshotReadRequest` derives the original operation binding
and checks exact independently declared `list_canister_snapshots` target/digest.
Status and mutation methods reject. The constructor accepts a separate expected
observer digest from its exact intent/reservation owner; equality validates bytes,
not retained accounting or dispatch. Original mutation digest and allowances stay
unchanged. The canonical request hash binds full original intent, operation sequence,
exact list wire digest, caller-owned challenge and a 0–1,024 descriptive call ceiling.
The ceiling grants no paid calls; provider observations and the list call itself
need separately approved prior per-call accounting and coherent command custody.

The model owns a normalized, sorted unique 0–10 `SnapshotViewerSet` and explicit
`SnapshotVisibility::{Controllers, Public, AllowedViewers}`. No unknown/default
visibility or status/log substitution exists. Passive input and immutable observation
retain exact current request, actually observed context/target, snapshot visibility,
optional complete known controllers, opaque evidence and reported calls. Target text
canonicalizes on model admission. `None` controllers means unobserved; known empty
is distinct. Public or exact viewer membership needs no controller projection;
unobserved controllers never establish controller access. All new types lack Serde
and persisted fresh-authority admission. Existing schemas remain unchanged.

`ports::snapshot_read::SnapshotReadProvider` is fallible with no installed/default
implementation. Actual authenticated snapshot settings/context, freshness, challenge
uniqueness/timing, evidence meaning and custody remain integration-owned. Unknown
visibility must fail. Unavailable/Unsupported reject before effects; Indeterminate
retains consumed accounting/evidence and stops without retry or replenishment.
An observation of permissions does not perform the list or settle a lost list reply.
Terminal replay must never invoke the provider; live verification is separate.

Pure `policy::snapshot_read::validate` matches current request, actual context/target
and reported calls, then requires a known exact caller-controller path, public
visibility or exact allowed-viewer membership. Controller evidence takes precedence
when present. A private-field matching view grants no mutation control, signing,
dispatch, fresh spending, snapshot identity/extent completeness, lifecycle safety,
application fence or terminal proof. Root-configured/Proven declarations, parents
and other viewers/controllers do not substitute for the original caller.
The visibility semantics and viewer bound follow the
[primary management interface](https://docs.internetcomputer.org/references/ic-interface-spec/management-canister/).
Only the existing list codec is admitted; metadata/data request and response codecs,
actual providers and complete read preflight remain unimplemented. See
[the typed Rust contract](contracts/snapshot-read-port.json).

Fresh native tests cover independent binary request goldens, original mutation vs
read binding, every registered unsupported method, exact target/hash/context/challenge/
ceiling mismatch, viewer aliases/duplicates/bounds, unknown vs empty controllers,
all three caller paths and Root-only/revoked-caller denial. A public local fixture
reopens original plan and spent pending mutation/observation, reads the exact observer
digest from the retained reservation, rejects stale results without provider use and
preserves exact bytes/allowances across typed failures. Permission evidence cannot
settle the lost observation or reserve another. These qualify local binding/recovery,
not actual IC permissions or effects.

### Original consistency requirements and current capture/fence checks

`model::consistency::ConsistencyRequirementRecord` retains exactly v1 `version`,
canonical `plan_intent` and explicit `guarantee` (`per_canister` or
`application_coordinated`). Missing/unknown fields, obsolete flag names and other
versions reject. The original full plan intent binds context/inventory/selection/
graph/requests/allowances. A domain-separated hash binds that intent and guarantee;
neither the declaration nor its digest proves current consistency. See
[the v1 schema and binary goldens](contracts/consistency-requirement.schema.json).

`create_consistency_requirement` requires the exact original plan already retained
in the held layout, then durably creates fixed `consistency-requirement.json` under
journal exclusion with no replacement. Reads require the exact expected requirement
digest and original retained plan; raw input and canonical output are bounded to
1 KiB. Missing/corrupt/changed evidence cannot silently downgrade or recreate the
original requirement. Lost local create replies reconcile by an exact retained read.
This storage owns no fence, fresh allowance or dispatch authority.

Ephemeral `ConsistencyRequest` binds the original requirement/operation, caller-owned
challenge, explicit BeforeCapture/AfterCapture boundary, 0–1,024 descriptive remote-call
ceiling and optional exact expected `ApplicationFenceBinding`. Per-canister requests
require no fence; coordinated requests require the retained fence identity AND original
membership revision recovered by the integration's durable obligation owner. Constructors
match declarations but inspect no retained fence evidence and interpret no capture wire
payload. The original mutation digest and allowances remain unchanged. The request hash
binds both expected fence fields. Requests, parameters, observations, target/fence
evidence and views have no Serde/default or persisted fresh-authority admission.

Model observation admission canonicalizes/sorts a nonempty set of at most 1,024
unique actual physical targets backed by the actual inventory. Passive provider input
retains current request, actual canonical context/full inventory, exact selected target
state and opaque stopped/drained evidence, actual membership revision, explicit evidence
lane, opaque qualified observation evidence and reported calls. Coordinated evidence
includes actual Active/Inactive fence state, identity, revision and required whole-unit
write/membership/timer/external-work fencing and drained-work evidence digests.
No parent/component ordering or unit grouping implies a distributed checkpoint.

`ports::consistency::ConsistencyProvider` observes existing obligations, with no
installed/default implementation or acquisition/release API. Providers qualify actual
authenticated context/selection/state, drain and continuously retained whole-selection
fencing across capture/interruption. Fresh challenges, timing, custody, evidence meaning
and prior approved per-call accounting remain integration-owned. Unknown custody fails;
known inactive fences produce denial. Unavailable/Unsupported reject before effects;
Indeterminate retains consumed allowance/evidence and obligations, then stops without
retry/reset/release. Failure, timeout, process death or dropping model values must not
release the integration's fence. Terminal replay invokes no provider.

Pure `policy::consistency::validate` matches exact current request/context/full inventory/
selection/call reporting, requires every selected target Stopped and the exact original
guarantee. Coordinated results must have the exact retained Active fence, with both actual
and fence revisions equal to the original retained revision. It performs no IO, serialization,
provider calls, plan/journal transition, dispatch, restart or release. Opaque evidence hashes
are integration attestations, not signatures or independently proven active fencing.
Sequential stops and before/after identity/revision equality cannot prove continuity.
The observed lifecycle names follow the
[primary management interface](https://docs.internetcomputer.org/references/ic-interface-spec/management-canister/);
their actual backend decoding/qualification remains unimplemented. See
[the typed current consistency contract](contracts/consistency-port.json).

Fresh native cases cover strict records and independent hash goldens, original plan/
budget/request sensitivity, fence/challenge/boundary/revision binding, inactive fences,
running/stopping members, complete inventory drift including unselected metadata,
exact selection/lane mismatches, canonical aliases/duplicates and 1,024-target admission.
Local persistence tests cover original requirement retention/no downgrade, contention,
1 KiB raw bounds, malformed/rebound records, symlinks and replaced layouts. A public
native provider fixture reopens original requirement and spent pending journals,
rejects stale/wrong-fence results and preserves exact retained integration obligation
fixture bytes across all typed failures/drop. These establish local contracts only;
application fence acquisition/release, actual lifecycle/drain/continuous custody,
capture completion and same-release restore/external-payment settlement remain pending.

| Canic surface | Extraction disposition |
| --- | --- |
| Hash helpers and artifact IO | Copied into pure checksum records and artifact ops; canonical decoding and UTF-8 identity strengthened |
| JSON IO, journal/file locking, artifact publication | Copied into persistence ops; record byte bounds/private permissions added; direct regressions rerun |
| Layout lifetime locks and reference retention | Copied with model-owned v1 validation/transitions, stable parent-side locking, directory identity and explicit bounds |
| Command lifetime locks | Copied with owned inheritance, exact v1 sidecar identity, one-spawn allowance and retained exclusive quiescence; fresh native process evidence |
| Reference release and prune | Remain in Canic until terminal completion, backend custody and runner integration govern release/deletion |
| Local download journal lifecycle | Adapted exact v1 identities, state/checksum transitions and derived views; guarded local verification/publication and explicit original-plan/exact-set published-byte checks with fresh recovery evidence |
| Pending claims and operation receipts | Adapted local exact-identity ledger, immutable separate allowances and chronological replay; guarded reservations/receipts with fresh native recovery evidence |
| Remote transfer extents, execution/restore journals, plans and manifests | Require complete backend metadata, generic v1 identity/budget binding and model-owned transitions before runner import |
| Topology hashing and declared registry target expansion | Adapted into bounded canonical forest records, unambiguous v1 binary hashes, pure exact/direct-child/subtree selection and immutable local persistence |
| Authoritative discovery, registry revisions and routing | Fleet/Root observation, membership authority and application fencing stay integration-owned; no live provider imported |
| Backup phase and restore member ordering | Adapted explicit bounded DAG records, deterministic planning order and pure causal declared-progress views; application dependency semantics remain adapter-owned |
| Plan structure, operation projection and original-plan resume binding | Adapted immutable declared context/inventory/selection/graph/request/allowance binding, canonical full intent and derived original attempt authority; complete backup/restore safety semantics and transfer/response codecs remain pending |
| Plan/journal integrity and resume reporting | Adapted complete exact journal binding, retained causal Applied checks and pure graph-ordered condition/assigned-accounting projections; no accepted-preflight, retry or terminal flags imported |
| Typed backup executor and snapshot/lifecycle command payloads | Adapted closed IC host-ingress method/target/raw-snapshot declarations, official Candid encoding and exact wire digests; argv rendering, transport selection, actual permissions/receipts and transfer remain with their owners |
| Topology preflight projection/admission and provider boundary | Adapted into ephemeral original-plan/challenge/boundary membership requests, fallible provider results and pure actual-context/full-inventory matching; freshness, permissions, fence/revision semantics and actual provider remain integration-owned |
| Control authority declarations, receipt headers and controller projections | Adapted into exact original IC mutation/challenge requests, complete known canonical controller sets and pure direct caller-controller admission; Root/Proven upgrades, read visibility, proxy/admin lanes, live observations and full preflight remain with their owners |
| Snapshot-read declarations/receipts and visibility projections | Adapted into original mutation intent plus independent exact list/challenge binding and pure actual-context/target/controller/public/viewer read paths; no Root-configured/Proven upgrades, status/log fallback, provider or paid-call admission imported |
| Quiescence declarations, receipt admission and consistency-unit projection | Adapted immutable original requested guarantee plus ephemeral exact stopped/drained/fence evidence matching; accepted/expiry/RootCoordinated flags and parent-derived application consistency are not copied; real fencing/custody and acquisition/release remain integration-owned |
| Backup and restore runners | Require the reviewed generic ports and uncertain-effect reconciliation; not copied in this batch |
| ICP subprocess transport | Narrow extraction into the transport package after executor contracts and selected backend capabilities are qualified |
| Local prune and CLI integration | Generic retention belongs here after layout/reference contracts; Fleet-facing commands remain Canic-owned |
| Operational-readiness/IC fixtures | Port against the actual extracted behavior as its owners move; current native evidence does not qualify IC effects |
| Timestamp and strict-field serialization helpers | Introduce with the records that need them; no unused helpers or compatibility wrappers copied |

The source consumer trace identifies `canic-cli` as the dependent package. Its
backup/restore commands consume layout, manifest, plan, journal and runner APIs;
list/cycles/metrics also consume discovery failures. Canic Host owns the ICP
transport used by the CLI executors. Existing prune depends on layout lifetime
and durable restore references. No consumer was changed or old source removed.

Source plans still bind Fleet, environment, Root identity/control and
Root-coordinated quiescence. Restore has relocatable identity/mapping surfaces
outside this product's same-ID scope. The restore executor accepts rendered
process output, while backup injects typed status/snapshot methods. These
contracts must be reshaped before moving runners. The current Canic backup
executor's topology preflight still rejects; copying local IO does not fix it.

Native qualification reruns the copied checksum, JSON, publication and lock
regressions. New cases cover canonical record decoding, private staging, UTF-8
identity, bounded reads and real acknowledged child-process death before/after
JSON and directory publication. A public integration journey persists original
checksum intent, adopts a published tree after a lost reply and rejects changed
bytes without changing that intent. Full B1 authority/backend qualification,
full B2 journal/runner extraction and live backup/restore remain incomplete.
