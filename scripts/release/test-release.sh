#!/usr/bin/env bash
set -Eeuo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TEST_REAL_MAKE="$(command -v make)"
export TEST_REAL_MAKE
mkdir -p "$ROOT/target"
TEMPORARY="$(mktemp -d "$ROOT/target/release-tests.XXXXXX")"
mkdir -p "$TEMPORARY/bin"
RUNNER_PID="$BASHPID"
CASE_NAME=setup
CASE_LOG=/dev/null
export TEST_LOG=/dev/null TEST_EFFECTS=/dev/null
export TEST_SOURCE=1111111111111111111111111111111111111111

finish() {
    local status=$?
    [[ "$BASHPID" == "$RUNNER_PID" ]] || return "$status"
    if [[ "$status" == 0 ]]; then
        rm -rf "$TEMPORARY"
    else
        printf 'Release-helper tests: FAIL [%s]\n' "$CASE_NAME" >&2
        tail -n 80 "$CASE_LOG" >&2
        printf 'Fixture command trace:\n' >&2
        tail -n 40 "$TEST_LOG" >&2
        printf 'Full logs and isolated fixture retained: %s\n' "$TEMPORARY" >&2
    fi
}
trap finish EXIT

run_case() {
    CASE_NAME="$1"
    shift
    CASE_LOG="$TEMPORARY/$CASE_NAME.log"
    FIXTURE="$TEMPORARY/$CASE_NAME"
    DATA="$FIXTURE/scripts/release/release-data.pl"
    TEST_LOG="$FIXTURE/commands.log"
    TEST_EFFECTS="$FIXTURE/effects.log"
    # Do not wrap this subshell in if/! or an OR-list: those suppress errexit
    # inside test functions and can turn failed assertions into passing cases.
    (
        trap 'printf "Failed at line %s: %s\n" "$LINENO" "$BASH_COMMAND" >&2' ERR
        create_fixture
        "$@"
    ) >"$CASE_LOG" 2>&1
}

create_fixture() {
    mkdir "$FIXTURE"
    mkdir -p "$FIXTURE/scripts/release" "$FIXTURE/docs" "$FIXTURE/target/debug" "$FIXTURE/crates/ic-backup"
    cp "$ROOT/scripts/release/release.sh" "$ROOT/scripts/release/release-data.pl" "$FIXTURE/scripts/release/"
    cd "$FIXTURE"
    cat > Cargo.toml <<'TOML'
[workspace]
members = ["crates/ic-backup"]
[workspace.package]
version = "0.1.0"
edition = "2024"
publish = ["crates-io"]
TOML
    cat > crates/ic-backup/Cargo.toml <<'TOML'
[package]
name = "ic-backup"
version.workspace = true
edition.workspace = true
publish.workspace = true
TOML
    cat > Cargo.lock <<'LOCK'
version = 4

[[package]]
name = "ic-backup"
version = "0.1.0"
LOCK
    cat > CHANGELOG.md <<'NOTES'
# Changelog

## [Unreleased]

- Test release notes.
NOTES
    : > "$TEST_LOG"
    : > "$TEST_EFFECTS"
    printf 'retained build artifact\n' > target/debug/cache-sentinel
    unset TEST_DIRTY TEST_TAG_EXISTS TEST_GATE_FAIL TEST_UPDATE_FAIL TEST_GATE_DIRTY TEST_GATE_HEAD TEST_METADATA_FAIL TEST_PUSH_FAIL TEST_FETCH_FAIL TEST_PUBLISH_FAIL
}

expect_failure() {
    local status
    if "$@" >target/rejection.log 2>&1; then
        echo "expected rejection: $*" >&2
        cat target/rejection.log >&2
        exit 1
    else
        status=$?
    fi
    cat target/rejection.log
    # A missing executable or an unsupported substitute command is a broken
    # fixture, not proof that the release guard rejected the requested action.
    case "$status" in 1 | 255) ;; *) echo "unexpected rejection status: $status" >&2; exit 1 ;; esac
}

fingerprint() {
    sha256sum Cargo.toml Cargo.lock crates/ic-backup/Cargo.toml CHANGELOG.md
}

assert_unchanged() {
    [[ "$(fingerprint)" == "$before" ]]
    [[ ! -e docs/release.json ]]
}

