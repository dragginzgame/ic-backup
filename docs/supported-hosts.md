# Supported Hosts And Dependencies

Support claims follow executed evidence. An installer branch or an upstream
release asset does not by itself make a host supported.

## Portable script baseline

The portable scripts target Bash 3.2 or newer and standard Unix userland.
Repository CI exercises the offline regression set on:

| Host | Scope |
| --- | --- |
| Ubuntu 24.04 GitHub-hosted runner | Portable scripts, ShellCheck, workflow lint, installer downloads, and secret scan |
| macOS 15 GitHub-hosted runner | Portable offline regression set with the runner-provided Bash |

The table describes the intended CI contract. A revision is supported only
after its matching workflow run passes.

Windows and non-Bash shells are not supported.

## Tool-specific dependencies

| Tool | Additional dependencies |
| --- | --- |
| `scripts/dev/cloc.sh` | Git, Cargo, `cloc`, `jq`, `awk`, `find`, `grep`, and `sort` |
| `scripts/dev/gh-ci.sh` | Git and an authenticated GitHub CLI |
| `scripts/ci/run-validation-targets.sh` | GNU Make plus `awk`, `grep` or `rg`, `sed`, `tail`, and `tee` |
| Installer scripts | `curl`, `tar`, a SHA-256 implementation, and the archive codec used by the selected tool |
| `scripts/ci/run-sccache.sh` | An executable `sccache` binary |
| Snapshot verification | A SHA-256 implementation |
| Snapshot refresh | Git, a clean Shared Tooling checkout, and a SHA-256 implementation |

## Installer-capable platforms

The actionlint, Gitleaks, and ShellCheck installers contain asset mappings for
Linux and Darwin on x86-64 and ARM64. Branches not exercised by the repository's
installer-download CI are install-capable, not support claims.

Consumers own the exact tool versions and platform digests they admit.
