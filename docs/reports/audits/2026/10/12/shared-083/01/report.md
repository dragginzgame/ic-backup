# Shared Tooling 0.8.3 consumer adoption

The uncommitted Backup 0.20.2 draft now selects committed Shared Tooling 0.8.3
`8cca7e67b03ce6e701b6e7dcec2e9753eb98337e`, verified against GitHub
main. The source commit subject says `0.8.2`, but its committed `VERSION`,
changelog and canonical export all identify 0.8.3. A clean isolated source
checkout supplied the refresh. Preview and export retain the existing 105
entrypoints and payloads, update eight selected files and retire none. The
manifest SHA-256 is
`3ae0ace8be23ab43ed35e12247e4c8e36e1d0dae4d2468512929920f1f1f0130`.
Preview/export logs remain under ignored `target/shared-083/`.

The selected changes align formatting admission with the Rust tool-version
catalog, reject redirected Cargo build-observation destinations before writing
or activating a selection, and silence successful ripgrep PCRE2 probes while
retaining failure diagnostics. The common Make helper now declares its installer
and default LOC companions for future minimal snapshots. The source exporter
also replaces individual payloads atomically; it is upstream tooling, not a
new selected consumer payload. Backup's local Cargo-version inspection command
from the prior 0.8.2 phase remains explicit. No package pin, Backup runtime
code or public library contract changed in this refresh.

The root `Cargo.toml`, `Cargo.lock` and `ci/tool-versions.env` match their
pre-export hashes. The selected lock SHA-256 is
`de4f91081c0a84437fd5e96b550773d53e67edc8b9ff9f6794a912d28c067646`;
it contains the separately incoming Host 0.14.7, Testkit 0.34.4 and Metrics
0.7.2 graph. This snapshot work does not qualify that dependency graph.

Focused checks pass: `make shared-tooling-check`, changed host and Rust
installer fixtures, `make format-tools-check`, `make shell-check` and final
`make tools-check`. The first aggregate admission found the Testkit 0.34.4 CLI
missing. Two explicit setup attempts under workspace sandboxing reported a
shared-store installation lock; their failure logs remain retained. Because
the store is outside the writable workspace and no lock file persisted, setup
was retried with reviewed access to the user tool store. That exact locked
selection installed successfully, and aggregate offline admission then passed.
No lockfile, manifest or tool pin changed during setup. The full `make ci` gate
was not run for this tooling-only continuation. No Backup commit, push, release
or package publication was performed.