assert_cache_retained() {
    [[ "$(cat target/debug/cache-sentinel)" == 'retained build artifact' ]]
}

# These substitutes exercise local sequencing and failure handling without
# creating commits, tags, network requests, or registry publications.
cat > "$TEMPORARY/bin/git" <<'MOCK'
#!/usr/bin/env bash
set -euo pipefail
echo "git $*" >> "$TEST_LOG"
case "$*" in
    'rev-parse --verify HEAD' | 'rev-parse HEAD')
        if [[ -f target/mock-head ]]; then
            cat target/mock-head
        elif [[ -n "${TEST_GATE_HEAD:-}" && -f target/gate-ran ]]; then
            echo 2222222222222222222222222222222222222222
        else
            echo "$TEST_SOURCE"
        fi
        ;;
    'rev-parse --verify --quiet refs/tags/'*) [[ "${TEST_TAG_EXISTS:-0}" == 1 ]] ;;
    'status --porcelain --untracked-files=all')
        if [[ "${TEST_DIRTY:-0}" == 1 ]] ||
            [[ "${TEST_GATE_DIRTY:-0}" == 1 && -f target/gate-ran ]]; then
            echo ' M src/lib.rs'
        fi
        ;;
    'tag --list v*') ;;
    'merge-base --is-ancestor '*) ;;
    'diff --name-only '*) printf '%s\n' Cargo.toml Cargo.lock CHANGELOG.md ;;
    'ls-files --others --exclude-standard') echo docs/release.json ;;
    'add -- Cargo.toml Cargo.lock CHANGELOG.md docs/release.json')
        echo stage >> "$TEST_EFFECTS"
        touch target/mock-staged
        ;;
    'diff --quiet') ;;
    'diff --cached --quiet') [[ ! -f target/mock-staged ]] ;;
    'commit -m Release '*)
        echo commit >> "$TEST_EFFECTS"
        [[ -f target/mock-staged ]]
        echo 3333333333333333333333333333333333333333 > target/mock-head
        rm target/mock-staged
        ;;
    'tag -a v'*)
        echo tag >> "$TEST_EFFECTS"
        [[ -f target/mock-head ]]
        cp target/mock-head target/mock-tag
        ;;
    'rev-parse --verify refs/tags/'*) cat target/mock-tag ;;
    'cat-file -t refs/tags/'*) echo tag ;;
    'rev-parse HEAD^') echo "$TEST_SOURCE" ;;
    'symbolic-ref --quiet --short HEAD') echo main ;;
    'remote get-url origin') echo 'https://invalid.example/no-network-fixture.git' ;;
    'push --atomic origin HEAD:refs/heads/main refs/tags/v'*)
        echo push >> "$TEST_EFFECTS"
        [[ "$*" == "push --atomic origin HEAD:refs/heads/main refs/tags/v$(perl scripts/release/release-data.pl version)" ]]
        [[ "${TEST_PUSH_FAIL:-0}" != 1 ]]
        ;;
    *) echo "unexpected Git command: $*" >&2; exit 97 ;;
esac
MOCK
cat > "$TEMPORARY/bin/make" <<'MOCK'
#!/usr/bin/env bash
set -euo pipefail
echo "make $*" >> "$TEST_LOG"
[[ "$*" == '--no-print-directory release-verify' ]] || exit 97
echo validate >> "$TEST_EFFECTS"
[[ "${TEST_GATE_FAIL:-0}" != 1 ]]
touch target/gate-ran
MOCK
cat > "$TEMPORARY/bin/cargo" <<'MOCK'
#!/usr/bin/env bash
set -euo pipefail
echo "cargo $*" >> "$TEST_LOG"
case "$*" in
    'fetch --locked')
        echo fetch >> "$TEST_EFFECTS"
        [[ "${TEST_FETCH_FAIL:-0}" != 1 ]]
        touch target/mock-cargo-cache
        ;;
    'check --offline --locked -p ic-backup --all-targets --all-features')
        [[ -f target/mock-cargo-cache ]]
        echo check >> "$TEST_EFFECTS"
        ;;
    'update --offline -p ic-backup')
        [[ "${TEST_UPDATE_FAIL:-0}" != 1 ]]
        next="$(perl scripts/release/release-data.pl version)"
        sed -i "s/^version = \"0.1.0\"$/version = \"$next\"/" Cargo.lock
        ;;
    'metadata --offline --locked --no-deps --format-version 1')
        [[ "${TEST_METADATA_FAIL:-0}" != 1 ]]
        echo '{}'
        ;;
    'publish --locked --registry crates-io -p ic-backup' | \
    'publish --locked --registry crates-io -p ic-backup --dry-run')
        if [[ "${TEST_PUBLISH_FAIL:-0}" == 1 ]]; then
            echo 'error: simulated Cargo package rejection' >&2
            exit 101
        fi
        echo publish >> "$TEST_EFFECTS"
        ;;
    clean) echo clean >> "$TEST_EFFECTS"; rm -rf target/debug ;;
    *) echo "unexpected Cargo command: $*" >&2; exit 97 ;;
