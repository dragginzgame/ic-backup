SHELL := /bin/bash
.DEFAULT_GOAL := help
export CARGO_TARGET_DIR := $(CURDIR)/target
LOCAL_TOOL_INSTALL_TARGETS := deps install-testkit-server install-release-editor
LOCAL_TOOL_CHECK_TARGETS := testkit-server-check release-editor-check
RELEASE_EDITOR_VERSION := 0.70.5
export RELEASE_EDITOR_MAX_BYTES := 8388608
RELEASE_EDITOR_COMMAND = bash "$(CURDIR)/scripts/dev/install-rust-tools.sh" --consumer "$(CURDIR)" \
    --package ic-build --version "$(RELEASE_EDITOR_VERSION)" --bin ic-build-release-metadata --profile debug --features cli
VERSION ?=
RELEASE_REMOTE ?= origin
RELEASE_BRANCH ?= main
RELEASE_DELIVERY ?= direct
export RELEASE_DELIVERY
RELEASE := bash scripts/release/release.sh
override RELEASE_RESUME_CHECK_COMMAND = $(RELEASE) resume-check "$(VERSION)"
override SHARED_TOOLING_ROOT := $(CURDIR)
include make/consumer.mk
CI_TARGETS := shared-tooling-check tools-check repository-files-check dependency-pins-check dependency-boundaries-check check-doc-links shell-check fmt-check clippy tooling-check release-check hooks-check check test doc check-msrv package

.PHONY: check check-cargo-tool-versions check-doc-links check-msrv ci clean clippy deps doc ensure-clean \
        help hooks-check package publish publish-dry-run \
        release-check release-plan \
        release-version release-preflight release-prepare-version release-prepared-check \
        release-files release-commit-check release-committed-check release-tagged-check \
        release-push-check release-tag-check release-verify shell-check \
        tags tasks test tooling-check validate version
.PHONY: dependency-boundaries-check dependency-pins-check install-testkit-server testkit-server-check release-delivery-check
.PHONY: install-release-editor release-editor-check repository-files-check

help:
	@echo "check                       Compile this library's native targets"
	@echo "check-cargo-tool-versions   Inspect Cargo tool versions online without installing"
	@echo "check-doc-links             Check maintained local Markdown targets"
	@echo "check-msrv                  Check Rust 1.91.0 and independent consumers"
	@echo "ci                          Run the full gate against prepared dependencies"
	@echo "clean                       Explicitly remove build artifacts"
	@echo "clippy                      Lint native targets with warnings denied"
	@echo "cloc                        Report Rust workspace source and test LOC"
	@echo "dependency-boundaries-check Check direct IC and WAT dependency ownership"
	@echo "dependency-pins-check       Check dependency declarations and exceptions"
	@echo "deps                        Fetch locked dependencies"
	@echo "doc                         Build documentation with warnings denied"
	@echo "ensure-clean                Check the worktree is committed and clean"
	@echo "fmt                         Sort Cargo manifests and format Rust"
	@echo "fmt-check                   Check Cargo ordering and Rust formatting"
	@echo "format-tools-check          Check prepared Cargo formatters offline"
	@echo "help                        Show available commands"
	@echo "hooks-check                 Test the pre-commit formatter"
	@echo "host-tools-check            Verify local jq/yq/ripgrep/cloc offline"
	@echo "ic-tools-check IC_TOOLS=... Optional: verify the selected IC executables"
	@echo "install-hooks               Enable the tracked pre-commit formatter"
	@echo "install-host-tools          Prepare pinned local jq/yq/ripgrep/cloc"
	@echo "install-ic-tools IC_TOOLS=... Optional: prepare selected IC executables"
	@echo "install-release-editor      Prepare the published Cargo release editor"
	@echo "install-rust-tools          Prepare the complete shared Cargo tool set"
	@echo "install-testkit-server       Prepare the locked Testkit CLI and its PocketIC server"
	@echo "install-tools               Prepare host/Cargo tools, locked Testkit and release editor"
	@echo "package                     Verify both library packages locally"
	@echo "publish                     Publish both libraries to crates.io"
	@echo "publish-dry-run             Verify Cargo publication without uploading"
	@echo "release-check               Test release helpers with isolated substitutes"
	@echo "release-delivery-check      Verify the supported direct release policy without effects"
	@echo "release-editor-check        Admit the prepared Cargo release editor offline"
	@echo "release-major               Maintainer: prepare, commit, tag and push a major"
	@echo "release-minor               Maintainer: prepare, commit, tag and push a minor"
	@echo "release-patch               Maintainer: prepare, commit, tag and push a patch"
	@echo "release-plan VERSION=minor  Preview patch/minor/major or an exact version"
	@echo "release-resume VERSION=x.y.z Maintainer: resume the exact saved release"
	@echo "release-tag-check           Verify the current annotated release tag"
	@echo "release-verify              Run the full release validation gate"
	@echo "repository-files-check      Check source links and complete package licenses"
	@echo "rust-tools-check            Verify the complete local Cargo tools offline"
	@echo "shared-tooling-check        Verify the reviewed local tooling snapshot"
	@echo "shell-check                 Check maintained shell and Perl tooling"
	@echo "tags                        List local version tags"
	@echo "tasks                       Read the shared maintenance catalog"
	@echo "test                        Run native library tests and doctests"
	@echo "testkit-server-check        Check the locked Testkit CLI/server offline; print server path"
	@echo "tooling-check               Test consumer CI wiring and retained evidence"
	@echo "tools-check                 Verify host/Cargo tools, locked Testkit and release editor"
	@echo "tools-report                Diagnose every offline tool check and retain failures"
	@echo "validate                    Run the full validation gate"
	@echo "version                     Print the workspace package version"

