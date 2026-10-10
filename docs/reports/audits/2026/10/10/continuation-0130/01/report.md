# Host 0.11 and Shared Tooling 0.2.14 adoption

Released base: IC Backup 0.12.1 at `f5b0b093dd10ec6b25934e80ceaadb4f32d7506e`.
Select the whole pending draft as 0.13.0 because direct registry Artifact/FS
0.11.0 changes the public `PersistenceError::Publication` Rust error identity.
Consumers sharing publication errors must align Host 0.11. Package versions and
release receipt remain 0.12.1. Initial manifest/lock bytes remain unchanged through
the first full gate; the subsequent incoming Testkit update is preserved exactly
through final qualification.

Published Host source is `1d768c80a5bb87e3330a6b7bacfdfc543063968f`. Both cached
archive SHA-256 values match the selected lock and published VCS evidence.
Artifact/FS API shapes used here are retained; durable error Display now includes
secondary staging cleanup failures while preserving primary source identity. The
new Process limits/cleanup API has no production caller here. Normal consumers
link Artifact/FS alone, with optional archive/Wasm disabled. Initial Testkit 0.28.1
retains its separate Host 0.10.2 dev graph. After that full gate finished, an incoming
update selects Testkit 0.29.0 from published `e15cc2acfd9324f6877f854415005f91d169a031`,
unifying all four Host crates at 0.11.0. Explicitly prepare/admit its exact CLI;
preserve the prior selection and qualify this final graph separately. No sibling patch or process
dependency is introduced. Original v1 records, spending, private permissions and
synchronized publication barriers stay with their existing owners.

The canonical exporter adopts the same 95 selected files from Shared Tooling
0.2.14, committed `fd11692f31e7dfd44dcc2ca56634eaeab3569825`. The optional upstream
CI observer remains unselected; its missing-failure-log rule is adopted. The
repository description remains accurate about unfinished full runners.

Original release preflight fetches locked sources, prepares the selected Testkit
CLI, checks it offline, then rechecks source/manifest/lock/receipt. Native
qualification targets depend on offline admission even under parallel Make.
Prepared/committed/tagged recovery does not replay setup. Consumer fixtures prove
N-installed/N+1-selected preparation before gate, one installation across retry,
offline/network refusal, invalid/locked selection refusal, saved recovery skip and
parallel refusal before compiler/test dispatch. Installer substitutes prove
consumer ordering; canonical Shared tests own actual receipt/lock integrity.
Existing tools and failed fixtures are retained.

Static checks, focused consumer fixtures and the full unmodified `make ci` roster
pass on the initial Host 0.11/Testkit 0.28.1 graph. Preserve its exact original
lock separately. Every graph-dependent target passes again on final Testkit 0.29.0:
pins/links/locked fetch, selected CLI/server admission, formatting, native check,
strict both-library Clippy, all tests/doctests, warning-denied docs, MSRV and both
packages. Reuse unchanged passing snapshot/tools/shell/tooling/release/hook checks
from the first full gate. This is complete delivery-suite evidence with a separately
qualified final graph, rather than relabeled initial proof.

The core has 492 unit cases, 22 actual simulator journeys and remaining public
integration cases; Agent has eight HTTP-boundary and three actual simulator cases.
Both independent Rust 1.88 consumers pass without simulator/process/tool packages.
A separate Rust 1.88 consumer directly shares Host FS 0.11 publication errors,
verifying original primary source identity and secondary cleanup error display.
It performs no filesystem publication. Final normal features retain only Artifact/FS
with optional archive/Wasm disabled. Testkit 0.29.0 CLI receipt/bytes and offline
server admission pass, with the prior CLI/receipt bytes unchanged.

[qualification.json](qualification.json) binds technical input hashes, exact
package selections, logs and native status. Final source/lock remain unchanged
through qualification. Documentation-only evidence completion receives link and
whitespace checks afterward.
Logs and exact graph are retained under `target/host-shared-review/`. The initial
new fixture wrongly assumed one preflight per successful release; the runner also
rechecks original-source preflight before version preparation. The corrected
fixture proves one installation and preserves those repeated admission checks.
Its failed evidence remains separately retained.

The previous allocation review remains historical proof of its original Host
0.10.2 graph; it is not relabeled as Host 0.11 proof. Current released-source CI
has Linux/Apple Silicon success; Intel ended cancelled in
[run 38037385719](https://github.com/dragginzgame/ic-backup/actions/runs/38037385719).
The uncommitted candidate has no hosted result. No root Git write, release, registry
publication, production IC effect, sibling edit or recovery cleanup is authorized.