esac
MOCK
chmod +x "$TEMPORARY/bin/"*
export PATH="$TEMPORARY/bin:$PATH"

test_dependency_bootstrap() {
    # Exercise the actual Make gate/recipes against an initially empty cache.
    # Skip unrelated validation; neither network nor compilation is needed.
    cp "$ROOT/Makefile" Makefile
    cat > target/gate-make <<'MOCK'
#!/usr/bin/env bash
set -euo pipefail
echo "gate $*" >> "$TEST_LOG"
case "$*" in
    '--no-print-directory deps' | '--no-print-directory check')
        exec "$TEST_REAL_MAKE" "$@" ;;
esac
MOCK
    chmod +x target/gate-make
    before="$(fingerprint)"
    if [[ "$1" == failure ]]; then
        export TEST_FETCH_FAIL=1
        if "$TEST_REAL_MAKE" --no-print-directory validate "MAKE=$FIXTURE/target/gate-make" >target/fetch-rejection.log 2>&1; then
            echo 'expected fetch failure to stop validation' >&2
            exit 1
        else
            [[ "$?" == 2 ]]
        fi
        cat target/fetch-rejection.log
        [[ "$(cat "$TEST_EFFECTS")" == fetch ]]
        [[ "$(rg '^gate ' "$TEST_LOG")" == 'gate --no-print-directory deps' ]]
        [[ ! -f target/mock-cargo-cache ]]
    else
        "$TEST_REAL_MAKE" --no-print-directory release-verify "MAKE=$FIXTURE/target/gate-make"
        printf '%s\n' fetch check >target/expected-effects
        diff -u target/expected-effects "$TEST_EFFECTS"
        [[ "$(rg '^gate ' "$TEST_LOG" | head -n 1)" == 'gate --no-print-directory deps' ]]
    fi
    assert_unchanged
    assert_cache_retained
}

test_versions() {
    local requested expected
    for requested in patch minor major 0.1.0; do
        case "$requested" in
            patch) expected=0.1.1 ;; minor) expected=0.2.0 ;;
            major) expected=1.0.0 ;; 0.1.0) expected=0.1.0 ;;
        esac
        [[ "$(perl "$DATA" next "$requested")" == "$expected" ]]
    done
    for requested in 0.0.9 01.2.3 0.1.1-extra; do
        expect_failure perl "$DATA" next "$requested"
    done
}

test_invalid_changelog() {
    case "$1" in
        duplicate) printf '\n## [Unreleased]\n' >> CHANGELOG.md ;;
        empty) printf '# Changelog\n\n## [Unreleased]\n' > CHANGELOG.md ;;
        competing)
            perl "$DATA" set-version 0.9.0
            cat >> CHANGELOG.md <<'NOTES'

## [0.10.0]

- A different future draft must still block this patch.

## [0.9.0]

- Current undated release.
NOTES
            ;;
    esac
    before="$(fingerprint)"
    expect_failure perl "$DATA" changelog-check "$(perl "$DATA" next patch)" 2026-09-25
    assert_unchanged
}

