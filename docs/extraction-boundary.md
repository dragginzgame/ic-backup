# Implemented extraction boundary

The first extraction copies and reshapes Canic's local artifact and persistence
mechanisms into `ic-backup`. It performs no IC calls and introduces no Canic
dependency. [Fresh provenance](extraction-source.json) records inspected
working-tree hashes, copied source/destinations and consumer references. The
original [planning baseline](source-baseline.json) remains retained evidence.
MIT contributor notices remain in the root license.

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

## Source ownership and remaining contract work

| Canic surface | Extraction disposition |
| --- | --- |
| Hash helpers and artifact IO | Copied into pure checksum records and artifact ops; canonical decoding and UTF-8 identity strengthened |
| JSON IO, journal/file locking, artifact publication | Copied into persistence ops; record byte bounds/private permissions added; direct regressions rerun |
| Layout lifetime locks, restore references, command custody | Remain in Canic until their generic schema and runner integration are specified |
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
