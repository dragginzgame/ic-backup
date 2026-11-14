.DEFAULT_GOAL := help
REPO_ROOT := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))
TOKO_TOOLING_ROOT := $(REPO_ROOT)vendor/toko-tooling
include $(TOKO_TOOLING_ROOT)/make/consumer.mk
include $(TOKO_TOOLING_ROOT)/make/cargo-dependencies.mk

PACKAGE ?=
CARGO_SELECTION = $(if $(strip $(PACKAGE)),-p "$(PACKAGE)",--workspace)
export RUSTUP_AUTO_INSTALL := 0

.PHONY: help check doc-links-check fetch-dependencies clippy rustdoc test
help:
	@printf '%s\n' \
	  'check               Aggregate offline checks (all packages by default)' \
	  'check PACKAGE=name  Select one package for Rust checks' \
	  'deps-list           List local workspace package versions' \
	  'deps-update         Explicit Cargo dependency update (network)' \
	  'doc-links-check     Check local documentation file links' \
	  'fetch-dependencies  Explicit locked Cargo cache preparation (network)' \
	  'install-tools       Prepare shared host and Rust developer tools' \
	  'toko-tooling-check  Admit Cargo pin, members, lockfile and snapshot'
	@printf '\n%s\n' 'LOC report (PATH command): toko-cloc [-d N] [directory | crate[::module]]'

check: toko-tooling-check doc-links-check fmt-check clippy rustdoc test
ci: check

fetch-dependencies: | toko-execution-check
	cargo fetch --locked

clippy: toko-tooling-check
	cargo clippy $(CARGO_SELECTION) --all-targets --locked --offline -- -D warnings

rustdoc: toko-tooling-check
	RUSTDOCFLAGS="-D warnings" cargo doc $(CARGO_SELECTION) --no-deps --locked --offline

test: toko-tooling-check
	RUSTFLAGS="-D warnings" cargo test $(CARGO_SELECTION) --locked --offline

doc-links-check: toko-tooling-check
	@perl "$(TOKO_TOOLING_ROOT)/scripts/ci/check-documentation-links.pl" \
	  --root "$(REPO_ROOT)" README.md AGENTS.md CHANGELOG.md docs
