# Current handoff — 2026-10-23

## Ownership and source

Core owns bounded backup records, guarded persistence, reconciliation and workflow
coordination. Agent owns reserved signed transport. Platform owns external IC
identities/codecs/SDKs; Testkit owns simulator assets and managed processes.
Applications retain membership, permissions, consistency, fencing and external
obligations. Current contracts belong to [implementation contracts](../implementation-contracts.md)
and [the record/API boundary](../extraction-boundary.md).

Released source is **0.39.0**, `0342c67`. Pending notes select **0.40.0** because
public Host checksum/publication error identities change from 0.65 to 0.66.
Consumers exchanging those types must align Host dependencies. Own versions
remain **0.39.0**. No staging, commit, release, publication, deployment, live IC
effect or sibling source edit occurs here. Production Rust, wire/storage formats,
bounds and recovery semantics are unchanged; only two simulator fixture call sites
change. Incoming `cc` 1.10.0 and `webpki-root-certs` 1.0.11 updates are preserved.

## Published dependency adoption

Host Artifacts/FS/Process, Testkit and the Build release editor select published
**0.66.0**. Registry checksums and archive VCS identify
`d71f4690c7e8e3ffe99f72467bc35cbadce33042`. Host libraries and Build have no Rust
source changes from 0.65.0. Testkit makes client/server fields private and exposes
explicit ownership transfer; its baseline/error/standalone callers migrate too.
See [the upstream migration](https://github.com/dragginzgame/ic-tooling/blob/d71f4690c7e8e3ffe99f72467bc35cbadce33042/docs/changelog/0.66.md).

Core and Agent fixtures call `into_parts()` and retain their original managed
server. The synchronous fixture still drops its client before its server. The
nonblocking fixture still stops its owned server and all private instances at
fixture drop; no automatic asynchronous client deletion or custom destructor is
added. Readiness/request bounds, cleanup custody, raw server output and original
application evidence are unchanged. No borrowed-server construction is used.

Platform **0.6.10**, Agent/Transport **0.49.3**, management DTO **0.11.0**, external
Principal **0.1.5** and PocketIC **16.1.0** remain selected. The Host adoption changes
only its four Host/Testkit lock entries; the subsequent concurrent Platform patch
is preserved separately. Library graphs enable neither
Testkit CLI/Wire nor development-only Host/Platform features. CLI tools retain
explicit `cli` selection.

## Focused qualification

Linux Rust **1.99.1** passes strict Clippy, all-target/all-feature compilation and
warning-denied Rustdoc. Actual Rust **1.91.0** owning and independent normal-consumer
checks pass, including public Host error identity probes. Both unpacked package
archives compile together with original selected identities and verified checksums.
Normal graphs contain only Host 0.66 and exclude Testkit/PocketIC/WAT/Wire and
development-only features; Core remains Agent-free.

**19 selected runtime tests pass**: nine public artifact/recovery/workflow
cases, six connected Core PocketIC cases, both independent Agent cold-process
journeys, complete authenticated same-ID round trip and lost-stop refusal.
These exercise both changed startup sites, actual client/server lifetime, complete
restored bytes, original pending spending, retained source references and
no-redispatch/effect-free replay. Source, manifest and lock fingerprints remain
unchanged throughout the runtime selection. No new product safety default is added.

The unchanged Host implementation's prior 0.65 artifact/persistence and HTTP
regressions retain their original graph scope; they are not claimed as new 0.66
execution. Shared Tooling now selects reviewed committed **0.26.8**, revision
**`0b07c2a3163a5a66328e100b553baf93afd8ed07`**, verified against remote main.
All **122 payloads** verify; only the cache/build helper, slot-wait helper and
setup guide change. Admitted owner withdrawal is tolerated only for the same
physical claim. Cache initialization has no global installation lock; Cargo still
serializes downloads. Invalid claims, atomic acquisition failures and offline
admission remain strict. See [the snapshot review](../shared-tooling-review.json).

The changed build-policy digest requires explicit Cargo-tool setup.
`make install-tools` refreshed the common tools and selected Testkit/Build tools,
retaining old generations/source-preparation slots. All offline admission, consumer
shell/tooling/hook checks and matching formatting pass. The actual refreshed editor
renders independently checked manifest/lock candidates in a disposable directory;
real package versions/metadata remain unchanged. No release operation is run.

The earlier Host-adoption Testkit setup initially timed out waiting 60 seconds for the admitted
ic-blob-storage owner of the same tool slot. That owner's build completed; a retry
rechecked and admitted the exact cached tool. No owner lock, process or failed
evidence was removed. Both selected tools and PocketIC pass offline admission.

During Shared Tooling qualification, a concurrent Cargo.lock edit selected Platform
0.6.10. The first run's input guard correctly rejected changed inputs. The update
is preserved; its authenticated archive has no Rust-source differences from 0.6.9,
and its updated optional Wire dependency is not enabled. Locked preparation,
selected tooling/product checks and all 19 runtime cases restarted successfully
against the final graph. No further input changes occurred.

Current evidence: `/tmp/backup-shared-0268-20261023-3ep0swa_/`, including refresh provenance, explicit setup,
read-only editor probes, retained input-change failure, restarted product/runtime
results, final normal-consumer metadata, package archives and simulator fixtures.
Validated lock SHA-256:
`5c60401b984b472b3eb4487270191b3004ebf4bb3de9cdb6320218a889a31e35`.
Earlier `/tmp/backup-host-066-20261023-rh83v4s4/` results retain their Host 0.66 /
Platform 0.6.9 scope. Pending Host source/manifest changes and unrelated work are
preserved exactly; Shared adoption changes no Rust source or product protocol.
The [previous handoff](https://github.com/dragginzgame/ic-backup/blob/0342c67/docs/status/current.md)
retains the 0.65 qualification and earlier history.

The full local workspace test suite/make ci, new hosted/native macOS, registry-only
published 0.40, installed-IC and Canic application acceptance are not run here.
No publication or live backup attempt is authorized by this preparation.

## Canic adoption preparation

The following retains the 2026-10-22 published-0.38.0 qualification scope.

[The adoption guide](../canic-adoption.md) maps actual consumers to public
interfaces and the remaining Root/application acceptance. Canic's dependency
adoption is already present; live creation still refuses before effects. Its
terminal provider already exists. Production Root custody, authenticated whole-unit
orchestration and management dispatcher identity remain Canic-owned work.

Both existing cold-process journeys pass again on Linux with the child selecting
only registry Core/Agent **0.38.0**, not local paths. The isolated parent export
uses the released harness; its explicit adaptation changes only child dependency
source selection, corresponding graph admission and the evidence scope label.
All other selected package identities and simulator exclusions remain enforced.
The first run stopped at Testkit admission because the export used a tool-directory
symlink; explicit setup in a physical directory fixed preparation before rerun.
Success proves complete cold capture/restore/accepted start/terminal replay and
death after authenticated load with retained pending spending/no redispatch for
the isolated application. It is not installed Canic or live-provider acceptance.

Evidence: `/tmp/backup-canic-prep-20261022-7hg9oc6n/`, including the adaptation,
registry manifests/locks/metadata, binary digest, original journals, raw ingress/
replies, server logs and `registry-journeys-retry.log`. The ordinary maintained
test harness still selects candidate paths. Registry-only Linux proof does not
supply native macOS execution of this adapted harness or installed-IC evidence.

The maintainer plans the live backup attempt after Blob storage is complete.
Prepare Canic's application authority/fencing under its existing issues; no live
command or new no-external-effects assertion is authorized by this handoff.
[#29 — Complete original-plan execution and stage binding before Canic runner adoption](https://github.com/dragginzgame/ic-backup/issues/29)
retains application acceptance: current authority, whole-selection fencing,
command/byte custody and external-obligation settlement. This dependency adoption
does not establish those production facts or qualify Canic.
