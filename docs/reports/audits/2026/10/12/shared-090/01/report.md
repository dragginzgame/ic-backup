# Shared Tooling 0.9.0 consumer adoption

The uncommitted Backup 0.20.3 draft selects clean committed Shared Tooling
0.9.0 at `aec7a149f581a42b9531fc747b46024fc0314fcf`. The local source
checkout records that commit as `origin/main`, and its committed `VERSION` and
changelog identify 0.9.0. An independent remote-tip check was attempted but
GitHub DNS did not resolve; this adoption does not claim fresh remote proof.

The canonical preview and export preserve all 105 selected files. Four selected
paths change: the snapshot consumption guide and verifier, plus the local LOC
helper and its fixture. No payload is added or retired. The v1 manifest now
records the exact source revision and has SHA-256
`9eecc75844aa440fdbcf621accabd7e876ee1f065e3c389ed1150116b1af0f31`.
The new verifier reads an entire manifest once, rejects NUL bytes before
splitting records, and can emit validated records with `--read-manifest` without
claiming payload integrity. The upstream refresh and fleet reporter use that
reader; the reporter itself is not a selected consumer tool. The selected
`cloc.sh` gains complete numeric JSON output while retaining its existing text
report and failure behavior.

`make tooling-check` passes, including exact snapshot bytes/modes, hostile
consumer fixtures, the new LOC JSON cases and retained-error checks.
`make shell-check`, offline `make tools-check` and `make check-doc-links` pass.
The reader admits all 105 manifest file records and rejects a NUL-bearing
manifest without emitting partial records. The root Cargo manifest, incoming
lockfile and tool pin
catalog retain their pre-export SHA-256 digests; the lockfile digest is
`04dba590354b9af576bac9ed344bd0727c7a0d233c7754a1917f1775d41139bc`.
No package version, public Backup API or runtime behavior changed. The full
product `make ci` gate and native hosted qualification have not been rerun for
this tooling-only batch. No commit, push, release or publication was performed.
