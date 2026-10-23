# Current handoff — 2026-10-23

## Ownership and source

Core owns bounded backup records, guarded persistence, reconciliation and workflow
coordination. Agent owns reserved signed transport. Platform owns external IC
identities/codecs/SDKs; Testkit owns simulator assets and managed processes.
Applications retain membership, permissions, consistency, fencing and external
obligations. Current contracts belong to [implementation contracts](../implementation-contracts.md)
and [the record/API boundary](../extraction-boundary.md).

Released base remains **0.38.0**, `43e65df`. Pending notes select **0.39.0** because
public Host checksum/publication error identities change from 0.64 to 0.65.
Consumers exchanging those types must align Host dependencies. Own package
versions remain **0.38.0**. No staging, commit, release, publication, deployment,
live IC effect or sibling source edit occurs here. Production Rust, wire/storage
formats, bounds and recovery semantics are unchanged. Incoming Canic preparation
and unrelated dependency work are preserved.

## Published dependency adoption

Host Artifacts/FS/Process, Testkit and the Build release editor select published
**0.65.0**. Registry checksums and archive VCS identify `378ad712`. Host libraries
have no Rust-source changes from incoming 0.64.2; Build has none from 0.64.0.
Testkit changes only its unused optional Wire module/reexport and tests. Backup
has no Wire-helper calls and enables no Testkit Wire feature; no adapter is added.
The selected CLI tools retain explicit `cli`; library consumers do not enable it.

Incoming Platform **0.6.9** and Agent/Transport **0.49.3** are preserved. Platform's
optional Wire schema feature remains off; its Rust 1.91 floor matches Backup.
Management DTO **0.11.0**, external Principal **0.1.5** and PocketIC **16.1.0** stay
unchanged. Relative to the incoming lock, only the four Host/Testkit entries change.

## Shared Tooling

The canonical exporter adopts committed **0.26.7** at
**`3e8d7cd3e7e9b7b79a5e800c30daf0be2710dab0`**, verified against remote main.
All **122 payloads** verify. Only the Cargo installer, tool-store helper and setup
guide change. Explicit selected-tool setup waits up to 60 seconds for an admitted
slot owner, with a canonical 0..300 override, then rechecks build/selection/source/
receipt inputs. It removes no owner lock or failed generation. Offline checks and
preflight remain immediate. Tool pins, build identity policy, release mechanics
and asynchronous CI cadence are unchanged. See [the review](../shared-tooling-review.json).

## Focused qualification

Linux Rust **1.99.1** passes strict Clippy, all-target/all-feature compilation and
warning-denied Rustdoc. Actual Rust **1.91.0** owning and independent normal-consumer
checks pass, including Host error identity probes. Both unpacked package archives
compile together with original selected identities and verified checksums. Normal
graphs exclude Testkit/PocketIC/WAT/Wire and development-only Host/Platform features;
Core also excludes Agent.

**203 selected runtime tests pass**: four artifact and 166 persistence units,
nine public artifact/recovery/workflow cases, six connected Core PocketIC cases,
14 Agent HTTP cases and four Agent PocketIC cases. The latter include both
independent cold-process acceptance/uncertainty journeys, complete same-ID round
trip and lost-stop refusal. Exact journals, original spending, byte evidence,
source references and effect-free terminal replay remain qualified at fixture scope.

Snapshot, dependency inheritance/boundaries, shell, consumer tooling/evidence and
hooks pass. Canonical tool setup and offline admission pass; the read-only release
preview selects 0.39.0 without effects. Evidence:
`/tmp/backup-host-shared-20261023-07cbuc2g/`, including source/lock comparisons, product and runtime results,
independent consumer metadata, archives and retained simulator fixtures. The
validated lock SHA-256 is `65a5ea1720954f4f869489597d559e0e00e03f776d51dd15830d316dbc76aef3`.

The complete local workspace test suite and make ci are not run. New hosted/native
macOS, registry-only published 0.39, installed-IC and Canic application acceptance
remain unqualified. [Released 0.38 CI](https://github.com/dragginzgame/ic-backup/actions/runs/39759606409)
passed Linux and both macOS architectures for its earlier Host 0.64 graph; that is
separate evidence. The prior [published handoff](https://github.com/dragginzgame/ic-backup/blob/43e65df388d6b6b8b3ba5b5ac5cab0788de67845/docs/status/current.md)
retains earlier adoption history.

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
