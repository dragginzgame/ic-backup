# Implemented local inventory and payload integrity

The Unix-only `ic_backup::store` API implements initialization, opening,
`list_backups`, `latest_completed_backup` and `verify_payload`. Its
[public documentation and example](../../crates/ic-backup/src/store/mod.rs) are
compiled with the library. Capture, restore, completion writing, operation ledgers,
target claims and full operation-evidence verification are not implemented. The wider
[local-store design](../design/local-store.md) remains their implementation input.

## Custody and initialization

The caller supplies an existing empty private directory and excludes external
namespace/content writers. Host capability handles refuse parent traversal and
symlinks; directories forbid group/other permissions. The initializer creates a
0600 `format.pending` exclusively, writes and syncs the marker, creates private
`bundles`, `operations` and `targets` directories, then publishes `format`
create-only through Host's durable primitive. The exact marker is
`ic-backup/local-store/1` followed by one LF. Failures retain partial work.
There is no reset or automatic initialization recovery.

Opening performs no writes and requires the marker and exactly those three child
directories. Missing, unsupported, occupied or unsafe structure refuses. Held
handles remain attached after a root rename; the caller owns location continuity
and private custody. Initialization contention permits one exclusive claimant;
this is not an implemented per-canister operation lock.

## Inventory semantics

Each bundle directory name is a 32-character lowercase hexadecimal operation ID.
A missing `manifest.json` means no published completion observed and is omitted.
Observed manifests must be private regular files without hard-link aliases.
Unknown bundle names/kinds, malformed records and read errors fail the scan,
including for other targets. Inventory does not inspect payload, reference or
operation trees and is not complete-bundle filesystem admission.

Callers supply positive entry, cumulative manifest-byte and result limits.
Unfinished bundle directories consume entries; nonmatching records consume bytes
and are decoded before filtering. Each manifest is additionally capped at 1 MiB.
List results sort descending by source capture timestamp, host completion timestamp
and operation ID. Latest scans the same candidates but retains only one row.
No limit failure returns partial rows or an older fallback. No-match describes
only what this bounded scan observed in this store. A concurrently published
manifest may appear on the next refresh; this is not a fleet transaction.

## Record boundary

The [private manifest reader](../../crates/ic-backup/src/store/manifest/mod.rs)
admits `ic-backup/local-manifest/1` with typed JSON and rejects duplicate,
unknown or missing fields. It checks canonical Platform principal text, lowercase
SHA-256 values, the operation/directory binding and a 1–256 byte snapshot identity
encoded as lowercase hex. Unsigned timestamps preserve the source and host clocks
independently; timestamp order supplies no effect attribution.

Fields bind target, source snapshot, both timestamps, release, module hash, total
payload length, plan/ledger/metadata argument/reply commitments and payload members.
Evidence length caps are respectively 1 MiB, 128 MiB, 4 KiB and 1 MiB; declarations
must be nonempty. The payload declares exactly one module, heap and stable-memory
region, plus at most 1,024 named chunks, with at most 1 GiB total. Module bytes are
nonempty and its hash agrees with the summary; chunks are at most 1 MiB and bind
the hash in their name. Member names are closed, unique and path-free. These are
reader bounds, not measured capture capacity or supported management-call sizes.

The summary includes the hash of the exact observed manifest bytes. It is a
passive completion declaration: hashes/lengths are checked for internal consistency,
not against files. Listing establishes neither fresh integrity, authenticated
provenance, prior durable completion, restore authority nor application safety.
There is deliberately no public API for manufacturing a completion receipt.

## Payload integrity

`verify_payload(bundle_id, expected_manifest, maximum_payload_bytes)` is a
read-only byte check. It admits the exact manifest against the caller's trusted
original SHA-256 before interpreting its record. An observed inventory hash alone
cannot establish provenance. The positive caller byte limit is checked against
the admitted total before opening the payload directory; the manifest's own
1 MiB and 1 GiB payload caps still apply.

The checker traverses the held private `payload` directory with at most the
declared number of entries. Every declared member must exist exactly once, with
no extra entries or subdirectories. Files must be private regular files with no
symlink or hard-link aliases. Host's exact bounded streaming hash checks every
file's complete length and digest, including empty regions and named empty chunks.
The scanner retains only bounded metadata and constant hashing scratch space,
not a payload-sized buffer. The first failure stops the check; detecting growth
may read one byte past that file's declared length. Files are never repaired.
After each file read, the original descriptor stays open while Host identity and
length are compared with the current private no-follow name. Closing admission
rechecks the complete payload member set, rehashes the bounded manifest while
retaining its original descriptor, and compares the payload/bundle/bundles child
directory identities under the held root. A detected change refuses success.
The root's absolute pathname may still be renamed as documented above.

