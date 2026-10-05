SHELL := /bin/bash
.DEFAULT_GOAL := help
export CARGO_TARGET_DIR := $(CURDIR)/target
VERSION ?=
RELEASE := bash scripts/release/release.sh
CI_TARGETS := shared-tooling-check deps shell-check tooling-check release-check hooks-check fmt-check check clippy test doc check-msrv package

.PHONY: bump-x check check-msrv ci clean clippy deps doc ensure-clean fmt fmt-check \
        help hooks-check hooks-install major minor package patch publish publish-dry-run \
        release-check release-commit release-major release-minor release-patch release-plan \
        release-push release-stage release-tag-check release-verify release-x shared-tooling-check shell-check \
        tags test tooling-check validate version

help:
	@echo "bump-x VERSION=x.y.z         Prepare an exact release version"
	@echo "check                       Compile this library's native targets"
	@echo "check-msrv                  Check Rust 1.91.0"
	@echo "ci                          Fetch dependencies and run the full gate"
	@echo "clean                       Explicitly remove build artifacts"
	@echo "clippy                      Lint native targets with warnings denied"
	@echo "deps                        Fetch locked dependencies"
	@echo "doc                         Build documentation with warnings denied"
	@echo "ensure-clean                Check the worktree is committed and clean"
	@echo "fmt                         Format Rust"
	@echo "fmt-check                   Check Rust formatting"
	@echo "help                        Show available commands"
	@echo "hooks-check                 Test the pre-commit formatter"
	@echo "hooks-install               Enable the tracked pre-commit formatter"
	@echo "major                       Validate and prepare a major release"
	@echo "minor                       Validate and prepare a minor release"
	@echo "package                     Verify the standalone package locally"
	@echo "patch                       Validate and prepare a patch release"
	@echo "publish                     Publish the current library to crates.io"
	@echo "publish-dry-run             Verify Cargo publication without uploading"
	@echo "release-check               Test release helpers with isolated substitutes"
	@echo "release-commit              Maintainer: commit and tag prepared release files"
	@echo "release-major               Maintainer: prepare, commit, tag and push a major"
	@echo "release-minor               Maintainer: prepare, commit, tag and push a minor"
	@echo "release-patch               Maintainer: prepare, commit, tag and push a patch"
	@echo "release-plan VERSION=minor  Preview patch/minor/major or an exact version"
	@echo "release-push                Push exactly main and its current release tag"
	@echo "release-stage               Stage only prepared release files"
	@echo "release-tag-check           Verify the current annotated release tag"
	@echo "release-verify              Run the full release validation gate"
	@echo "release-x VERSION=x.y.z     Maintainer: release an exact version"
	@echo "shared-tooling-check        Verify the reviewed local tooling snapshot"
	@echo "shell-check                 Check maintained shell and Perl tooling"
	@echo "tags                        List local version tags"
	@echo "test                        Run native library tests and doctests"
	@echo "tooling-check               Test snapshot integrity and CI diagnostics"
	@echo "validate                    Run the full validation gate"
	@echo "version                     Print the workspace package version"

bump-x:
	$(RELEASE) bump "$(VERSION)"

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
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

hooks-check:
	bash scripts/hooks/test-hooks.sh

hooks-install:
	bash scripts/hooks/install.sh

major:
	$(RELEASE) bump major

minor:
	$(RELEASE) bump minor

package:
	cargo package --offline --locked --allow-dirty -p ic-backup

patch:
	$(RELEASE) bump patch

publish:
	$(RELEASE) publish

publish-dry-run:
	$(RELEASE) publish --dry-run

release-check:
	bash scripts/release/test-release.sh

release-commit:
	$(RELEASE) commit

release-major:
	$(RELEASE) release major

release-minor:
	$(RELEASE) release minor

release-patch:
	$(RELEASE) release patch

release-plan:
	@$(RELEASE) plan "$(if $(VERSION),$(VERSION),patch)"

release-push:
	$(RELEASE) push

release-stage:
	$(RELEASE) stage

release-tag-check:
	$(RELEASE) tag-check

release-verify: ci

release-x:
	$(RELEASE) release "$(VERSION)"

shared-tooling-check:
	bash scripts/ci/verify-shared-tooling-snapshot.sh

shell-check:
	@for script in scripts/ci/*.sh scripts/release/*.sh scripts/hooks/*.sh .githooks/pre-commit; do \
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
	printf '%s\n' "$$shellcheck_bin scripts/ci/*.sh scripts/release/*.sh scripts/hooks/*.sh .githooks/pre-commit"; \
	"$$shellcheck_bin" scripts/ci/*.sh scripts/release/*.sh scripts/hooks/*.sh .githooks/pre-commit
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
