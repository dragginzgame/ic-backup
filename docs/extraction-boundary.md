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
Original fence obligation retention is a local contract refinement; its
[fresh source inspection](fence-obligation-source.json) distinguishes Canic
terminal/custody behavior from unimplemented generic acquisition/release.

## Maintained local contracts

`model::artifacts::ArtifactChecksumRecord` is the maintained v1 checksum record.
Its JSON fields are exactly `algorithm` and `hash`; missing/unknown fields reject.
The algorithm is exactly `sha256`. Digests have 64 hexadecimal digits and are
normalized to lowercase during construction/deserialization. Record equality
uses canonical values. ASCII case normalization precedes shared raw digest parsing;
typed malformed-hash errors retain the original input. No predecessor readers
or alternative schema generation is maintained.

Raw digest-text fields share the same crate-internal normalizer without constructing
a checksum record solely for string projection. Plan, attempt identity/history,
inventory, requirement, download and restore-reference boundaries retain their
original typed errors, validation order and canonical bytes. Journal reopen still
admits caller-supplied intent before locking/reading retained evidence.

`ops::artifacts` delegates raw stream checksums and private staging copies to
`ic-host-artifacts` using constant working memory. Both helpers retry interrupted
reads. Copying retains partial output on input or sink failure; neither helper
returns a complete checksum on failure.
The IC artifact verifier supplies its admitted descriptor length to the shared
bounded hash owner and checks the returned byte count; short and excess streams
reject as `FileShape`. Descriptor/path identity checks remain independent.
The copy has no new total artifact-size ceiling. Unix traversal opens
descriptors without following artifact symlinks, checks actual entry types and
rejects traversal components and special entries. Directory digests preserve
the source recipe: sort relative UTF-8 paths, then hash each path, NUL, its
lowercase file digest and LF. Empty directories and filesystem attributes do
not participate. Non-UTF-8 entry names reject rather than collapse into the
same lossy path representation. The list of file digests is collected in memory;
this primitive does not implement an operation-wide resource budget.

The public I/O-free `ops::artifacts::checksum_relative_files` consumes owned
`(PathBuf, ArtifactChecksumRecord)` rows under the same framing owner used by
traversal, staging, durable directory sync and exact IC-tree verification. It
sorts by the original path-component order, which differs from complete-string
ordering. Exact UTF-8 relative names must have nonempty normal components, no
NUL, dot/parent/trailing/repeated separators or duplicate identity. Newline and
literal Unix backslash filename bytes remain exact; Unicode is not normalized.
Empty file sets retain the empty SHA-256. Declared rows do not prove a materialized
tree, byte custody, completeness, synchronization or publication.

`DirectoryChecksumError` owns non-UTF-8, malformed and duplicate input refusals.
The existing exhaustive `ArtifactError` keeps its variants: internal non-UTF-8
conversion retains `NonUtf8Path`, while other impossible internal row refusals
retain their typed cause under `Io(InvalidData)`. Valid records and digest bytes
do not change. Public goldens distinguish path ordering from string ordering,
retain nested/Unicode/control-byte identities and reject aliases/duplicate rows.
A real file tree matches composition; retained declared rows still compose after
the original tree is removed, without granting fresh integrity or custody.
The public composed identity also passes through durable publication and canonical
recovery with component-order-sensitive filenames. Changed staging or canonical
bytes reject under the declaration, retaining their actual paths and bytes without
publishing or recopying. This is Backup's public integration proof; downstream
Canic descriptor custody and crash barriers still need their own qualification.

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

`create_json_durable` and `write_json_durable` delegate sibling allocation, identity
checks, cleanup, atomic create-only/replacement and final parent synchronization
to published `ic-host-fs::durable::write_at_with` beneath a held syncable directory.
Backup retains explicit 0600 files, private 0700 parent creation/link syncs and
one-pass pretty-JSON serialization before filesystem effects. The resulting byte
buffer is intentional: streaming a generic serializer into staging cannot preserve
that ordering, and preflight followed by streaming would invoke it twice.

`PersistenceError::Publication` retains Host's original typed producer, cleanup
and before/after-publication errors. Parent-preparation IO and serialization JSON
errors keep their own boundaries. The public exhaustive enum changed in
0.6.0; callers must update matches and reconcile visible output after failed
completion. All v1 bytes, original limits, spending and journal transitions remain
unchanged. Neither helper grants operation or retry authority.

Released 0.8.1 selects the Host 0.7.1 `NamedWriteError` Rust type identity. The
released 0.9.0 selected Host 0.8. Pending 0.11.0 selects incoming direct Host 0.9,
changing that public Rust identity again while preserving variant shapes and v1
records. Testkit retains its independently published dev-only Host 0.8 graph.
Direct Host dependencies sharing that value must use the same compatible line.
The new closed-writer executable-admission API has no record-writing purpose;
JSON publication keeps this single held-parent adapter and preflight contract.

Native artifact, publication and lock syscall failures use Rustix's canonical
conversion into `io::Error`, retaining the original OS code before the existing
typed local projection. No local errno conversion, retry or cleanup owner remains.

Ordinary calls and crash fixtures use one adapter and the same Host engine. The
producer writes and synchronizes bytes before acknowledging the pre-publication
barrier; Host then independently verifies/synchronizes staging before publishing.
Successful Host return includes the held-parent sync and precedes the second
acknowledged barrier. Real child-death cases retain attempt/download/reference/
manifest/settlement evidence. Foreign staging replacements preserve both original
and foreign bytes with typed cleanup errors; moved-parent publication stays beneath
the held descriptor. This is sequential local evidence, not a namespace fence.

Typed record output admission and restore-reference retention use the shared
`BoundedWriter` over a counting sink to check the exact pretty-JSON budget without
collecting an encoded record. Inclusive limits and `RecordTooLarge` retain their
original meaning; other serialization errors remain JSON failures. Schema and
transition owners choose the existing budgets, and durable encoding/publication
remains separate. See [the shared-tool adoption review](ic-host-tools-adoption.json).

