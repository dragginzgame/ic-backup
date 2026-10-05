<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-backup/ic-backup-readme-header.svg" alt="IC Backup — Verified backups and safe recovery for Internet Computer apps" width="100%">
</p>

<!-- helper-navigation:start -->
<p align="center">
  <a href="https://github.com/dragginzgame/canic"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/canic.svg" width="18" height="18" alt=""> <strong>canic</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/icydb"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/icydb.svg" width="18" height="18" alt=""> <strong>icydb</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-timers"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-timers.svg" width="18" height="18" alt=""> <strong>ic-timers</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-memory"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-memory.svg" width="18" height="18" alt=""> <strong>ic-memory</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-query"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-query.svg" width="18" height="18" alt=""> <strong>ic-query</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-backup"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-backup.svg" width="18" height="18" alt=""> <strong>ic-backup</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-blob-storage"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-blob-storage.svg" width="18" height="18" alt=""> <strong>ic-blob-storage</strong></a>
  &nbsp;&middot;&nbsp;
  <a href="https://github.com/dragginzgame/ic-testkit"><img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/icons/ic-testkit.svg" width="18" height="18" alt=""> <strong>ic-testkit</strong></a>
</p>
<!-- helper-navigation:end -->


# Supported Hosts And Dependencies

Support claims follow executed evidence. An installer branch or an upstream
release asset does not by itself make a host supported.

This document covers repository tooling and the native library development lane.
It does not claim that end-to-end IC backup or restore is available on any host.

## Scope at a glance

| Scope | Current position |
| --- | --- |
| Native Rust library development | Operator-host lane using the pinned Rust toolchains and repository-local build directory |
| Portable repository scripts | Bash 3.2 or newer with standard Unix userland |
| CI evidence | Ubuntu 24.04 and macOS 15 within the exact scopes below |
| Installer mappings | Linux and Darwin on x86-64 and ARM64; an untested mapping is not a support claim |
| Windows and non-Bash shells | Not supported |
| Complete backup/restore product | Not yet implemented or host-qualified |

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