# Refuse a missing/changed selected CLI before native qualification, including
# parallel standalone invocations. Validation never provisions it implicitly.
check check-msrv clippy test: testkit-server-check

check:
	+cargo check --offline --locked -p ic-backup -p ic-backup-agent --all-targets --all-features

check-doc-links:
	bash scripts/ci/check-doc-links.sh

check-msrv:
	RUSTUP_AUTO_INSTALL=0 rustc +1.91.0 --version
	+RUSTUP_AUTO_INSTALL=0 cargo +1.91.0 --version
	+RUSTUP_AUTO_INSTALL=0 cargo +1.91.0 check --offline --locked -p ic-backup -p ic-backup-agent --all-targets --all-features
	+bash scripts/ci/check-msrv-consumers.sh 1.91.0

ci:
	+@bash scripts/ci/run-validation-targets.sh --fail-fast --input Cargo.toml --input Cargo.lock $(CI_TARGETS)

clean:
	+cargo clean

clippy:
	+cargo clippy --offline --locked -p ic-backup -p ic-backup-agent --all-targets --all-features -- -D warnings

deps:
	+cargo fetch --locked

doc:
	+RUSTDOCFLAGS="-D warnings" cargo doc --offline --locked -p ic-backup -p ic-backup-agent --no-deps --all-features

ensure-clean:
	+@$(RELEASE) ensure-clean

hooks-check:
	bash scripts/hooks/test-hooks.sh

dependency-boundaries-check:
	+bash scripts/ci/check-cargo-dependency-boundaries.sh --consumer "$(CURDIR)" --external-ic

dependency-pins-check:
	+bash scripts/ci/check-dependency-pins.sh --cargo-inheritance

install-release-editor:
	+@$(RELEASE_EDITOR_COMMAND)

release-editor-check:
	+@$(RELEASE_EDITOR_COMMAND) --check

repository-files-check:
	perl scripts/ci/check-no-symlinks.pl --consumer "$(CURDIR)"
	@for license in LICENSE crates/ic-backup/LICENSE crates/ic-backup-agent/LICENSE; do \
		perl scripts/ci/check-license.pl --kind mit --file "$$license" || exit "$$?"; done

# The canonical Cargo installer selects the exact registry Testkit from Cargo.lock
# CLI receipts; Testkit owns server assets, admission and provisioning.
define TESTKIT_SERVER
	+@set -eo pipefail; \
	cli="$$(bash scripts/dev/install-rust-tools.sh --consumer "$(CURDIR)" \
	  --package ic-testkit --lockfile Cargo.lock --bin ic-testkit-server --profile release --features cli $(1))" || { \
	  status=$$?; \
	  if [[ "$$status" == 75 ]]; then \
	    printf 'Locked Testkit CLI is busy; retry after its installation owner finishes.\n' >&2; \
	    exit "$$status"; \
	  fi; \
	  printf 'Locked Testkit CLI admission failed. Prepare a missing selection with make install-testkit-server; inspect retained evidence if it is changed.\n' >&2; \
	  exit "$$status"; \
	}; \
	"$$cli" $(2) --directory "$(CURDIR)/.tools/testkit-server"
