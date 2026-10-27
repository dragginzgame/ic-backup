#!/usr/bin/env bash
set -Eeuo pipefail

ROOT="${BASH_SOURCE[0]}"
[[ "$ROOT" == /* ]] || ROOT="$PWD/$ROOT"
ROOT="$(cd -P "${ROOT%/*}/../.." && printf '%s/.' "$PWD")"
ROOT="${ROOT%/.}"
mkdir -p "$ROOT/target"
TEMPORARY="$(mktemp -d "$ROOT/target/hook-tests.XXXXXX")"
CASE_NAME=setup
CASE_LOG=/dev/null
fixture_complete=false
finish() {
    local status=$?
    [[ "$fixture_complete" == true || "$status" != 0 ]] || status=1
    if [[ "$status" == 0 ]]; then
        printf 'Hook fixtures retained: %s\n' "$TEMPORARY"
    else
        printf 'Hook tests: FAIL [%s]; fixtures retained at %s\n' "$CASE_NAME" "$TEMPORARY" >&2
        tail -n 80 "$CASE_LOG" >&2
    fi
    exit "$status"
}
trap finish EXIT

# Exercise actual Git indexes and rustfmt, but never create a commit or tag.
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_COMMON_DIR
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1

create_fixture() {
    mkdir -p "$FIXTURE/crates/ic-backup/src" "$FIXTURE/scripts/dev" "$FIXTURE/scripts/ci" "$FIXTURE/ci" "$FIXTURE/.githooks" "$FIXTURE/make"
    bash "$ROOT/scripts/ci/prepare-script-fixture.sh" "$ROOT" "$FIXTURE" \
        Makefile make/consumer.mk ci/tool-versions.env .githooks/pre-commit \
        scripts/dev/install-git-hooks.sh rust-toolchain.toml
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
    git add -- Makefile rust-toolchain.toml Cargo.toml crates scripts make .githooks README.md
    make --no-print-directory install-hooks
    [[ "$(git config --local --get core.hooksPath)" == .githooks ]] || exit 1
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
    [[ "$(index_fingerprint)" == "$before" && "$(sources_fingerprint)" == "$sources" ]] || exit 1
    rg -q 'Error 13' rejection.log
}

test_dependency_role_groups() {
    cat >> Cargo.toml <<'TOML'

# Production dependencies
[workspace.dependencies]
zzz = "1"
aaa = "1"

# Tests and tooling only
yyy = "1"
bbb = "1"
TOML
    make --no-print-directory fmt > write.log 2>&1
    rg -U '# Production dependencies\n\[workspace.dependencies\]\naaa = "1"\nzzz = "1"\n\n# Tests and tooling only\nbbb = "1"\nyyy = "1"' Cargo.toml
    cp Cargo.toml sorted.toml
    make --no-print-directory fmt > second-write.log 2>&1
    cmp Cargo.toml sorted.toml
    make --no-print-directory fmt-check > check.log 2>&1
    perl -0777 -i -pe 's/aaa = "1"\nzzz = "1"/zzz = "1"\naaa = "1"/ or die "missing role fixture\n"' Cargo.toml
    cp Cargo.toml unsorted.toml
    if make --no-print-directory fmt-check > rejection.log 2>&1; then
        echo 'grouped check accepted unsorted production dependencies' >&2; exit 1
    fi
    cmp Cargo.toml unsorted.toml
}

test_formatting_output() {
    make --no-print-directory fmt > write.log 2>&1
    rg -q '^Manifest sorting \[\.\]\.\.\. ok \(' write.log
    rg -q '^Rust formatting \[\.\]\.\.\. ok \(' write.log
    local before sources lock_before
    before="$(index_fingerprint)"
    sources="$(sources_fingerprint)"
    printf '# unrelated retained lock\nversion = 4\n\n[[package]]\nname = "hook-fixture"\nversion = "0.0.0"\n' > Cargo.lock
    lock_before="$(checksum Cargo.lock)"
    make --no-print-directory fmt-check > check.log 2>&1
    rg -q '^Manifest sorting \[\.\]\.\.\. ok \(' check.log
    rg -q '^Rust formatting \[\.\]\.\.\. ok \(' check.log
    mkdir failing-bin diagnostics
    export HOOK_TEST_REAL_CARGO
    HOOK_TEST_REAL_CARGO="$(command -v cargo)"
    cat > failing-bin/cargo <<'CARGO'
#!/usr/bin/env bash
if [[ "$*" == 'sort --workspace --grouped --check' ]]; then
    echo 'sorter stdout diagnosis'
    echo 'sorter stderr diagnosis' >&2
    exit 17
fi
if [[ "$*" == 'fmt --all -- --check' ]]; then touch unexpected-rustfmt; fi
exec "$HOOK_TEST_REAL_CARGO" "$@"
CARGO
    chmod +x failing-bin/cargo
    if RUNNER_TEMP="$FIXTURE/diagnostics" make --no-print-directory fmt-check \
        "FORMAT_CARGO=$FIXTURE/failing-bin/cargo" > failure.log 2>&1; then
        echo 'expected formatting refusal' >&2; return 1
    fi
    rg -q '^Manifest sorting \[\.\]\.\.\. FAILED \(exit 17,' failure.log
    rg -q 'Details: .*formatting\.' failure.log
    local logs=(diagnostics/formatting.*)
    [[ ${#logs[@]} == 1 && -s "${logs[0]}" && ! -e unexpected-rustfmt ]] || exit 1
    rg -q 'sorter stdout diagnosis' "${logs[0]}"
    rg -q 'sorter stderr diagnosis' "${logs[0]}"
    [[ "$(index_fingerprint)" == "$before" && "$(sources_fingerprint)" == "$sources" ]] || exit 1
    [[ "$(checksum Cargo.lock)" == "$lock_before" ]] || exit 1
}

test_checkout_local_tools() {
    local original_tool before sources
    original_tool="$(command -v cargo-sort)"
    mkdir -p .tools/rust/common/bin fallback-bin
    cp "$original_tool" .tools/rust/common/bin/cargo-sort
    export HOOK_TEST_REAL_CARGO
    HOOK_TEST_REAL_CARGO="$(command -v cargo)"
    # Dispatch sort through PATH to expose the checkout/export distinction,
    # while preserving the real pinned sorter and Rust formatter behavior.
    cat > fallback-bin/cargo <<'CARGO'
#!/usr/bin/env bash
case "${1:-}" in
    sort) exec cargo-sort "$@" ;;
    *) exec "$HOOK_TEST_REAL_CARGO" "$@" ;;
esac
CARGO
    printf '#!/usr/bin/env bash\necho cargo-sort 0.0.0\n' > fallback-bin/cargo-sort
    chmod +x fallback-bin/cargo fallback-bin/cargo-sort
    export PATH="$FIXTURE/fallback-bin:$PATH"
    printf 'unformatted unrelated working edit\n' > README.md
    git hook run pre-commit
    rg -q --fixed-strings 'pub fn answer() -> u32' crates/ic-backup/src/lib.rs
    [[ "$(git show :crates/ic-backup/src/lib.rs)" == "$(cat crates/ic-backup/src/lib.rs)" ]] || exit 1
    [[ "$(cat README.md)" == 'unformatted unrelated working edit' ]] || exit 1
    [[ -z "$(git ls-files -- .tools fallback-bin)" ]] || exit 1
    for state in missing wrong; do
        printf 'pub fn answer()->u32{42}\n' > crates/ic-backup/src/lib.rs
        git add -- crates/ic-backup/src/lib.rs
        if [[ "$state" == missing ]]; then
            rm .tools/rust/common/bin/cargo-sort
        else
            cp fallback-bin/cargo-sort .tools/rust/common/bin/cargo-sort
        fi
        before="$(index_fingerprint)"
        sources="$(sources_fingerprint)"
        expect_hook_failure
        [[ "$(index_fingerprint)" == "$before" && "$(sources_fingerprint)" == "$sources" ]] || exit 1
        [[ "$(cat README.md)" == 'unformatted unrelated working edit' ]] || exit 1
    done
}

run_case() {
    CASE_NAME="$1"
    shift
    FIXTURE="$TEMPORARY/$CASE_NAME"
    CASE_LOG="$TEMPORARY/$CASE_NAME.log"
    printf '%s\n' "$CASE_NAME" >> "$TEMPORARY/cases.txt"
    # Do not place the subshell in an if/OR-list: assertions must keep errexit.
    (
        case_complete=false
        trap 'status=$?; [[ "$case_complete" == true || "$status" != 0 ]] || status=1; exit "$status"' EXIT
        trap 'printf "Failed at line %s: %s\n" "$LINENO" "$BASH_COMMAND" >&2' ERR
        create_fixture
        "$@"
        case_complete=true
    ) > "$CASE_LOG" 2>&1
}

# Qualify the exact shared checker with our real formatter inputs, including
# uncommitted module files. Native released consumer coverage qualifies this
# owner; the independent exact formatter-status case remains local.
CASE_NAME=shared-formatting
CASE_LOG="$TEMPORARY/$CASE_NAME.log"
printf '%s\n' "$CASE_NAME" >> "$TEMPORARY/cases.txt"
# Perturb the current inherited dependency table without retaining retired names.
perl -0777 -pe 's/^(\[dependencies\]\n)([A-Za-z0-9_-]+\.workspace[^\n]*\n)([A-Za-z0-9_-]+\.workspace[^\n]*\n)/$1$3$2/m or die "expected two inherited production dependencies\n"' \
    "$ROOT/crates/ic-backup/Cargo.toml" > "$TEMPORARY/unsorted-Cargo.toml" 2> "$CASE_LOG"
formatter_inputs=(Cargo.toml Cargo.lock rust-toolchain.toml ci/tool-versions.env scripts/ci/StableVersion.pm scripts/ci/ic-tool-pins.awk scripts/ci/tool-catalog-records.pl scripts/ci/check-format-tools.sh scripts/ci/run-formatting.sh scripts/ci/run-rust-formatting.pl make/consumer.mk make/release.mk make/tools.mk make/rust-format.mk make/execution.mk)
bash "$ROOT/scripts/ci/check-formatting-hooks.sh" "$ROOT" \
    crates/ic-backup/src/lib.rs crates/ic-backup/Cargo.toml \
    "$TEMPORARY/unsorted-Cargo.toml" --rust-sources "${formatter_inputs[@]}" > "$CASE_LOG" 2>&1

run_case formatter-failure test_formatter_failure
run_case dependency-role-groups test_dependency_role_groups
run_case formatting-output test_formatting_output
run_case checkout-local-tools test_checkout_local_tools
echo 'Hook tests: PASS (selected auto-formatting, index refresh, partial-stage rejection, unrelated edit preservation, formatter failure and local installation).'
fixture_complete=true
