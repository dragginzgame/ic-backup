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
        printf 'Hook fixtures retained: %s\n' "$TEMPORARY"
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
    mkdir -p "$FIXTURE/crates/ic-backup/src" "$FIXTURE/scripts/dev" "$FIXTURE/.githooks"
    cp "$ROOT/.githooks/pre-commit" "$FIXTURE/.githooks/"
    cp "$ROOT/scripts/dev/install-git-hooks.sh" "$FIXTURE/scripts/dev/"
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
    git -C "$ROOT" show HEAD:README.md > "$FIXTURE/README.md"
    cd "$FIXTURE"
    git init --quiet
    # Borrow the existing source commit read-only; never create fixture commits.
    mkdir -p .git/objects/info
    printf '%s\n' "$(git -C "$ROOT" rev-parse --path-format=absolute --git-path objects)" > .git/objects/info/alternates
    git update-ref HEAD "$(git -C "$ROOT" rev-parse HEAD)"
    git read-tree HEAD
    # The current source retires the old symlink, which may remain in HEAD
    # before maintainer commit. Qualify the maintained regular license bytes.
    cp "$ROOT/crates/ic-backup/LICENSE" crates/ic-backup/LICENSE
    git add -- Makefile rust-toolchain.toml Cargo.toml crates scripts .githooks README.md
    make --no-print-directory install-hooks
    [[ "$(git config --local --get core.hooksPath)" == .githooks ]]
}

checksum() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$@"
    else
        shasum -a 256 "$@"
    fi
}

index_fingerprint() { git ls-files --stage | checksum; }
sources_fingerprint() { checksum Cargo.toml crates/ic-backup/Cargo.toml crates/ic-backup/src/lib.rs; }

expect_hook_failure() {
    if git hook run pre-commit > rejection.log 2>&1; then
        echo 'expected hook to reject commit' >&2
        exit 1
    fi
    cat rejection.log
}

test_format_and_retry() {
    local before readme
    readme="$(git show :README.md)"
    printf 'Unrelated working edit.\n' >> README.md
    printf 'Untracked retained content.\n' > untracked.rs
    before="$(index_fingerprint)"
    git hook run pre-commit
    [[ "$(index_fingerprint)" != "$before" ]]
    [[ "$(git show :README.md)" == "$readme" ]]
    [[ "$(cat untracked.rs)" == 'Untracked retained content.' && -z "$(git ls-files -- untracked.rs)" ]]
    rg -q '^Unrelated working edit.$' README.md
    make --no-print-directory fmt-check
    git diff --exit-code -- crates/ic-backup/src/lib.rs Cargo.toml crates/ic-backup/Cargo.toml
    before="$(index_fingerprint)"
    local sources
    sources="$(sources_fingerprint)"
    git hook run pre-commit
    [[ "$(index_fingerprint)" == "$before" && "$(sources_fingerprint)" == "$sources" ]]
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
    rg -q 'partially staged file' rejection.log
    [[ "$(index_fingerprint)" == "$index_before" && "$(sources_fingerprint)" == "$sources_before" ]]
}

test_formatter_failure() {
    mkdir failing-bin
    export HOOK_TEST_REAL_CARGO
    HOOK_TEST_REAL_CARGO="$(command -v cargo)"
    cat > failing-bin/cargo <<'CARGO'
#!/usr/bin/env bash
if [[ "$*" == 'fmt --all' ]]; then exit 13; fi
exec "$HOOK_TEST_REAL_CARGO" "$@"
CARGO
    chmod +x failing-bin/cargo
    local before sources
    before="$(index_fingerprint)"
    sources="$(sources_fingerprint)"
    export PATH="$FIXTURE/failing-bin:$PATH"
    expect_hook_failure
    [[ "$(index_fingerprint)" == "$before" && "$(sources_fingerprint)" == "$sources" ]]
    rg -q 'Error 13' rejection.log
}

test_existing_hooks() {
    git config --local core.hooksPath custom-hooks
    if make --no-print-directory install-hooks > rejection.log 2>&1; then
        echo 'installer overwrote existing hooks configuration' >&2
        exit 1
    fi
    [[ "$(git config --local --get core.hooksPath)" == custom-hooks ]]
    rg -q 'refused to replace core.hooksPath=custom-hooks' rejection.log
}

test_symlink_checkout() {
    local linked="$TEMPORARY/linked-checkout" before
    before="$(index_fingerprint)"
    ln -s "$FIXTURE" "$linked"
    bash "$linked/scripts/dev/install-git-hooks.sh"
    [[ "$(git config --local --get core.hooksPath)" == .githooks ]]
    [[ "$(index_fingerprint)" == "$before" ]]
}

run_case() {
    CASE_NAME="$1"
    shift
    FIXTURE="$TEMPORARY/$CASE_NAME"
    CASE_LOG="$TEMPORARY/$CASE_NAME.log"
    printf '%s\n' "$CASE_NAME" >> "$TEMPORARY/cases.txt"
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
run_case symlink-checkout test_symlink_checkout
echo 'Hook tests: PASS (selected auto-formatting, index refresh, partial-stage rejection, unrelated edit preservation, formatter failure and local installation).'
