<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-backup/ic-backup-readme-header.svg" alt="IC Backup — Verified backups and safe recovery for Internet Computer apps" width="100%">
</p>

<!-- helper-navigation:start -->
<p align="center">
  <a href="https://github.com/dragginzgame/canic"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/canic.svg" width="18" height="18" alt=""> <strong>canic</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/icydb"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/icydb.svg" width="18" height="18" alt=""> <strong>icydb</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-timers"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-timers.svg" width="18" height="18" alt=""> <strong>ic-timers</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-memory"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-memory.svg" width="18" height="18" alt=""> <strong>ic-memory</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-query"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-query.svg" width="18" height="18" alt=""> <strong>ic-query</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-backup"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-backup.svg" width="18" height="18" alt=""> <strong>ic-backup</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-blob-storage"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-blob-storage.svg" width="18" height="18" alt=""> <strong>ic-blob-storage</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-testkit"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-testkit.svg" width="18" height="18" alt=""> <strong>ic-testkit</strong></a>
</p>
<!-- helper-navigation:end -->

`ic-backup` is being built to help application operators save verified copies of
an Internet Computer application's data on their own computer or server, then
safely restore the same canisters if something goes wrong.

On the Internet Computer, applications run in programs called **canisters**.
Canisters can hold both application code and important data. A canister snapshot
captures that state so it can be restored later. `ic-backup` is intended to keep
an independently verified recovery copy outside the running application.

> **Current status:** This is an early-stage Rust library, not yet a working
> backup application. It can securely prepare, record and verify local files and
> recovery plans, but it does not yet connect to the Internet Computer or perform
> a complete canister backup or restore.

## At a glance

| Question | Answer |
| --- | --- |
| What is it? | A planned tool for creating verified local copies of Internet Computer canister snapshots |
| Who is it for? | Teams responsible for operating an Internet Computer application |
| Where does it run? | On an operator-controlled computer or server, outside the canisters |
| What can it recover? | The same canisters using the same application release |
| Is it ready for backups today? | No. Local safety components exist, but the IC connection, runners and command-line application are not implemented |

## When might it be useful?

An Internet Computer application can consist of several related canisters, each
with its own state and responsibilities. Recovering them safely involves more
than downloading some files: the operator must know exactly what was captured,
keep related state consistent, verify every local copy and avoid repeating an
operation when its result is uncertain.

| It may be useful when… | It is not designed for… |
| --- | --- |
| An application stores important state in canisters | Backing up personal files from an ordinary computer |
| A team wants independently retained recovery copies | Copying an application to new canister identities |
| Several related canisters must be captured and restored consistently | Migrating state between different software releases |
| Interrupted operations must resume without blindly repeating actions | Replacing application-specific disaster-recovery planning |
| Operators need reviewable plans and verifiable local files | Providing a ready-to-use backup command today |

The initial recovery scope is deliberately narrow: restore the same application
release into the same existing canister identities. Creation of replacement
canisters, relocation, cross-release migration and automatic rollback are outside
that scope.

<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-backup/ic-backup-scope-guide.svg" alt="Decision guide for whether an application needs independent local recovery of the same Internet Computer canisters and software release" width="800">
</p>

## How it is intended to work

<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-backup/ic-backup-how-it-works.svg" alt="Backup workflow from selecting canisters and reviewing a plan through snapshot creation, download, verification, local retention and recovery of the same canisters" width="800">
</p>

### Creating a backup

1. Select the exact network, application release and canisters to protect.
2. Build a bounded backup plan for the operator to review before anything changes.
3. Check current authority and the application's consistency requirements.
4. Safely pause or fence the selected application components when required.
5. Create snapshots and download their exact files into private local staging.
6. Verify the downloaded files, then publish a durable backup bundle without
   overwriting conflicting evidence.
7. Record completion and retain any remote snapshots or local files still needed
   by unfinished work.

### Restoring a backup

1. Select and verify one complete backup bundle.
2. Confirm that it belongs to the same network, release and canister identities.
3. Build and review a separate restore plan; planning alone changes nothing.
4. Recheck current control and application safety before stopping or changing a
   canister.
5. Upload and load the exact retained snapshots in the application's required
   order.
6. Verify the restored state before allowing the application to resume normal
   operation.

If an operation is interrupted or a response is lost, the intended workflow
keeps its original plan and evidence. It inspects what actually happened instead
of assuming failure and blindly repeating a potentially expensive or destructive
operation.

<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-backup/ic-backup-interruption-recovery.svg" alt="Interruption recovery records exact intent, performs one operation and reconciles retained evidence instead of blindly repeating an operation after a lost reply" width="800">
</p>

## Safety boundaries

<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-backup/ic-backup-responsibility-boundary.svg" alt="Responsibility boundary between the local IC Backup workflow, application-specific safety decisions and Internet Computer platform operations" width="800">
</p>

`ic-backup` is intended to own the generic backup machinery:

- exact, reviewable operation plans;
- local journals that survive interruption;
- checksums and verified backup directories;
- bounded attempts and retained uncertain outcomes;
- safe local staging, publication and retention; and
- request and reply formats for supported Internet Computer operations.

The application integration still owns information that a generic backup library
cannot safely guess:

- which canisters belong to the application;
- who is currently allowed to control and inspect them;
- how the application is paused or fenced consistently;
- the correct order for application-specific operations;
- whether restored application data is valid; and
- how external payments and other effects are reconciled.

