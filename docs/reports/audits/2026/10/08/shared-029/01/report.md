# Shared Tooling 0.1.29 and published Host review

Release 0.9.0 is HEAD at `32f091a3cc576dc7e0d8b80aea6a484abf905984`.
The compatible next draft is 0.9.1; package versions and the release receipt stay
unchanged. This batch changes release diagnostics and Linux lint preparation.
No product function, method or type is removed.

The canonical exporter runs from a clean isolated checkout at Shared Tooling
`1a54fb625d6e47efa64c4384808ecbc87be84e7e`, excluding dirty sibling proposals.
[Source proof](source-proof.json) independently checks all 97 Git blobs and
executable modes. Fifteen existing paths change; the release-source checker and
two canonical CI-installer files are added explicitly. The qualified consumer IC
matrix stays unchanged. [The prior review](previous-review.json) retains its scope.

The consumer release adapter delegates source admission to the canonical checker.
Initial preflight permits pending changelog notes only and explains that validation
and version preparation have not started on refusal. Prepared checks retain the
original metadata allowance and require the original HEAD; committed/resumed checks
require clean source. Real private-index fixtures qualify hidden staged changes,
ordinary working edits, lock-only edits, untracked newline names, native Git errors
and unchanged index/file/HEAD bytes. Existing receipts and recovery stay local.
All consumer adapters and canonical runner simulations pass without new commits.

The delivered 0.9.0 Linux job fails before Rust validation: Ubuntu ShellCheck 0.9
reports SC2015 in the shared dependency checker. Local 0.11 accepts the same
expression. The retained actual job log owns this failure; it is not a failed
product test. Linux CI now uses the already-reviewed 0.11 pin through the canonical
checksum installer. Local shell-check prefers the prepared checkout binary, then
its existing PATH/user-local choices. Actual installation, selected binary/version,
shell lint, full tooling and workflow lint pass. macOS keeps Homebrew setup.
No vendored patch or diagnostic suppression is introduced.

Common issue authority now covers only `dragginzgame/*`, and the adopted maintenance
prompts carry that boundary. No issue or scheduled agent is activated here. The
optional npm checker is not selected by this Rust-only consumer; its existing
Cargo/Action path passes. Shared's sibling dashboard and repository-specific source
fixture are unselected; consumer private-index cases qualify the adopted helper.

[Host proof](host-proof.json) records fresh official sparse registry rows: all four
selected packages are still non-yanked 0.8.2, and cached archive digests match the
locked identities and retained committed-source proof. Exact Host owner CI now
passes Linux, both macOS architectures and MSRV. The read-only local committed
0.8.3 draft adds child-output observation and an unlocked durable path opener;
dirty fixture changes are excluded. GitHub has no such draft commit at the final
read, and registry/main remain 0.8.2. Nothing unpublished is selected. Backup's
existing-only custody admission and private 0600 sidecar creation cannot be replaced
by an opener that always creates missing files/parents with general permissions.

A concurrent external lock update selects Metrics 0.2.15. It is preserved and
prepared through an explicit locked fetch after initial offline metadata refusal.
[Metrics proof](metrics-proof.json) matches its non-yanked registry/archive and exact
committed source. Every published Rust byte equals 0.2.14; selected host features
remain empty. Fresh Rust 1.88 compilation covers both libraries and independent
public consumers without simulator/dev feature unification. All five focused
diagnostic/distribution cases pass. Source-bound earlier evidence remains historical.

[Qualification](qualification.json) owns exact inputs, logs, original CI failure,
registry preparation limits and native observations. Shared 0.1.29 Linux and
lint/security pass; both macOS jobs are queued at the final read. The current dirty
consumer needs its own committed native CI. No complete CI/release gate, dependency
reselection, sibling edit, Git/release/publication or live IC effect occurs. Full
workflow/stage binding, fresh Canic adoption and terminal/fence/reference release
retain their separate owners.