test_preparation() {
    if [[ "$1" == named ]]; then
        cat > CHANGELOG.md <<'NOTES'
# Changelog

## [Unreleased]

## [0.1.1]

- Named release.
NOTES
    fi
    # Undated imported history remains supported regardless of today's version.
    cat >> CHANGELOG.md <<'NOTES'

## [0.1.0]

- Initial undated release; keep these bytes unchanged.

## [0.0.9]

- Earlier imported history.
NOTES
    sed -n '/^## \[0.1.0\]$/,$p' CHANGELOG.md > target/historical-notes
    before="$(fingerprint)"
    perl "$DATA" changelog-check 0.1.1 2026-09-25
    assert_unchanged
    bash scripts/release/release.sh bump 0.1.1
    [[ "$(perl "$DATA" version)" == 0.1.1 ]]
    rg -q '^version = "0.1.1"$' Cargo.lock
    perl "$DATA" verify
    [[ "$(perl "$DATA" source)" == "$TEST_SOURCE" ]]
    cmp target/historical-notes <(sed -n '/^## \[0.1.0\]$/,$p' CHANGELOG.md)
    expect_failure perl "$DATA" finalize 0.1.1 2026-09-25
    expect_failure bash scripts/release/release.sh bump 0.1.1
    # This must hold for each fixture, not just the last reset in the suite.
    [[ "$(cat "$TEST_EFFECTS")" == validate ]]
    assert_cache_retained
}

test_rejected_preparation() {
    before="$(fingerprint)"
    export "$1=1"
    expect_failure bash scripts/release/release.sh bump patch
    assert_unchanged
    case "$(cat "$TEST_EFFECTS")" in
        "" | validate) ;;
        *) echo "rejected preparation caused release effects" >&2; exit 1 ;;
    esac
    assert_cache_retained
}

test_staging() {
    bash scripts/release/release.sh bump patch
    : > "$TEST_EFFECTS"
    bash scripts/release/release.sh stage
    [[ -f target/mock-staged && "$(cat "$TEST_EFFECTS")" == stage ]]
    printf '\n- Unvalidated change.\n' >> CHANGELOG.md
    rm target/mock-staged
    expect_failure bash scripts/release/release.sh stage
    [[ ! -f target/mock-staged ]]
}

test_initial_version() {
    bash scripts/release/release.sh bump 0.1.0
    [[ "$(perl "$DATA" version)" == 0.1.0 ]]
    perl "$DATA" verify
    [[ "$(cat "$TEST_EFFECTS")" == validate ]]
}

prepare_tagged_release() {
    bash scripts/release/release.sh release minor
}

test_release() {
    local requested="$1" expected
    case "$requested" in
        patch) expected=0.1.1 ;;
        minor) expected=0.2.0 ;;
        major) expected=1.0.0 ;;
        *) echo "unexpected release kind: $requested" >&2; exit 1 ;;
    esac
    cp "$ROOT/Makefile" Makefile
    # Run the real public Make target; its release effects remain substituted.
    "$TEST_REAL_MAKE" --no-print-directory "release-$requested"
    [[ "$(perl "$DATA" version)" == "$expected" ]]
    perl "$DATA" verify
    # Exact observable effects; incidental read-only commands do not matter.
    printf '%s\n' validate stage commit tag push > target/expected-effects
    diff -u target/expected-effects "$TEST_EFFECTS"
    assert_cache_retained
}

test_publish() {
    local mode="$1" state="$2" expected receipt_before
    case "$state" in
        no-release) ;;
        tagged) prepare_tagged_release ;;
        stale-changelog)
            prepare_tagged_release
            printf '\n- Next development notes.\n' >> CHANGELOG.md
            ;;
        publication-config)
            sed -i 's/publish = \["crates-io"\]/publish = false/' Cargo.toml
            prepare_tagged_release
            # A committed publishing correction after a repository release
            # leaves its receipt and tag behind without changing crate version.
            sed -i 's/publish = false/publish = ["crates-io"]/' Cargo.toml
            echo 4444444444444444444444444444444444444444 > target/mock-head
            ;;
        missing-tag)
            prepare_tagged_release
            rm target/mock-tag
            ;;
        missing-receipt)
            prepare_tagged_release
            rm docs/release.json
            ;;
        *) echo "unexpected publication state: $state" >&2; exit 1 ;;
    esac
    : > "$TEST_LOG"
    : > "$TEST_EFFECTS"
    before="$(fingerprint)"
    receipt_before="$(if [[ -f docs/release.json ]]; then sha256sum docs/release.json; fi)"
    cp "$ROOT/Makefile" Makefile
    if [[ "$mode" == dry-run ]]; then
        "$TEST_REAL_MAKE" --no-print-directory publish-dry-run
        expected='cargo publish --locked --registry crates-io -p ic-backup --dry-run'
    else
        "$TEST_REAL_MAKE" --no-print-directory publish
        expected='cargo publish --locked --registry crates-io -p ic-backup'
    fi
    [[ "$(cat "$TEST_LOG")" == "$expected" ]]
    [[ "$(cat "$TEST_EFFECTS")" == publish ]]
    [[ "$(fingerprint)" == "$before" ]]
    [[ "$(if [[ -f docs/release.json ]]; then sha256sum docs/release.json; fi)" == "$receipt_before" ]]
    assert_cache_retained
}