The caller excludes namespace and content writers throughout the check. Success
returns the summary after observing matching payload bytes; it does not hold
verified files for later consumption or guarantee their future contents. Any later
use must admit them again under its own custody. The checker does not inspect
operation plans, ledgers, metadata evidence or source references, and does not
replay authority, sync files or publish completion. Those remain prerequisites
for full bundle admission and restore. Payload integrity is a useful diagnostic,
not a `BackupVerified` capture receipt or a restore handle.

## Focused legacy review — 2026-11-14

Reviewed the pinned legacy revision `4ae087fad8c23de712aa60bf569bde4b2b932651`:
[artifact verification](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/ops/persistence/download_journal/ic_snapshot_artifact/verification/mod.rs),
[member admission](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/ops/persistence/download_journal/ic_snapshot_artifact/member/mod.rs),
[manifest replay](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/ops/persistence/download_journal/manifest/mod.rs)
and their native regression tests. The problems were stale file names after reads,
changed tree membership, lost original-record association and confusion between
record replay and fresh byte verification.

History reviewed: [a3d520c](https://github.com/dragginzgame/ic-backup-legacy/commit/a3d520c2cfb768b4d10f2b4a51058ca6f9a00349)
centralized original admission and adopted Host file identities while preserving
checks around byte IO. [9f7b886](https://github.com/dragginzgame/ic-backup-legacy/commit/9f7b886a824eec67dda8f81792a560436419bb2a)
reduced upload-planning scans but deliberately retained closing verification and
added post-scan mutation regressions. These checks were not redundant scaffolding.

| Legacy regression/source | Adaptation in this library |
| --- | --- |
| [Same-length replacement after read](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/ops/persistence/download_journal/ic_snapshot_artifact/verification/tests/mod.rs) | `legacy_same_bytes_file_replacement_during_read_is_refused` reproduces and now refuses the stale-name success. |
| Closing tree/namespace checks and post-scan mutations in 9f7b886 | Extra-member, child-directory, post-hash extent and manifest-change cases exercise the shared production closing path. |
| [Changed/unsafe/unbounded children and maximum chunks](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/ops/persistence/download_journal/ic_snapshot_artifact/tests/verification/mod.rs) | Existing corrupt/short/long/missing/extra/link cases retained; new FIFO closing refusal and 1,024-chunk/1 MiB-chunk acceptance. Originals remain in test fixtures. |
| [Manifest replay without artifact reads](https://github.com/dragginzgame/ic-backup-legacy/blob/4ae087fad8c23de712aa60bf569bde4b2b932651/crates/ic-backup/src/ops/persistence/download_journal/manifest/tests/mod.rs) | Existing inventory test admits a declaration without payloads; fresh payload verification stays explicit. |

The replacement/extra-entry regressions failed against the previous checker, then
passed after adapting closing checks through Host capabilities and `FileIdentity`.
Cases are re-authored for this format, not a copied legacy suite or qualification
receipt. No production source was copied and no dependency was added.

Changed assumptions matter: legacy replay binds original plans and journals;
our inventory is only an untrusted completion projection. Payload checking binds
a caller-trusted manifest but does not yet admit those operation records. Legacy
requires the original layout pathname; this store deliberately follows a held root
through rename, while now refusing changed children within it. Legacy permits an
empty module; this reader retains its selected nonempty-module profile. Empty heap,
stable-memory regions and named chunks remain covered.

Legacy publication/death recovery, original-plan/journal/metadata mismatches and
upload admission cases remain required for their future owners, not claimed by
these diagnostics. No generic journal framework or legacy compatibility reader
was introduced. Sequential closing checks still do not fence concurrent writers
or establish a point-in-time immutable tree.

## Evidence

[Native tests](../../crates/ic-backup/src/store/tests/mod.rs) cover initialization
contention/interruption, held-root replacement, link/permission refusal, budgets,
ordering/filtering, malformed records and refresh after visible publication.
[Payload tests](../../crates/ic-backup/src/store/tests/payload.rs) additionally
exercise exact-budget success with empty and nonempty chunks, corruption,
truncation/growth, missing/extra/unsafe files, root replacement and refusal of a
self-consistent replacement under the original trusted manifest commitment.
The synthetic [golden manifest](../../crates/ic-backup/src/store/tests/fixtures/completion.json)
has SHA-256 `d6ae354787c3955deffe807213f2ac6b953d96456abd8eb8413ba5944804d64e`,
checked independently with `sha256sum`. Its payload/evidence hashes are illustrative;
it is not an IC capture fixture. Tests intentionally demonstrate that inventory
can read a declaration without those payload or evidence files.

Local Linux results do not qualify power-loss durability, hosted macOS, IC effects
or Canic integration. Host owns OS primitives; Backup's native tests exercise its
selected composition. Failed fixture directories remain under the owning target.
