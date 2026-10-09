# 0.11.5 complete original data streaming and issue review

Released base: `a585ae1064b1db49cd33f5c02dfe76637bd9f1b9` (0.11.4).
The compatible undated 0.11.5 draft extends [#29](https://github.com/dragginzgame/ic-backup/issues/29)
with a complete original planned data-transfer loop and finishes the remaining
formatting declaration cleanup in [#34](https://github.com/dragginzgame/ic-backup/issues/34).
Package/receipt remain 0.11.4. No release or root Git writes occur.

## Transfer behavior

`workflow::ic_snapshot_download::download_snapshot` consumes an existing private
writer under an exact retained nonempty metadata-derived data stage. Its model owner
re-admits the original binding and retained metadata evidence. Its artifact owner
joins the original layout/intent and descriptor custody. Changed metadata/intent,
prior coverage and any already consumed original attempt reject before dispatch;
missing/held originals never supply zero spending. Read-free plans create no fake
operation or stage and are refused here.

For each original ordinal, call the existing single-read coordinator. Durable
reservation, fresh admission, bounded decoding/association and one provider call
retain their existing owners. Reopen the selected original journal and require
an explicit integration-qualified exact Applied receipt for the admitted response.
Append decoded bytes before the sole attempt owner records that receipt; only then
can a dependent read run. Wire success alone never generates an outcome.

Failures consume the writer and preserve partial bytes, original spending and
recorded receipts. Post-read rejection retains the bounded returned response.
Failed receipt persistence can leave accepted bytes with pending spending, without
restart or completion authority. No partial-read resume, implicit provider reissue,
coverage reconstruction, schema, journal, allowance or progress owner is added.
Complete original Applied progress and coverage precede existing fresh checksum/
durable publication; a closing stage rejection retains the returned checksum.
Immutable manifests/checkpoints remain explicit. References/fences are unchanged.

The actual isolated capture-to-download caller now delegates its data loop to this
public owner. Its stopped/no-external-effects admission, token/raw-ID association,
authenticated exact ingress and receipt semantics remain fixture-owned. Complete
publication and lost/malformed second-read cases pass; neither establishes arbitrary
application consistency, stable noncooperating byte custody or a complete product
runner. Core's synchronous provider loop does not install an async Agent bridge or
bypass the transport's required original signed-envelope/request-ID retention.

## Maintained owners and selected graph

Remove only redundant local PHONY declarations for `format-tools-check`, `fmt` and
`fmt-check`; the adopted Shared formatting include already owns them. Actual hook,
check-only formatting, Testkit-routing and release-adapter fixtures pass. Preserve
the default goal, exact prepared formatter admission and post-run release-resume
receipt/tag/saved-validation check. The 93-file committed Shared 0.2.6 snapshot is
unchanged. No Rust function, method or type was removed or renamed; existing
caller-side loop assembly is retired, with canonical models/ports/decoders retained.

The incoming lock already selects Host 0.9.4. Cached published artifacts/FS Rust
sources are byte-identical to 0.9.3. During qualification an external lock update
selects Metrics 0.3.3 instead of 0.3.2. Preserve it and prepare its locked cache
without reselection; earlier input/qualification remains separate. Final checks
use Host 0.9.4 / Metrics 0.3.3 / Testkit 0.27.1 / Agent 0.49.2 and the existing
prepared Testkit CLI/PocketIC 16.1.0 server. Metrics stays instruction-free on the
host graph; original exact spending never depends on diagnostics.

## Focused evidence and issue acceptance

[qualification.json](qualification.json) binds sources, selections and retained logs
under `target/continuation-0115/`. Eight new streaming tests and 49 total focused
workflow/stage cases pass, along with four metric cases, five actual planned core
simulator cases and three actual Agent gateway cases. Eight Agent HTTP cases,
warning-denied core tests/Agent library Clippy, core rustdoc, both independent
Rust 1.88 consumers and a standalone new-API caller pass. Formatting, snapshot/
dependency pins, selected Testkit offline admission and maintained local links pass.
Tests retain unsuccessful type-inference/Clippy fixture attempts separately.
Offline qualification correctly refused the newly selected uncached Metrics crate;
the locked preparation and final graph results are distinct. Sandbox-only HTTP
listeners were refused before transport behavior; local-loopback rerun passes.

Released 0.11.4 has one main-push
[exact-source matrix](https://github.com/dragginzgame/ic-backup/actions/runs/37949516046).
Linux passes; both macOS jobs remain queued. That run does not qualify this dirty
candidate. Keep [#32](https://github.com/dragginzgame/ic-backup/issues/32) and
[#33](https://github.com/dragginzgame/ic-backup/issues/33) open for native acceptance;
#34 still requires delivery/consumer acceptance of its final cleanup.
[Agent #25](https://github.com/dragginzgame/ic-backup/issues/25) retains installed
consumer identity/root/endpoint and never-dispatched custody qualification;
#29 retains full capture/metadata/upload/restore orchestration, actual application
admission and terminal/custody proof. No overlapping open pull request was found
in the preceding review. No new duplicate issue or premature closure is warranted.

No full workspace/CI/release gate, root hook activation, version transaction,
publication, live production IC effect, Canic/sibling edit or recovery cleanup ran.
