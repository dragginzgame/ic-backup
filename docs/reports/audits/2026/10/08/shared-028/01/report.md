# Shared Tooling 0.1.28 adoption and MSRV qualification

Reviewed committed source is `1872ed2c20f6c70689bb2249050b1d673c60bfa0`.
The canonical exporter preserves the incoming uncommitted Testkit work, refreshes
17 existing paths and explicitly adds 15 catalog/runner-fixture paths. All 94
snapshot bytes/modes match [official source proof](source-proof.json). The one
consumer-owned PocketIC 16.1 matrix remains outside the immutable snapshot and
unchanged. The refresh preserves the original incoming dependency graph. A later
external update selects Host 0.8.2; preserve that lock and qualify it freshly.
Only Host's private macOS cleanup fixture changes from 0.8.1; production Rust
remains identical. [Published source proof](published-source-proof.json) binds
all six selected Testkit/Host/Metrics packages to exact official source/archives.
No vendored patch, sibling edit or root Git write occurs.

New common rules require std paths for std-dependent code and an actual qualified
low MSRV independent of the development compiler. All maintained Rust already
uses std paths; no function, method or type is removed. The selected Host and
Agent normal dependencies declare Rust 1.88, so an isolated candidate lowers only
its copied root declaration, retaining exact source and lock. Both libraries pass
actual Rust 1.88 all-target/all-feature compilation before changing the real
inherited support claim from 1.91 to 1.88. Development remains Rust 1.99.

`make check-msrv` explicitly records rustc/Cargo versions, compiles both libraries
and invokes two retained independent consumers. Each standalone fixture copies
the original lock, prepares only its own metadata offline, admits every selected
package/version/source against the original graph, excludes Testkit/PocketIC and
compiles with the same actual minimum compiler under --offline --locked. The
normal Host feature sets are empty; no workspace dev-feature unification supplies
support proof. CI installs/selects 1.88 on all declared native hosts. Current-source
native macOS execution remains pending; a Linux build does not qualify it.

The first candidate omitted six JSON schema fixtures used by include_str and
failed before complete all-target qualification. Its corrected retained copy
includes those original contracts and passes. The first consumer preparation used
generate-lockfile, which reselected cached compatible versions; admission refused
that graph before compilation, leaving the root lock untouched. Preparation now
uses seeded metadata to preserve the original selections, followed by locked
checks. ShellCheck also rejected the inline trap's status assignment; a normal
local-status exit handler fixes that diagnostic. All original logs remain retained
separately from corrected results in [qualification](qualification.json).

New installer cases qualify malformed literal active-link refusal before commands
or downloads. Full local tooling passes, including both synthetic installer suites,
closed governance links and the new maintenance runner through a substitute CLI.
The catalog is accessible through make tasks/README; no actual agent or scheduler
runs. Shared release fixtures are now simulation-only; consumer release adapters
retain their existing private-index checks and do not create commits or publish.
The separately selected real-Git tracking suite stays with Shared Tooling's owner.
No complete CI or release gate is invoked by this adoption.

The prior Testkit 0.25.1 / Host 0.8.1 / Metrics 0.2.14 runtime checks retain their
original Rust source, selected graph, compiler and assertions. Their package
archives are copied before refreshing archives for the new MSRV metadata; exact
checksums and paths are retained in qualification. They are not relabelled as
Rust 1.88 runtime tests. New focused MSRV/tooling/package evidence owns this batch. The Host 0.8.1
stage's inputs, logs and archives remain under
`target/shared-028-review/qualified-host-081`; final Host 0.8.2 receives fresh
MSRV, independent-consumer, declaration, Clippy and package qualification. Both
current archives advertise 1.88 and retain all exact Rust/WAT bytes. Earlier
1.91 archives remain retained; no runtime simulator rerun is claimed for this
production-identical patch. Actionlint and the read-only task command also pass.

Exact Shared 0.1.28 CI was queued at inspection; the previous db039347 correction
now has passing owner CI. Neither establishes current native consumer acceptance.
The draft remains 0.9.0 for the existing Host public type-identity cut; package
versions and release receipt remain 0.8.1. Full workflow/stage binding, Canic
application adapters and terminal/fence/reference release remain separate.
