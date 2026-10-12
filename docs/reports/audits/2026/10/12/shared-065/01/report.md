# Shared Tooling 0.6.5 and Host 0.14.3 adoption

Preserve the pending Metrics 0.7 work and unchanged Backup package versions 0.19.2.
Released base is `2ebf8a2e6ea25061bc9a04a2ecca28ea5b910e54`. Select the same
pending 0.20.0 notes because Metrics changes public summary/aggregate widths;
this compatible tooling/Host patch adds no further public cut or stored-record
conversion.

Canonically export exact Shared Tooling 0.6.5
`42f2091768b5f71c98a684d3dab563f408ca2dd9` from the clean reviewed source.
Nine selected files change; the roster stays at 102. Local `make ci` opts in the
root Cargo.toml/Cargo.lock at its existing stage boundaries. Shared checkpoint
hashing/error/retention owns drift detection, including nested same-checkout gates.
It detects persistent changes at checkpoints rather than arbitrary edits within
a running stage. Failure retains current source and original/observed identities;
restart requires preparation and a complete current gate, without automatic updates.
The release fixture now exports the checksum helper with its runner. Original
resume receipts, product setup/order and spending/recovery owners remain intact.

New consumer tests copy the actual Makefile/includes and append unique fixture
mutation/next-stage targets. Both declared input changes refuse later dispatch and
retain actual failure logs without recipe overrides or Rust/IC effects. Focused
consumer tooling and hook/index checks pass on Bash 5 and genuine Linux Bash 3.2.57.
This does not qualify native macOS behavior. Shared staged-format receipt context
is available; no stronger local receipt prerequisite or setup is silently added.
No symbols or public targets are retired.

All four Host crates select registry 0.14.3. Official API requests returned HTTP
403; Cargo's registry resolution/fetch succeeds. Every archive matches its locked
checksum and all published source files match committed producer source. Compared
with 0.14.2, Rust files are unchanged and normalized manifests differ only by package
version. The producer release adopts Shared 0.6.5. Restore incidental Windows edge
changes from targeted Cargo resolution, preserving every non-Host package selection
and dependency edge. There is no new offload opportunity, publication-error identity
cut, command custody or artifact/journal change.

Detailed source/graph/setup/check evidence stays under ignored `target/shared-065/`.
Every registered full `make ci` target passes with the canonical input checkpoints
enabled, on unchanged final technical/graph inputs. Strict Clippy, both actual
simulator suites, Rust 1.88 independent consumers and paired archives pass.
[Compact qualification evidence](qualification.json) binds registered membership
and retained logs to actual source/graph inputs. The real index/HEAD and incoming
manifest remain unchanged; the final selected lock remains unchanged across the gate. No Git/index,
version/release effect, production IC operation, sibling edit or evidence cleanup
is performed. Common tool pins/build identities stay unchanged; no reinstall is
needed. Selected Testkit remains its explicitly prepared 0.34.1 CLI/server.

Released Backup 0.19.2 passes Linux and Apple Silicon; Intel macOS was cancelled
at final inspection. Shared 0.6.5 passes lint/Linux with both macOS jobs cancelled.
Host 0.14.3 passes Linux/MSRV with both native macOS jobs queued.
These are separate exact-source results; uncommitted adoption has no hosted/native
macOS acceptance. Earlier Metrics/inspection reports retain their original scopes.
Coordination: [Backup #36](https://github.com/dragginzgame/ic-backup/issues/36),
[Backup #40](https://github.com/dragginzgame/ic-backup/issues/40),
[Shared #129](https://github.com/dragginzgame/shared-tooling/issues/129),
[Shared #130](https://github.com/dragginzgame/shared-tooling/issues/130).