test_publish_failure() {
    : > "$TEST_LOG"
    : > "$TEST_EFFECTS"
    before="$(fingerprint)"
    export TEST_PUBLISH_FAIL=1
    local args=()
    [[ "$1" == dry-run ]] && args=(--dry-run)
    if bash scripts/release/release.sh publish "${args[@]}" >target/cargo-rejection.log 2>&1; then
        echo 'expected Cargo publication failure' >&2
        exit 1
    else
        [[ "$?" == 101 ]]
    fi
    rg -q '^error: simulated Cargo package rejection$' target/cargo-rejection.log
    [[ "$(cat "$TEST_LOG")" == "cargo publish --locked --registry crates-io -p ic-backup${args[*]:+ ${args[*]}}" ]]
    [[ ! -s "$TEST_EFFECTS" ]]
    assert_unchanged
    assert_cache_retained
}

test_invalid_release() {
    prepare_tagged_release
    : > "$TEST_LOG"
    : > "$TEST_EFFECTS"
    case "$1" in
        dirty) export TEST_DIRTY=1 ;;
        missing-tag) rm target/mock-tag ;;
        changed-release) printf '\n- Changed after release.\n' >> CHANGELOG.md ;;
        changed-library-manifest) printf '\n# Changed after release.\n' >> crates/ic-backup/Cargo.toml ;;
    esac
    expect_failure bash scripts/release/release.sh push
    [[ ! -s "$TEST_EFFECTS" ]]
    assert_cache_retained
}

test_push_retry() {
    export TEST_PUSH_FAIL=1
    expect_failure bash scripts/release/release.sh release patch
    [[ -f target/mock-head && -f target/mock-tag ]]
    perl "$DATA" verify
    assert_cache_retained
    before="$(fingerprint)"
    : > "$TEST_EFFECTS"
    unset TEST_PUSH_FAIL
    bash scripts/release/release.sh push
    # Retrying a push must not repeat validation, commit or tag creation.
    [[ "$(cat "$TEST_EFFECTS")" == push ]]
    [[ "$(fingerprint)" == "$before" ]]
    assert_cache_retained
}

echo "Release-helper tests: isolated fixtures; no real commits, tags or uploads."
for outcome in success failure; do
    run_case "dependency-bootstrap-$outcome" test_dependency_bootstrap "$outcome"
done
run_case versions test_versions
for invalid in duplicate empty competing; do
    run_case "changelog-$invalid" test_invalid_changelog "$invalid"
done
for notes in unreleased named; do
    run_case "prepare-$notes" test_preparation "$notes"
done
for failure in TEST_DIRTY TEST_TAG_EXISTS TEST_GATE_FAIL TEST_UPDATE_FAIL TEST_GATE_DIRTY TEST_GATE_HEAD TEST_METADATA_FAIL; do
    run_case "reject-$failure" test_rejected_preparation "$failure"
done
run_case staging test_staging
run_case initial-version test_initial_version
for kind in patch minor major; do
    run_case "release-$kind" test_release "$kind"
done
for mode in dry-run upload; do
    for state in no-release tagged stale-changelog publication-config missing-tag missing-receipt; do
        run_case "publish-$mode-$state" test_publish "$mode" "$state"
    done
    run_case "publish-$mode-cargo-failure" test_publish_failure "$mode"
done
for invalid in dirty missing-tag changed-release changed-library-manifest; do
    run_case "release-push-$invalid" test_invalid_release "$invalid"
done
run_case push-retry test_push_retry
echo "Release-helper tests: PASS (dependency bootstrap, preparation, rollback, publication and retry)."
