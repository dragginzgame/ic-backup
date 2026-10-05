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

# Documentation

Start with the [project overview](../README.md) for a plain-language explanation
of what IC Backup is intended to do and its current limitations. Use the guide
below to find the document that matches your task.

## Understand the project

| Document | Use it for |
| --- | --- |
| [Current status](status/current.md) | Released baseline, current implementation evidence and remaining work |
| [Implemented extraction boundary](extraction-boundary.md) | Exact behavior currently provided by the library and what it does not establish |
| [Extraction and implementation design](extraction-design.md) | Detailed proposed product architecture, workflows and completion criteria |

## Develop and maintain it

| Document | Use it for |
| --- | --- |
| [Development](development.md) | Quick start, code ownership, validation commands and build-directory rules |
| [Supported hosts](supported-hosts.md) | Qualified CI and script environments and their dependencies |
| [Releasing](releasing.md) | Preview, preparation, maintainer release and registry publication workflows |

## Understand shared engineering inputs

| Document | Use it for |
| --- | --- |
| [Shared Tooling adoption](shared-tooling.md) | Reviewed upstream identity, local overlay and validation evidence |
| [Consuming Shared Tooling snapshots](consuming-snapshots.md) | Creating, refreshing and verifying vendored tooling files |
| [Engineering principles](principles/README.md) | Repository-neutral guidance used across participating projects |

Machine-readable schemas and provenance records under `contracts/` and the
remaining JSON files support the detailed implementation evidence. They are not
introductory documentation.
