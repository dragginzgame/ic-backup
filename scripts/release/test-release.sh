#!/usr/bin/env bash
set -Eeuo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
# Fixtures own their release identities and Make invocations. A real release
# exports both environment values and command-line overrides through GNU Make;
# neither may reach the fixture's standalone gates or select a parent helper.
# This child-only reset leaves the enclosing release and its evidence intact.
unset RELEASE_SOURCE RELEASE_PREVIOUS RELEASE_VERSION RELEASE_DATE RELEASE_KIND RELEASE_COMMIT \
    RELEASE_REMOTE RELEASE_BRANCH RELEASE_MAKE VERSION \
    MAKEFLAGS MAKEOVERRIDES MFLAGS MAKELEVEL
TEST_REAL_MAKE="$(command -v make)"
TEST_REAL_GIT="$(command -v git)"
export TEST_REAL_MAKE TEST_REAL_GIT
mkdir -p "$ROOT/target"
TEMPORARY="$(mktemp -d "$ROOT/target/shared-release-tests.XXXXXX")"
mkdir -p "$TEMPORARY/bin"
CASE_NAME=setup
CASE_LOG=/dev/null
export TEST_SOURCE=1111111111111111111111111111111111111111
finish() {
    local status=$?
    if [[ "$status" != 0 ]]; then
        printf 'Release adapter tests failed [%s]; retained: %s\n' "$CASE_NAME" "$TEMPORARY" >&2
        tail -n 80 "$CASE_LOG" >&2
    else
        printf 'Release adapter fixtures and command traces retained: %s\n' "$TEMPORARY"
    fi
}
trap finish EXIT
run_case() {
    CASE_NAME="$1"; shift
    CASE_LOG="$TEMPORARY/$CASE_NAME.log"
    printf '%s\n' "$CASE_NAME" >> "$TEMPORARY/cases.txt"
    export FIXTURE="$TEMPORARY/$CASE_NAME"
    (create_fixture; "$@") > "$CASE_LOG" 2>&1
}
create_fixture() {
    mkdir -p "$FIXTURE/scripts/release" "$FIXTURE/scripts/ci" "$FIXTURE/crates/ic-backup" "$FIXTURE/docs" "$FIXTURE/target/debug"
    cp "$ROOT/Makefile" "$FIXTURE/"
    cp "$ROOT/scripts/release/release.sh" "$ROOT/scripts/release/release-data.pl" "$FIXTURE/scripts/release/"
    cp "$ROOT/scripts/ci/run-release.sh" "$ROOT/scripts/ci/next-release-version.sh" "$ROOT/scripts/ci/run-validation-targets.sh" "$FIXTURE/scripts/ci/"
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
    printf 'version = 4\n\n[[package]]\nname = "ic-backup"\nversion = "0.1.0"\n\n[[package]]\nname = "retained-dependency"\nversion = "9.8.7"\n' > Cargo.lock
    printf '# Changelog\n\n## [0.1.1]\n\n- Completed notes.\n\n## [0.1.0] - 2026-10-01\n\n- Retained history.\n\n## [0.0.9]\n\n- Imported undated history.\n' > CHANGELOG.md
    sed -n '/^## \[0.1.0\]/,$p' CHANGELOG.md > target/history
    export TEST_LOG="$FIXTURE/target/commands.log" TEST_EFFECTS="$FIXTURE/target/effects.log"
    : > "$TEST_LOG"; : > "$TEST_EFFECTS"
    printf 'retained build artifact\n' > target/debug/cache-sentinel
    export VALIDATION_REPOSITORY_ROOT="$FIXTURE" VALIDATION_FAILURE_LOG_DIR="$FIXTURE/target/validation-failures" GITHUB_STEP_SUMMARY="$FIXTURE/target/summary.md"
    unset TEST_DIRTY TEST_TAG_EXISTS TEST_GATE_FAIL TEST_METADATA_FAIL TEST_PUSH_FAIL TEST_FETCH_FAIL TEST_PUBLISH_FAIL TEST_LOST_EFFECT TEST_GATE_DIRTY TEST_GATE_HEAD TEST_DESTINATION TEST_PREPARED_FORMAT_FAIL TEST_REMOTE_FAIL
}
expect_failure() {
    local status
    if "$@" > target/rejection.log 2>&1; then echo "unexpected acceptance: $*" >&2; exit 1; else status=$?; fi
    cat target/rejection.log
    [[ "$status" != 97 && "$status" != 126 && "$status" != 127 ]] || { echo 'broken substitute' >&2; exit 1; }
}
checksum() {
    if command -v sha256sum >/dev/null; then sha256sum "$@"; else shasum -a 256 "$@"; fi
}
fingerprint() { checksum Cargo.toml Cargo.lock crates/ic-backup/Cargo.toml CHANGELOG.md; }
assert_unchanged() { [[ "$(fingerprint)" == "$before" && ! -e docs/release.json ]]; }
assert_cache_retained() { [[ "$(cat target/debug/cache-sentinel)" == 'retained build artifact' ]]; }

