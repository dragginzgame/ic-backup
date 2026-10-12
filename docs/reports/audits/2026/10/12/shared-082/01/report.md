# Shared Tooling 0.8.2 consumer adoption

Backup 0.20.1 is committed at
`7a8096edd0d1a07b71aa652c1c9a6374695c7e42`. The next compatible draft is
0.20.2, with own package versions unchanged. A clean isolated checkout of
committed Shared Tooling 0.8.2
`aaf75a37717357dd62f4b0b15001eaa3ba1629cf` supplied the canonical
snapshot. Its default preview updated only `scripts/distribution/governance-files.txt`,
`tasks/README.md` and `tasks/maintenance-prompt.md` among the 102 prior files.
The new catalog link would have lacked its target. The reviewed export explicitly
selected `tasks/cargo-tool-versions.md`,
`scripts/dev/check-cargo-tool-versions.pl` and
`scripts/ci/test-cargo-tool-versions.sh`. The resulting v1 manifest has 105
entrypoints and 105 payloads, with no retirements; its SHA-256 is
`118daec8938fd0c0fcc21328853dd0ade3e65fc31b74e991fa3477e3b009006d`.

`make check-cargo-tool-versions` now invokes that shared checker explicitly for
this checkout. It is an online, read-only comparison of literal common Cargo
pins, local selection receipts and user-installed packages against current
non-yanked stable crates.io releases. It retains registry evidence, reports
missing or unavailable observations, and does not install `cargo-modules` or
other tools, alter pins or replace offline `make rust-tools-check`. The new
fixture runs in Backup's tooling gate. Immediately after export, the lock,
root Cargo manifest and tool pins matched their pre-export hashes. During
final review, `Cargo.lock` changed independently to Host 0.14.7, Testkit
0.34.4 and Metrics 0.7.2. The lock is preserved; this tooling-focused review
does not qualify that later dependency graph. Root Cargo manifest and tool pins
still match their pre-export hashes. The final lock SHA-256 is
`de4f91081c0a84437fd5e96b550773d53e67edc8b9ff9f6794a912d28c067646`;
focused `make dependency-pins-check` and offline locked Cargo metadata pass on
those bytes, without compiling or testing the selected packages.

Focused checks passed: `make shared-tooling-check`, the new Cargo-version
fixture, `make tooling-check`, `make tools-check`, `make shell-check`,
`make check-doc-links` and `git diff --check`. The checker itself was not run
against live crates.io; that optional online inspection has a different result
scope than offline admission. The full `make ci` gate was not rerun for this
tooling-only batch. Detailed previews, export and check logs remain under
ignored `target/shared-082/`.

At the 2026-10-12 10:37 UTC inspection, Shared Tooling's local `main` was one
commit ahead of GitHub `main`; the GitHub commit API had not yet found 0.8.2.
At 10:38 UTC, GitHub main returned the exact exported commit
`aaf75a37717357dd62f4b0b15001eaa3ba1629cf`. Remote source provenance is
therefore resolved. Native acceptance still needs a delivered exact consumer
run. No Backup commit, push, release or package publication was performed.
