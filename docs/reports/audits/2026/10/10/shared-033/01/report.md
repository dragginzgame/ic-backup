# Original restore steps and Shared 0.3.3 review

Released base is 0.14.1 at `434454b2923c2cd004ffe2697770a7f3b9b07d7e`.
The compatible uncommitted draft is 0.14.2; package/receipt stay 0.14.1.

Canonically export committed Shared Tooling 0.3.3 at
`d63f0cfaba8ab2961d6012064adbf051c1898bc1` from a clean isolated checkout.
Only the runner and its regression owner change within the same 96-file roster.
Canonical bounded decimal nesting depth rejects before dispatch; explicit
completion refuses premature success and preserves available source/log evidence.
Completed target failures retain their original status. Local dependency discovery
preserves Make jobserver descriptors for Cargo metadata. Dirty sibling work is
excluded; no vendored path is patched locally.

Preserve the final incoming lock's four Host 0.12.4 selections, Testkit 0.32.2
and Metrics 0.5.3. Published Host and Testkit Rust source trees match 0.12.3 and
0.32.1 respectively; Metrics matches 0.5.2. These releases add no product API
to offload. Exact published Host/Testkit revisions are
`5400f159474cebac1ec7ae7c8763abfd258bde03` and
`9e697b6363441e3fa96477adae5b07820461c3c5`, verified remotely. Explicit canonical
setup prepares the exact Testkit CLI and admits the existing PocketIC 16.1.0
server. Earlier installations, receipts and failed evidence remain. Agent 0.49.2
stays selected; normal consumers retain no Testkit/PocketIC/process/tools dependency
or Metrics feature.

Host 0.12.4 also adopts upstream Shared 0.3.4's Binaryen 133 tooling pins.
That is a separate consumer toolset selection, not a Rust API change or an implicit
change to Backup's requested Shared 0.3.3 snapshot. Remote Shared main is now
`169d77b8440568c5200eede971625126181f7bb2` (0.3.4). Deliberate adoption and
qualification of those tool bytes remain the next tooling refresh; sibling 0.3.5
work addressing #105 is not treated as a committed snapshot.

The new public `workflow::ic_snapshot_restore::restore_snapshot` coordinates an
exact singleton original load/start with both retained source/restore plans and
immutable safety requirement. Existing spending, current safety, reply codecs,
explicit qualified receipts and checkpoints remain their sole owners. Fresh
admission must independently qualify actual permissions, complete uploaded source
and command/byte/application custody. A mandatory actual observation passes existing
safety policy before one update; independent durable original request/reply retention
precedes the exact Applied receipt. Load settlement alone never admits later start.

Real Testkit complete-upload journeys now continue through load, independently
verified complete stopped state, and start. Original dependency 17 → 3 qualifies
ordering independent of numeric sequence. Lost/malformed load stops before creating
start; lost/malformed start retains pending originals without restart. Source bytes,
references and original spending remain. Existing uncertain-effect recovery tests
retain their distinct reconciled observations; no implicit recovery is installed.

Focused local reservation/safety/receipt/checkpoint cases and actual simulator
success/lost/malformed journeys pass. Two local tests cover successful fenced
load/start and twelve refusal boundaries. Full `make ci` passes all sixteen gates
after fixture isolation. Incoming Metrics 0.5.3 arrives before compiled gates;
Host 0.12.4/Testkit 0.32.2 arrive during the suite's tail. That earlier suite is
retained for its actual inputs, not relabeled as frozen final-graph qualification.
After explicit setup, all twelve graph-dependent gates pass again on the unchanged
final inputs: snapshot/tools/pins/links/fetch/format/check/Clippy/test/doc/MSRV/package.
Reuse unchanged passing shell/tooling/release/hook evidence from the full suite.
The final graph passes 502 core unit cases, 29 core simulator journeys, eight Agent
HTTP cases and three Agent simulator journeys, strict Clippy/docs, Rust 1.88
workspace and independent normal consumers, and both packages. A separate locked
Rust 1.88 consumer compiles the complete new public restore API on an exact normal
subset of the final graph. Adjacent [machine evidence](qualification.json) binds
source inputs, graphs, CLI receipt, archives and logs.
The sandbox initially refused simulator loopback startup; that evidence remains.
An initial outer upload assertion was applied after intentional restore-reference
retention; correcting its phase preserves both the unchanged-upload proof and
the restore owner's additional durable reference. No recovery evidence was erased.

An initial full suite also exposed inherited `VALIDATION_LOG_DIR` in the shared
runner regression's fallback test. Exact standalone execution passes; inherited
log selection reproduces status 1 at the expected temporary-log assertion. The
local consumer adapter now isolates fixture logger identities and requires explicit
completion before success/cleanup. Inherited-parent adapter qualification passes.
Report the reusable source finding in
[Shared #105](https://github.com/dragginzgame/shared-tooling/issues/105); exported
fixture bytes remain unchanged. Initial suite and both traced runs are retained.

Evidence is retained under `target/shared-033-review/`. This review does not qualify
native macOS acceptance, a default application lane, an async Agent bridge, full
product runners, terminal custody or fence/source-reference release. No root Git
write, release, registry upload, sibling edit or recovery cleanup ran.

Released 0.14.1 has one [exact-source native run](https://github.com/dragginzgame/ic-backup/actions/runs/38053403854):
Linux succeeds and both macOS jobs remain queued. This uncommitted 0.14.2 candidate
has no hosted result. Keep #32/#36 open for native acceptance; #29 retains full
coordination/terminal custody and #25 the async Agent/application bridge.