cat > "$TEMPORARY/bin/git" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
printf 'git %s\n' "$*" >> "$TEST_LOG"
head() { if [[ -f target/mock-head ]]; then cat target/mock-head; else echo "$TEST_SOURCE"; fi; }
resolve() { if [[ "$1" == HEAD ]]; then head; else printf '%s\n' "$1"; fi; }
ancestor() {
    local cursor
    cursor="$(resolve "$2")"
    while [[ "$cursor" != "$1" ]]; do
        [[ -f "target/commits/$cursor/parent" ]] || return 1
        cursor="$(cat "target/commits/$cursor/parent")"
    done
}
dirty() {
    [[ "${TEST_DIRTY:-0}" != 1 ]] || return 0
    local current file
    current="$(head)"
    [[ -d "target/commits/$current" ]] || return 1
    for file in Cargo.toml Cargo.lock CHANGELOG.md docs/release.json crates/ic-backup/Cargo.toml; do
        cmp -s "$file" "target/commits/$current/files/$file" || return 0
    done
    return 1
}
case "$1" in
    check-ref-format) [[ "$2" == refs/heads/main ]] ;;
    symbolic-ref) echo main ;;
    remote) echo "${TEST_DESTINATION:-https://example.invalid/no-network-release}" ;;
    hash-object) exec "$TEST_REAL_GIT" hash-object --stdin ;;
    rev-parse)
        value="${*: -1}"
        case "$value" in
            --show-toplevel) pwd ;;
            release-state) echo target/release-state ;;
            HEAD) head ;;
            HEAD^) cat "target/commits/$(head)/parent" ;;
            *'^{tree}') echo dddddddddddddddddddddddddddddddddddddddd ;;
            'refs/tags/'*'^{commit}')
                [[ -f target/mock-tag ]]; name="${value#refs/tags/}"; name="${name%\^\{commit\}}"
                cat "target/tags/$name.commit" ;;
            refs/tags/*) [[ -f target/mock-tag ]]; cat "target/tags/${value#refs/tags/}.object" ;;
            *) [[ "$value" =~ ^[0-9a-f]{40}$ && -d "target/commits/$value" ]]; echo "$value" ;;
        esac ;;
    show)
        value="$2"; sha="${value%%:*}"; path="${value#*:}"
        [[ "$sha" =~ ^[0-9a-f]{40}$ ]]; cat "target/commits/$sha/files/$path" ;;
    status) if dirty; then echo ' M retained-source'; fi ;;
    diff)
        case "${2:-}" in
            --name-only) if [[ "${TEST_DIRTY:-0}" == 1 ]]; then printf 'src/changed.rs\0'; fi ;;
            --binary) cat Cargo.toml Cargo.lock CHANGELOG.md crates/ic-backup/Cargo.toml ;;
            --cached) [[ ! -f target/mock-staged ]] ;;
            --quiet) if [[ "${3:-}" == HEAD ]]; then ! dirty; else [[ "${TEST_DIRTY:-0}" != 1 ]]; fi ;;
            *) exit 97 ;;
        esac ;;
    ls-files) ;;
    rev-list)
        range="${*: -1}"; base="${range%..HEAD}"; cursor="$(head)"; history=''
        while [[ "$cursor" != "$base" ]]; do
            [[ -f "target/commits/$cursor/parent" ]] || exit 1
            history="$cursor${history:+$'\n'$history}"
            cursor="$(cat "target/commits/$cursor/parent")"
        done
        [[ -z "$history" ]] || printf '%s\n' "$history" ;;
    merge-base) ancestor "$3" "$4" ;;
    tag)
        case "$2" in
            --list) if [[ ( -f target/mock-tag && -f "target/tags/$3.object" ) || "${TEST_TAG_EXISTS:-0}" == 1 ]]; then echo "$3"; fi ;;
            -a)
                mkdir -p target/tags
                printf '%s\n' "$4" > "target/tags/$3.commit"
                printf '%s\n' "$3" | "$TEST_REAL_GIT" hash-object --stdin > "target/tags/$3.object"
                cp "target/tags/$3.object" target/mock-tag
                echo tag >> "$TEST_EFFECTS"
                if [[ "${TEST_LOST_EFFECT:-}" == tag && ! -e target/lost-tag ]]; then touch target/lost-tag; exit 9; fi ;;
            *) exit 97 ;;
        esac ;;
    cat-file) [[ -f target/mock-tag && -f "target/tags/${3#refs/tags/}.object" ]]; echo tag ;;
    ls-remote)
        [[ "${TEST_REMOTE_FAIL:-0}" != 1 ]] || exit 9
        for ref in "$@"; do
            case "$ref" in
                refs/heads/main) if [[ -f target/remote-head ]]; then printf '%s\t%s\n' "$(cat target/remote-head)" "$ref"; fi ;;
                refs/tags/*) if [[ -f "target/remote-tags/${ref#refs/tags/}" ]]; then printf '%s\t%s\n' "$(cat "target/remote-tags/${ref#refs/tags/}")" "$ref"; fi ;;
            esac
        done ;;
    add) [[ "$*" == 'add -- Cargo.toml Cargo.lock CHANGELOG.md docs/release.json' ]]; touch target/mock-staged; echo stage >> "$TEST_EFFECTS" ;;
    write-tree) echo dddddddddddddddddddddddddddddddddddddddd ;;
    log)
        sha="$(resolve "${*: -1}")"
        case "$3" in
            --format=%P) cat "target/commits/$sha/parent" ;;
            --format=%s) cat "target/commits/$sha/subject" ;;
            *) exit 97 ;;
        esac ;;
    commit)
        [[ "$*" == "commit -m Release $(perl scripts/release/release-data.pl version)" ]]
        parent="$(head)"; sha=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
        if [[ -d "target/commits/$sha" ]]; then sha="$(printf '%s\n' "$parent" "$3" | "$TEST_REAL_GIT" hash-object --stdin)"; fi
        mkdir -p "target/commits/$sha/files/crates/ic-backup" "target/commits/$sha/files/docs"
        printf '%s\n' "$parent" > "target/commits/$sha/parent"
        printf '%s\n' "$3" > "target/commits/$sha/subject"
        cp Cargo.toml Cargo.lock CHANGELOG.md "target/commits/$sha/files/"
        cp crates/ic-backup/Cargo.toml "target/commits/$sha/files/crates/ic-backup/"
        cp docs/release.json "target/commits/$sha/files/docs/"
        printf '%s\n' "$sha" > target/mock-head
        echo commit >> "$TEST_EFFECTS"
        if [[ "${TEST_LOST_EFFECT:-}" == commit && ! -e target/lost-commit ]]; then touch target/lost-commit; exit 9; fi ;;
    push)
        [[ "$#" == 6 && "$2" == --no-follow-tags && "$3" == --atomic && "$4" == origin && "$5" == *:refs/heads/main && "$6" == refs/tags/v*:refs/tags/v* ]]
        sha="$(resolve "${5%:refs/heads/main}")"; tag="${6%%:*}"; tag="${tag#refs/tags/}"
        if [[ -f target/remote-head ]]; then ancestor "$(cat target/remote-head)" "$sha"; fi
        printf '%s %s\n' "$sha" "$tag" >> target/pushes.log
        echo push >> "$TEST_EFFECTS"
        [[ "${TEST_PUSH_FAIL:-0}" != 1 ]] || exit 9
        echo "$sha" > target/remote-head
        mkdir -p target/remote-tags
        cp "target/tags/$tag.object" "target/remote-tags/$tag"
        cp "target/tags/$tag.object" target/remote-tag
        if [[ "${TEST_LOST_EFFECT:-}" == push && ! -e target/lost-push ]]; then touch target/lost-push; exit 9; fi ;;
    *) echo "unsupported Git substitute: $*" >&2; exit 97 ;;
esac

STUB
cat > "$TEMPORARY/bin/make" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
printf 'make %s\n' "$*" >> "$TEST_LOG"
target=''
for argument in "$@"; do
    case "$argument" in -*|*=*) ;; *) target="$argument" ;; esac
done
if [[ "$target" == shared-tooling-check ]]; then exit 0; fi
if [[ "$target" == fmt-check ]]; then
    echo prepared-format-check >> "$TEST_EFFECTS"
    [[ "${TEST_PREPARED_FORMAT_FAIL:-0}" != 1 ]]
    exit
fi
if [[ "$target" == release-verify ]]; then
    echo validate >> "$TEST_EFFECTS"
    [[ "${TEST_GATE_FAIL:-0}" != 1 ]] || exit 7
    if [[ "${TEST_GATE_DIRTY:-0}" == 1 ]]; then printf '\n- Changed in gate.\n' >> CHANGELOG.md; fi
    if [[ "${TEST_GATE_HEAD:-0}" == 1 ]]; then echo eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee > target/mock-head; fi
    exec "$TEST_REAL_MAKE" "$@" CI_TARGETS=check
fi
exec "$TEST_REAL_MAKE" "$@"
STUB
cat > "$TEMPORARY/bin/cargo" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
printf 'cargo %s\n' "$*" >> "$TEST_LOG"
case "$*" in
    'fetch --locked') echo fetch >> "$TEST_EFFECTS"; [[ "${TEST_FETCH_FAIL:-0}" != 1 ]] || exit 7; touch target/mock-cache ;;
    'check --offline --locked -p ic-backup --all-targets --all-features') [[ -f target/mock-cache ]] || exit 7; echo check >> "$TEST_EFFECTS" ;;
    'metadata --offline --locked --no-deps --format-version 1') [[ "${TEST_METADATA_FAIL:-0}" != 1 ]] || exit 7; echo '{}' ;;
    'publish --locked --registry crates-io -p ic-backup'|'publish --locked --registry crates-io -p ic-backup --dry-run')
        if [[ "${TEST_PUBLISH_FAIL:-0}" == 1 ]]; then exit 101; fi
        echo publish >> "$TEST_EFFECTS" ;;
    *) echo "unsupported Cargo substitute: $*" >&2; exit 97 ;;
esac
STUB
chmod +x "$TEMPORARY/bin/"*
export PATH="$TEMPORARY/bin:$PATH"

test_dependency_bootstrap() {
    before="$(fingerprint)"
    if [[ "$1" == failure ]]; then
        export TEST_FETCH_FAIL=1
        expect_failure "$TEST_REAL_MAKE" --no-print-directory validate 'CI_TARGETS=deps check'
        [[ "$(cat "$TEST_EFFECTS")" == fetch ]]
        [[ -s target/validation-failures/latest.log && -s target/validation-failures/latest-errors.log ]]
    else
        "$TEST_REAL_MAKE" --no-print-directory release-verify 'CI_TARGETS=deps check'
        printf '%s\n' fetch check > target/expected
        cmp target/expected "$TEST_EFFECTS"
    fi
    assert_unchanged; assert_cache_retained
}
test_nested_dependency_bootstrap() {
    mkdir -p target/parent
    printf 'parent evidence\n' > target/parent/summary
    export VALIDATION_RUNNER_DEPTH=1
    test_dependency_bootstrap "$1"
    [[ "$(cat target/parent/summary)" == 'parent evidence' ]]
}
test_validation_source_identity() {
    before="$(fingerprint)"
    # Explicit per-call identity still reaches the real production guard after
    # the suite has excluded enclosing release/Make context at its entry.
    expect_failure "$TEST_REAL_MAKE" --no-print-directory release-verify \
        'CI_TARGETS=deps check' RELEASE_SOURCE=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
        RELEASE_PREVIOUS=0.1.0 RELEASE_VERSION=0.1.1 RELEASE_DATE=2026-10-05
    assert_unchanged
    [[ ! -e target/release-state/0.1.1.validation.json ]]
    "$TEST_REAL_MAKE" --no-print-directory release-verify 'CI_TARGETS=deps check' \
        "RELEASE_SOURCE=$TEST_SOURCE" RELEASE_PREVIOUS=0.1.0 \
        RELEASE_VERSION=0.1.1 RELEASE_DATE=2026-10-05
    perl scripts/release/release-data.pl validation-check \
        target/release-state/0.1.1.validation.json "$TEST_SOURCE" 2026-10-05 0.1.0 0.1.1 original
    assert_unchanged
    [[ ! -e target/release-state/0.1.1.plan && ! -e target/mock-staged && ! -e target/mock-tag && ! -e target/remote-head ]]
    assert_cache_retained
}
test_versions() {
    [[ "$(bash scripts/ci/next-release-version.sh 0.1.0 patch)" == 0.1.1 ]]
    [[ "$(bash scripts/ci/next-release-version.sh 0.1.0 minor)" == 0.2.0 ]]
    [[ "$(bash scripts/ci/next-release-version.sh 0.1.0 major)" == 1.0.0 ]]
    for value in 01.2.3 0.1.1-extra 0.0.9; do expect_failure perl scripts/release/release-data.pl next "$value"; done
}
test_invalid_changelog() {
    case "$1" in
        conflict) sed 's/\[0.1.1\]/[0.2.0]/' CHANGELOG.md > target/notes; cp target/notes CHANGELOG.md ;;
        duplicate) printf '\n## [0.1.1]\n\n- Other.\n' >> CHANGELOG.md ;;
        empty) printf '# Changelog\n\n## [0.1.1]\n' > CHANGELOG.md ;;
        dated) sed 's/## \[0.1.1\]/## [0.1.1] - 2026-10-01/' CHANGELOG.md > target/notes; cp target/notes CHANGELOG.md ;;
        unnumbered) sed 's/\[0.1.1\]/[Draft]/' CHANGELOG.md > target/notes; cp target/notes CHANGELOG.md ;;
    esac
    before="$(fingerprint)"
    expect_failure perl scripts/release/release-data.pl changelog-check 0.1.1 2026-10-05
    assert_unchanged
}
test_preparation() {
    export TEST_LOST_EFFECT=commit
    expect_failure "$TEST_REAL_MAKE" --no-print-directory release-patch
    perl scripts/release/release-data.pl verify
    [[ -f target/release-state/0.1.1.validation.json ]]
    rg -q '^name = "retained-dependency"$' Cargo.lock
    rg -q '^version = "9.8.7"$' Cargo.lock
    cmp target/history <(sed -n '/^## \[0.1.0\]/,$p' CHANGELOG.md)
    local backups=(target/release-backup.*)
    [[ -d "${backups[0]}" ]]
    assert_cache_retained
}
test_rejected_preparation() {
    before="$(fingerprint)"
    export "$1=1"
    expect_failure "$TEST_REAL_MAKE" --no-print-directory release-patch
    if [[ "$1" != TEST_GATE_DIRTY ]]; then assert_unchanged; fi
    [[ ! -f target/mock-tag && ! -f target/remote-head ]]
    assert_cache_retained
}
test_staging() {
    before="$(fingerprint)"
    expect_failure "$TEST_REAL_MAKE" --no-print-directory release-patch release-minor
    assert_unchanged
    [[ ! -s "$TEST_EFFECTS" ]]
    "$TEST_REAL_MAKE" --no-print-directory -n release-patch
    [[ ! -s "$TEST_EFFECTS" && ! -e target/release-state ]]
    expect_failure "$TEST_REAL_MAKE" --no-print-directory release-stage
}
test_initial_version() {
    "$TEST_REAL_MAKE" --no-print-directory release-patch
    [[ "$(perl scripts/release/release-data.pl version)" == 0.1.1 ]]
}
prepare_tagged_release() {
    sed 's/\[0.1.1\]/[0.2.0]/' CHANGELOG.md > target/notes; cp target/notes CHANGELOG.md
    "$TEST_REAL_MAKE" --no-print-directory release-minor
}
test_release() {
    local kind="$1" candidate
    case "$kind" in patch) candidate=0.1.1 ;; minor) candidate=0.2.0 ;; major) candidate=1.0.0 ;; esac
    sed "s/\[0.1.1\]/[$candidate]/" CHANGELOG.md > target/notes; cp target/notes CHANGELOG.md
    "$TEST_REAL_MAKE" --no-print-directory "release-$kind"
    [[ "$(perl scripts/release/release-data.pl version)" == "$candidate" ]]
    perl scripts/release/release-data.pl verify
    awk '/^(validate|stage|commit|tag|push)$/ {print}' "$TEST_EFFECTS" > target/actual
    printf '%s\n' validate stage commit tag push > target/expected
    cmp target/expected target/actual
    [[ "$(tail -n 1 "target/release-state/$candidate.plan")" == complete ]]
    cmp target/history <(sed -n '/^## \[0.1.0\]/,$p' CHANGELOG.md)
    assert_cache_retained
}
test_publish() {
    before="$(fingerprint)"
    if [[ "$1" == tagged ]]; then prepare_tagged_release; before="$(fingerprint)"; fi
    : > "$TEST_EFFECTS"
    "$TEST_REAL_MAKE" --no-print-directory publish
    [[ "$(cat "$TEST_EFFECTS")" == publish && "$(fingerprint)" == "$before" ]]
    assert_cache_retained
}
test_publish_failure() {
    before="$(fingerprint)"; export TEST_PUBLISH_FAIL=1
    expect_failure "$TEST_REAL_MAKE" --no-print-directory publish-dry-run
    assert_unchanged
    [[ ! -s "$TEST_EFFECTS" ]]
}
test_invalid_release() {
    prepare_tagged_release
    case "$1" in
        notes) printf '\n- Post-release drift.\n' >> CHANGELOG.md ;;
        tag) rm target/mock-tag ;;
        head) echo eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee > target/mock-head ;;
    esac
    : > "$TEST_EFFECTS"
    expect_failure "$TEST_REAL_MAKE" --no-print-directory release-resume VERSION=0.2.0
    [[ ! -s "$TEST_EFFECTS" ]]
    expect_failure bash scripts/release/release.sh tag-check
}
test_validation_custody() {
    export TEST_LOST_EFFECT=commit
    expect_failure "$TEST_REAL_MAKE" --no-print-directory release-patch
    unset TEST_LOST_EFFECT
    : > "$TEST_EFFECTS"
    case "$1" in
        missing) mv target/release-state/0.1.1.validation.json target/retained-validation.json ;;
        member) printf '\n# Changed source after validation.\n' >> crates/ic-backup/Cargo.toml ;;
    esac
    expect_failure "$TEST_REAL_MAKE" --no-print-directory release-resume VERSION=0.1.1
    [[ ! -s "$TEST_EFFECTS" && ! -f target/mock-tag ]]
    assert_cache_retained
}
test_destination_custody() {
    export TEST_PUSH_FAIL=1
    expect_failure "$TEST_REAL_MAKE" --no-print-directory release-patch
    unset TEST_PUSH_FAIL
    : > "$TEST_EFFECTS"
    export TEST_DESTINATION=https://example.invalid/changed-destination
    expect_failure "$TEST_REAL_MAKE" --no-print-directory release-resume VERSION=0.1.1
    [[ ! -s "$TEST_EFFECTS" && ! -f target/remote-head ]]
    assert_cache_retained
}
test_push_retry() {
    export TEST_LOST_EFFECT="$1"
    expect_failure "$TEST_REAL_MAKE" --no-print-directory release-patch
    before="$(fingerprint)"
    unset TEST_LOST_EFFECT
    "$TEST_REAL_MAKE" --no-print-directory release-resume VERSION=0.1.1
    [[ "$(fingerprint)" == "$before" ]]
    for event in validate stage commit tag push; do
        [[ "$(awk -v event="$event" '$0==event {n++} END {print n+0}' "$TEST_EFFECTS")" == 1 ]]
    done
    assert_cache_retained
}
test_gate_retry() {
    before="$(fingerprint)"
    export TEST_GATE_FAIL=1
    expect_failure "$TEST_REAL_MAKE" --no-print-directory release-patch
    assert_unchanged
    [[ ! -e target/release-state/0.1.1.plan ]]
    printf 'Earlier validation failure evidence.\n' > target/failed-gate-evidence
    unset TEST_GATE_FAIL
    export TEST_SOURCE=eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee
    printf '\n- Reviewed source fix.\n' >> CHANGELOG.md
    "$TEST_REAL_MAKE" --no-print-directory release-patch
    perl scripts/release/release-data.pl verify
    [[ "$(perl scripts/release/release-data.pl source)" == "$TEST_SOURCE" ]]
    [[ "$(awk '$0=="validate" {n++} END {print n+0}' "$TEST_EFFECTS")" == 2 ]]
    [[ "$(cat target/failed-gate-evidence)" == 'Earlier validation failure evidence.' ]]
    assert_cache_retained
}
test_early_plan_retry() {
    # Prior runner plans exist only as genuine retained interruption evidence.
    local destination
    destination="$(printf '%s\n' https://example.invalid/no-network-release | "$TEST_REAL_GIT" hash-object --stdin)"
    mkdir -p target/release-state
    printf '%s\n' release-plan-1 patch 0.1.0 0.1.1 2026-10-01 "$TEST_SOURCE" origin main "$destination" '' validate > target/release-state/0.1.1.plan
    printf 'Earlier exact proof bytes.\n' > target/release-state/0.1.1.validation.json
    cp target/release-state/0.1.1.plan target/earlier-plan
    cp target/release-state/0.1.1.validation.json target/earlier-proof
    export TEST_SOURCE=eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee
    "$TEST_REAL_MAKE" --no-print-directory release-patch
    local archives=(target/release-state/0.1.1.attempt.*/0.1.1.plan)
    local proofs=(target/release-state/0.1.1.validation.*/validation.json)
    cmp target/earlier-plan "${archives[0]}"
    cmp target/earlier-proof "${proofs[0]}"
    [[ "$(perl scripts/release/release-data.pl source)" == "$TEST_SOURCE" ]]
    assert_cache_retained
}
test_prepared_normal_retry() {
    export TEST_LOST_EFFECT=commit
    expect_failure "$TEST_REAL_MAKE" --no-print-directory release-patch
    unset TEST_LOST_EFFECT
    before="$(fingerprint)"
    : > "$TEST_EFFECTS"
    "$TEST_REAL_MAKE" --no-print-directory release-patch
    [[ "$(fingerprint)" == "$before" && "$(cat "$TEST_EFFECTS")" == $'tag\npush' ]]
    [[ ! -e target/release-state/0.1.2.plan ]]
    "$TEST_REAL_MAKE" --no-print-directory release-resume VERSION=0.1.1
    assert_cache_retained
}
record_reviewed_fix() {
    local current fix=ffffffffffffffffffffffffffffffffffffffff
    current="$(cat target/mock-head)"
    printf '\n# Reviewed member metadata fix.\n' >> crates/ic-backup/Cargo.toml
    {
        printf '# Changelog\n\n## [%s]\n\n- Reviewed follow-up fix.\n\n' "$1"
        sed -n '/^## \[0.1.1\]/,$p' CHANGELOG.md
    } > target/new-notes
    cp target/new-notes CHANGELOG.md
    mkdir -p "target/commits/$fix/files/crates/ic-backup" "target/commits/$fix/files/docs"
    printf '%s\n' "$current" > "target/commits/$fix/parent"
    printf 'Reviewed source fix\n' > "target/commits/$fix/subject"
    cp Cargo.toml Cargo.lock CHANGELOG.md "target/commits/$fix/files/"
    cp crates/ic-backup/Cargo.toml "target/commits/$fix/files/crates/ic-backup/"
    cp docs/release.json "target/commits/$fix/files/docs/"
    printf '%s\n' "$fix" > target/mock-head
}
test_older_release_recovery() {
    local mode="$1" next=0.1.2
    [[ "$mode" != minor ]] || next=0.2.0
    export TEST_PUSH_FAIL=1
    expect_failure "$TEST_REAL_MAKE" --no-print-directory release-patch
    unset TEST_PUSH_FAIL
    cp target/release-state/0.1.1.validation.json target/original-proof.json
    record_reviewed_fix "$next"
    expect_failure perl scripts/release/release-data.pl verify
    : > "$TEST_EFFECTS"
    if [[ "$mode" == explicit ]]; then
        "$TEST_REAL_MAKE" --no-print-directory release-resume VERSION=0.1.1
        [[ "$(cat "$TEST_EFFECTS")" == push ]]
        [[ "$(cat target/remote-head)" == bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb ]]
        [[ "$(perl scripts/release/release-data.pl version)" == 0.1.1 ]]
        [[ ! -e "target/release-state/$next.plan" ]]
    else
        if [[ "$mode" == gate-failure ]]; then
            export TEST_GATE_FAIL=1
            expect_failure "$TEST_REAL_MAKE" --no-print-directory release-patch
            [[ "$(perl scripts/release/release-data.pl version)" == 0.1.1 ]]
            [[ ! -e target/release-state/0.1.2.plan ]]
            [[ "$(cat target/remote-head)" == bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb ]]
            unset TEST_GATE_FAIL
            "$TEST_REAL_MAKE" --no-print-directory release-patch
        else
            "$TEST_REAL_MAKE" --no-print-directory "release-$mode"
        fi
        [[ "$(perl scripts/release/release-data.pl version)" == "$next" ]]
        [[ "$(perl scripts/release/release-data.pl source)" == ffffffffffffffffffffffffffffffffffffffff ]]
        [[ "$(tail -n 1 "target/release-state/$next.plan")" == complete ]]
        [[ "$(cat "target/tags/v$next.commit")" == "$(cat target/mock-head)" ]]
        rg -q 'release-verify.*RELEASE_SOURCE=ffffffffffffffffffffffffffffffffffffffff' "$TEST_LOG"
        perl scripts/release/release-data.pl verify
    fi
    [[ "$(cat target/tags/v0.1.1.commit)" == bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb ]]
    [[ "$(tail -n 1 target/release-state/0.1.1.plan)" == complete ]]
    [[ "$(sed -n '2p' target/pushes.log)" == 'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb v0.1.1' ]]
    cmp target/original-proof.json target/release-state/0.1.1.validation.json
    perl scripts/release/release-data.pl verify --commit bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
    assert_cache_retained
}
test_selected_proof_rejection() {
    export TEST_PUSH_FAIL=1
    expect_failure "$TEST_REAL_MAKE" --no-print-directory release-patch
    unset TEST_PUSH_FAIL
    record_reviewed_fix 0.1.2
    if [[ "$1" == missing ]]; then
        mv target/release-state/0.1.1.validation.json target/retained-proof.json
    elif [[ "$1" == identity ]]; then
        cp target/release-state/0.1.1.validation.json target/retained-proof.json
        perl -i -pe 's/"candidate" : "0.1.1"/"candidate" : "0.9.9"/' target/release-state/0.1.1.validation.json
    else
        export TEST_REMOTE_FAIL=1
    fi
    : > "$TEST_EFFECTS"
    expect_failure "$TEST_REAL_MAKE" --no-print-directory release-patch
    [[ ! -s "$TEST_EFFECTS" && ! -f target/remote-head ]]
    [[ "$(tail -n 1 target/release-state/0.1.1.plan)" == push ]]
    assert_cache_retained
}
test_completed_evidence_replay() {
    "$TEST_REAL_MAKE" --no-print-directory release-patch
    mv target/release-state/0.1.1.validation.json target/retained-proof.json
    : > "$TEST_EFFECTS"
    expect_failure "$TEST_REAL_MAKE" --no-print-directory release-resume VERSION=0.1.1
    [[ ! -s "$TEST_EFFECTS" ]]
    mv target/retained-proof.json target/release-state/0.1.1.validation.json
    "$TEST_REAL_MAKE" --no-print-directory release-resume VERSION=0.1.1
    [[ ! -s "$TEST_EFFECTS" ]]
    expect_failure perl scripts/release/release-data.pl set-version 9.9.9 --commit bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
    expect_failure perl scripts/release/release-data.pl verify --commit invalid
    expect_failure perl scripts/release/release-data.pl verify "$TEST_SOURCE" 1900-01-01 0.1.1 --commit bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
    [[ "$(perl scripts/release/release-data.pl version)" == 0.1.1 ]]
    assert_cache_retained
}
test_dependency_failure_conditional_status() {
    export TEST_FETCH_FAIL=1
    # Conditional evaluation can disable errexit even after a sourced script
    # enables it. The substitute must explicitly reject before creating a cache.
    cat > target/conditional-fetch.sh <<'SH'
if source "$1" fetch --locked; then exit 0; else exit $?; fi
SH
    expect_failure bash target/conditional-fetch.sh "$TEMPORARY/bin/cargo"
    [[ ! -e target/mock-cache && "$(cat "$TEST_EFFECTS")" == fetch ]]
    assert_cache_retained
}

