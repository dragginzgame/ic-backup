# Shared Tooling 0.6.3 adoption

Review committed `4fd979d13c32adb3cb2f51f213ebfc4f98339b3d` from a clean isolated
checkout and refresh through the canonical exporter. Eight selected files change;
the 99-file roster stays intact.

Normal direct release retries archive a saved prepare intent only for clean
descendant source with unchanged destination, absent staging evidence and current
metadata equal to the old base or candidate. Restart selects the requested
increment from current metadata and repeats preflight/full validation. Exact resume
stays bound to its original identity. Backup preparation mutates local metadata
only, retains backups and rolls back ordinary failures. No real release runs here.

Adoption reconciliation distinguishes exact delivered commits and required native
checks from upstream producer qualification. Pending/cancelled checks remain open.

Preserve the incoming direct Host Artifact/FS 0.14.0 graph. Published Rust sources
match 0.13.5; public publication-error identity still changes, selecting a 0.19.0
draft. Package versions remain 0.18.4. Testkit 0.33.7 retains separate dev-only
Host 0.13.5; Metrics 0.6.4, Agent 0.49.2 and PocketIC 16.1.0 remain selected.

All 16 delivery targets pass on unchanged technical/graph inputs, including real
simulator journeys, strict Clippy, independent Rust 1.88 consumers and paired
archives. An external Rust 1.88 consumer constructs/extracts the public Host 0.14
publication error without dev unification. Root manifest/lock, index and HEAD stay
unchanged. Local adapter tests use actual Backup Make/metadata owners with Git/
Cargo substitutes; generic runner cases remain canonical. No live IC or real release
is executed.

Released 0.18.4 exact-source CI passes Linux; both macOS jobs are queued. Upstream
0.6.3 Linux/lint pass while macOS remains pending. Keep native acceptance distinct
from this local candidate. Detailed evidence stays under `target/shared-063`;
see [the compact proof](qualification.json). Work remains uncommitted.
