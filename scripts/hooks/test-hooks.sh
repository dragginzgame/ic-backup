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
    mkdir -p "$FIXTURE/crates/ic-backup/src" "$FIXTURE/scripts/dev" "$FIXTURE/scripts/ci" "$FIXTURE/ci" "$FIXTURE/.githooks" "$FIXTURE/make"
    cp "$ROOT/scripts/ci/check-format-tools.sh" "$ROOT/scripts/ci/check-make-execution.sh" "$FIXTURE/scripts/ci/"
    cp "$ROOT/make/tools.mk" "$FIXTURE/make/"
    cp "$ROOT/ci/tool-versions.env" "$FIXTURE/ci/"
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
    git add -- Makefile rust-toolchain.toml Cargo.toml crates scripts make .githooks README.md
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

test_checkout_local_tools() {
    local original_tool before sources
    original_tool="$(command -v cargo-sort)"
    mkdir -p .tools/rust/bin fallback-bin
    cp "$original_tool" .tools/rust/bin/cargo-sort
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
    [[ "$(git show :crates/ic-backup/src/lib.rs)" == "$(cat crates/ic-backup/src/lib.rs)" ]]
    [[ "$(cat README.md)" == 'unformatted unrelated working edit' ]]
    [[ -z "$(git ls-files -- .tools fallback-bin)" ]]
    for state in missing wrong; do
        printf 'pub fn answer()->u32{42}\n' > crates/ic-backup/src/lib.rs
        git add -- crates/ic-backup/src/lib.rs
        if [[ "$state" == missing ]]; then
            rm .tools/rust/bin/cargo-sort
        else
            cp fallback-bin/cargo-sort .tools/rust/bin/cargo-sort
        fi
        before="$(index_fingerprint)"
        sources="$(sources_fingerprint)"
        expect_hook_failure
        [[ "$(index_fingerprint)" == "$before" && "$(sources_fingerprint)" == "$sources" ]]
        [[ "$(cat README.md)" == 'unformatted unrelated working edit' ]]
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
        trap 'printf "Failed at line %s: %s\n" "$LINENO" "$BASH_COMMAND" >&2' ERR
        create_fixture
        "$@"
    ) > "$CASE_LOG" 2>&1
}

# Qualify the exact shared checker with our real formatter inputs, including
# uncommitted module files. Native released consumer coverage qualifies this
# owner; the independent exact formatter-status case remains local.
CASE_NAME=shared-formatting
CASE_LOG="$TEMPORARY/$CASE_NAME.log"
printf '%s\n' "$CASE_NAME" >> "$TEMPORARY/cases.txt"
perl -0777 -pe 's/^(candid\.workspace[^\n]*)\n(ic-host-artifacts\.workspace[^\n]*)$/$2\n$1/m or die "expected dependency ordering fixture\n"' \
    "$ROOT/crates/ic-backup/Cargo.toml" > "$TEMPORARY/unsorted-Cargo.toml"
formatter_inputs=(Cargo.toml Cargo.lock rust-toolchain.toml ci/tool-versions.env scripts/ci/check-format-tools.sh make/tools.mk)
while IFS= read -r -d '' path; do
    formatter_inputs[${#formatter_inputs[@]}]="$path"
done < <(git -C "$ROOT" ls-files -z --cached --others --exclude-standard -- '*.rs')
bash "$ROOT/scripts/ci/check-formatting-hooks.sh" "$ROOT" \
    crates/ic-backup/src/lib.rs crates/ic-backup/Cargo.toml \
    "$TEMPORARY/unsorted-Cargo.toml" "${formatter_inputs[@]}" > "$CASE_LOG" 2>&1

run_case formatter-failure test_formatter_failure
run_case checkout-local-tools test_checkout_local_tools
echo 'Hook tests: PASS (selected auto-formatting, index refresh, partial-stage rejection, unrelated edit preservation, formatter failure and local installation).'