for outcome in success failure; do
    run_case "dependency-$outcome" test_dependency_bootstrap "$outcome"
    run_case "nested-dependency-$outcome" test_nested_dependency_bootstrap "$outcome"
done
run_case validation-source-identity test_validation_source_identity
run_case versions test_versions
for invalid in conflict duplicate empty dated unnumbered; do run_case "notes-$invalid" test_invalid_changelog "$invalid"; done
run_case preparation test_preparation
for failure in TEST_FETCH_FAIL TEST_GATE_FAIL TEST_METADATA_FAIL TEST_DIRTY TEST_TAG_EXISTS TEST_GATE_DIRTY TEST_GATE_HEAD TEST_PREPARED_FORMAT_FAIL; do run_case "reject-$failure" test_rejected_preparation "$failure"; done
run_case staging test_staging
run_case initial-version test_initial_version
for kind in patch minor major; do run_case "release-$kind" test_release "$kind"; done
for state in no-release tagged; do run_case "publish-$state" test_publish "$state"; done
run_case publish-failure test_publish_failure
for drift in notes tag head; do run_case "completed-$drift" test_invalid_release "$drift"; done
for custody in missing member; do run_case "validation-$custody" test_validation_custody "$custody"; done
run_case destination-custody test_destination_custody
for effect in commit tag push; do run_case "lost-$effect" test_push_retry "$effect"; done
run_case gate-retry-current-source test_gate_retry
run_case early-plan-retry-retention test_early_plan_retry
run_case prepared-normal-retry test_prepared_normal_retry
for mode in explicit patch minor gate-failure; do run_case "older-release-$mode" test_older_release_recovery "$mode"; done
for reason in missing identity remote; do run_case "selected-proof-$reason" test_selected_proof_rejection "$reason"; done
run_case completed-evidence-replay test_completed_evidence_replay
run_case dependency-failure-conditional-status test_dependency_failure_conditional_status
printf 'Release adapters: PASS (actual Make entry points, original metadata, gates, rollback, recovery and exact push; Git/Cargo substitutes only).\n'
