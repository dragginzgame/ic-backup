# Changelog

## [Unreleased]

- Document the extraction of host-side backup/restore from Canic, including
  ownership boundaries, same-release recovery to the same canister IDs,
  source provenance and the implementation sequence.
- Establish an independent Rust 2024 workspace with the `ic-backup` library,
  inherited package metadata and lints, Rust 1.99.0 development tooling and
  Rust 1.91.0 minimum support.
- Add native build, lint, test, documentation and standalone package commands,
  Linux CI and a repository-local build directory.
- Add the pre-commit formatter with unstaged-source protection and explicit
  review/restaging of formatting changes.
- Provide `make release-patch`, `make release-minor` and `make release-major`
  to validate, prepare, commit, tag and atomically push a release, alongside
  preview and individual-step commands. Retain release-file hash receipts,
  rollback, build artifacts and isolated tooling regressions.
- Preserve MIT contributor attribution and source hashes for adapted sibling
  tooling. This initial foundation has no backup/restore APIs or CLI;
  registry publication remains disabled.
