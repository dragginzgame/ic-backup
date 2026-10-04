# Implemented extraction boundary

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
owned child descriptor setup and exact local quiescence admission.

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

| Canic surface | Extraction disposition |
| --- | --- |
| Hash helpers and artifact IO | Copied into pure checksum records and artifact ops; canonical decoding and UTF-8 identity strengthened |
| JSON IO, journal/file locking, artifact publication | Copied into persistence ops; record byte bounds/private permissions added; direct regressions rerun |
| Layout lifetime locks and reference retention | Copied with model-owned v1 validation/transitions, stable parent-side locking, directory identity and explicit bounds |
| Command lifetime locks | Copied with owned inheritance, exact v1 sidecar identity, one-spawn allowance and retained exclusive quiescence; fresh native process evidence |
| Reference release and prune | Remain in Canic until terminal completion, backend custody and runner integration govern release/deletion |
| Download/execution/restore journals, plans and manifests | Require generic v1 records, exact identity/budget binding and model-owned transitions before runner import |
| Topology, discovery and registry contracts | Generic graph/explicit-set mechanics can move; authoritative Fleet/Root discovery and routing stay integration-owned |
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
