# Native IO and diagnostics hard-cut review

2026-10-08. Method: `audits/flow-convergence-and-duplication.md`, adopted through
Shared Tooling `0ba0ad00ed94848e54ecc82629b6b7873b7284c0` (0.1.23).
Provisional product overlay: current AGENTS.md's native filesystem, private
publication, descriptor custody and diagnostic invariants; no dedicated reusable
audit overlay is claimed. The maintainer requested continued implementation and
hard-cut improvements. [Exact source/graph identity](artifacts/source-identity.json)
binds the dirty 0.7.0 batch to the released 0.6.0 base and the original overlay.

## Scope and result

PASS for the inspected convergence obligations after authorized cleanup and
focused Linux checks. The finding was LOW: four private errno adapters plus inline
copies repeated an upstream conversion without a distinct policy. They now call
Rustix's existing `From<Errno> for io::Error` directly. The locked 1.1.5 source
confirms that this retains exactly the original raw OS code. The selected Metrics
0.2.11 [archive and committed-source proof](artifacts/metrics-source-proof.json)
admits the actual graph; its Rust is unchanged from 0.2.9/0.2.10 and features are empty.
No throughput, memory, complete product or all-host qualification claim is made.

## Owner trace and disposition

| Behavior | Canonical owner | Consumers | Disposition |
| --- | --- | --- | --- |
| Native errno to IO error | Rustix `From<Errno>` | Safe artifact traversal/copy, durable directory publication, IC artifact creation/verification/upload, sidecar/layout/command locks | Delete duplicate helpers; preserve existing typed projections and contention branch. |
| Diagnostic arithmetic | Registry Metrics histogram/summary | Guard-local success/failure and prepared-size reporting | Retain one byte histogram and six independent duration summaries; no local arithmetic or persisted progress. |
| JSON preflight/publication | Backup serialization/private parents and Host descriptor publisher | Create/replace plus crash fixtures | Retain serialize-once byte buffer and synchronized barriers; streaming does not preserve the original order. |
| Original source/custody admission | Existing guarded journal, retained plan and descriptor owners | Fresh IC verification/upload and restore-source/copy recovery | Keep before/after checks at their independent corruption/custody boundaries. |

No compatibility alias or dual reader is present in the inspected metrics/artifact/
lock scope. General model codecs, live transport, terminal/reference release and
application qualification are outside this narrow structural review. Their retained
v1 contracts, unfinished effects and external obligations are not disposable.
The known production transport gap remains on GitHub issue #25; this cleanup does
not implement a provider or qualify dispatch/retry/authentication.

Removed functions (not moves or renames), all replaced by upstream IO conversion:

- `ops::artifacts::secure::unix::errno_to_io`
- `ops::persistence::artifact_commit::supported::errno_to_io`
- `ops::persistence::file_lock::errno_to_io`
- `ops::persistence::download_journal::ic_snapshot_artifact::errno_to_io`

No method or type is removed. Public shapes, persisted bytes, original allowances,
references, obligations and platform admission are unchanged. The 0.7.0 draft
remains necessary for the independent Host publication-error Rust identity cut;
package/receipt stay 0.6.0. No source changes outside Backup, dependency update,
Git mutation, release, upload, live IC effect or destructive cleanup runs.

## Focused verification and limits

Evidence owner: `target/hard-cut-070-review`. Incoming source/manifest/lock snapshots,
actual registered cases, source hashes and full logs are retained. Offline metadata
admitted the selected cached graph before compilation. Checks pass:

- Four Metrics units and 21 guarded IC artifact units, plus public upload recovery,
  before cleanup (`metrics.log`, `ic-artifact.log`, `public-upload.log`).
- All 142 affected ops cases after cleanup (`ops-cases.log`, `ops.log`), including
  real lock/process/publication failures, child death and retained evidence.
- Eight public cases across artifact, upload, restore source, local recovery and
  execution settlement (`public.log`).
- Configured package all-target/all-feature Clippy with warnings denied, plus
  Rust 1.91 compilation (`clippy.log`, `msrv.log`).

The initial public command named a nonexistent `local_restore_artifact` test target;
Cargo rejected it. Its log is retained, and the actual public targets passed.
No new test merely prohibits the removed symbol. No broad CI/release gate, standalone
package or simulator rerun is claimed. Earlier PocketIC runs keep their original
source/dependency identities. Current consumer native macOS delivery is still
required independently of upstream CI.