endef

install-testkit-server:
	$(call TESTKIT_SERVER,,setup)

testkit-server-check:
	$(call TESTKIT_SERVER,--check,check)

package:
	+bash scripts/ci/check-packages.sh

publish:
	+$(RELEASE) publish

publish-dry-run:
	+$(RELEASE) publish --dry-run

release-check:
	+bash scripts/release/test-release.sh

release-delivery-check:
	@[[ "$${RELEASE_DELIVERY:-direct}" == direct ]] || { echo 'This repository supports direct release delivery; PR delivery needs a qualified consumer adapter.' >&2; exit 2; }

# Product delivery admission and exact resumed-receipt verification remain local;
# the common include owns dispatch and failure propagation.
release-patch release-minor release-major release-resume: release-delivery-check

release-plan:
	+@$(RELEASE) plan "$(if $(VERSION),$(VERSION),patch)"

release-tag-check:
	+$(RELEASE) tag-check

release-verify: ci
	+@if [[ -n "$(RELEASE_SOURCE)" ]]; then $(RELEASE) validation-record; fi

release-version:
	+@$(RELEASE) version

release-preflight:
	+@$(RELEASE) preflight

release-prepare-version:
	+@$(RELEASE) prepare

release-prepared-check:
	+@$(RELEASE) prepared-check

release-files:
	+@$(RELEASE) files

release-commit-check:
	+@$(RELEASE) commit-check

release-committed-check:
	+@$(RELEASE) committed-check

release-tagged-check:
	+@$(RELEASE) tagged-check

release-push-check:
	+@$(RELEASE) push-check

export RELEASE_SOURCE RELEASE_PREVIOUS RELEASE_VERSION RELEASE_DATE RELEASE_COMMIT

shell-check:
	@for script in scripts/ci/*.sh scripts/dev/*.sh scripts/release/*.sh scripts/hooks/*.sh .githooks/pre-commit; do \
		bash -n "$$script" || exit $$?; \
	done
	@set -e; \
	if [[ -x "$(CURDIR)/.tools/ci/bin/shellcheck" ]]; then \
		shellcheck_bin="$(CURDIR)/.tools/ci/bin/shellcheck"; \
	elif command -v shellcheck >/dev/null 2>&1; then \
		shellcheck_bin=shellcheck; \
	elif [[ -x "$$HOME/.local/bin/shellcheck" ]]; then \
		shellcheck_bin="$$HOME/.local/bin/shellcheck"; \
	else \
		printf '%s\n' 'error: ShellCheck is required; install it on PATH or at ~/.local/bin/shellcheck' >&2; \
		exit 127; \
	fi; \
	printf '%s\n' "$$shellcheck_bin -x scripts/ci/*.sh scripts/dev/*.sh scripts/release/*.sh scripts/hooks/*.sh .githooks/pre-commit"; \
	"$$shellcheck_bin" -x scripts/ci/*.sh scripts/dev/*.sh scripts/release/*.sh scripts/hooks/*.sh .githooks/pre-commit
	perl -c scripts/release/release-data.pl
	perl -c scripts/ci/check-documentation-links.pl
	perl -c scripts/ci/test-documentation-links.pl
	perl -c scripts/dev/check-cargo-tool-versions.pl

check-cargo-tool-versions:
	@perl scripts/dev/check-cargo-tool-versions.pl --consumer "$(CURDIR)" --versions "$(RUST_TOOL_VERSIONS)"

tags:
	@git tag --sort=-version:refname

test:
	+cargo test --offline --locked -p ic-backup -p ic-backup-agent --all-features

tasks:
	@cat tasks/README.md

# Shared owns generic checksum/archive matrices; this gate qualifies Backup callers.
tooling-check: shared-tooling-check
	bash scripts/ci/test-tooling.sh
	bash scripts/ci/test-testkit-commands.sh
	bash scripts/ci/test-failure-evidence.sh

validate: ci

version:
	+@$(RELEASE) version