A saved plan is not current permission to act, and a downloaded file is not by
itself proof of a complete backup. Authority, application consistency and the
actual file contents must be checked at their appropriate boundaries.

## What is implemented today

The repository currently contains one Rust library at `crates/ic-backup`. Its
implemented foundation includes:

- checksums, secure staging and durable publication of verified local files;
- bounded local records, journal locking and interruption-safe state transitions;
- retention of backup files required by unfinished restores;
- command custody that prevents cooperating processes from assuming an external
  operation has stopped;
- exact canister inventories, target selection and dependency graphs;
- immutable operation plans with bounded mutation and observation attempts;
- local progress derived from the original plan and retained journals;
- typed Internet Computer request and reply encoding for selected snapshot and
  lifecycle operations;
- original-attempt-bound snapshot metadata/data read requests, a single-call provider
  contract and bounded passive response checks, without automatic receipts or retries;
- bounded snapshot metadata decoding that retains exact global values, optional
  timer/hook values and chunk identities without claiming complete transfer;
- metadata-bound snapshot data reads with checked ranges, exact lengths and chunk
  hashes, plus bounded incremental coverage checks that reject missing or repeated
  data without claiming durable transfer;
- local streaming of admitted snapshot bytes into private verified artifacts, with
  durable publication, local interruption recovery through the original journal,
  and explicit retained metadata/extent/chunk verification;
- source-bound snapshot upload metadata and bounded byte preparation, with a single-update
  provider contract and passive replies under separate pending attempts; live upload is pending;
- per-guard host timing summaries and bounded prepared chunk-size distributions using `ic-metrics`, with
  diagnostics kept separate from retained progress, spending and completion evidence;
- contracts for exact originally reserved IC updates and bounded passive reply
  association, preserving pending spending without automatic settlement;
- exact reserved status/snapshot-list observation contracts that retain both attempt
  identities and leave lost observations pending without automatic outcomes;
- fresh verification of durable local backup files against their original plan;
- retained same-release source/safety requirements and application evidence checks
  before snapshot load or controlled start;
- retained original application fence obligations and exact acquisition-journal
  recovery, exact application update envelopes and reserved reconciliation checks
  across interruption;
- immutable original execution settlement checkpoints with exact journal histories
  for local replay;
- immutable manifests of the exact verified local download set, with local record
  replay that preserves original snapshot/checksum evidence;
- fresh local restore-source verification under the original requirement, exact
  manifest and existing selected canister IDs; and
- private operation-bound restore artifact copies, explicit durable publication
  and retained-copy verification that preserve partial evidence and original obligations.

These components have native regression evidence for their local filesystem,
record and process behavior. They do not establish real Internet Computer backup
or restore behavior.

The following product components are still missing:

- capture and restore runners;
- application-qualified transport/provider integration;
- the standalone command-line application;
- application-specific membership, authority and consistency adapters; and
- complete product and application recovery qualification on real IC backends.

## Project boundaries

The implementation began by extracting generic backup mechanisms from Canic's
`canic-backup` crate. Generic snapshot, artifact, planning and recovery behavior
belongs here. Canic continues to own its Fleet discovery, authorization and
application-consistency integration. Other applications can provide their own
adapters without depending on Canic.

The package layout is:

| Package | Intended responsibility | Status |
| --- | --- | --- |
| `ic-backup` | Generic planning, records, policy and local backup mechanisms | Implemented in part |
| `ic-backup-agent` | Direct single-update Rust IC transport | Implemented; isolated local qualification |
| `ic-backup-cli` | Standalone `ic-backup` command | Design name; not yet created |

Package names and availability must be checked before publication. New packages
will be added only when their implementation is required.

## Development

Development uses Rust 1.99.0, edition 2024 and a minimum supported Rust version
of 1.91.0. The workspace owns package metadata, dependency declarations and
lints. Builds stay in this repository's `target/` directory.

```bash
cargo install cargo-sort --version 2.1.4 --locked
make install-hooks
make check
make test
```

`make help` lists the available development and release commands. CI runs the
native library and repository-tooling gate. A real PocketIC integration target
qualifies single-canister capture, complete streamed transfer and same-ID restore,
including deliberately lost replies. See [its exact scope](docs/pocketic-qualification.md).
Direct Agent transport has separate [HTTP/gateway qualification](docs/agent-transport.md).
Complete workflows and application qualification remain unfinished.

See [contribution guidance](CONTRIBUTING.md) for topic branches, PR delivery and
the separately authorized release workflow.

## Documentation

| Document | Use it for |
| --- | --- |
| [Documentation index](docs/README.md) | Choose the right product, development, evidence or governance document |
| [Current handoff](docs/status/current.md) | Current implementation state, evidence and next work |
| [Extraction and implementation design](docs/extraction-design.md) | Complete proposed product behavior and safety design |
| [Implemented extraction boundary](docs/extraction-boundary.md) | Exact behavior currently provided by the library |
| [Development commands](docs/development.md) | Local development and validation workflow |
| [Release workflow](docs/releasing.md) | Maintainer release process |
| [Agent and contributor instructions](AGENTS.md) | Repository-specific engineering and safety rules |
| [Source baseline and provenance](docs/source-baseline.json) | Original Canic source baseline |
| [Fresh extraction provenance](docs/extraction-source.json) | Source and destination identities for the first extraction |
| [Repository tooling provenance](docs/tooling-provenance.json) | Reviewed shared tooling inputs |

## License

MIT licensed, with Canic and sibling-tooling contributor attribution retained.
