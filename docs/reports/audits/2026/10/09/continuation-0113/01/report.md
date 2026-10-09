# 0.11.3 single snapshot read step and Shared 0.2.5 review

Released base: `2f441df35bffb0e9a8d4ea93e826045d0e71424d` (0.11.2).
The compatible undated 0.11.3 draft extends the incoming CI-source fix for
[#33](https://github.com/dragginzgame/ic-backup/issues/33). Package/receipt remain
0.11.2. Manifest bytes remain exact. Initial qualification used the incoming
Metrics 0.3.2 / Host 0.9.2 / Testkit 0.27.0 lock. During verification the maintainer
changed the lock to Host 0.9.3 and Testkit 0.27.1, retaining Metrics 0.3.2.
Preserve that new selection rather than restoring the incoming lock. Capture both
input hashes and qualify the final graph separately. The agent reselected no package.

## Implemented step and ownership

`workflow::ic_snapshot_transfer_read::read_snapshot` now coordinates a single
metadata/data replicated update under the exact retained execution stage.
Canonical payload binding rejects before consumption; the original journal is
opened and complete original-plan admission owns durable reservation. A mandatory
fallible integration callback qualifies fresh access, original snapshot/metadata,
application requirements and never-dispatched command custody. The selected
journal stays locked through admission, one provider invocation and bounded
passive response association. Retained stage and ancestor checks bracket dispatch.

Successful responses remain pending. The existing explicit receipt owner alone
may record independently qualified outcomes; the coordinator never creates a
receipt, refund or additional journal. Lost/failed/malformed replies preserve
original consumption. Association and post-reply rejection errors retain the
bounded returned response. Pending/Applied attempts, missing/held originals and
unfulfilled prerequisites reject before another callback/provider invocation.
Original request/decoder/attempt limits, schemas, source references and fence
obligations remain unchanged. Sequential checks do not fence noncooperating actors.

The isolated public Testkit driver now calls the installed step for metadata and
every planned data request. It retains its fixture-specific authentication,
stopped/no-external-effects admission, original ingress history and explicit
qualified receipts. Lost/malformed second reads preserve partial Created staging
and source references; attempting the dependent step and resuming invoke no new
provider. This qualifies an isolated application, not arbitrary Canic or a full
backup/restore runner. No default Agent provider, fresh-permission flag, complete
product manifest, terminal proof or fence/reference release is installed.
No Rust function, method or type was removed. Direct caller-side reservation/
dispatch assembly is replaced by the coordinator; canonical model, port, policy
and receipt APIs remain their distinct owners.

## Shared snapshot

Canonically refresh the existing 92-file roster from a clean isolated source of
committed Shared 0.2.5 `04e07b4bf54e7aeb03eb7804a845cee27b7305df`.
The pre-commit hook, installer and hook rule are the only selected changes.
Literal trailing-newline paths are preserved; failed Git configuration reads
stop setup rather than replace an existing selection. Other common rules,
production helpers and pins are unchanged. No snapshot file was patched locally.
The complete owner hook regression runs from the isolated source; consumer hook
fixtures independently exercise actual formatting/index preservation and local
setup. Root hook activation/configuration remain untouched.

## Focused evidence

Retained logs and exact checksums are bound in [qualification.json](qualification.json).
`target/continuation-0113/` holds earlier attempts and final commands. The focused
runtime set passes: seven new workflow cases, five original passive read cases,
four metrics cases and three actual Testkit planned-download cases. Warning-denied
core test/library and Agent library Clippy pass, as do public API rustdoc, both
independent Rust 1.88 consumers and a standalone caller of the new entry point.
Snapshot/pin/ShellCheck, formatting, selected Testkit offline admission and local
Markdown links pass. Final checks use the selected Host 0.9.3, Metrics 0.3.2 and
explicitly prepared Testkit 0.27.1 CLI with the retained PocketIC 16.1.0 server.
Earlier Host 0.9.2/Testkit 0.27.0 evidence stays separate. Metrics' actual locked
graph remains instruction-free.

The initial native simulator attempt failed at sandbox loopback binding; its
retained fixtures/log are separate from the successful local-server reruns.
Initial compile/Clippy attempts exposed a fixture include path, an error conversion
and test-only lint/type issues; their logs remain retained. Subsequent focused
qualification applies to the final sources, not those failed attempts. After the
external lock change, offline qualification correctly rejected uncached Host 0.9.3;
prepare only the newly selected locked cache and explicitly run
`make install-testkit-server` for 0.27.1 before its offline admission and simulator
checks. Keep the older CLI/server installations. Committed Host crate Rust sources
and Testkit runtime sources are unchanged in these patch updates; Testkit's own
Make CLI-build fix and native CI changes require no new consumer shim or API offload.

Released 0.11.2 Linux CI passes in
[its exact-source run](https://github.com/dragginzgame/ic-backup/actions/runs/37936454297);
both macOS jobs are queued. That release run does not qualify this uncommitted
candidate. Keep [#29](https://github.com/dragginzgame/ic-backup/issues/29) open for
full provider-driven capture/transfer/upload/restore coordination, application
qualification and terminal/custody proof;
[#32](https://github.com/dragginzgame/ic-backup/issues/32) retains native acceptance.
No full workspace/CI/release gate, root Git write, release/version transaction,
push/publication, production IC effect, cleanup of recovery evidence or sibling
edit occurred. All candidate work remains uncommitted for maintainer review.
