# Upstream refresh after 0.8.0

The compatible 0.8.1 draft adopts committed Shared Tooling 0.1.27 and qualifies
the incoming Host artifacts/FS 0.7.1 and Metrics 0.2.13 lock selection. Package
versions and release receipt remain 0.8.0. Ordinary work stays uncommitted.

The canonical exporter reads clean exact source
`b866d41041a1986eeec95bde9af4c6ba0853d2e3` and writes the existing 77-path
selection. All bytes/executable modes match committed source; 32 paths change.
Common baseline/rules remain unchanged. A retained isolated consumer passes
consecutive uncommitted refresh, then rejects edited input while retaining the
manifest, edited bytes and unrelated work. The source checkout remains read-only.
[Snapshot source proof](snapshot-source-proof.json) owns the exact inventory.

All 46 published Rust/original-manifest blobs from the selected patch libraries
match official committed source and non-yanked registry checksums. Their Rust
implementation bytes are unchanged from the previous selected versions; selected
features remain empty. Host 0.7 error identity is unchanged. No process dependency
is selected. [Published source proof](published-source-proof.json) retains exact
archive/source associations. Previous [Shared](previous-shared-review.json) and
[Host](previous-host-review.json) reviews keep their original scope.

Upstream bootstrap and fixture isolation changes also apply to local entry points.
The original relative release command fails under `CDPATH`; original metadata
fixtures fail under inherited `MAKEFILES`/`GNUMAKEFLAGS`. Both failures remain
retained. Local scripts now resolve physical absolute roots without capturing
directory-search output; release fixtures exclude enclosing Make context. The
actual version entry point and metadata fixtures pass in those same environments.
Production Make selections are preserved. No function, method or type is removed.

The frozen graph passes 158 focused Rust cases: affected persistence/diagnostics,
public recovery, actual Agent HTTP/PocketIC and core lost-reply/same-ID journeys.
Both libraries pass Clippy, Rust 1.91 and verified package builds. All 61 release
adapter cases, shared release/validation runners, snapshot/retention, tooling LOC,
hooks, formatting, dependency declarations, snapshot, shell and documentation
checks pass. Shell cases run under inherited `CDPATH`. The existing explicit CI
archive roots/retention remain selected; upstream's new compact tool-evidence
selector is outside this consumer's file set and policy.

[Qualification](qualification.json) binds actual source, graph, archives, commands,
case registries, logs and failed/corrected attempts. Current exact-source Shared,
Host and released Backup CI is queued; Metrics CI is in progress at the last read.
Prior Shared 0.1.26 Intel CI timed out under
[upstream #71](https://github.com/dragginzgame/shared-tooling/issues/71).
Local Linux qualification does not supply current consumer native macOS proof.
Full CI/release gates were not requested or run.

The public description is corrected because Agent transport is now released.
Generic orchestration/stage binding remains
[#29](https://github.com/dragginzgame/ic-backup/issues/29), and fresh downstream
Canic trust/application qualification remains separate. No sibling edits, root
Git writes, live IC effects, release/version transaction or registry upload run.
