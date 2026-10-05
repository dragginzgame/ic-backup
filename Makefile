SHELL := /bin/bash
.DEFAULT_GOAL := help
export CARGO_TARGET_DIR := $(CURDIR)/target
VERSION ?=
RELEASE_REMOTE ?= origin
RELEASE_BRANCH ?= main
RELEASE := bash scripts/release/release.sh
CI_TARGETS := shared-tooling-check deps shell-check tooling-check release-check hooks-check fmt-check check clippy test doc check-msrv package

ifneq ($(word 2,$(filter release-patch release-minor release-major release-resume,$(MAKECMDGOALS))),)
$(error Select exactly one release target)
endif

.PHONY: check check-msrv ci clean clippy deps doc ensure-clean fmt fmt-check \
        help hooks-check install-hooks package publish publish-dry-run \
        release-check release-major release-minor release-patch release-plan release-resume \
        release-version release-preflight release-prepare-version release-prepared-check \
        release-files release-commit-check release-committed-check release-tagged-check \
        release-push-check release-tag-check release-verify shared-tooling-check shell-check \
        tags test tooling-check validate version

help:
	@echo "check                       Compile this library's native targets"
	@echo "check-msrv                  Check Rust 1.91.0"
	@echo "ci                          Fetch dependencies and run the full gate"
	@echo "clean                       Explicitly remove build artifacts"
	@echo "clippy                      Lint native targets with warnings denied"
	@echo "deps                        Fetch locked dependencies"
	@echo "doc                         Build documentation with warnings denied"
	@echo "ensure-clean                Check the worktree is committed and clean"
	@echo "fmt                         Sort Cargo manifests and format Rust"
	@echo "fmt-check                   Check Cargo ordering and Rust formatting"
	@echo "help                        Show available commands"
	@echo "hooks-check                 Test the pre-commit formatter"
	@echo "install-hooks               Enable the tracked pre-commit formatter"
	@echo "package                     Verify the standalone package locally"
	@echo "publish                     Publish the current library to crates.io"
	@echo "publish-dry-run             Verify Cargo publication without uploading"
	@echo "release-check               Test release helpers with isolated substitutes"
	@echo "release-major               Maintainer: prepare, commit, tag and push a major"
	@echo "release-minor               Maintainer: prepare, commit, tag and push a minor"
	@echo "release-patch               Maintainer: prepare, commit, tag and push a patch"
	@echo "release-plan VERSION=minor  Preview patch/minor/major or an exact version"
	@echo "release-resume VERSION=x.y.z Maintainer: resume the exact saved release"
	@echo "release-tag-check           Verify the current annotated release tag"
	@echo "release-verify              Run the full release validation gate"
	@echo "shared-tooling-check        Verify the reviewed local tooling snapshot"
	@echo "shell-check                 Check maintained shell and Perl tooling"
	@echo "tags                        List local version tags"
	@echo "test                        Run native library tests and doctests"
	@echo "tooling-check               Test snapshot integrity and CI diagnostics"
	@echo "validate                    Run the full validation gate"
	@echo "version                     Print the workspace package version"

check:
	cargo check --offline --locked -p ic-backup --all-targets --all-features

check-msrv:
	cargo +1.91.0 check --offline --locked -p ic-backup --all-targets --all-features

ci:
	+@bash scripts/ci/run-validation-targets.sh --fail-fast $(CI_TARGETS)

clean:
	cargo clean

clippy:
	cargo clippy --offline --locked -p ic-backup --all-targets --all-features -- -D warnings

deps:
	cargo fetch --locked

doc:
	RUSTDOCFLAGS="-D warnings" cargo doc --offline --locked -p ic-backup --no-deps --all-features

ensure-clean:
	@$(RELEASE) ensure-clean

fmt:
	@[[ "$$(cargo sort --version)" == 'cargo-sort 2.1.4' ]] || { echo 'error: install cargo-sort 2.1.4 with --locked during setup' >&2; exit 1; }
	cargo sort --workspace
	cargo fmt --all

fmt-check:
	@[[ "$$(cargo sort --version)" == 'cargo-sort 2.1.4' ]] || { echo 'error: install cargo-sort 2.1.4 with --locked during setup' >&2; exit 1; }
	cargo sort --workspace --check
	cargo fmt --all -- --check

hooks-check:
	bash scripts/hooks/test-hooks.sh

install-hooks:
	bash scripts/dev/install-git-hooks.sh

package:
	cargo package --offline --locked --allow-dirty -p ic-backup

publish:
	$(RELEASE) publish

publish-dry-run:
	$(RELEASE) publish --dry-run

release-check:
	bash scripts/release/test-release.sh
	bash scripts/ci/test-release-runner.sh

release-patch release-minor release-major:
	@bash scripts/ci/run-release.sh "$(@:release-%=%)" "$(RELEASE_REMOTE)" "$(RELEASE_BRANCH)"

release-resume:
	@$(RELEASE) resume-check "$(VERSION)"
	@bash scripts/ci/run-release.sh resume "$(VERSION)" "$(RELEASE_REMOTE)" "$(RELEASE_BRANCH)"
	@$(RELEASE) tag-check

release-plan:
	@$(RELEASE) plan "$(if $(VERSION),$(VERSION),patch)"

release-tag-check:
	$(RELEASE) tag-check

release-verify: ci
	@if [[ -n "$(RELEASE_SOURCE)" ]]; then $(RELEASE) validation-record; fi

release-version:
	@$(RELEASE) version

release-preflight:
	@$(RELEASE) preflight

release-prepare-version:
	@$(RELEASE) prepare

release-prepared-check:
	@$(RELEASE) prepared-check

release-files:
	@$(RELEASE) files

release-commit-check:
	@$(RELEASE) commit-check

release-committed-check:
	@$(RELEASE) committed-check

release-tagged-check:
	@$(RELEASE) tagged-check

release-push-check:
	@$(RELEASE) push-check

export RELEASE_SOURCE RELEASE_PREVIOUS RELEASE_VERSION RELEASE_DATE

shared-tooling-check:
	bash scripts/ci/verify-shared-tooling-snapshot.sh

shell-check:
	@for script in scripts/ci/*.sh scripts/dev/*.sh scripts/release/*.sh scripts/hooks/*.sh .githooks/pre-commit; do \
		bash -n "$$script" || exit $$?; \
	done
	@set -e; \
	if command -v shellcheck >/dev/null 2>&1; then \
		shellcheck_bin=shellcheck; \
	elif [[ -x "$$HOME/.local/bin/shellcheck" ]]; then \
		shellcheck_bin="$$HOME/.local/bin/shellcheck"; \
	else \
		printf '%s\n' 'error: ShellCheck is required; install it on PATH or at ~/.local/bin/shellcheck' >&2; \
		exit 127; \
	fi; \
	printf '%s\n' "$$shellcheck_bin scripts/ci/*.sh scripts/dev/*.sh scripts/release/*.sh scripts/hooks/*.sh .githooks/pre-commit"; \
	"$$shellcheck_bin" scripts/ci/*.sh scripts/dev/*.sh scripts/release/*.sh scripts/hooks/*.sh .githooks/pre-commit
	perl -c scripts/release/release-data.pl

tags:
	@git tag --sort=-version:refname

test:
	cargo test --offline --locked -p ic-backup --all-features

tooling-check: shared-tooling-check
	bash scripts/ci/test-tooling.sh

validate: ci

version:
	@$(RELEASE) version
