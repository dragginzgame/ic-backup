# Publication of the direct-controller library

Backup's first registry release is an experimental Unix library for the scope
described in the [packaged README](../../crates/ic-backup/README.md). Canic manager
execution and application maintenance/fence adapters remain unfinished. This
workflow neither enables them nor migrates legacy stores or release state.

The maintainer authorized the experimental 0.102.1 release on 2026-11-16. Only
the root product workspace admits `crates-io`; both qualification packages remain
`publish = false`. Later uploads still require release authority. Select the release version and
update the product manifest, both locks and qualification's Backup requirement
together, preserve external supplier selections, and finalize the corresponding
changelog entry. Commit and deliver that exact source under Git authorization.
No version bump, Git commit/tag/push or registry mutation is hidden in these
publication commands. Automated version/tag release orchestration is not installed.

## Preparation and checks

```bash
make install-tools
make fetch-dependencies
make package-check
make test-publication-adapter
make publication-verify
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
verification and the publication-caller regression. Its recipe is sequential
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
the installed Toko 0.44.5 publication adapter, the fixed `ic-backup` roster and
`crates-io` registry, exact versions from locked metadata and the complete local
gate. Unknown, duplicate, empty or private package selections refuse. Only a
command-line `PACKAGE=ic-backup` may narrow the roster; qualification packages
can never be selected. Cargo registry access is needed even for its dry run;
an explicit offline request refuses if those observations are unavailable.

Use `publish-check` as a preflight when needed; it does not issue a reusable
release-validation receipt. A later `publish` repeats the complete gate. Do not
silently skip that gate based on old logs. A single authorized publication can
run `publish` directly after source preparation; Cargo still packages and verifies
before dispatch. Future automated version/tag orchestration or validation reuse
must adopt the shared release contract with both workspace locks, rather than
copying the legacy runner or adding a second local release journal.

The adapter records selection, source commit, exact arguments, dispatch intent,
Cargo output and separate Cargo/logger statuses under `target/publication/`.
Upload requires clean source; source identity and status are rechecked before
dispatch. Caller credentials and registry configuration stay outside the repo.
Publication does not automatically create tags, push Git or enable eligibility.

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
