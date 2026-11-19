# Publication of the direct-controller library

Backup's first registry release is an experimental Unix library for the scope
described in the [packaged README](../../crates/ic-backup/README.md). Canic manager
execution and application maintenance/fence adapters remain unfinished. This
workflow neither enables them nor migrates legacy stores or release state.

The maintainer authorized the experimental 0.102.1 release on 2026-11-16. Only
the root product workspace admits `crates-io`; both qualification packages remain
`publish = false`. Toko’s [scope and authorization](../../TOKO-TOOLING.md#scope-and-authorization)
governs delivery. For Backup, the publishable group contains only `ic-backup`.
Select the release version and
update the product manifest, both locks and qualification's Backup requirement
together, preserve external supplier selections, and finalize the corresponding
changelog entry. Commit and deliver that exact source under Git authorization.
No version bump, Git commit/tag/push or registry mutation is hidden in these
publication commands. Automated version/tag release orchestration is not installed.

## Preparation and checks

```bash
make install-tools
make publish
```

`package-check` is available before publication eligibility is enabled. It packages
and compiles the actual archive, compares its README, admits the license, then
builds the existing consumer source against the extracted archive with normal
dependencies. Its generated unpublished manifest projects the qualification
catalog; the isolated lock starts from the qualification lock. Testkit/PocketIC
must not enter that consumer's normal graph. Each archive and command log is
retained separately under `target/package-consumer/`; neither maintained lock
is edited. This working-source check permits dirty packaging and does not prove
clean Git provenance or authorize upload.

`publication-verify` runs offline tool admission, strict Clippy for both workspaces
before native checks, all actual-IC journeys (including 20 MiB), package
verification. Its recipe is sequential
under parallel Make. This is the complete local gate; it does not wait for hosted
CI and does not establish actual-IC qualification on macOS. IC journeys use the
optimized release profile; see [capacity evidence](../architecture/capture-transfer.md).

## One selected publication attempt

After eligibility and exact committed source have been authorized:

```bash
make publish-check
make publish
```

`publish-check` invokes Cargo's real `--dry-run`; `publish` can upload. Both use
the installed Toko 0.45.15 publication adapter, the fixed `ic-backup` roster and
`crates-io` registry and exact versions from locked metadata. Unknown, duplicate,
empty or private package selections refuse. Only a
command-line `PACKAGE=ic-backup` may narrow the roster; qualification packages
can never be selected. Cargo registry access is needed even for its dry run;
an explicit offline request refuses if those observations are unavailable.

Use `publish` directly for the normal one-pass delivery. Optional `publish-check`
runs `publication-preflight`: offline host/Rust tool admission and product Clippy,
then Cargo's fresh archive compilation and registry dry run. It does not run
native tests, installed IC journeys or qualification-workspace lint. A dry-run
success establishes archive/registry preflight, not complete product qualification.
`publish` independently runs the complete `publication-verify` gate once, then
Cargo's fresh archive verification and upload. Thus check-plus-upload performs one
full qualification, rather than two. The target-specific selection is consumer
policy; shared selection, dispatch, exclusion and attempt recovery are unchanged.

No validation receipt or skip switch is introduced. Missing or tampered prior
dry-run records cannot authorize omitting upload's gate. Product lint runs again
at upload because the installed shared adapter has no qualified preflight-to-upload
proof handoff; prior output is not a reusable stage proof. `publication-verify`
remains an explicit full readiness check, so running it separately before
`publish` will repeat that work. Use the optional narrow preflight instead.
Future full-gate reuse must qualify shared evidence mechanics against both locks,
tools, server, environment and complete input inventory; never infer reuse from
old logs or implement a private proof framework.

The independent normal consumer in `package-check` proves extracted-package
interoperability and excludes Testkit/PocketIC. Cargo publication separately
admits the final archive, clean source and registry destination. These guarantees
remain separate; neither compilation substitutes for the other.

When release timing is enabled, the Cargo publication span records the exact
selected package and version. The adapter records selection, source commit, exact arguments, dispatch intent,
Cargo output and separate Cargo/logger statuses under `target/publication/`.
Upload requires clean source; source identity and status are rechecked before
dispatch. Caller credentials and registry configuration stay outside the repo.
Publication does not automatically create tags, push Git or enable eligibility.

## Check ownership and triggers

| Flow | Owning command / scope | Trigger and guarantee |
| --- | --- | --- |
| Development | `make check [PACKAGE=ic-backup]` | Root documentation, format, lint, Rustdoc and native tests; use focused commands for ordinary edits. |
| Changed publication tooling | `make test-publication-adapter` | Real Make/shared-adapter wiring with substituted costly validation and registry effects; run when this caller changes and in Linux scaffold CI. |
| Native CI | Product Clippy/Rustdoc/tests and qualification Clippy per configured host; actual-IC target on Linux | Host-specific evidence, asynchronously; no local gate waits for this matrix. |
| Optional preflight | `make publish-check [PACKAGE=ic-backup]` | Fixed product roster, lint, fresh archive/registry dry run; no runtime qualification. |
| Release/publication | `make publish [PACKAGE=ic-backup]` | Complete product/qualification gate once, exact archive/normal consumer and one fresh Cargo upload attempt. Version preparation and Git delivery remain explicit. |
| Explicit readiness | `make publication-verify` | Full gate without upload; no reusable admission is claimed. |
| Uncertain upload | Existing attempt evidence plus remote/registry readback | Reconcile the original dispatch first; a network failure does not trigger runtime tests or an automatic upload retry. |

The one publishable product is the complete selected local release scope. Its
separate qualification graph is the required reverse consumer, not an additional
publishable group. Product selection reaches native Cargo leaves; both
qualification packages remain private. Generic tooling regression matrices remain
upstream. The local publication-caller fixture is now owned by scaffold CI and
changed-tooling qualification, not every publication.

## Failure and recovery

Retain all publication attempt directories and locks. A nonzero status, timeout
or missing completion record does not prove the version was absent from crates.io.
Inspect the exact attempt, independently check the registry version/archive and
its source identity, and reconcile before selecting a fresh attempt. Never retry
an upload blindly, delete a surviving lock to proceed, change a published version
or overwrite legacy release/recovery records. This one-package workflow has no
partial multi-package batch or resume command. Registry readback and a fresh
registry consumer remain release-execution evidence, not pre-publication claims.

## Review provenance

Reviewed legacy `4ae087fad8c23de712aa60bf569bde4b2b932651`:
`scripts/release/{release.sh,test-publication.sh}`, its group policy and standalone
consumer guidance. History `0d4671ed9bf7a5e217e06a67e3fba490871daff4` consolidates
qualification; `87f2a7869075bc76ac25783f9c602128848d12d2` adopts vendor ownership.
The retained guarantees are exact package/version admission, original source,
failure propagation, one dispatch and preserved uncertain evidence. The restart
has one product package and no legacy release receipts to interpret. It reuses
the shared publication adapter and existing consumer instead of importing the
legacy two-package release framework. The caller fixture substitutes validation
cost and registry dispatch; actual packaging/dry-run checks are separate evidence.

### One-pass flow qualification — 2026-11-17

For Backup #3, reread the frozen legacy publication caller and tests at
`4ae087fad8c23de712aa60bf569bde4b2b932651`, including history
`87f2a7869075bc76ac25783f9c602128848d12d2` and
`1af7e219c3073395992c316ecaa5e83e74b1aecc`. Exact package/version admission,
early lint failure, source drift refusal and preserved single dispatch remain.
Legacy multi-group release receipts are not needed for this one-product flow.
No legacy source or state is imported.

The changed assumption is that optional dry-run establishes archive/registry
readiness, not a second complete product qualification. Upload retains every
product/native/IC and independent archive-consumer obligation. Only its recurring
publication-caller regression is retired from the release gate and moved to CI.
No shared input-proof mechanism is changed or implemented locally. Installed
Toko Tooling 0.44.11 is retained; unpublished 0.44.12 policy is not claimed.

The disposable caller runs actual Make/shared-adapter dispatch under parallel
Make with substituted gate cost and Cargo effects. It checks private/invalid
selections, fixed registry/roster/gate policy, missing/tampered prior check records,
failed preflight and upload gates, source drift, retained failed-command status,
single dispatch and surviving exclusion locks. Check-plus-upload must record one
preflight, one complete qualification and two fresh Cargo calls (dry-run/upload).
Only successful command exits can proceed; the check record never skips work.
Failed or uncertain uploads still require destination reconciliation before a new
attempt; these fixtures simulate effects and do not establish live recovery.

Timings in caller output measure warm locked-metadata/admission and substituted
effects, not product/IC runtime or network delivery. No cold run or end-to-end
speed improvement is claimed. Source review proves the full-gate count changes
from two to one for check-plus-upload; timing that count alone would not establish
actual latency savings. Actual archive/consumer and IC evidence remains owned by
the unchanged product tests and release gate. The handoff records current checks.
