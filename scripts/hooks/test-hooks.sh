#!/usr/bin/env bash
set -Eeuo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
mkdir -p "$ROOT/target"
TEMPORARY="$(mktemp -d "$ROOT/target/hook-tests.XXXXXX")"
CASE_NAME=setup
CASE_LOG=/dev/null
finish() {
    local status=$?
    if [[ "$status" == 0 ]]; then
        rm -rf "$TEMPORARY"
    else
        printf 'Hook tests: FAIL [%s]; fixtures retained at %s\n' "$CASE_NAME" "$TEMPORARY" >&2
        tail -n 80 "$CASE_LOG" >&2
    fi
}
trap finish EXIT

# Exercise actual Git indexes and rustfmt, but never create a commit or tag.
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_COMMON_DIR
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1

create_fixture() {
    mkdir -p "$FIXTURE/crates/ic-backup/src" "$FIXTURE/scripts/hooks" "$FIXTURE/.githooks"
    cp "$ROOT/.githooks/pre-commit" "$FIXTURE/.githooks/"
    cp "$ROOT/scripts/hooks/install.sh" "$FIXTURE/scripts/hooks/"
    cp "$ROOT/Makefile" "$FIXTURE/"
    cp "$ROOT/rust-toolchain.toml" "$FIXTURE/"
    cat > "$FIXTURE/crates/ic-backup/Cargo.toml" <<'TOML'
[package]
name = "hook-fixture"
version.workspace = true
edition.workspace = true
TOML
    cat > "$FIXTURE/Cargo.toml" <<'TOML'
[workspace]
members = ["crates/ic-backup"]

[workspace.package]
version = "0.0.0"
edition = "2024"
TOML
    printf 'pub fn answer()->u32{42}\n' > "$FIXTURE/crates/ic-backup/src/lib.rs"
    cd "$FIXTURE"
    git init --quiet
    git add -- Makefile rust-toolchain.toml Cargo.toml crates scripts .githooks
    bash scripts/hooks/install.sh
    [[ "$(git config --local --get core.hooksPath)" == .githooks ]]
}

index_fingerprint() { git ls-files --stage | sha256sum; }
sources_fingerprint() { sha256sum crates/ic-backup/src/lib.rs; }

expect_hook_failure() {
    if git hook run pre-commit > rejection.log 2>&1; then
        echo 'expected hook to reject commit' >&2
        exit 1
    fi
    cat rejection.log
}

test_format_and_retry() {
    local before
    before="$(index_fingerprint)"
    expect_hook_failure
    rg -q 'review, stage the formatting' rejection.log
    [[ "$(index_fingerprint)" == "$before" ]]
    make --no-print-directory fmt-check
    if git diff --quiet -- crates/ic-backup/src/lib.rs; then
        echo 'formatter did not change the staged fixture source' >&2
        exit 1
    fi
    git add -- crates/ic-backup/src/lib.rs
    before="$(index_fingerprint)"
    local sources
    sources="$(sources_fingerprint)"
    git hook run pre-commit
    [[ "$(index_fingerprint)" == "$before" && "$(sources_fingerprint)" == "$sources" ]]
    git diff --exit-code
}

test_unstaged_protection() {
    local index_before sources_before
    case "$1" in
        root) printf '\n// Unstaged edit.\n' >> crates/ic-backup/src/lib.rs ;;
        manifest) printf '\n# Unstaged config.\n' >> crates/ic-backup/Cargo.toml ;;
    esac
    index_before="$(index_fingerprint)"
    sources_before="$(sources_fingerprint)"
    expect_hook_failure
    rg -q 'unstaged Rust sources/configuration' rejection.log
    [[ "$(index_fingerprint)" == "$index_before" && "$(sources_fingerprint)" == "$sources_before" ]]
}

test_formatter_failure() {
    mkdir failing-bin
    printf '#!/usr/bin/env bash\nexit 13\n' > failing-bin/cargo
    chmod +x failing-bin/cargo
    local before
    before="$(index_fingerprint)"
    export PATH="$FIXTURE/failing-bin:$PATH"
    expect_hook_failure
    [[ "$(index_fingerprint)" == "$before" ]]
    rg -q 'Error 13' rejection.log
}

test_existing_hooks() {
    git config --local core.hooksPath custom-hooks
    if bash scripts/hooks/install.sh > rejection.log 2>&1; then
        echo 'installer overwrote existing hooks configuration' >&2
        exit 1
    fi
    [[ "$(git config --local --get core.hooksPath)" == custom-hooks ]]
    rg -q 'already custom-hooks' rejection.log
}

run_case() {
    CASE_NAME="$1"
    shift
    FIXTURE="$TEMPORARY/$CASE_NAME"
    CASE_LOG="$TEMPORARY/$CASE_NAME.log"
    # Do not place the subshell in an if/OR-list: assertions must keep errexit.
    (
        trap 'printf "Failed at line %s: %s\n" "$LINENO" "$BASH_COMMAND" >&2' ERR
        create_fixture
        "$@"
    ) > "$CASE_LOG" 2>&1
}

run_case format-and-retry test_format_and_retry
for location in root manifest; do
    run_case "unstaged-$location" test_unstaged_protection "$location"
done
run_case formatter-failure test_formatter_failure
run_case existing-hooks test_existing_hooks
echo 'Hook tests: PASS (format/retry, index preservation, unstaged protection, formatter failure, hook installation).'
