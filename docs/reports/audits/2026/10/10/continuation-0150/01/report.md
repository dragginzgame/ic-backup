# Async mutation integration and Shared 0.3.7 qualification

Released base is 0.14.2 at `c157d43e542dc27dc5f7781850c7b8bb169b5a0c`.
The uncommitted 0.15.0 draft hard-cuts the public mutation port and capture/load/start
coordinators to futures, with async admission/qualification and the currently
guarded original journal. Consumers must await them and migrate their callbacks;
package versions and release receipt remain 0.14.2.

The configured `AgentMutationProvider` reuses canonical request/reservation admission,
requires durable exact signed-envelope/request-ID retention, then consumes one
existing transport submission. Retention refusal sends nothing; accepted/lost
responses remain indeterminate and pending, without observation, retry or automatic
receipt. Async admission and submission retain selected journal exclusion.
Cancellation before/after dispatch releases that lock, retains spending and refuses
reentry; restore reply retention must precede cancellable qualification work.
No core runtime/Send bound, compatibility facade, persisted schema or allowance owner
is added. Other provider families remain synchronous and full runners/application
custody/terminal/fence/reference release remain independent work in #25/#29.

Canonically export committed Shared Tooling 0.3.7 at
`34e5ad7aac3599306c9572bb547f2239d09df1a3` from an isolated clean checkout.
The 96-file roster remains unchanged. The baseline now requires workflow/ref
concurrency with cancellation of older routine checks; Backup already preserves
all three hosts under that group. Cancelled jobs qualify no source. Upstream fixture
logger isolation/completion and failed Git tree observation refusal are adopted
byte-for-byte. Its Cargo installer owns exact registry selection and drift admission
from `--lockfile Cargo.lock`; Backup removes local metadata/JQ package discovery.
Consumer fixtures retain ordered Make routing, offline refusal and jobserver checks.
Shared and local mandatory assertions explicitly fail under Bash 3.2. A contradicted
actual Testkit routing result rejects and retains original inputs; the actual CI
collector round-trips its exact command logs and lockfile. Routing and package
fixtures honor CI's retained temporary root instead of leaving diagnostic logs
outside the selected archive paths.

Binaryen 133 selects all three committed official archive hashes. Explicit setup
retains prior bundles. Both actual simulator suites retain original/optimized Wasm
and optimizer identity, then run the optimized code with their original state,
transfer, same-ID restore and lost-reply checks. This is fixture qualification,
not a production optimizer policy or blanket arbitrary application guarantee.

Preserve incoming Host 0.12.6, Testkit 0.33.0 and Metrics 0.5.4. Host FS rejects NUL
paths before parent creation/producer invocation; published Artifact and Metrics
Rust trees remain byte-identical to 0.12.4 and 0.5.3 respectively. Testkit recognizes
scoped HTTP connection resets for dead instance recovery classification; Backup
uses managed server ownership without those recovery recipes or automatic rebuilds.
All four Host 0.12.6 Rust source trees match 0.12.5, with committed archive identity
`5f356effea97fbc31dfcca5b9f1b2834f35325a9`. Another incoming lock update during
the preceding suite selected uncached 0.12.6 and stopped offline checking. Preserve
that failure separately, fetch the unchanged selected graph, and qualify it afresh.
Explicit setup prepares/admit the exact 0.33 CLI and existing PocketIC 16.1 server;
older tools/receipts/server bytes and all failed evidence remain retained.

Focused checks pass all four Agent and 29 core simulator cases on this graph,
including actual async admission, durable signed retention and passive pending
capture. An independent locked Rust 1.88 consumer compiles both public async
coordinators with the Agent provider, on an exact normal graph subset without
Testkit/PocketIC or Metrics features. The prior full suite passed its first fifteen gates;
ordinary Cargo package verification then selected published same-version core and
rejects the Agent's new trait implementation. Explicit registry selection reproduces
that failure. Cargo's [package contract](https://doc.rust-lang.org/cargo/commands/cargo-package.html)
normalizes path dependencies to registry dependencies; the observed same-version
fallback is retained as consumer evidence, not an authority to bump root versions.

The corrected local package owner lets Cargo admit/create both archives, then
mandatorily builds their exact unpacked pair in a retained independent consumer.
A fixture-only core override excludes published/live/sibling fallback. Strict
metadata checks preserve original dependency selections and exact archive paths,
exclude simulator packages, then build offline/locked. Both archive and 327 unpacked
file hashes remain unchanged, as do root manifest/lock. Ordinary registry publication
retains Cargo admission. Reusable guidance feedback is
[Shared #109](https://github.com/dragginzgame/shared-tooling/issues/109).
The complete 16-target `make ci` passes on Shared 0.3.7 and the fixed final graph:
504 core unit cases, 29 core simulator journeys, nine Agent HTTP cases, four Agent
simulator journeys, strict Clippy/docs, Rust 1.88 and both current archives. All
technical inputs remain byte/mode-identical throughout that run. The subsequent
two-file routing-retention adjustment passes affected shell/tooling checks and
actual Bash 3.2 routing/negative-retention checks on unchanged inputs; all other
passing evidence is reused. Retained evidence lives under
`target/continuation-0143/`. Initial compiler/lint/CLI-selection, release-substitute,
cancellation fixture-path and package failures remain separate evidence.
Exact inputs, archive/tool identities and results are in
[the machine evidence](qualification.json).

Released 0.14.2 has one [native CI run](https://github.com/dragginzgame/ic-backup/actions/runs/38057022179):
Linux succeeds; both macOS jobs failed in system-Bash tooling immediately as the
Testkit routing check began. Downloaded artifacts omit its fixture directory, so
the exact failed command remains a diagnosis gap rather than a proven source cause.
The draft fixes that evidence selection and passes local Bash 3.2 checks; matching
native macOS acceptance remains required under #32/#33/#36. This uncommitted draft
has no hosted result and supplies no native macOS proof. No root Git write, release, package
upload, production IC effect, sibling edit or recovery cleanup ran.