`read_json` requires an explicit byte limit, reads at most that limit plus one,
and rejects excess bytes before decoding. Unix reads delegate to registry
`ic-host-fs::read::read_file_no_follow`, using final-component no-follow,
nonblocking descriptor admission and bounded, fallibly allocated regular-file
reads. Local projection retains exact `RecordTooLarge` limits, original IO errors
and `InvalidInput` for nonregular entries; empty bytes still fail JSON decoding.
Record parents are operator-owned trusted directories; ancestor aliases remain
caller-selected and are not new confinement admission. No records are written by
this read. Durable publication, tree hashing, locks/journal transitions and command
descriptor custody remain local. See [the shared-reader review](ic-host-tools-adoption.json).
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
Success keeps an exclusive non-spawning guard until dropped. Dropping that guard
explicitly unlocks its file description, so a transient unrelated fork/descriptor
copy cannot extend exclusion. Dispatched owner custody still closes without
unlocking, preserving real inherited child/descendant custody. See
[the native release finding](https://github.com/dragginzgame/ic-backup/issues/13).
A replaced sidecar,
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

Complete snapshot transfer coverage, full status metadata, live transport,
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
This permission port admits only the existing list codec. Separate metadata/data
codecs grant no fresh read permission; actual providers and complete read preflight
remain unimplemented. See
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

### Original restore safety requirements and current load/start checks

`model::restore_safety::RestoreSafetyRequirementRecord` retains strict v1 `version`,
canonical full `plan_intent`, full `source_plan_intent`, exact `source_artifacts`,
explicit `safety` lane and required nullable `expected_fence`. Both declared plans
must share network/release, and every selected restore ID must have been selected
in the source. A restore subset is allowed only with application qualification;
source caller may differ from the current caller. The application-fenced lane
retains exact original identity, membership revision and external-obligations
revision; the no-irreversible-effects lane requires explicit null. Missing/unknown
fields, other versions, inappropriate fence lanes and malformed hashes reject.
Canonical domain-separated hashing binds every original field. See
[the schema and independent binary goldens](contracts/restore-safety-requirement.schema.json).

`create_restore_safety_requirement` requires both exact plans already retained
under their unchanged layout guards, then durably creates fixed
`restore-safety-requirement.json` under journal exclusion without replacement.
`read_restore_safety_requirement` checks both retained plans, the original source
binding and exact expected requirement digest. Raw input and canonical output
have a 1 KiB ceiling. Lost local creation replies reconcile through exact reads;
missing/corrupt/changed evidence never changes the lane, source or obligation.
This declaration does not verify artifacts or automatically retain a source
reference; those remain separate existing owners before any recoverable effects.

`RestoreSafetyRequest` derives exact original operation authority and validates
original load/start target/method/Candid bytes, including the raw load snapshot ID.
It binds the original requirement, operation, wire digest, integration-owned fresh
challenge and 0–1,024 descriptive call ceiling. The wire digest owns the effect
boundary, with no independent mutable phase flag. Canonical current observations
admit 1–1,024 unique normalized inventory-backed target rows, upstream
`CanisterStatusType`, lifecycle/drain evidence and nullable source-specific restored
acceptance, actual context/full inventory/source bindings, application safety lane,
opaque evidence and actual calls. Requests, observations and views have no Serde,
default provider, persisted Proven/expiry or fresh-authority flag.

Pure `policy::restore_safety::validate` requires exact request/context/full
inventory/original selection/source/lane and bounded actual calls. Load needs every
selected target Stopped with integration-qualified drain evidence. Start needs its
exact target Stopped and every selected restored state accepted for this source;
other selected targets may already be Running under the integration's controlled
execution contract, while a still-Stopping member rejects. Graph order and actual
receipt/dispatch chronology remain
separate, with no universal parent or canonical lifecycle order.

The explicit no-irreversible-effects lane requires actual application knowledge
about restored intent/timers; there is no generic standalone assumption/default.
The fenced lane requires current Active exact original fence/revisions plus
whole-selection write/membership/timer/external-work/drain, continuous
rewind-independent custody and qualified external-obligation/replay prevention
evidence. Before start it additionally requires current controlled-execution
evidence. Known unresolved external obligations and inactive/rebound fences deny.
Settlement alone cannot qualify restored work that may replay an irreversible
effect. Hash equality and passive DTO construction cannot authenticate these facts.

`ports::restore_safety::RestoreSafetyProvider` has no installed implementation.
Integrations qualify complete original source artifacts and target-local uploaded
snapshot association, actual freshness/release/lifecycle/drain, outside-snapshot
fence custody/revisions/replay safety, restored acceptance and prior per-call
accounting. A descriptive ceiling grants no spending. Unavailable/Unsupported
reject before effects; Indeterminate retains evidence, original spent authority
and obligations and stops without retry, reset or release. Dropping values,
timeouts and process death release no obligation. Terminal replay invokes no
provider. No view settles a lost load, proves complete transfer, signs/dispatches
load/start or grants terminal, fence or source-reference release. See
[the current typed contract](contracts/restore-safety-port.json).

Fresh native tests cover strict schema and independent hash goldens, source and
original attempt sensitivity, canonical bounded targets, exact load/start payloads,
source/network/release/selection/lane mismatch, stale challenges, full inventory
drift, active original fence revisions and required restored/controlled-execution
evidence. Persistence tests reject downgrade/replacement, missing original plans,
oversized/malformed/rebound records, contention, symlinks and either replaced layout.
A public passive provider fixture reopens original source/safety/spent attempts,
rejects stale/inactive/rebound results and preserves exact requirement/journal/
outside-source obligation bytes and unfinished restore references across failures
and drop/reopen. These qualify local contracts, not actual IC or application safety.
[Fresh Canic inspection](restore-safety-source.json) identifies source ownership;
no runner, framework mapping or hash-only restored verification was copied.

### Original application fence obligation retention

`model::fence_obligation::FenceObligationRecord` is the immutable v1 declaration
for the full original selected unit. It binds the canonical original plan digest,
explicit application acquisition sequence and exact purpose. Capture binds the
original coordinated requirement, fence identity and membership revision. Restore
derives its original requirement and fence/membership/external-obligation revisions
from the existing fenced safety owner, including source/artifact identity. Per-canister
capture and no-irreversible-effects restore lanes reject; absent operations and
zero original mutation allowances reject. The physical operation target describes
routing, never whole-unit coverage or authority. Integrations qualify the opaque
application request's exact semantics and chosen identity before dispatch.

The strict [schema and independent binary goldens](contracts/fence-obligation.schema.json)
reject unknown/missing fields, other versions, invalid checksums and invented
purposes. SHA-256 uses the NUL-terminated `ic-backup/fence-obligation/v1` domain,
64 ASCII original intent bytes, big-endian u64 acquisition sequence, purpose byte
(capture=0, restore=1), then 64 ASCII requirement, fence identity and membership
revision bytes. Restore appends 64 ASCII external-obligations revision bytes.
Mutable status, receipts and release flags are absent.

`create_fence_obligation` publishes fixed `fence-obligation.json` with no replacement
under layout and journal exclusion. `read_fence_obligation` admits raw/canonical
1 KiB bounds, the expected digest and exact original plan/requirement bindings.
Capture checks the retained consistency requirement; restore checks both retained
plans/unchanged layout guards and the retained restore safety requirement. Changed,
missing, malformed, oversized, unsafe or rebound evidence rejects without repair.
Local lost publication replies reconcile through the exact read, never recreation.
Publish this declaration before acquisition reservation/dispatch. Source-reference
retention remains separately required before restore journals/effects.

`policy::fence_obligation::acquisition_progress` checks exact original journal
authority, including context, request, operation and both original limits, and
returns its existing `AttemptJournalView`. No second spending counter/state machine
exists. Missing evidence cannot be passed as zero consumption. Lost observation
replies stay pending; settled Uncertain observations leave the mutation unresolved
and no outcome refunds attempts. Full dependency admission remains with
`execution_progress`; this projection proves no cross-journal chronology or receipt
authenticity. Applied acquisition evidence is not fresh Active custody. Pending,
Applied, NotApplied and Uncertain outcomes all retain the original obligation.
There is no automatic release/disposition on drop, timeout or process death.

Fresh native tests cover strict schema/goldens, purpose/lane/source/revision/plan
binding, original authority/allowance denial, immutable publication, retained
requirements, both layouts, private permissions, contention, unsafe and excessive
bytes. A public filesystem journey reopens exhausted pending acquisition and
observation reservations, records each passive native settled outcome and preserves
exact obligation/journal bytes and unfinished source references through drop/reopen.
These are local persistence/recovery checks, not application/IC effects. No
`ConsistencyCoordinator` implementation, authenticated reconciliation, signing,
dispatch, current Active proof, terminal proof or fence/reference release is
implemented. [Fresh Canic inspection](fence-obligation-source.json) copies no code
and supplies no generic application-fence release qualification.

### Reserved fence acquisition reconciliation

`model::fence_reconciliation::FenceReconciliationIntent` binds the retained original
obligation and full-plan-derived acquisition authority, including original limits,
to an existing pending mutation and caller-owned challenge. Intent hashing precedes
observation reservation; `FenceReconciliationRequest` requires that exact pending
observation. The journal's `pending_observation_request` reads its existing validated
projection without changing v1 records, counters or transitions. Structural admission
is not a dispatch permit; durable custody and no prior dispatch remain integration-owned.

The [typed contract and independent binary goldens](contracts/fence-reconciliation-port.json)
hash the NUL-terminated `ic-backup/fence-reconciliation/v1` domain, lowercase ASCII
obligation/authority hashes, big-endian u32 mutation attempt and challenge hash.
Actual results bind both mutation and observation IDs separately, rejecting replay
across reservations even when a challenge is reused. Passive observations require
canonical unique actual targets within 1,024 entries and the existing combined
1,024-attempt bound. The single-remote-observation descriptive ceiling grants no
call allowance or fresh permissions; batching paid calls under one reservation is forbidden.

`policy::fence_reconciliation::validate` rechecks the unchanged current journal and
matches exact actual context, full original inventory, selected unit and request/IDs.
Acquired capture/restore evidence reuses existing whole-unit safety fields, requiring
the exact Active fence and original revisions plus exclusive original-request
attribution. Restore additionally retains rewind-independent custody and replay
safety; fresh load/start admission remains separate. Current absence or inactivity
cannot establish nonapplication: negative evidence must exclude transient acquisition
and release. Only an authenticated settled observation can claim unresolved uncertainty.

The returned view is a passive outcome claim. It performs no IO, serialization,
provider call or automatic journal receipt. Only a separately qualified integration
can submit an actual receipt to existing guarded transition owners. Provider failure,
drop or process death leaves the observation pending and spent; retained late replies
may be admitted without another provider call. All outcomes retain obligations and
source references, with no refund, acquisition retry or fence/reference release.
Terminal replay invokes no provider. There is no installed authenticated provider,
acquisition/release implementation or terminal proof.

Ten focused unit cases and two public native recovery journeys cover exact identities,
bounds, negative/uncertain claims, replay rejection, all provider failures, exhausted
spending, late replies and reopen retention. Passive native fixtures qualify local
contracts and persistence only. [Fresh Canic inspection](fence-reconciliation-source.json)
copies no product code and supplies no generic application-fence attribution proof;
actual freshness, authenticated association, exclusive chronology, accounting and
custody require application/backend qualification.

### Exact application fence acquisition requests

`model::fence_acquisition::FenceAcquisitionPayload` owns an immutable exact
application receiver/method/argument envelope. Receiver equals effective routing
target; the management canister is excluded. Method spelling is 1..128 visible ASCII
bytes and opaque arguments are at most 1 MiB, checked before copying. Debug output
retains length but excludes argument content. There is no query/default/proxy lane
or generic application argument codec. Integrations own retained original bytes and
their whole-unit purpose/fence/revision semantics; the routing target narrows no scope.

The [typed contract and independent goldens](contracts/fence-acquisition-port.json)
specify SHA-256 over the NUL-terminated `ic-backup/application-fence-acquisition/v1`
domain, u8 principal length/raw canonical identity, fixed update byte 1, big-endian
u32 method length/exact bytes and u64 argument length/exact bytes. Payload construction
precedes the plan; later original plan/requirement/obligation owners bind the payload
without recursive hashes. No persisted schema, counter or new accounting owner is added.

`FenceAcquisitionRequest` validates exact original obligation/plan and journal
authority/limits, payload target/digest and already pending mutation. A pending
observation rejects mutation association; current journal admission is rechecked for
acknowledgements. The structural request supplies no fresh dispatch permission or
proof of no earlier dispatch. Qualified integrations must retain original inputs
and source references, establish actual context/whole-unit membership/permissions,
application semantics, prerequisites, exclusive custody and no prior dispatch before
one original accounted update. Hidden retries, batching, proxy/query substitutions,
funding and extra remote observations are forbidden within that invocation.

`FenceAcquisitionAcknowledgement` is passive full authority/attempt/reply-evidence
association. Pure `validate_acknowledgement` performs no IO, serialization, provider
call or transition and returns no outcome. An associated acknowledgement proves no
acquisition, actual authentication, Active custody, exclusive attribution or negative
effect. Qualified direct settlement remains integration-owned through existing
journal transition owners. Provider failures, drop and death retain pending spent
attempts, obligations and source references. Unknown dispatch/lost replies require
reserved reconciliation; request reconstruction never authorizes another mutation.
Terminal replay remains provider-free; no installed provider, live effects or release
API is implemented.

Focused native cases cover independent envelope bytes, canonical targets, bounds,
original authority/budget drift, pending/settled/recovering requests and passive
acknowledgement association. A public native journey retains exact inputs, obligations
and references across acknowledgements/all provider errors, reopen and handoff to
reserved reconciliation without another mutation invocation. These fixtures establish
local association/persistence only. [Fresh Canic inspection](fence-acquisition-source.json)
imports no production code, default command, effect flag or dispatch/outcome proof.
Actual application/IC backend qualification remains required.

### Immutable local download manifests

`DownloadJournalGuard::publish_download_manifest` publishes the exact guarded
Durable download record at fixed `download-manifest.json`. It reuses the existing
strict v1 `DownloadJournalRecord` schema and model/policy owners, adding no second
progress ledger or mutable manifest state. Existing bounds remain 1 MiB raw and
canonical IO, 1,024 artifacts and 256 ASCII snapshot-token bytes. Every selected
target needs its retained exact snapshot metadata, derived paths and checksum.

`DownloadJournalRecord::digest` owns canonical binary identity for all four local
states. A NUL-terminated `ic-backup/download-journal/v1` domain precedes the 64 ASCII
intent and big-endian u64 entry count. Canonical principal-text rows each encode four
u32-length-prefixed UTF-8 strings (principal, exact token, staging and artifact paths),
two u64 metadata values, a state byte (Created=0 through Durable=3), a checksum
presence byte and the 64 ASCII hash when present. All integers are big-endian.
Identity normalization comes from existing record admission; no timestamp, release
tool metadata or snapshot provenance is regenerated. See the
[contract and independent goldens](contracts/download-manifest.json).

Publication borrows exclusive layout/journal custody, holds a manifest lock and
calls the existing fresh durable-artifact verifier before private 0600 durable
no-replace publication. All selected trees are checked with existing no-follow
traversal. Missing/changed originals, incomplete sets, unsafe trees, checksum drift,
contention and existing destinations reject without repairing records or bytes.
Cooperating writers use layout exclusion; noncooperating stable byte custody remains
integration-owned. Sequential checksum observations are not an atomic snapshot.

`read_download_manifest` is explicit exact local replay after publication or a lost
reply. It admits the bounded regular no-follow manifest under its expected digest,
the full original retained plan, exact selected Durable set and equal retained
download journal. Active download guards must be dropped first. Artifact trees can
be absent or changed: replay reads records only and makes no provider calls, fresh
byte checks, writes, repairs, progress changes or allowance resets. Conflict and
unknown publication outcomes preserve all recovery evidence and source references.

Native tests cover independent hashes for every state and maximum metadata/token
values, canonical order/checksum drift, changed final-tree bytes, changed originals,
unsafe/excessive evidence, contention, immutable publication/lost replies and
acknowledged process death before publication and after directory sync. A public
reopen/replay journey retains unfinished restore references and fence obligations
with artifact trees moved aside. These are local model/filesystem/process checks;
they qualify no complete IC transfer, authenticated snapshots or application effects.

[Fresh read-only source inspection](download-manifest-source.json) records Canic's
manifest construction/publication and actual consumers. Its tool/time provenance,
parent-derived unit consistency, completed-operation receipt and exact-file adoption
are not imported. The local mechanism reuses maintained download owners and releases
no fence/reference. A full product backup manifest, authenticated capture/transfer,
consistency/effect evidence, command custody and terminal admission remain pending.

### Fresh original local restore-source verification

`policy::local_restore_source::validate` joins the existing original restore/source
plans, immutable safety requirement and exact local download manifest. It reuses
`RestoreSafetyRequirementRecord::validate_plans` for full original identities and
same declared network/release/existing selected-ID subset admission, then the
canonical durable-download policy for complete original source intent/coverage and
all Durable checksums. This opt-in local source binding requires `source_artifacts`
to equal the existing manifest digest; generic integration artifact digests keep
their meaning under existing owners. No record or persistence schema is added.

`LocalRestoreSourceView` borrows the complete original durable source and exposes
the exact restore artifacts in canonical principal-text order. Exact snapshot tokens,
timestamp/size, paths and checksums remain owned by their original records. Source
and restore callers may differ; matching context is no current control permission.
A restore subset needs actual application qualification rather than inferred parent
order or universal unit consistency. Policy performs no IO, serialization, transitions,
provider calls or scheduling. See [the typed contract](contracts/local-restore-source.json).

`DownloadJournalGuard::read_download_manifest` adds record-only replay borrowing
an already-held source journal. It reuses the manifest owner's bounded decoder and
digest/plan/Durable admission, requiring the retained journal unchanged before/after
the read. It acquires no second journal lock and reads no artifact trees. Existing
standalone replay retains its exact behavior and failure order.

`verify_local_restore_source` is explicit fresh local verification. The source journal
borrows source-layout exclusion; the returned view also borrows the restore layout,
both plans and requirement. The retained exact requirement/plans/manifest/journal are
admitted before/after existing no-follow verification of every original source tree,
including source artifacts outside the restore subset. Root replacement, changed
records/bytes, missing or unsafe evidence and contention return typed failures without
repair, copying, resetting progress or releasing obligations. Existing 1 KiB requirement,
1 MiB plan/manifest/journal IO, 1,024-target and 256-token-byte bounds remain unchanged.

Native tests qualify exact original/source digest and snapshot metadata, complete
source coverage, same-ID subset projections, differing source callers, non-Durable
or generic incompatible local bindings, changed unselected trees and unsafe/missing
originals, root replacement and manifest contention. A public recovery journey retains
spent pending attempts, exact requirement/manifest/journal/fence bytes and unfinished
source references through checksum failure and drop/reopen, with no provider calls.

[Fresh read-only source inspection](local-restore-source.json) traces Canic's restore
planner, snapshot validation, descriptor-copied source staging and actual consumers.
No relocation/mapping, parent ordering, optional-checksum readiness, staging/cleanup
or runner behavior is imported. This join proves local sequential byte observations,
not an atomic snapshot, future copied/upload bytes, authenticated original capture or
complete backend extents. Stable noncooperating bytes, application subset safety,
current permissions/dispatch and command custody remain independently qualified.
No upload/load/start, receipt, terminal or fence/reference release permit is created.

### Private original-operation restore artifacts

`DownloadJournalGuard::stage_local_restore_artifact` resolves one exact original
opaque operation sequence and its selected source artifact through the existing
original-source view. Complete fresh source verification precedes the existing
descriptor-based no-follow copy. The destination is fixed as
`restore-artifact-{sequence}.tmp`, directly under the held restore layout and
excluded by its existing journal-lock owner. New directories/files use 0700/0600
permissions. Source and destination hashes must equal the original retained checksum;
exact retained requirement/plans/manifest/journal are re-admitted before returning.
No record, schema, digest encoder, accounting owner or dispatch permit is added.

An occupied destination rejects without adoption, replacement or deletion. Copy
errors, checksum drift, changed original admission and drop retain partial/full
bytes as recovery evidence. This is staging without fsync/durable publication.
Operation association proves neither an upload request nor actual transfer or load.
The returned `LocalRestoreArtifactView` borrows both layout lifetimes, original
plans/requirement and source journal; it exposes original operation/artifact identity
and a checked path. Holding the view does not freeze the filesystem or preserve a
snapshot against noncooperating writers. The operator-selected destination parent
and stable future byte custody remain integration-owned.

Explicit `verify_staged_local_restore_artifact` takes the original sequence and
re-admits exact retained original metadata before/after fresh no-follow copy hashing.
It reads no source trees and performs no copy: originals may be absent while their
immutable metadata remains required. Missing, unsafe, incomplete or conflicting
copies reject without repair/recreation. This is fresh retained-copy verification;
ordinary resume and terminal replay do not invoke it. Partial evidence needs separate
operator-owned disposition. Both paths preserve exact original spending, unfinished
source references and fence obligations. Existing original-source IO/target/token
bounds remain unchanged. See [the typed contract](contracts/local-restore-artifact.json).

Native cases qualify private permissions/exact bytes and original snapshot identity,
occupied/unknown-operation rejection, source changes during copy, missing/unsafe or
corrupt retained copies, requirement changes after copy, contention, injected partial
and lost copy replies, and acknowledged process death before/after copying. A public
journey reopens a retained copy with source trees moved aside and preserves pending
spent attempts and all original journal/manifest/requirement/fence/reference evidence.
[Fresh source inspection](local-restore-artifact-source.json) traces Canic's staging
and actual callers. Its stale-copy deletion, drop cleanup, upload gating and runner
execution are not imported. No transfer/extent/authentication, current permissions,
command dispatch, terminal proof or fence/reference release is qualified here.

### Durable original-operation restore artifacts

`DownloadJournalGuard::publish_staged_local_restore_artifact` is the explicit local
durable operation after private staging. It resolves the original opaque operation
and selected artifact through canonical retained original-source admission under both
layout lifetimes and the source-journal guard. It shares staging/verification's lock
on `restore-artifact-{sequence}.tmp` and delegates to `commit_artifact_directory`
with that fixed temporary path, canonical sibling `restore-artifact-{sequence}` and
the exact original checksum. Descriptor synchronization, finite-buffer hashing,
atomic no-replace rename and parent-directory synchronization keep their existing
owner. No new records, hashes, schemas or progress/accounting state appear.

Publication returns the existing checked original-artifact view plus explicit
Published/Recovered outcome. The canonical bytes and original retained plans,
requirement, manifest and journal are re-admitted after publication. Recovery
synchronizes and verifies the matching canonical tree when temporary bytes are
absent, without copying or reading source artifact trees. Original metadata is still
required even when source trees are unavailable. Both paths present, neither present,
changed/unsafe bytes, missing/changed original declarations and contention reject
without replacement, repair, automatic cleanup or effects. IO/admission failure may
leave the canonical publication complete; retain it for exact-path recovery.

`verify_published_local_restore_artifact` freshly admits retained originals and hashes
the canonical tree under the same operation lock. It performs no fsync/publication,
source-tree reads or recopying and does not infer durability from the path's existence.
Neither operation runs as ordinary resume or terminal replay. Existing original-source
IO/target/token limits and private copy permissions remain unchanged; stable
noncooperating parent/byte custody remains integration-owned. All journals, spent
attempts, fence obligations and source references remain unchanged. Success grants
no complete backend transfer, authenticated snapshot, current permission, command
dispatch, upload/load/start, application safety, terminal or fence/reference release.

Native tests qualify publication/recovery with source trees moved aside, exact bytes/
private permissions, missing/conflicting/changed/unsafe trees, lost replies before/
after actual durable publication, closing original/canonical drift and shared lock
contention. Acknowledged child death before/after actual publication retains exact
paths and recovers without cleanup. The public source journey now publishes and
checks the retained copy while preserving exact original journals/manifest/requirement,
fence obligation and unfinished references. Existing publication-owner regressions
also run freshly against the reused production implementation. See
[the maintained contract](contracts/local-restore-artifact.json) and
[fresh source inspection](local-restore-artifact-publication-source.json). Canic staging
cleanup, upload integration and all consumers remain unchanged; no live effect or
native macOS qualification is claimed.

### Exact originally reserved IC mutation updates

`model::ic_mutation::IcMutationRequest` derives the exact original operation authority
from the complete plan and binds an existing pending mutation to the unchanged
`IcManagementRequestRecord` target/digest/effect class. Only capture, load, start and
stop are admitted; status/list are separately accounted observations. Missing or
different original identity/allowances, changed bytes and pending observation recovery
reject. Binding and repeated reconstruction create no journal, spending or undispatched
proof. The request retains immutable plan/payload borrows and rechecks the current
pending reservation before association.

`ports::ic_mutation::IcMutationProvider::submit_mutation` describes one originally
accounted host replicated update with exact management receiver, routing target,
method and Candid bytes. No implementation is installed. Integrations retain originals
and qualify fresh actual permissions/context, complete prerequisite evidence,
capture consistency/quiescence, exact load-origin permissions and same-release
load/start source/upload/application safety, and exclusive dispatch/command/byte
custody. Pending state alone never admits
a repeat. Hidden observations/retries/funding/proxy/query substitution are excluded;
lost outcomes require original-effect reconciliation. Provider errors/drop/death
retain consumed pending mutations, obligations and source references.

`IcMutationAcknowledgement` retains a full original authority digest, mutation attempt,
actual claimed network/caller/release/target, exact raw reply and opaque evidence.
The existing 1,024-attempt and 1 MiB raw-reply bounds apply; target admission reuses
canonical principals and Debug redacts raw bytes. No Serde or Default flag exists.
Pure `policy::ic_mutation::validate_acknowledgement` rechecks current original evidence
and exact association claims, then delegates capture to `IcSnapshotReply` and
stop/start/load to `IcLifecycleReply`. Existing raw IDs, required metadata, exact empty
tuple, finite decoding work and request/raw-reply digests retain their single owners.
No additional codec, record, spending ledger or evidence hash is introduced.

Successful matching is a passive read-only wire projection. It authenticates no
network/caller/target/reply or timing, proves no exclusive attribution, and produces
no Applied/NotApplied/Uncertain receipt, fresh authority, restart/load safety, terminal
proof or fence/reference release. Retained late replies may be associated under the
same pending mutation before observation recovery; this performs no provider call.
Once a recovery observation is pending, association rejects rather than bypass its
original owner. Terminal replay never invokes a provider.

Native tests cover all four original envelopes, exact opaque load bytes, original
authority/budget/target/attempt/context drift, absent/replaced/settled/recovering
reservations, raw/attempt bounds, canonical targets, redacted Debug and method-specific
invalid wire. A public journey qualifies one passive fixture invocation and exact
retained journal/plan/raw argument/reply/reference bytes across drop/reopen for each
method and provider failure. It implements no IC behavior. See
[the typed contract](contracts/ic-mutation-port.json) and
[fresh read-only inspection](ic-mutation-source.json). Canic executor dispatch,
automatic completion/receipt conversion, restore upload and all consumers remain
unchanged. Actual providers, capture/upload/load/reconciliation and runners/transport
remain independently qualified.

### Exact originally reserved IC recovery observations

`model::ic_observation::IcObservationRequest` joins the full original plan/operation
authority, exact original mutation and canonical status/list observation payload with
both already pending attempts. It reuses the existing observation digest and journal
owner; original mutation target/digest/class and observation target/class/reserved
digest must match. Absent, changed or settled reservations reject. Construction and
reconstruction spend nothing, create no journal or record and grant no repeat call.
This is recovery of an existing mutation, not a fresh preflight reservation API.

`ports::ic_observation::IcObservationProvider::observe` describes one originally
accounted host replicated observation at the exact management receiver/routing target,
method and Candid bytes. Integrations retain durable originals and qualify actual
context, method-specific current read permission, recovery chronology and exclusive
command/dispatch custody proving this observation was never dispatched. Hidden
retries, extra observations, queries/proxies, funding and mutations are excluded.
Reconstructed pending state supplies no proof of prior nondispatch. Failures/drop/
death retain consumed pending observations and mutations, obligations and source
references; a lost observation cannot automatically mean settled Uncertain.

`IcObservationResponse` owns bounded passive original authority, both chronological
attempt IDs, exact observation digest, actual claimed context/target, raw Candid
reply and opaque evidence. Admission preserves the 1,024 total-attempt, 1 MiB raw
reply and canonical principal boundaries; Debug redacts bytes, and no Serde or
Default admission exists. Pure `policy::ic_observation::validate_response` rechecks
current reservations and every exact association claim, then delegates inventory to
`IcSnapshotReply` and status to `IcLifecycleReply`. Existing 1,024-entry/raw-ID/
metadata/controller/finite-work decoder limits, payload hashes and reply digests
retain their canonical owners.

Successful association is read-only wire evidence, not authenticated freshness,
exclusive attribution or Applied/NotApplied/Uncertain settlement. Stopped/controller
projections do not prove drain/load safety or current authority. Zero/one/multiple
snapshots do not identify the original capture's outcome. No receipt, retry, refund,
mutation/restart, terminal proof or fence/reference release follows. Retained late
replies can be matched under the exact pending observation without calling a provider;
settled/replaced original reservations reject. Terminal replay invokes no provider.

Native unit tests exercise exact identities, changed budgets/bytes/classes/targets/
attempts/context, missing/settled/replaced reservations, response bounds, Debug
redaction, malformed method-specific wire and unchanged spending for zero/one/multiple
inventory entries. The public recovery journey covers both observations for all four
original mutations, passive replies and three provider failure variants; exact
retained original journals, arguments, plan, opaque fixture obligation marker and
source references survive drop/reopen with no reissue or automatic outcome. These
fixtures simulate no IC effects or application fence. See
[the contract](contracts/ic-observation-port.json) and
[fresh source inspection](ic-observation-source.json). Canic consumers and its status/
inventory-to-completion conversions remain unchanged and are not imported. Actual
providers, authenticated reconciliation, upload/load and transport/runners remain
independently qualified.

### Bounded IC snapshot metadata

`model::ic_snapshot_metadata::IcSnapshotMetadataRequest` owns a separate ephemeral
transfer read for a canonical target and exact raw snapshot ID. It encodes the
pinned SDK `ReadCanisterSnapshotMetadataArgs`, names the management receiver and
`read_canister_snapshot_metadata` replicated host update method, and reuses the
existing management payload digest. The six-method v1 lifecycle/recovery record,
its schema, bytes and existing digests remain unchanged. No new persisted record,
provider, observation port or spending owner is introduced.

`IcSnapshotMetadataReply::decode` retains one pinned metadata argument under 1 MiB
raw input, 2 MiB decoder work, zero skipped work, 64 type-table entries and a
16 KiB header limit. Required nat64 fields retain their full range without summing
sizes or equating them with local artifact lengths. Bounded visitors admit at most
4,096 ordered globals, 1,024 distinct ordered SHA-256 chunk identities and 32
certified-data bytes, without allocating from untrusted length hints. The v128
global must fit unsigned 128 bits; floating globals retain exact bits. The source
adapter reads the actual Candid reserved marker rather than using the SDK's
arbitrary-value skipping deserializer. Malformed/unknown optional values, skipped
extensions, extra arguments and trailing bytes reject.

The pinned SDK's optional source/global slots and timer/hook values preserve actual
absence; required source/global wire values also admit losslessly into those slots.
Absent optional fields/slots grant no upload default or reconstruction completeness.
Chunk order and global order remain unchanged. Read-only access exposes the pinned
DTO while the evidence digest binds the exact request hash and raw byte checksum
in `ic-backup/ic-snapshot-metadata-reply/v1` followed by NUL and two 64-byte lowercase
ASCII hashes. Equivalent decoded views can retain different wire evidence.
Debug and typed errors omit snapshot bytes, globals, certified data and raw decoder
diagnostics. Requests/replies have no Serde admission or non-neutral Default.

Independent DIDL/field-hash/LEB128/numeric/SHA-256 fixtures qualify every registered
case against production and official SDK decoding. Native regressions cover exact
request/reply hashes, full numeric values, float bits, optional states, ordered
chunks, bounds, malformed/unknown fields, truncation and finite headers. The public
journey reopens exact retained metadata with original plan, pending consumed attempts
and source references unchanged. These tests call no IC backend and authenticate
no origin. Fresh read access/accounting, snapshot association, all data extents,
application/byte/command custody and upload/load/start/terminal/release admission
remain integration-owned. See [the generated contract](contracts/ic-snapshot-metadata.json)
and [fresh review](ic-snapshot-metadata-source.json). Canic's whole-command download
and its completed-transfer/receipt assumptions are not imported; consumers remain
unchanged. No existing public/private function, method or type is removed.

### Bounded metadata-bound IC snapshot data

`model::ic_snapshot_data::IcSnapshotDataRequest` borrows exact retained metadata
and its canonical target/raw snapshot ID. It encodes the pinned SDK's data-read
arguments for Wasm module, Wasm heap, stable memory or a complete chunk-store
entry. Range size must be nonzero and at most 1 MiB; checked nat64 offset plus size
must fit its own retained region. A chunk identity must be exactly 32 bytes and
present in original metadata. No generic token, unchecked region, hash fallback or
default request appears. The sole replicated host update method is
`read_canister_snapshot_data`; routing and argument digests reuse the original
management owner without changing the six-method v1 record or metadata schema.

`IcSnapshotDataReply::decode` admits exactly one record with a required chunk blob.
Bounded visitors retain at most 1 MiB data without allocating from untrusted length
hints. The separate raw limit is 2 MiB to allow wire overhead around maximum chunks;
decoder work is 8 MiB, skipped work zero, type-table entries 16 and headers 4 KiB.
Existing lifecycle/recovery raw limits remain unchanged. Unknown extensions, missing/
wrong fields, extra arguments, trailing bytes and hostile work/lengths reject.
Range replies require exact requested size; chunk-store bytes must hash to the
requested original SHA-256, including known empty chunks. These checks authenticate
no target or origin and cannot attribute or settle a lost mutation or observation.

Data checksum and raw-wire checksum stay distinct. The reply evidence uses
`ic-backup/ic-snapshot-data-reply/v1` plus NUL, followed by the exact original metadata
reply, data request and raw reply hashes, each 64 lowercase ASCII hex bytes.
Wire request identity excludes metadata from its nonrecursive payload hash;
changed metadata remains visible in reply evidence even with equal request/data
bytes. Read-only owners expose no Serde, persisted progress or non-neutral Default.
Debug and errors omit actual data, arguments, raw IDs/hashes and decoder diagnostics.

Independent generated DIDL/field-hash/LEB128/nat64/SHA-256 fixtures admit every
registered kind through production and official SDK decoding. Native cases check
exact bytes/hashes, all region boundaries, zero/oversized/overflowing ranges,
maximum range/chunk data, exact lengths and known empty/nonempty chunk hashes,
changed metadata, malformed/truncated/skipped wire and finite headers. The public
journey reopens retained metadata/data for every registered kind while preserving
original plan/journal bytes, pending consumption and source references. It performs
no provider call, simulated IC effect or automatic settlement.

This is single-call codec/association qualification, not aggregate transfer coverage.
Complete no-gap/no-overlap regions and all chunk bytes, authenticated current
snapshot/context association, fresh read access and prior per-call reservations,
stable byte/command custody, upload/load/start/application safety and terminal or
fence/reference release remain separately qualified. Canic whole-command downloads
and directory-to-completion assumptions remain unchanged and are not imported.
See [the generated contract](contracts/ic-snapshot-data.json) and
[fresh review](ic-snapshot-data-source.json). No existing public/private function,
method or type is removed; no journal, manifest or original allowance changes.

### Incremental declared snapshot data coverage

`model::ic_snapshot_coverage::IcSnapshotDataCoverage` is the ephemeral owner of
coverage over already admitted replies. It compares exact metadata request/raw
evidence digests, advances three independent nat64 cursors only from each region's
next contiguous offset, and admits each original chunk-store identity once.
Regions can interleave; chunk arrival order is independent of metadata order.
Failed admission leaves all state unchanged. Empty regions need no reads, but
known empty chunks still need their actual hash-checked reply. `complete()` exposes
a borrowed read-only view only when all sizes and chunks are exactly covered.

Retained state is three cursors and at most 1,024 presence bits; no data buffers,
aggregate-size sum, new hash, serialization, journal or accounting owner appears.
Each admission reuses the existing bounded request/reply owners. Reconstructing
coverage starts empty; it cannot reconstruct missing durable evidence or reset
attempts. Admission order is not dispatch order or an allowance. Complete local
coverage proves no authenticated origin, stable bytes, durable artifact, backend
transfer, mutation settlement, upload/load/start, terminal or fence/reference release.

Native tests exercise interleaved exact coverage, gap/overlap/duplicate rejection,
unchanged state on errors, changed metadata/target/raw ID, exact metadata redecoding,
zero-size regions, empty stored chunks, all 1,024 chunks and streamed 1 MiB replies
under independent maximum-nat64 regions. Public replay retains original spent
journals and source references without importing coverage as persisted progress.
Real-backend transfer and application qualification remain pending. Source review
extends the existing [data review](ic-snapshot-data-source.json), without importing
Canic whole-command completion assumptions or changing its consumers.

### Durable metadata-bound IC snapshot artifacts

`DownloadJournalGuard::stage_ic_snapshot_artifact` borrows the original guarded
Created entry and exact decoded metadata. The canonical target and timestamp must
match; the caller's generic backend token remains unchanged and its association
with the distinct raw IC ID remains integration-owned. Independently observed
total snapshot size is preserved, without summing metadata regions. Raw metadata
must match the decoder's retained checksum and 1 MiB bound before any creation.

The distinct `ic-backup/ic-snapshot-artifact/v1` tree has fixed direct children:
`format`, exact `metadata.candid` and `metadata-arguments.candid`, three region
files (`wasm-module.bin`, `wasm-memory.bin`, `stable-memory.bin`), and one
`chunk-{lowercase-sha256}.bin` per original chunk. Original raw metadata retains
globals, optional fields, certified data and ordered chunk identities without
re-encoding or upload defaults. Empty regions have actual empty files; empty
known chunks require their actual admitted reply. New directories/files use
0700/0600 permissions and descriptor-relative exclusive/no-follow creation.

`IcSnapshotArtifactWriter::append` reuses the canonical coverage owner, retaining
each admitted reply's exact bytes with three incremental region hash states and
at most 1,024 chunk checksum rows. Each chunk is bounded to 1 MiB; no aggregate
buffer, total-size sum, new progress record or spending owner appears. Root,
artifact-parent and staging identities are rechecked. Before admitting another
reply or writing, all three region names must still select their held regular-file
descriptors at the exact already-covered lengths. Replacement, symlink/directory
substitution and truncation/extension reject with `FileShape`; a missing name
retains the native IO error. The check performs at most six metadata syscalls,
without new opens, allocations or retained counters. Sequential identity/extent
checks do not fence noncooperating writers or detect same-length byte changes;
final fresh checksum verification remains required. See
[issue #21](https://github.com/dragginzgame/ic-backup/issues/21).
An error consumes the writer, closes descriptors and retains partial evidence
with no journal transition.
Drop does not delete files. Occupied staging/canonical paths reject recreation.

Explicit `finish` requires complete coverage and a closed fixed tree. Fresh
no-follow hashing must match checksums derived from actual admitted bytes,
including exact original metadata/request. Existing model transitions derive
Downloaded then ChecksumVerified; one atomic journal publication retains their
expected checksum before the existing descriptor synchronization/no-replace
artifact publisher and Durable transition. There is no new digest encoder or
journal/schema flag. Failed persistence can require reopen; failed publication
can retain exact staging or canonical bytes. ChecksumVerified recovery uses
ordinary `finalize_artifact` without another transfer. Created partial staging
is not reconstructed as coverage and supplies no read retry permission; its
disposition remains operator-owned. Ordinary Durable replay reads retained
progress without artifact IO. Fresh verification remains an explicit operation.
Closing root, parent and published-directory identity checks can reject after
Durable evidence was retained; such a failure preserves that completed local
evidence without repairing or replacing paths. Holding guards does not fence
noncooperating file mutations.

`DownloadJournalGuard::verify_ic_snapshot_artifact` is the distinct opt-in fresh
retained IC-tree check. It requires the full original persisted plan, unchanged held
journal and canonical complete Durable selection through the existing integrity
policy. Only the requested target's artifact is read. Exact target/token/timestamp,
format, original metadata/request hashes, three nat64 region lengths and every known
bounded chunk hash must match; the closed direct-child tree uses the existing retained
whole-tree checksum. Generic tokens remain separate from raw IDs. This is stronger
shape admission for the opt-in format, without changing generic artifact verification.

Descriptor-relative no-follow/nonblocking opens reject symlinks, directories and
special files. Observed length bounds apply before streaming through the canonical
64 KiB checksum buffer; reads stop at the observed length plus one even if a writer
grows a file. Missing/extra children, changed lengths/hashes, replaced file/directory
identity and closing original plan/journal changes reject without repair. Metadata
and chunks retain their 1 MiB bounds; region lengths remain independent nat64 values
without aggregate allocation or a sum. A passive returned checksum holds no future
byte custody, full-set byte verification or upload permit. Checks are sequential,
so integrations still fence noncooperating changes. No new record or state owner is
introduced; ordinary reopen/terminal replay performs no fresh artifact reads.

Native tests exercise interleaved ranges/chunks, exact raw metadata/request and
private bytes, empty regions/chunks, maximum replies/1,024 chunk identities,
incomplete or mixed coverage, duplicate/occupied entries, IO errors, changed or
unsafe files and replaced layout/parent/staging custody. Acknowledged native child
death during transfer, after retained checksum and after actual publication
qualifies local recovery only. Public replay preserves original pending mutation/
observation allowances, plan bytes and a nonempty restore-reference record.
Fresh retained checks additionally exercise exact reopen, changed/missing/extra
children (including empty directories), symlinks/FIFOs, pre-read size rejection,
wrong original metadata/request/plan/journal and non-durable/replaced custody.
Independent shape and metadata/chunk checks reject even when a deliberately corrupted
journal checksum matches an invalid tree. Maximum chunk rows and region replies,
empty files/chunks and a nat64 maximum extent retain bounded local checks. Public
fresh verification preserves original pending spending, plans and source references;
missing artifact replay succeeds while explicit verification rejects.

Integrations qualify authentic snapshot association and complete backend transfer,
generic token/raw-ID mapping, fresh permissions and prior per-call spending,
stable noncooperating bytes/command custody, consistency and upload/load/start
safety. No provider, authenticated receipt, effect settlement, full product manifest,
terminal proof or fence/reference release is installed. Canic consumers remain
unchanged. See [the local tree contract](contracts/ic-snapshot-artifact.json) and
[fresh source review](ic-snapshot-artifact-source.json).

### Original source-bound IC snapshot upload

`model::ic_snapshot_upload::IcSnapshotUploadRequest` borrows the full original source
plan and metadata reply, retains the declared tree checksum and encodes official SDK
upload arguments for that same canonical target. Metadata uploads have no replacement
ID. Ordered globals retain exact float/v128 values; an unavailable global rejects.
Optional timer/hook values remain absent rather than inventing states. Read-only source,
timestamp, version and chunk inventory remain original evidence, without becoming
upload fields. These mappings follow the pinned SDK and
[management interface](https://docs.internetcomputer.org/references/ic-interface-spec/management-canister/).

Data construction needs the original metadata-upload declaration and an exact new
1..256-byte raw destination distinct from the original source. It reuses the data
owner's nonzero checked ranges and known 32-byte hashes. Actual bytes must match the
requested region size or chunk SHA-256; empty known chunks are allowed. Only one
at-most-1-MiB chunk is buffered. Upload chunk wire carries no source hash, so the
source declaration retains it while exact encoded bytes bind its actual checksum.
Separate upload ceilings are 2 MiB arguments and 4 KiB raw replies. Existing
lifecycle arguments, schemas and exhaustive enums retain their original bounds.

`digest()` reuses the management wire owner. `binding_digest()` additionally binds
the full source-plan digest, exact metadata evidence and retained tree checksum,
then a metadata/data tag, the original metadata wire digest for data, and current
wire digest. All hashes are 64 lowercase ASCII bytes after the NUL-terminated
`ic-backup/ic-snapshot-upload-binding/v1` domain. Retain that binding as the operation's
request in its original upload plan before consuming its mutation allowance.
`IcSnapshotUploadAttempt` derives full original authority from that plan, checks
exact target/binding and original source network/release, and requires the unchanged
pending mutation with no pending recovery observation. Reconstruction permits no
dispatch. A newly allocated destination must precede each later data intent and its
independent reservation; the metadata allowance cannot be reused or replenished.
Integration-owned allocation attribution and cross-operation accounting are not
inferred from an ID, matching declarations or a passive acknowledgement.

The explicit `DownloadJournalGuard::prepare_ic_snapshot_upload_metadata` entry point
uses existing full retained source-plan/journal and complete Durable selected-set
admission, then verifies the exact opt-in IC tree. `prepare_ic_snapshot_upload_data`
requires unchanged original evidence/checksum, verifies before/after reading a fixed
direct child through no-follow/nonblocking descriptors, and encodes one exact source
slice or known chunk. Existing checks reject changed, missing, extra or unsafe files.
The already selected descriptor/offset and exact `take` extent use shared
`read_reader` for bounded, fallibly allocated collection. The existing 1 MiB
ceiling and exact returned-length check still apply; original IO errors pass
through, and allocation failures use the existing typed IO boundary. See
[the shared primitive review](ic-host-tools-adoption.json).
No record, progress, allowance, obligation, source reference or artifact changes.
The checks are sequential and hold no future noncooperating byte custody. Ordinary
resume stays artifact-free and supplies no upload input or permit.

`IcSnapshotUploadReply` admits an exact metadata ID through the existing bounded raw-ID
visitor, or the lifecycle owner's canonical empty Candid tuple for data. Metadata
decoding has 64 KiB work, zero skipped work, 16 type-table entries and 4 KiB header
bounds. Extra arguments, skipped extensions and trailing bytes reject. The evidence
hash domain is `ic-backup/ic-snapshot-upload-reply/v1` plus NUL, original binding hash
and exact raw-reply SHA-256. Pure `policy::ic_snapshot_upload::validate_acknowledgement`
rechecks the current journal and exact claimed original authority, attempt, actual
context and target through the existing passive acknowledgement owner. Its result
authenticates no provider or allocated ID, records no receipt and grants no new call.

Lost metadata/data replies remain pending. Provider failure, absent bytes or zero/one/
multiple snapshot entries never prove NotApplied or Uncertain. Qualified settled
reconciliation and proof of no prior dispatch remain integration-owned; no upload
observation backend, blind retry, refund or default provider is installed. Authentic
complete source/new snapshot transfer, fresh controllers, command/byte custody,
application/fence obligations, same-release load/start safety and terminal/reference
release remain independently qualified. See [the generated contract](contracts/ic-snapshot-upload.json)
and [fresh review](ic-snapshot-upload-source.json). Independent wire fixtures and
native public reopen tests qualify declarations/local bytes only, not IC effects.

### Single originally reserved IC snapshot upload port

`ports::ic_snapshot_upload::IcSnapshotUploadProvider::submit_upload` accepts the
existing exact `IcSnapshotUploadAttempt` and returns the existing bounded passive
`IcMutationAcknowledgement` or canonical `IcMutationProviderError`. No additional
request, receipt, failure enum, journal or accounting owner is introduced.
Integrations retain source/upload originals and durably reserve each metadata/data
update before invocation. Qualify authentic complete source, actual fresh controllers
and prerequisites, stable byte/command custody and proof of no prior dispatch.
Data additionally needs exclusive original allocation attribution for the new ID;
passive IDs or inventory cardinality cannot supply it. Metadata spending supplies
no data allowance or replacement/deletion lane.

One invocation permits only the exact management receiver, routing target, method
and Candid bytes as one host replicated update. No hidden retries, observations,
funding, query/proxy substitutions or allocation/data batching occur. Additional
calls need independent prior accounting and admission. Pure acknowledgement checks
still re-admit the current journal, exact claims and tighter upload wire bounds;
they authenticate no provider and perform no settlement. Errors/drop/death preserve
pending spending, originals, source references and application/fence obligations.
Lost replies need qualified reconciliation; local reopen invokes no provider.

Native tests exercise refusal at this interface and exact retained local bytes,
not IC behavior. There is no installed provider, transport, runner, complete-upload
proof or release authority. See [the port contract](contracts/ic-snapshot-upload-port.json)
and [source review](ic-snapshot-upload-port-source.json). The released upload codec
contract and existing v1 records/digests/bounds remain unchanged.

### Exact originally reserved metadata-upload observations

`model::ic_snapshot_upload_observation::IcSnapshotUploadObservationRequest` binds
the complete original upload plan, exact source-bound metadata bytes and immutable
allowances to an independent original `ListCanisterSnapshots` payload. Both original
attempt IDs must remain pending; the list digest must match its exact retained
reservation. Data uploads and status observations reject. Construction performs no
IO, reservation or settlement. Existing upload authority checks and observation
reservation checks have one canonical private implementation shared by both ports.

`ports::ic_snapshot_upload_observation::IcSnapshotUploadObservationProvider::observe_upload`
permits one previously accounted exact host replicated list update. Integrations
retain original source/upload/list bytes before reservation and independently qualify
fresh actual context/read permission, timing, authentication, command custody and
proof this observation has never been dispatched. Reconstruction supplies no reissue
permission. No provider implementation or hidden observation/retry is installed.

Pure `policy::ic_snapshot_upload_observation::validate_response` reuses the existing
passive response, exact claim association and bounded inventory decoder. Its view
retains exact raw evidence, unique canonical raw IDs and required nat64 timestamp/
size. Original 1,024 attempts/entries, 256 ID bytes, 4 KiB arguments, 8 KiB list
record, 1 MiB reply and finite decoder-work bounds remain unchanged. Zero, one or
many snapshots and matching timestamps/sizes establish no exclusive original
allocation attribution or upload outcome. There is no automatic receipt or Applied
transition. Original baseline custody and authenticated reconciliation remain owned
by integrations; Canic's singleton-new-ID success inference is not imported.

Failures and lost replies retain pending mutation/observation spending, references
and application/fence obligations. Lost observation replies stay pending, rather
than becoming settled Uncertain. Local reopen and association spend nothing and
invoke no provider. Native tests cover structural denials, bounded wire, interface
refusal and exact retained local bytes/reopen. They qualify no IC effect, complete
upload, data reconciliation, terminal proof or fence/reference release. See
[the observation contract](contracts/ic-snapshot-upload-observation.json) and
[fresh source review](ic-snapshot-upload-observation-source.json).

### Exact originally reserved data-upload observations

`model::ic_snapshot_upload_data_observation::IcSnapshotUploadDataObservationRequest`
joins the full original source-bound data upload plan/bytes/allowances with an
independently retained `IcSnapshotDataRequest`. The target, new destination raw ID
and exact original region/offset/length or chunk hash must match. Destination
metadata must declare `MetadataUpload` and the original three region sizes; unknown/
taken source or different sizes reject without supplying defaults. The existing
data request owns bounded extent/hash admission. Destination authentication and
exclusive original allocation attribution remain integration-owned.

Both original attempt IDs must remain pending, with the exact read digest retained
by its already consumed observation. Construction and current-journal checks reuse
canonical upload authority and observation reservation admission and perform no IO,
spending or settlement. Retain destination metadata/read originals before reservation.

The single `IcSnapshotUploadDataObservationProvider::observe_upload_data` contract
allows only that original replicated host data-read update. Fresh read permissions,
actual context/chronology/authentication, backend readback capability, stable command/
byte custody and proof of no prior observation dispatch remain separately qualified.
There is no installed provider, hidden retry, metadata/list/status read, query/proxy
substitution or upload. Every failure retains both pending attempts and obligations;
lost observation replies never mean settled Uncertain or renewed allowance.

Its passive response reuses existing claim fields, chronological/principal checks
and redacted diagnostics under the existing 2 MiB data-reply bound. The released
status/list response keeps its 1 MiB bound. Pure association rechecks current
reservations and exact actual claims through the canonical common owner, then uses
the existing data decoder: 1 MiB chunk, 4 KiB arguments/header, 16 type-table entries,
8 MiB decoder work and no skipped fields. Raw IDs remain bounded to 256 bytes.
The existing data evidence digest includes exact retained destination metadata,
read payload and raw reply; no new hash recipe or persisted schema is introduced.

The read-only view compares actual chunk SHA-256 with the original encoded upload
chunk. Matching bytes can predate this upload or come from another writer; the
[IC state-transition specification](https://docs.internetcomputer.org/references/ic-interface-spec/abstract-behavior/)
initializes uploaded regions with zeros. Different or absent bytes alone cannot
prove a negative outcome either. Chunk-store replies still require the exact known
hash, including valid empty chunks. No comparison automatically creates a receipt,
Applied/NotApplied outcome, retry, complete upload or source-reference release.

Native tests cover all region/chunk kinds, exact denials, optional source rejection,
raw/decoder bounds and a full 1 MiB chunk plus wire overhead. Public local evidence
survives provider refusal, matching/different passive bytes and journal reopen with
unchanged originals, source references and diagnostics. These checks qualify no IC
effect, complete transfer, terminal proof, load/start safety or fence/reference release.
See [the contract](contracts/ic-snapshot-upload-data-observation.json) and
[fresh source review](ic-snapshot-upload-data-observation-source.json).

### Exact data-upload settlement claims

`IcSnapshotUploadDataSettlement` and `IcSnapshotUploadDataAttribution` are passive
integration evidence for an already spent successful exact data read. Pure
`policy::ic_snapshot_upload_data_observation::validate_settlement` reuses the
existing readback association, current reservation/actual claim admission and
bounded data decoder. It additionally requires the original full-plan authority,
both exact attempt IDs, the caller's current qualification challenge, the existing
data-reply digest and unchanged opaque observation evidence. That digest includes
exact destination metadata, request and raw reply; equal bytes with changed
metadata or provenance cannot rebind a settlement.

Applied claims require matching original bytes and independent exclusive
original-write attribution. Integrations exclude preexisting bytes and other writers,
qualify the original allocation and retain stable destination custody through the
read. NotApplied claims require proof the original update never applied, including
exclusion of a transient write later overwritten. Equal initialized/preexisting
bytes remain compatible with nonapplication; different bytes alone establish no
negative outcome. Unresolved claims require an actually settled authenticated read.
Lost, absent or malformed read replies stay outside this successful-read boundary.
The library matches passive claims; it authenticates none of those proofs.

The read-only view retains original readback/attribution and projects the existing
`ObservationOutcomeRecord`. It performs no IO, calls, serialization, automatic
receipt, retry or refund. Qualification here uses retained evidence only; extra
remote observations need their own prior accounting and are outside this operation.
Only the integration explicitly calls the existing journal transition after
qualifying all evidence. Recording settled uncertainty clears that observation but
retains the pending mutation; every original allowance remains consumed. Reopen
reads only local history and never repeats the provider call.

No provider, spending/progress owner, schema or hash recipe is added. Original
1,024 attempts, 1 MiB chunks and 2 MiB raw data replies retain their existing owners.
Single-write attribution establishes no full transfer, authenticated backend,
load/start safety, terminal proof or fence/source-reference release.
Native cases check exact identities, changed evidence/metadata, every region,
known empty chunks, byte conflict, all claimed outcomes and explicit existing-owner
spending retention. The public local journey preserves lost replies, then records
synthetic settled uncertainty solely to qualify durable local accounting/reopen;
it simulates no IC behavior. See
[the contract](contracts/ic-snapshot-upload-data-settlement.json) and
[fresh source review](ic-snapshot-upload-data-settlement-source.json).

### Exact independently qualified lifecycle settlement

`IcLifecycleSettlement` and `IcLifecycleAttribution` are passive integration
claims for original stop/start/load operations under an already reserved successful
status read. `policy::ic_observation::validate_lifecycle_settlement` reuses the exact
original plan/authority/payload and current journal association, bounded lifecycle
decoder and existing status request/raw-reply digest. Capture mutations and inventory
reads reject. Authority, both attempt IDs, caller-owned current challenge, exact
status digest and unchanged opaque observation evidence must match.

Applied claims independently attribute the original request, excluding preexisting
state and independent effects. For load, the integration qualifies exact original
snapshot state; code hash or Stopped projection cannot substitute. NotApplied claims
exclude transient application followed by reversal or overwrite. Unresolved claims
require an actually settled authenticated successful observation. Lost, absent,
unavailable or malformed replies never enter this boundary as uncertainty.

Status describes current lifecycle, while the claim concerns the original effect.
All current status variants remain compatible with independently qualified historical
outcomes; the library infers none from a status value. Current execution prerequisites,
application drain/fence, restored-state acceptance and permission checks remain
independently required at subsequent effect boundaries. Canic's desired-status-to-
completed-receipt inference is not imported. The
[IC management specification](https://docs.internetcomputer.org/references/ic-interface-spec/management-canister/)
describes status and snapshot loading; this local admission qualifies no actual backend.

The read-only view projects the existing `ObservationOutcomeRecord`, retains exact
original observation and complete qualification evidence, and performs no IO,
provider call, serialization, automatic receipt, retry or refund. Only a qualified
integration explicitly invokes the existing journal transition. Settled uncertainty
clears its observation but keeps the original mutation pending. Every original
allowance, obligation and source reference remains retained; extra remote evidence
needs separately accounted calls. Existing 1,024 attempt, 4 KiB argument, 1 MiB raw
reply and finite status decoder bounds remain with their original owners.

Native unit cases cover all methods/statuses/claimed outcomes, exact typed denials,
capture/inventory rejection, changed raw skipped metadata/evidence and stale journals.
The public local journey proves unchanged admission bytes, explicit receipt recording,
exhausted allowances and original references/obligations through reopen; its synthetic
claims simulate no IC effect. No provider, transport, schema, hash recipe, accounting
owner, load/start safety, terminal or fence/reference-release permit is introduced.
See [the contract](contracts/ic-lifecycle-settlement.json),
[fresh source review](ic-lifecycle-settlement-source.json) and
[issue #14](https://github.com/dragginzgame/ic-backup/issues/14).

### Exact independently qualified capture settlement

`IcCaptureSettlement` and `IcCaptureAttribution` are passive original capture
claims over an already reserved successful exact list observation.
`policy::ic_observation::validate_capture_settlement` reuses full original
plan/authority/capture bytes, current reservations, actual response association,
bounded inventory decoding and the canonical closed-baseline comparison. Lifecycle
mutations and status observations reject. Both attempt IDs, current caller-owned
challenge, exact baseline/current request/raw digests and opaque observation
evidence must match. Every original baseline ID, timestamp and size stays unchanged;
missing or changed rows reject for all outcomes.

Applied names an explicitly independently attributed new raw ID from the exact
candidate set, retaining its timestamp/size. Several candidates can coexist;
singleton cardinality never supplies attribution. An ID outside that set, including
an original baseline ID or one outside existing codec bounds, rejects. NotApplied
requires qualified original nonapplication excluding transient capture then deletion.
Unresolved requires an actually settled authenticated successful list. Lost, absent,
unavailable or malformed replies stay pending. Opaque evidence is a declaration,
not self-authenticating proof.

The integration retains the complete original baseline before capture and qualifies
actual network/caller/target, original chronology, fresh read permissions, exclusive
attribution, challenge freshness and command custody. Canonical equality proves none
of those. Capture consistency, snapshot authenticity, full transfer and subsequent
application/restore safety remain independent. Canic's singleton-to-receipt inference
is not copied. The
[IC management specification](https://docs.internetcomputer.org/references/ic-interface-spec/management-canister/)
describes capture/list methods; no backend or actual effect is qualified here.

The borrowed view reads exact original evidence and projects the existing journal
outcome without IO, calls, serialization, automatic receipt, retry or refund. Only
qualified integration code explicitly invokes the existing receipt owner. Settled
uncertainty clears only its observation and retains the original pending mutation;
all consumption, obligations and source references survive reopen. Extra remote
evidence requires independently accounted calls. No provider, transport, schema,
hash recipe, spending owner, terminal or fence/reference-release permit is added.

Existing owners retain the 1,024 total-attempt/inventory-entry, 256 raw-ID-byte,
4 KiB argument, 1 MiB raw-reply, 2 MiB decoding-work, zero skipped-work and
16-type-table bounds. Fresh native cases cover explicit attribution among several
candidates, zero/one/many descriptive deltas, ID bounds, exact raw ordering/evidence,
typed identity/baseline/method/association denials and stale journals. Public local
recovery retains the original baseline before reservation, leaves admission bytes
unchanged, explicitly records all claimed outcomes and preserves exhausted spending,
plans, payloads, obligations and references through reopen. Synthetic native claims
qualify local binding/persistence only, with no simulated IC behavior. See
[the contract](contracts/ic-capture-settlement.json),
[fresh committed source review](ic-capture-settlement-source.json) and
[issue #16](https://github.com/dragginzgame/ic-backup/issues/16).

### Local IC snapshot diagnostics

`DownloadJournalGuard::ic_snapshot_metrics` returns a copied, read-only
`IcSnapshotLocalMetrics` view for that guard's lifetime. It reuses registry
`ic-metrics`'s `MeasurementSummary` arithmetic, re-exported as
`ic_backup::ops::persistence::MeasurementSummary` so callers can name returned
summaries without a separate dependency selection. The actual shared type is
declared once in the workspace
catalog and inherited for the Unix-host implementation. Default features are
disabled and the canister instruction-reader feature is absent. Consumers own
sampling/reporting; no sibling path, global registry, telemetry endpoint or
transport metric is introduced.

The existing explicit `verify_ic_snapshot_artifact`,
`prepare_ic_snapshot_upload_metadata` and `prepare_ic_snapshot_upload_data` calls
record their returned successes/rejections separately. Host monotonic elapsed
durations use nanoseconds and clamp to `u64::MAX` before shared saturating aggregation.
Only a successfully returned data payload contributes its actual bounded chunk
length in bytes. Empty known chunks contribute a measured zero; repeated preparation
contributes repeated work, not unique or remotely transferred bytes. Internal
verification samples are included, so enclosing preparation durations overlap those
checks and must not be added as exclusive work. Calls that do not return supply no
sample. No wall-clock threshold, IC instruction/cycle cost or receipt is inferred.

Successful prepared sizes have one `ic_metrics::MeasurementHistogram<4>` owner,
re-exported through the persistence facade. `prepared_chunk_bytes_histogram()`
returns its copied distribution; the existing `prepared_chunk_bytes()` reads its
summary. Inclusive bounds of zero, 32 KiB, 256 KiB and the 1 MiB payload ceiling
separate empty, small, larger and maximum-sized local preparation work. Disjoint
counts and separate overflow are descriptive, not exact percentiles or payload
admission. The production payload owner rejects oversized chunks; overflow routing
is tested without granting an oversized upload. Fixed arrays and overflow add
72 bytes per guard, with at most four comparisons per successful chunk and no
extra timing sample. The six duration owners remain summaries. No second byte
aggregate, local histogram arithmetic or duration distribution is introduced.

Sampling is empty on create/open. Ordinary retained record access and metric reads
perform no fresh artifact IO or sampling, even when artifact trees are missing.
Guard-local synchronization preserves `Send + Sync`; no metric lock spans filesystem
work or caller code. Diagnostic poison recovery changes no operation result.
Counts/totals saturate independently, and either at `u64::MAX` is unavailable for
exact interval arithmetic. There is no persistence, reset API, serialization,
identifier label or retained metric history. Existing v1 bytes/hashes, typed failures,
spending, obligations and source references keep their original owners.

Native tests check exact sample membership and units, error/zero/repeated-byte
semantics, duration clamping, copied views, guard traits and fresh empty reopen.
Public source/upload replay retains original record/reference bytes and exhausted
pending spending while diagnostic samples grow. This qualifies local arithmetic
and sampling only. [Dependency review](ic-metrics-adoption.json) binds the verified
registry archive; real IC effects and transport measurements remain unimplemented.

### Original-plan-bound retained execution admission

`read_execution_progress` reopens the exact persisted plan and every original
attempt journal under layout exclusion, then delegates complete identity,
coverage, accounting and causal projections to the existing policy owner.
Missing originals reject; no journal is created and no consumption defaults to
zero. Journal locks are acquired sequentially rather than held as an unbounded
set. The same internal reader serves execution-settlement admission/replay.

`AttemptJournalGuard::reserve_planned_mutation` and
`reserve_planned_observation` retain the selected journal lock while admitting
all original evidence. Mutations require every declared prerequisite's retained
Applied outcome; observations retain the existing exact pending mutation and
request binding. Ordinary attempt reservation/persistence remains the sole
spending owner. Wrong plans/authority, contention, missing evidence, premature
history, pending or exhausted attempts fail without a refund or reset.

Callers hold no other attempt guard. Reads and failures perform no provider call,
artifact traversal, terminal transition, source-reference/fence release or cleanup.
Local record locks and IO remain observable. Fresh authority, authenticated
receipts/prerequisites, never-dispatched command custody, actual backend capability
and application safety remain integration-qualified. Native tests cover blocked
and then Applied prerequisites, exact consumption, missing/wrong/held originals,
premature histories and lost-observation reopen. The public settlement journey
uses the new reservation boundary while retaining its fence/source obligations.
This is reusable execution admission, not an installed scheduler or IC transport.

### Original workflow allocations and learned-stage binding

`ExecutionWorkflowRecord` retains a distinct strict v1 stage catalog under fixed
`execution-workflow.json`. It reuses `OperationPlanRecord` for canonical context,
full inventory, selection, closed dependency graph and bounded allocations.
Catalog operation sequences identify stages; request hashes commit original
integration-owned input/purpose contracts. The catalog supplies no attempt
authority. Each stage's original per-operation ceiling is at most 1,024 combined
attempts; the full original allocation remains at most 65,536. Unassigned workflow
headroom is never transferred. Ordinary operation-plan v1 semantics remain intact.

`ExecutionStageBindingRecord` joins the original workflow digest and stage identity
to one exact ordinary child plan digest and the complete exact set of direct
predecessors. Each row retains the predecessor binding, chronological settlement
and opaque learned-input evidence digest. Child context/full inventory must equal
the original catalog; selection is exactly that stage's target. Aggregate mutation
and observation ceilings equal the original stage allocation, while the existing
plan owner bounds assigned operation allowances. Zero-total child operations
reject, bounding total child operations by original allocated attempts. Rebuilding
an in-memory plan grants no extra allowance and cannot replace the retained plan.

`ExecutionStageGuard::create` borrows the original workflow layout and creates
only its fixed direct `execution-stage-{sequence}` child with 0700 permissions.
It validates all records before filesystem allocation and requires each original
predecessor's exact retained binding/plan and complete Applied settlement through
the existing journal-history owner. Parent synchronization and existing durable
0600 JSON publication retain `operation-plan.json` and `stage-binding.json` before
return. Occupied directories reject; partial preparation is retained without
repair, cleanup or rebinding. An interrupted preparation with a missing document
stops safely. A lost successful creation response reopens the exact complete pair.

`open` and `layout` re-admit original workflow, child plan/binding and predecessor
settlements, retaining both layout lifetimes. Derived stage symlinks reject.
Admission traverses the complete original ancestor DAG iteratively, checking each
distinct stage's exact binding, plan, chronological settlement and complete original
journal set. Shared ancestors require identical binding/settlement identities;
their per-edge learned-input commitments may differ. At most one row per original
catalog node is retained, bounded by the existing 8,192-node catalog, and only one
ancestor layout/journal reader is held at a time. Missing or changed transitive
evidence rejects before descendant allocation or retained-layout access. Sequential
checks do not establish atomic evidence or noncooperating byte custody.
Only existing attempt journals own spending; these APIs never create a journal,
reserve a call, infer zero consumption, refund or automatically settle a receipt.
Resume must use complete original journal admission; a missing original rejects.
Stage creation is preparation, not dispatch. Integrations own the input contract,
exact learned snapshot/extent evidence and derivation, receipt authentication,
fresh effect-boundary permissions/safety and never-dispatched command/byte custody.
Noncooperating filesystem custody retains its existing operator-owned boundary.

`ExecutionStageGuard::prepare` additionally derives all original child authorities
before allocation and creates their complete attempt-journal set through the
existing durable create-only journal owner before returning. It takes one journal
lock at a time and re-admits original stage/predecessor records after publication.
`create` remains the record/layout primitive and creates no journals. Neither
entrypoint reserves attempts, authenticates receipts or supplies fresh dispatch
custody. Partial journal preparation retains the occupied stage and every original
record; calling `prepare` again refuses rather than filling gaps. A lost successful
response can reopen the complete exact set. `open` creates no journal, and execution
admission still rejects any missing, changed or unsafe original. The typed
`ExecutionStagePreparationError` preserves original stage, authority and journal
errors without granting repair, refunds or retries. No schema or digest changes.

`ExecutionStageGuard::resume` joins exact record-only `open` admission to the
existing complete original child-journal progress reader, then re-admits original
stage/ancestor records before returning the guard and canonical progress view.
Missing, changed or held journals reject through `ExecutionStageResumeError`,
preserving original stage and progress errors. Hold no attempt guards. Pending and
exhausted spending remains unchanged; unassigned headroom supplies no allowance.
Resume creates no journal, reads no artifact tree and calls no provider. `open`
retains its distinct record-only inspection purpose for incomplete preparation.
The returned sequential view is no atomic custody, fresh authority, receipt,
transfer completion or terminal/fence/reference-release proof.

`ExecutionStageGuard::checkpoint` publishes the existing all-Applied journal
checkpoint and returns the exact original stage/binding/settlement predecessor
commitment. The guard re-admits the retained workflow, binding, child plan and
complete ancestor histories before and after canonical checkpoint publication.
Post-publication failure keeps the checkpoint and all original spending; an
occupied checkpoint is never replaced. Missing/held/pending originals or changed
workflow/ancestor evidence return typed refusals. Drop active attempt guards at
admission. The caller's learned-evidence digest remains opaque and independently
qualified; this operation authenticates no learned ID, receipt or provider and
grants no successor dispatch, full completion or fence/reference-release proof.
The standalone original-plan checkpoint and exact identity-bound replay retain
their independent local recovery responsibilities.

The workflow and binding use separate NUL-terminated v1 digest domains. Workflow
hashing retains the full canonical allocation digest. Stage hashing retains exact
workflow/child identities, stage sequence and canonical predecessor rows; no
counters or terminal flags are copied. Workflow JSON is bounded to 1 MiB, binding
JSON to 512 KiB and rows to the existing 1,024 direct-dependency limit. Strict
[workflow](contracts/execution-workflow.schema.json) and
[binding](contracts/execution-stage.schema.json) schemas include independent
binary goldens. Native tests cover exact/missing/changed evidence, finite original
budgets, occupied/unsafe stages, pending reopen and acknowledged process death
between plan/binding publication and after durable binding completion. The public
journey binds real capture/metadata codec digests with a learned raw snapshot ID;
its native receipt fixtures qualify local admission, not IC effects. Full capture/
download orchestration, authenticated transfer, application adapters and terminal/
fence/reference release remain unimplemented.

### Metadata-derived original download planning

`model::ic_snapshot_download::IcSnapshotDownloadPlan` borrows an exact original
workflow stage and decoded metadata. It checks target equality and chunk size
within 1..=1 MiB, then counts all Wasm-module, heap, stable and known-chunk requests
with checked nat64 arithmetic before allocating or encoding payloads. The entire
read set must fit the original stage mutation ceiling; the existing stage owner
bounds combined attempts to 1,024. Oversized snapshots reject without iteration
proportional to remote sizes. This bounded planner does not partition a larger
snapshot into additional stages or create fresh allowance.

Requests cover contiguous module/heap/stable regions followed by known chunks in
the exact metadata order. Empty regions need no call; known empty chunks still
need their exact hash-checked reply. Existing data codecs own every extent, raw ID
and payload hash. The ordinary child plan copies original context/full inventory,
selects the exact original target, retains original aggregate ceilings and assigns
each zero-based ordinal one mutation and zero observations. Explicit sequential
dependencies admit the contiguous writer order. Spare headroom stays unassigned.
Entirely read-free metadata yields no child plan or binding, without a placeholder
operation or inference of complete transfer/publication.

Binding requires the exact single-request metadata child plan under the original
workflow, its direct predecessor binding and exact request/raw-reply metadata
evidence digest. Existing stage creation separately admits complete original
Applied predecessor journals and chronological settlement. These checks authenticate
neither opaque receipt semantics nor metadata provenance. Fresh permissions,
original capture/extent custody, purpose contracts and never-dispatched command
custody remain integration responsibilities. No new record, journal, spending
owner, provider or installed runner appears.

Real Testkit/PocketIC qualification freezes capture/metadata/data stage ceilings
before capture, creates every original data journal before the first data call,
and downloads the isolated stopped fixture through this exact plan. Success joins
existing coverage, durable IC-tree publication, fresh verification and immutable
local manifest replay. Lost/malformed replies after the first successful append
retain the second pending mutation, partial Created bytes and source references;
dependent reservations reject and reopen makes no provider call or fresh journal.
The fixture owns explicit simulator setup, single-ingress custody and its
application's stopped/no-external-effects admission. This is not generic application
qualification, Canic adoption, full restore or terminal/fence/reference release.

### Provider-driven snapshot capture step

`workflow::ic_snapshot_capture::capture_snapshot` accepts only the exact original
`TakeCanisterSnapshot` payload under a retained execution stage. Canonical target/
wire binding rejects before spending. Open the existing original journal, never
create it; complete original-plan progress owns the durable reservation. Hold the
selected journal through mandatory fallible fresh integration admission, one
`IcMutationProvider::submit_mutation` invocation and existing bounded passive
acknowledgement association. Recheck stage/ancestors before dispatch and after
the reply. Fresh actual control, capture consistency, quiescence/fence obligations,
authentication and never-dispatched command custody remain integration-owned.
Remote preflight has its own prior accounting; no default provider or lane exists.

Success stays pending until an independently qualified integration explicitly
records a receipt. Every post-reservation failure retains original consumption;
association and post-reply errors retain the bounded returned acknowledgement.
Pending/Applied, missing/held originals, changed binding or unfulfilled prerequisites
cannot invoke the callback/provider again. No new schema, journal, allowance,
decoder, refund, automatic uncertainty or reference/fence disposition is added.
Reuse existing 1,024-attempt, 256-ID-byte, 4 KiB argument and 1 MiB raw-reply bounds.
Sequential checks do not fence noncooperating actors.

The isolated Testkit download driver uses this public step before its explicit
qualified capture receipt and existing metadata/data read steps. Successful capture
joins durable artifact publication; lost/malformed capture replies preserve pending
originals and source references, with no recapture, checkpoint or successor stage.
These cases qualify the controlled stopped fixture only. Full backup/restore,
default Agent provider, application admission and terminal/custody/fence/reference
release remain open in [#29](https://github.com/dragginzgame/ic-backup/issues/29).
See the [mutation port contract](contracts/ic-mutation-port.json).

### Provider-driven snapshot transfer read step

`workflow::ic_snapshot_transfer_read::read_snapshot` coordinates exactly one
metadata/data replicated update under `ExecutionStageGuard`. Payload target/wire
binding reuses the model owner before spending. The existing selected journal is
opened, never created, and canonical `reserve_planned_mutation` admits the complete
original set and Applied prerequisites before durable consumption. The guard holds
the selected journal throughout mandatory fresh admission, one provider invocation
and bounded passive reply association. Retained stage and ancestor admission runs
again after the callback and after an associated reply.

The explicit fallible callback qualifies actual fresh read permission, original
snapshot/metadata custody, current application requirements and exclusive original
command custody. It has no library default or persisted permission flag. Any remote
preflight work needs its own prior accounting. The existing provider owns actual
authentication and a single exact ingress, without hidden calls or retries. These
sequential local checks do not fence noncooperating filesystem actors.

Success returns an immutable bounded raw response and leaves spending pending.
Only separately authenticated integration evidence may enter the existing receipt
owner. Lost, unavailable, unsupported or malformed replies never refund consumption,
settle uncertainty or permit repetition; a second invocation rejects the pending
attempt before callback/provider entry. Typed association and post-reply failures
retain the exact returned bounded response. No journal/schema, artifact progress,
fence obligation or source reference changes beyond the original reservation.
Original 1,024-attempt, 4 KiB argument, 1 MiB metadata and 2 MiB raw-data limits and
all decoder quotas remain with their existing owners.

The public simulator download fixture now uses this step for metadata and every
planned data call. Complete artifact publication and lost/malformed second-read
retention are qualified for that isolated stopped application only. This installs
one callable coordination step, not a complete backup/restore runner or default
Agent provider. Applications still own full transfer, fresh membership/consistency,
restore safety, stable custody and terminal/fence/reference release. See
[#29](https://github.com/dragginzgame/ic-backup/issues/29) and the
[port contract](contracts/ic-snapshot-transfer-read-port.json).

### Complete original snapshot data transfer

`workflow::ic_snapshot_download::download_snapshot` joins a nonempty exact
`IcSnapshotDownloadPlan`, retained data stage and existing private artifact writer.
Reject changed bindings/metadata/writer origins, prior writer coverage and any
original mutation/observation consumption before a provider call. Complete original
journal admission still owns missing/held evidence. A read-free plan creates no
placeholder operation; it is refused by this data-stage entrypoint.

For each original ordinal, reuse `read_snapshot` for durable reservation, mandatory
fresh read admission and one provider invocation. Reopen the selected original
journal, re-admit the exact bounded response and require an integration-qualified
exact Applied receipt. The integration independently authenticates attribution;
successful wire shape never generates that receipt. Append decoded bytes through
the existing writer before the sole attempt owner persists the receipt. Only then
can the next dependent read enter admission. Closing stage/ancestor checks remain
mandatory. No new schema, journal, allowance, decoder or progress owner is introduced.

Any failure consumes the writer, retaining partial bytes, every original reservation
and recorded receipt. Post-read errors retain the bounded returned response; lost
replies stay pending and stop without follow-up. Failed receipt persistence can leave
accepted bytes with pending spending; those bytes grant no restart or completion.
Do not reconstruct coverage or repeat reads after interruption. Complete original
Applied evidence and exact coverage precede existing fresh checksum/durable publication.
Post-publication stage rejection retains the returned checksum in its typed error.
Immutable manifest/checkpoint publication and ordinary effect-free resume remain
separate owners; source references and fence obligations are unchanged.

The actual isolated capture-to-download fixture now uses this loop, retaining
complete publication and lost/malformed second-read cases. Pure adversarial tests
also cover wrong receipts/outcomes, failed qualification/persistence, changed
metadata/intent/coverage and stage/byte custody. These qualify the stopped fixture,
not arbitrary application consistency, stable noncooperating byte custody, complete
product manifests, full Agent/backup/restore orchestration or terminal/fence/reference
release. [#29](https://github.com/dragginzgame/ic-backup/issues/29) remains open.

### Original execution settlement checkpoints

`model::execution_settlement::ExecutionSettlementRecord` is the immutable v1 local
checkpoint for the exact full original plan and unique journal-history fingerprints.
It has no copied counters, outcomes or completion/terminal flags. Required checksum
records reuse the existing owner; rows normalize and sort opaque u64 sequences,
rejecting duplicate identities even with different hashes. The 8,192-row ceiling
fits the 2 MiB raw/canonical IO bound, including maximum operation IDs. This is not
a product completion receipt. See the [strict schema and independent binary goldens](contracts/execution-settlement.schema.json).

`AttemptJournalRecord::digest` now hashes its full original authority and every
chronological reservation/receipt through the existing private event owner. The
NUL-terminated `ic-backup/attempt-journal-history/v1` domain precedes 64 ASCII
authority hash bytes and a big-endian u64 event count. Closed tags and integer
widths/outcome bytes bind every request/evidence hash. Derived progress, JSON
formatting and filesystem location are excluded; no v1 journal fields change.
Equal final views with changed receipt evidence therefore retain different histories.
The checkpoint's separate `ic-backup/execution-settlement/v1` domain binds original
plan hash, u64 row count and ascending u64 sequence/64 ASCII history pairs.

`policy::execution_settlement::validate` reuses canonical `execution_progress`
for complete exact original journal/context/operation/allowance admission and
retained Applied prerequisite checks. Every original operation must be Applied;
unused, pending, lost observations, Uncertain, NotApplied and exhausted unresolved
evidence cannot seal. Checkpoint coverage and every full history must match. Policy
performs no IO, serialization, mutation or scheduling and returns the existing view.
Original unused assigned allowances remain unchanged and confer no future authority.

Guarded `create_execution_settlement` and `read_execution_settlement` use fixed
`execution-settlement.json`, the original retained plan and every original journal
under layout exclusion. Sequential journal locks bound descriptor use, while Applied
owners prohibit subsequent transitions; drop active guards before invoking these
operations. `OperationPlanRecord::attempt_authorities` reuses scalar derivation through
one canonical helper, computing the full plan digest once per bulk scan. There is no
new cache or accounting owner. Existing no-follow bounded readers and durable private
no-replace publication retain prior bytes on conflict. Lost publication replies and
process death reopen exact local evidence, without paid calls or rewriting originals.

`checkpoint_execution_settlement` derives the existing record from the exact
expected retained plan and every original journal through the same sequential
reader. It then delegates to the existing immutable publisher, which re-admits
the plan and complete histories before writing. Its separate typed derivation
error preserves the existing publication/replay error contract. Pending,
NotApplied, Uncertain, missing or held originals cannot produce a checkpoint;
an occupied checkpoint is retained without replacement. Independent callers may
still construct a record for exact publication or retain its digest for replay.
No new schema, budget or outcome is introduced.

The checkpoint qualifies local ledger binding only. Authentication, cross-journal
dispatch chronology, stable noncooperating byte custody, transfer completeness,
manifest/application safety and command quiescence remain separate qualification.
No terminal, fence release, source-reference release or prune API is inferred or added.
All original spending, obligations and references remain retained across replay.

Fresh native cases cover independent hashes/schema, maximum bounds, missing/changed
originals, unsettled histories, exact receipt drift under identical final views,
contention, symlinks, layout replacement, failed publication and acknowledged process
death before publication/after directory sync. A public journey replays all-Applied
native fixture journals while retaining original fence/source dependencies. These
fixtures perform no IC/application effects. [Fresh Canic inspection](execution-settlement-source.json)
imports no completion counts, terminal flags, runner or release proof; actual backend
qualification and full product terminal evidence remain pending.

| Canic surface | Extraction disposition |
| --- | --- |
| Original all-Applied journal settlement | Adapted immutable full-plan/exact-history local checkpoint and guarded replay; completion counts/flags, command exit, actual terminal proof and release authority are not imported |
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
| Restore source identities, stopped-load checks and verification consumers | Adapted immutable same-network/release selected-source declarations and fresh exact load/start safety matching; remapping, Root fields, rendered commands and module-hash-only safety/settlement are not copied |
| Backup and restore runners | Require the reviewed generic ports and uncertain-effect reconciliation; not copied in this batch |
| ICP subprocess transport | Retired destination; direct `ic-backup-agent` reuses exact original wire/reservation owners; historical CLI evidence is preserved |
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

## Exact metadata-upload settlement claims

Passive `IcSnapshotUploadSettlement` and `IcSnapshotUploadAttribution` bind the
full original upload authority, both still-pending attempts, a current challenge,
exact independently retained baseline/current list digests and opaque observation
evidence. Pure `policy::ic_snapshot_upload_observation::validate_settlement` reuses
existing reservation/claim/decoder admission and the same closed-baseline comparison
as capture. Every original ID, timestamp and size must remain unchanged. Applied
requires exclusive original-allocation attribution to one explicitly named new ID,
within existing destination bounds and distinct from the source. Several candidates
may coexist; zero/one/many candidates never supply an outcome. Views retain exact
evidence and expose only the explicitly attributed descriptor. Raw claim IDs are
redacted from diagnostic formatting.

NotApplied requires qualified exclusion including transient allocation/deletion;
Unresolved requires a settled authenticated successful list, not a lost reply.
Integrations retain baseline evidence before mutation and qualify actual context,
authentication, chronology, freshness, stable custody and exclusive attribution.
Matching hashes cannot establish these properties. Pure admission writes no receipt
and performs no IO/calls/retries/refunds or complete-upload/load/start/release admission.
Qualified integrations explicitly use the unchanged journal transition. Settled
uncertainty clears only observation, retaining pending mutation, all consumption,
original plans/source evidence and references. The public native journey retains
baseline/list originals before mutation, records synthetic settled uncertainty and
reopens exhausted accounting without a new provider call. This qualifies local
persistence, not IC behavior. See [the contract](contracts/ic-snapshot-upload-settlement.json)
and [fresh source review](ic-snapshot-upload-settlement-source.json).

## Real single-canister platform qualification

The test-only driver uses published Testkit 0.25 for managed server startup/cleanup
and the complete PocketIC re-export, with retained private raw output. The
[qualified consumer pins](shared-tooling.md#consumer-owned-ic-pins) select 16.1.
Original calls, reservations and application safety remain local. It exercises
actual PocketIC capture, complete bounded
metadata/data reads, canonical durable artifact publication and retained original
manifest replay through the production library. Exact source-bound upload preparation
feeds real metadata/data calls, complete destination metadata/byte verification and
same-release load/start into the same existing ID. A separately accounted fresh
snapshot verifies the complete loaded state while stopped before restart. The
inspected fixture proves restored heap/global/stable/certified state and chunk-store
contents after deliberate pre-load changes, with exact timer/hook metadata.
Original intent and immutable spending precede
every tested management ingress. Deliberately discarded capture/allocation/data
and stop/load/start replies require reserved observations and independently controlled
original-request attribution before explicit existing receipts. Load attribution
includes full stopped-state verification. Losing its status reply retains both pending
reservations and stops before further verification or restart; no receipts, retries
or refunds occur.
Local replay preserves originals and source-reference bytes without remote calls.

This is an integration qualification driver, not an installed product provider,
transport, runner or CLI. The fixture owns exclusive simulator membership, caller,
permissions and no-external-effects/drain admission. It qualifies no Canic adapter,
application fence, arbitrary metadata configuration, process/network interruption,
application-specific lifecycle recovery or full product terminal/reference release.
Public APIs, v1 records, bounds and release identity are unchanged. See
[the qualification guide](pocketic-qualification.md),
[machine review](pocketic-qualification.json) and
[issue #17](https://github.com/dragginzgame/ic-backup/issues/17).

## Originally reserved snapshot transfer reads

`model::ic_snapshot_transfer_read` borrows the existing exact metadata or data
request under the full original plan and operation sequence. Its wire digest and
canonical target must match the original operation authority. The existing journal
must retain that authority, an exact pending update and no pending recovery
observation. Semantic reads still use replicated-update ingress, so the current
mutation reservation lane accounts for them. Reconstruction creates no journal,
spending, read permission or proof that the request was never dispatched.

`IcSnapshotTransferReadResponse` admits original attempt numbers 1..=1,024,
canonical actual claimed targets and at most 2 MiB raw wire. Pure
`policy::ic_snapshot_transfer_read::validate_response` rechecks the current journal
and exact authority/attempt/context/target claims, then delegates to the existing
metadata or data decoder. Metadata retains its tighter 1 MiB raw ceiling; data
retains exact range lengths, known chunk hashes and its 1 MiB actual-byte ceiling.
Existing 256 raw-ID bytes, 4 KiB arguments and decoder work/table/header quotas
remain. Data evidence retains the supplied metadata evidence separately from the
unchanged nonrecursive wire hash. Integrations durably retain and qualify that
original metadata; byte association alone authenticates none of it.

`IcSnapshotTransferReadProvider::read_snapshot` permits one originally accounted
exact host update. It reuses `IcObservationProviderError` and performs no hidden
retries, queries/proxies, status/list follow-ups, funding or extra calls. Actual
network/caller/target authentication, fresh method-specific snapshot access, stable
snapshot custody and proof of never-dispatched command custody remain integration
responsibilities. Lost replies stay pending. Passive success changes no journal;
integrations explicitly qualify and record any receipt through the existing owner.
No full transfer, durability, restart/load safety, terminal or fence/reference release
follows. There is no persisted schema, new evidence encoder or installed transport.

The private real PocketIC driver now uses this public boundary for every ordinary
metadata/data read before its explicit success receipt. Separate actual-ingress
reply-discard cases stop before download or retain Created partial artifact staging
and source references. Reopen changes no journal/reference bytes or spending and
invokes no provider. These are isolated single-canister simulator observations,
not process/network loss or arbitrary application qualification. See
[the port contract](contracts/ic-snapshot-transfer-read-port.json),
[the real backend scope](pocketic-qualification.md) and
[issue #22](https://github.com/dragginzgame/ic-backup/issues/22).


## Direct Agent transport hard cut

The maintainer selected `ic-backup-agent` as the sole product transport. Its async
preparation/submission binds existing reserved requests to exact current journals,
actual signer and declared original context. It sends exact receiver/effective-target/
method/update bytes without retries, polling, root fetching or receipt transitions.
Signed ingress evidence precedes dispatch; pending/lost/error replies retain original
spending, obligations and references. The ICP-specific executable probe is removed;
retained historical evidence and independent descriptor custody contracts remain.
[Transport scope and qualification](agent-transport.md) own current bounds and
host limits. Core model/policy contracts still install no provider or generic runner.
