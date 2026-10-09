# Shared Tooling 0.1.35 consumer review

Refresh the existing 95-file selection from committed 0.1.34
`3d33cd250fcae7dbe5cabe44b2abd6b2c91a1822` to committed 0.1.35
`be550afa57fe9e16872e5110b5cd69c24b4fa9e8`, verified against upstream main.
Canonical refresh uses a clean isolated committed checkout, preserving all
incoming dirty implementation and dependency work. Package versions and receipt
remain released 0.10.0; the changelog retains the compatible 0.10.1 draft.

Seven selected files change: the common baseline, consumer snapshot/setup/release
guides, Cargo dependency rules, Rust installer and its regression suite. Every
selected byte and executable mode matches the committed source. No vendored file
is patched locally, selection is unchanged and no function, method or type is
removed. Upstream's owner-only Cargo-install assessment stays unselected.

The optional installer admits exact stable Cargo package versions, named binaries
or examples and explicit debug/release profiles. Existing three-tool setup and
aggregate defaults remain unchanged. Shared fixtures use substitute Cargo to
check receipt/selection/byte identity, offline reuse, corruption/path refusal,
retained failed candidates and concurrent setup. These are consumer integration
checks, not actual registry installation or native package qualification.

The baseline clarifies that authorized dependency preparation includes registry
access. Explicit offline settings remain authoritative; ordinary checks neither
install nor gain network authority. Backup's shell/Perl release adapter already
runs root-workspace `cargo fetch --locked` after source/snapshot admission and
before offline validation. No compiled adapter or duplicate local registry-tool
resolver exists to migrate. Local release/development guidance records the rules.

Focused installer, shared command, local release-adapter, ShellCheck, actual
offline installed-tool and snapshot checks pass on Linux. Retained commands,
log hashes and source/preservation proof are in [qualification](qualification.json)
and `target/shared-035-review/`. Release fixtures use command substitutes and
private temporary Git boundaries; they do not release this repository.

Exact upstream 0.1.35 CI run
[37904190217](https://github.com/dragginzgame/shared-tooling/actions/runs/37904190217)
is in progress at inspection. Prior 0.1.34 CI passed but supplies no 0.1.35 native
acceptance. This uncommitted consumer candidate has no matching hosted run.
No broad CI/release gate, registry install, dependency update, sibling edit,
root Git write, publication or live IC effect occurs. Earlier stage qualification
retains its original source/graph/snapshot evidence separately.
