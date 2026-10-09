#!/usr/bin/env bash
set -Eeuo pipefail

ROOT="${BASH_SOURCE[0]}"
[[ "$ROOT" == /* ]] || ROOT="$PWD/$ROOT"
ROOT="$(cd -P "${ROOT%/*}/../.." && printf '%s/.' "$PWD")"
ROOT="${ROOT%/.}"
# Fixtures own their release identities and Make invocations. A real release
# exports both environment values and command-line overrides through GNU Make;
# neither may reach the fixture's standalone gates or select a parent helper.
# This child-only reset leaves the enclosing release and its evidence intact.
# Consumer assertions are mapped in docs/release-fixture-ownership.json. The
# canonical shared suite owns increment/phase and generic lost-reply matrices.
unset RELEASE_SOURCE RELEASE_PREVIOUS RELEASE_VERSION RELEASE_DATE RELEASE_KIND RELEASE_COMMIT \
    RELEASE_REMOTE RELEASE_BRANCH RELEASE_MAKE RELEASE_DELIVERY VERSION \
    MAKEFLAGS MAKEOVERRIDES MFLAGS MAKELEVEL GNUMAKEFLAGS MAKEFILES
unset VALIDATION_REPOSITORY_ROOT VALIDATION_RUNNER_SNAPSHOT_PATH
metadata_only=false
if [[ "$#" == 1 && "$1" == --metadata-only ]]; then
    metadata_only=true
elif [[ "$#" != 0 ]]; then
    echo 'usage: test-release.sh [--metadata-only]' >&2
    exit 2
fi
TEST_REAL_MAKE="$(command -v make)"
TEST_REAL_GIT="$(command -v git)"
TEST_REAL_CARGO="$(command -v cargo)"
export TEST_REAL_MAKE TEST_REAL_GIT TEST_REAL_CARGO
export PATH="$ROOT/.tools/host/bin:$PATH"
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
    mkdir -p "$FIXTURE/scripts/release" "$FIXTURE/scripts/ci" "$FIXTURE/crates/ic-backup/src" "$FIXTURE/docs" "$FIXTURE/target/debug"
    mkdir -p "$FIXTURE/make"
    cp "$ROOT/make/tools.mk" "$ROOT/make/rust-format.mk" "$ROOT/make/execution.mk" "$FIXTURE/make/"
    cp "$ROOT/scripts/ci/check-make-execution.sh" "$ROOT/scripts/ci/check-release-source.sh" "$FIXTURE/scripts/ci/"
    cp "$ROOT/Makefile" "$FIXTURE/"
    cp "$ROOT/scripts/release/release.sh" "$ROOT/scripts/release/release-data.pl" "$FIXTURE/scripts/release/"
    cp "$ROOT/scripts/ci/run-release.sh" "$ROOT/scripts/ci/next-release-version.sh" "$ROOT/scripts/ci/run-validation-targets.sh" "$FIXTURE/scripts/ci/"
    cp "$ROOT/scripts/ci/rewrite-local-lock-versions.pl" "$ROOT/scripts/ci/read-cargo-workspace-version.sh" "$FIXTURE/scripts/ci/"
    cp "$ROOT/scripts/ci/finalize-release-changelog.awk" "$FIXTURE/scripts/ci/"
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
    printf '// Native metadata fixture target; no canister behavior.\n' > crates/ic-backup/src/lib.rs
    printf 'version = 4\n\n[[package]]\nname = "ic-backup"\nversion = "0.1.0"\n\n[[package]]\nname = "retained-dependency"\nversion = "9.8.7"\n' > Cargo.lock
    printf '# Changelog\n\n## [0.1.1]\n\n- Completed notes.\n\n## [0.1.0] - 2026-10-01\n\n- Retained history.\n\n## [0.0.9]\n\n- Imported undated history.\n' > CHANGELOG.md
    sed -n '/^## \[0.1.0\]/,$p' CHANGELOG.md > target/history
    export TEST_LOG="$FIXTURE/target/commands.log" TEST_EFFECTS="$FIXTURE/target/effects.log"
    : > "$TEST_LOG"; : > "$TEST_EFFECTS"
    printf 'retained build artifact\n' > target/debug/cache-sentinel
    export VALIDATION_REPOSITORY_ROOT="$FIXTURE" VALIDATION_FAILURE_LOG_DIR="$FIXTURE/target/validation-failures" GITHUB_STEP_SUMMARY="$FIXTURE/target/summary.md"
    unset YQ
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
            --show-prefix) ;;
            release-state) echo target/release-state ;;
            HEAD) head ;;
            HEAD^) cat "target/commits/$(head)/parent" ;;
            *'^{tree}') echo dddddddddddddddddddddddddddddddddddddddd ;;
            'refs/tags/'*'^{commit}')
                [[ -f target/mock-tag ]] || exit 1
                name="${value#refs/tags/}"; name="${name%\^\{commit\}}"
                cat "target/tags/$name.commit" ;;
            refs/tags/*) [[ -f target/mock-tag ]] || exit 1; cat "target/tags/${value#refs/tags/}.object" ;;
            *) [[ "$value" =~ ^[0-9a-f]{40}$ && -d "target/commits/$value" ]] || exit 1; echo "$value" ;;
        esac ;;
    show)
        value="$2"; sha="${value%%:*}"; path="${value#*:}"
        [[ "$sha" =~ ^[0-9a-f]{40}$ ]] || exit 1; cat "target/commits/$sha/files/$path" ;;
    status)
        ending='\n'
        case " $* " in *' -z '*) ending='\0' ;; esac
        if [[ "${TEST_DIRTY:-0}" == 1 ]]; then
            printf " M src/changed.rs$ending"
        elif [[ -d "target/commits/$(head)" ]]; then
            for file in Cargo.toml Cargo.lock CHANGELOG.md docs/release.json crates/ic-backup/Cargo.toml; do
                if ! cmp -s "$file" "target/commits/$(head)/files/$file"; then
                    printf " M %s$ending" "$file"
                fi
            done
        fi ;;
    diff)
        case "${2:-}" in
            --name-only) if [[ "${TEST_DIRTY:-0}" == 1 ]]; then printf 'src/changed.rs\0'; fi ;;
            --binary) cat Cargo.toml Cargo.lock CHANGELOG.md crates/ic-backup/Cargo.toml ;;
            --cached)
                case "$3" in
                    --name-only) if [[ "${TEST_DIRTY:-0}" == 1 ]]; then printf 'src/changed.rs\0'; fi ;;
                    --quiet) if [[ "$#" == 3 ]]; then [[ ! -f target/mock-staged ]]; else [[ "${TEST_DIRTY:-0}" != 1 ]]; fi ;;
                    *) exit 97 ;;
                esac ;;
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
                ;;
            *) exit 97 ;;
        esac ;;
    cat-file) [[ -f target/mock-tag && -f "target/tags/${3#refs/tags/}.object" ]] || exit 1; echo tag ;;
    ls-remote)
        [[ "${TEST_REMOTE_FAIL:-0}" != 1 ]] || exit 9
        for ref in "$@"; do
            case "$ref" in
                refs/heads/main) if [[ -f target/remote-head ]]; then printf '%s\t%s\n' "$(cat target/remote-head)" "$ref"; fi ;;
                refs/tags/*) if [[ -f "target/remote-tags/${ref#refs/tags/}" ]]; then printf '%s\t%s\n' "$(cat "target/remote-tags/${ref#refs/tags/}")" "$ref"; fi ;;
            esac
        done ;;
    add) [[ "$*" == 'add -- Cargo.toml Cargo.lock CHANGELOG.md docs/release.json' ]] || exit 97; touch target/mock-staged; echo stage >> "$TEST_EFFECTS" ;;
    write-tree) echo dddddddddddddddddddddddddddddddddddddddd ;;
    log)
        sha="$(resolve "${*: -1}")"
        case "$3" in
            --format=%P) cat "target/commits/$sha/parent" ;;
            --format=%s) cat "target/commits/$sha/subject" ;;
            *) exit 97 ;;
        esac ;;
    commit)
        [[ "$*" == "commit -m Release $(perl scripts/release/release-data.pl version)" ]] || exit 97
        parent="$(head)"; sha=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
        if [[ -d "target/commits/$sha" ]]; then sha="$(printf '%s\n' "$parent" "$3" | "$TEST_REAL_GIT" hash-object --stdin)"; fi
        mkdir -p "target/commits/$sha/files/crates/ic-backup/src" "target/commits/$sha/files/docs"
        printf '%s\n' "$parent" > "target/commits/$sha/parent"
        printf '%s\n' "$3" > "target/commits/$sha/subject"
        cp Cargo.toml Cargo.lock CHANGELOG.md "target/commits/$sha/files/"
        cp crates/ic-backup/Cargo.toml "target/commits/$sha/files/crates/ic-backup/"
        cp crates/ic-backup/src/lib.rs "target/commits/$sha/files/crates/ic-backup/src/"
        cp docs/release.json "target/commits/$sha/files/docs/"
        printf '%s\n' "$sha" > target/mock-head
        echo commit >> "$TEST_EFFECTS"
        if [[ "${TEST_LOST_EFFECT:-}" == commit && ! -e target/lost-commit ]]; then touch target/lost-commit; exit 9; fi ;;
    push)
        [[ "$#" == 7 && "$2" == --no-follow-tags && "$3" == --atomic && "$4" == -- && "$5" == "${TEST_DESTINATION:-https://example.invalid/no-network-release}" && "$6" == *:refs/heads/main && "$7" == refs/tags/v*:refs/tags/v* ]] || exit 97
        sha="$(resolve "${6%:refs/heads/main}")"; tag="${7%%:*}"; tag="${tag#refs/tags/}"
        if [[ -f target/remote-head ]]; then ancestor "$(cat target/remote-head)" "$sha" || exit 1; fi
        printf '%s %s\n' "$sha" "$tag" >> target/pushes.log
        echo push >> "$TEST_EFFECTS"
        [[ "${TEST_PUSH_FAIL:-0}" != 1 ]] || exit 9
        echo "$sha" > target/remote-head
        mkdir -p target/remote-tags
        cp "target/tags/$tag.object" "target/remote-tags/$tag"
        cp "target/tags/$tag.object" target/remote-tag
        ;;
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
    [[ "${TEST_PREPARED_FORMAT_FAIL:-0}" != 1 ]] || exit 7
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
    'locate-project --workspace --message-format plain --manifest-path '*) exec "$TEST_REAL_CARGO" "$@" ;;
    'fetch --locked') echo fetch >> "$TEST_EFFECTS"; [[ "${TEST_FETCH_FAIL:-0}" != 1 ]] || exit 7; touch target/mock-cache ;;
    'check --offline --locked -p ic-backup -p ic-backup-agent --all-targets --all-features') [[ -f target/mock-cache ]] || exit 7; echo check >> "$TEST_EFFECTS" ;;
    'metadata --offline --locked --no-deps --format-version 1') [[ "${TEST_METADATA_FAIL:-0}" != 1 ]] || exit 7; echo '{}' ;;
    'publish --locked --registry crates-io -p ic-backup -p ic-backup-agent'|'publish --locked --registry crates-io -p ic-backup -p ic-backup-agent --dry-run')
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
    # Invoke the actual consumer entry point under a noisy directory-search environment.
    [[ "$(CDPATH="$PWD" bash scripts/release/release.sh version)" == 0.1.0 ]]
    [[ "$(bash scripts/ci/next-release-version.sh 0.1.0 patch)" == 0.1.1 ]]
    [[ "$(bash scripts/ci/next-release-version.sh 0.1.0 minor)" == 0.2.0 ]]
    [[ "$(bash scripts/ci/next-release-version.sh 0.1.0 major)" == 1.0.0 ]]
    for value in 01.2.3 0.1.1-extra 0.0.9; do expect_failure perl scripts/release/release-data.pl next "$value"; done
}
test_invalid_changelog() {
    case "$1" in
        conflict) sed 's/\[0.1.1\]/[0.2.0]/' CHANGELOG.md > target/notes; cp target/notes CHANGELOG.md ;;
        duplicate) printf '\n## [0.1.1]\n\n- Other.\n' >> CHANGELOG.md ;;
        dated) sed 's/## \[0.1.1\]/## [0.1.1] - 2026-10-01/' CHANGELOG.md > target/notes; cp target/notes CHANGELOG.md ;;
        historical-duplicate) printf '\n## [0.0.9]\n\n- Duplicate history.\n' >> CHANGELOG.md ;;
        competing) printf '\n## [0.1.2]\n\n- Competing draft.\n' >> CHANGELOG.md ;;
        same-date) sed 's/## \[0.1.1\]/## [0.1.1] - 2026-10-05/' CHANGELOG.md > target/notes; cp target/notes CHANGELOG.md ;;
        unnumbered) sed 's/\[0.1.1\]/[Draft]/' CHANGELOG.md > target/notes; cp target/notes CHANGELOG.md ;;
    esac
    before="$(fingerprint)"
    expect_failure perl scripts/release/release-data.pl changelog-check 0.1.1 2026-10-05 0.1.0
    assert_unchanged
}
test_shared_changelog() {
    # Keep rejected transformation evidence inside this retained case directory.
    export TMPDIR="$FIXTURE/target"
    case "$1" in
        empty) printf '# Changelog\n\n## [0.1.1]\n' > CHANGELOG.md ;;
        missing) cp target/history CHANGELOG.md ;;
        imported) sed 's/## \[0.1.0\] - 2026-10-01/## [0.1.0]/' CHANGELOG.md > target/notes; cp target/notes CHANGELOG.md ;;
        bumped) perl scripts/release/release-data.pl prepare-version 0.1.1 ;;
        failed-output)
            cat > scripts/ci/finalize-release-changelog.awk <<'AWK'
END { print "# Plausible partial candidate"; exit 13 }
AWK
            before="$(fingerprint)"
            expect_failure perl scripts/release/release-data.pl finalize 0.1.1 2026-10-05 0.1.0
            assert_unchanged
            rg -F '# Plausible partial candidate' "$TMPDIR"/ic-backup-changelog.*/candidate.md
            return ;;
        empty-output)
            printf 'END { exit 0 }\n' > scripts/ci/finalize-release-changelog.awk
            before="$(fingerprint)"
            expect_failure perl scripts/release/release-data.pl finalize 0.1.1 2026-10-05 0.1.0
            assert_unchanged
            local candidates=("$TMPDIR"/ic-backup-changelog.*/candidate.md)
            [[ "${#candidates[@]}" == 1 && -f "${candidates[0]}" && ! -s "${candidates[0]}" ]]
            return ;;
    esac
    cp CHANGELOG.md target/original-notes
    before="$(fingerprint)"
    perl scripts/release/release-data.pl changelog-check 0.1.1 2026-10-05 0.1.0
    assert_unchanged
    awk -v version=0.1.1 -v previous=0.1.0 -v date=2026-10-05 \
        -f scripts/ci/finalize-release-changelog.awk target/original-notes > target/expected-notes
    perl scripts/release/release-data.pl finalize 0.1.1 2026-10-05 0.1.0
    cmp CHANGELOG.md target/expected-notes
    # Finalization is never called again for prepared/committed recovery. Those
    # paths require exact retained payload/receipt admission instead.
    before="$(fingerprint)"
    expect_failure perl scripts/release/release-data.pl finalize 0.1.1 2026-10-05 0.1.0
    assert_unchanged
    assert_cache_retained
}
test_version_reader() {
    local mode="$1"
    if [[ "$mode" == comments ]]; then
        sed 's/version = "0.1.0"/version = '\''0.1.0'\'' # valid TOML comment/' Cargo.toml > target/commented.toml
        cp target/commented.toml Cargo.toml
        sed 's/version.workspace = true/version.workspace = true # valid inheritance comment/' crates/ic-backup/Cargo.toml > target/member.toml
        cp target/member.toml crates/ic-backup/Cargo.toml
        before="$(fingerprint)"
        [[ "$(perl scripts/release/release-data.pl version)" == 0.1.0 ]]
        assert_unchanged
    elif [[ "$mode" == selected ]]; then
        mkdir -p "target/commits/$TEST_SOURCE/files/crates/ic-backup/src"
        cp Cargo.toml "target/commits/$TEST_SOURCE/files/"
        cp crates/ic-backup/Cargo.toml "target/commits/$TEST_SOURCE/files/crates/ic-backup/"
        cp crates/ic-backup/src/lib.rs "target/commits/$TEST_SOURCE/files/crates/ic-backup/src/"
        printf 'invalid working TOML\n' >> Cargo.toml
        printf 'invalid working member TOML\n' >> crates/ic-backup/Cargo.toml
        before="$(fingerprint)"
        [[ "$(perl scripts/release/release-data.pl version --commit "$TEST_SOURCE")" == 0.1.0 ]]
        assert_unchanged
        expect_failure perl scripts/release/release-data.pl version
    else
        case "$mode" in
            duplicate) printf '\n[workspace.package]\nversion = "0.1.1"\n' >> Cargo.toml ;;
            failed-output) printf '%s\n' 'printf "0.1.0\n"; exit 13' > scripts/ci/read-cargo-workspace-version.sh ;;
            empty) printf '%s\n' 'exit 0' > scripts/ci/read-cargo-workspace-version.sh ;;
            failed-parser)
                printf '%s\n' '#!/usr/bin/env bash' 'printf '\''{"workspace":{"package":{"version":"0.1.0"}}}\n'\''; exit 9' > target/failed-parser
                chmod +x target/failed-parser
                export YQ="$PWD/target/failed-parser" ;;
        esac
        before="$(fingerprint)"
        expect_failure perl scripts/release/release-data.pl prepare-version 0.1.1
        assert_unchanged
        [[ ! -s "$TEST_EFFECTS" ]]
        if perl scripts/release/release-data.pl version > target/version-output 2> target/version-error; then
            echo 'invalid version read was accepted' >&2; exit 1
        fi
        [[ ! -s target/version-output ]]
    fi
    assert_cache_retained
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
test_multi_package_preparation() {
    perl -pi -e 's/members = \["crates\/ic-backup"\]/members = ["crates\/ic-backup", "crates\/ic-backup-agent"]/' Cargo.toml
    printf '\n[workspace.dependencies]\nic-backup = { version = "0.1", path = "crates/ic-backup" }\n' >> Cargo.toml
    mkdir -p crates/ic-backup-agent/src
    printf '[package]\nname = "ic-backup-agent"\nversion.workspace = true\nedition.workspace = true\n[dependencies]\nic-backup.workspace = true\n' > crates/ic-backup-agent/Cargo.toml
    printf '// Selected transport library fixture.\n' > crates/ic-backup-agent/src/lib.rs
    printf '\n[[package]]\nname = "ic-backup-agent"\nversion = "0.1.0"\ndependencies = ["ic-backup"]\n' >> Cargo.lock
    perl scripts/release/release-data.pl validation-receipt target/multi.validation.json "$TEST_SOURCE" 2026-10-05 0.1.0 0.2.0
    perl scripts/release/release-data.pl prepare-version 0.2.0
    perl -pi -e 's/0\.1\.1/0.2.0/' CHANGELOG.md
    perl scripts/release/release-data.pl finalize 0.2.0 2026-10-05 0.1.0
    perl scripts/release/release-data.pl receipt "$TEST_SOURCE" 2026-10-05
    perl scripts/release/release-data.pl validation-check target/multi.validation.json "$TEST_SOURCE" 2026-10-05 0.1.0 0.2.0 prepared
    perl scripts/release/release-data.pl verify
    jq -e '.files | has("crates/ic-backup-agent/Cargo.toml")' docs/release.json
    rg -F 'ic-backup = { version = "0.2.0", path = "crates/ic-backup" }' Cargo.toml
    [[ "$(rg -c 'version = "0.2.0"' Cargo.lock)" == 2 ]]
    mkdir -p "target/commits/$TEST_SOURCE/files"
    cp Cargo.toml Cargo.lock CHANGELOG.md "target/commits/$TEST_SOURCE/files/"
    cp -R crates docs "target/commits/$TEST_SOURCE/files/"
    printf 'invalid working member TOML\n' >> crates/ic-backup-agent/Cargo.toml
    perl scripts/release/release-data.pl verify --commit "$TEST_SOURCE"
    expect_failure perl scripts/release/release-data.pl validation-check target/multi.validation.json "$TEST_SOURCE" 2026-10-05 0.1.0 0.2.0 prepared
    assert_cache_retained
}
test_lockfile_rejection() {
    case "$1" in
        mismatch) perl -pi -e 's/0\.1\.0/0.0.9/' Cargo.lock ;;
        duplicate) printf '\n[[package]]\nname = "ic-backup"\nversion = "0.1.0"\n' >> Cargo.lock ;;
        missing) perl -pi -e 's/name = "ic-backup"/name = "other-package"/' Cargo.lock ;;
        failed-output)
            mv scripts/ci/rewrite-local-lock-versions.pl target/original-transformer.pl
            printf '%s\n' 'print "version = 4\n"; exit 13;' > scripts/ci/rewrite-local-lock-versions.pl ;;
    esac
    before="$(fingerprint)"
    expect_failure perl scripts/release/release-data.pl prepare-version 0.1.1
    assert_unchanged
    [[ ! -s "$TEST_EFFECTS" ]]
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
    for mode in -i -n -t -q; do
        expect_failure "$TEST_REAL_MAKE" --no-print-directory "$mode" release-patch
        assert_unchanged
        [[ ! -s "$TEST_EFFECTS" && ! -e target/release-state ]]
    done
    expect_failure "$TEST_REAL_MAKE" --no-print-directory release-stage
    for delivery in pr invalid; do
        expect_failure "$TEST_REAL_MAKE" --no-print-directory release-patch "RELEASE_DELIVERY=$delivery"
        expect_failure "$TEST_REAL_MAKE" --no-print-directory release-resume VERSION=0.1.1 "RELEASE_DELIVERY=$delivery"
        assert_unchanged
        [[ ! -s "$TEST_EFFECTS" && ! -e target/release-state ]]
    done
}
prepare_tagged_release() {
    sed 's/\[0.1.1\]/[0.2.0]/' CHANGELOG.md > target/notes; cp target/notes CHANGELOG.md
    "$TEST_REAL_MAKE" --no-print-directory release-minor
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
    cp docs/release.json target/original-receipt.json
    cp target/release-state/0.1.1.validation.json target/original-validation.json
    perl scripts/release/release-data.pl verify
    : > "$TEST_EFFECTS"
    "$TEST_REAL_MAKE" --no-print-directory release-patch
    [[ "$(fingerprint)" == "$before" && "$(cat "$TEST_EFFECTS")" == $'tag\npush' ]]
    cmp docs/release.json target/original-receipt.json
    cmp target/release-state/0.1.1.validation.json target/original-validation.json
    perl scripts/release/release-data.pl verify
    [[ "$(perl scripts/release/release-data.pl version)" == 0.1.1 ]]
    [[ "$(tail -n 1 target/release-state/0.1.1.plan)" == complete ]]
    cmp target/history <(sed -n '/^## \[0.1.0\]/,$p' CHANGELOG.md)
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
    mkdir -p "target/commits/$fix/files/crates/ic-backup/src" "target/commits/$fix/files/docs"
    printf '%s\n' "$current" > "target/commits/$fix/parent"
    printf 'Reviewed source fix\n' > "target/commits/$fix/subject"
    cp Cargo.toml Cargo.lock CHANGELOG.md "target/commits/$fix/files/"
    cp crates/ic-backup/Cargo.toml "target/commits/$fix/files/crates/ic-backup/"
    cp crates/ic-backup/src/lib.rs "target/commits/$fix/files/crates/ic-backup/src/"
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
test_conditional_substitute_rejections() {
    # Bash 3.2 also disables inherited errexit in conditional command substitutions.
    # Source under a condition to reproduce that rule on the selected native Bash.
    cat > target/conditional-substitute.sh <<'SH'
if source "$1" "${@:2}"; then exit 0; else exit $?; fi
SH
    expect_failure bash target/conditional-substitute.sh "$TEMPORARY/bin/git" cat-file -t refs/tags/v0.1.1
    expect_failure bash target/conditional-substitute.sh "$TEMPORARY/bin/git" rev-parse refs/tags/v0.1.1
    expect_failure bash target/conditional-substitute.sh "$TEMPORARY/bin/git" rev-parse 'refs/tags/v0.1.1^{commit}'
    expect_failure bash target/conditional-substitute.sh "$TEMPORARY/bin/git" rev-parse aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
    [[ ! -s "$TEST_EFFECTS" ]]
    export TEST_PREPARED_FORMAT_FAIL=1
    expect_failure bash target/conditional-substitute.sh "$TEMPORARY/bin/make" fmt-check
    [[ "$(cat "$TEST_EFFECTS")" == prepared-format-check ]]
    [[ ! -e target/mock-tag && ! -e target/mock-staged && ! -e target/mock-head ]]
    assert_cache_retained
}

test_real_index_boundaries() {
    # Reuse real existing history and private indexes; never create a commit.
    local native="$FIXTURE/target/real-worktree" source helper="$FIXTURE/target/native-guard.sh"
    "$TEST_REAL_GIT" clone --shared --quiet "$ROOT" "$native"
    source="$("$TEST_REAL_GIT" -C "$native" rev-parse HEAD)"
    mkdir "$FIXTURE/target/native-bin"
    cat > "$FIXTURE/target/native-bin/git" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${TEST_NATIVE_GIT_FAIL:-}" == "$1" ]]; then
    echo 'injected native Git observation failure' >&2
    exit 9
fi
exec "$TEST_REAL_GIT" "$@"
SH
    chmod +x "$FIXTURE/target/native-bin/git"
    cat > "$helper" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
export PATH="$FIXTURE/target/native-bin:$PATH"
source "$1" version
cd "$2"
export CARGO_TARGET_DIR="$PWD/target"
mkdir -p "$CARGO_TARGET_DIR"
case "$3" in
    paths) allowed_changes "$4" ;;
    clean) ensure_clean ;;
    preflight)
        export RELEASE_SOURCE="$4" RELEASE_PREVIOUS="$5" RELEASE_VERSION="$6" RELEASE_DATE=2026-10-06
        preflight ;;
    *) exit 97 ;;
esac
SH
    bash "$helper" "$ROOT/scripts/release/release.sh" "$native" paths "$source"
    printf '\n- Permitted pending notes.\n' >> "$native/CHANGELOG.md"
    bash "$helper" "$ROOT/scripts/release/release.sh" "$native" paths "$source"
    "$TEST_REAL_GIT" -C "$native" show HEAD:CHANGELOG.md > "$native/CHANGELOG.md"
    printf '\nstaged unrelated edit\n' >> "$native/README.md"
    "$TEST_REAL_GIT" -C "$native" add -- README.md
    "$TEST_REAL_GIT" -C "$native" show HEAD:README.md > "$native/README.md"
    expect_failure bash "$helper" "$ROOT/scripts/release/release.sh" "$native" paths "$source"
    rg -F 'staged: README.md' target/rejection.log >/dev/null
    "$TEST_REAL_GIT" -C "$native" diff --cached --name-only > target/native-staged.txt
    [[ "$(cat target/native-staged.txt)" == README.md ]]
    "$TEST_REAL_GIT" -C "$native" restore --staged -- README.md
    printf '\n# staged original metadata edit\n' >> "$native/Cargo.toml"
    "$TEST_REAL_GIT" -C "$native" add -- Cargo.toml
    "$TEST_REAL_GIT" -C "$native" show HEAD:Cargo.toml > "$native/Cargo.toml"
    local previous candidate
    previous="$(cd "$native" && perl scripts/release/release-data.pl version)"
    candidate="$(cd "$native" && perl scripts/release/release-data.pl next patch)"
    expect_failure bash "$helper" "$ROOT/scripts/release/release.sh" "$native" preflight "$source" "$previous" "$candidate"
    rg -F 'staged: Cargo.toml' target/rejection.log >/dev/null
    rg -F 'validation and version preparation have not started for this attempt' target/rejection.log >/dev/null
    [[ ! -f "$native/target/commands.log" ]]
    "$TEST_REAL_GIT" -C "$native" diff --cached --name-only > target/native-staged.txt
    [[ "$(cat target/native-staged.txt)" == Cargo.toml ]]
    "$TEST_REAL_GIT" -C "$native" restore --staged -- Cargo.toml

    # Report every observed category and quote unusual bytes without changing
    # the actual private index, lock, working files or retained untracked bytes.
    printf '\n' >> "$native/Cargo.lock"
    cp "$native/Cargo.lock" target/native-lock
    cp "$native/.git/index" target/native-index
    expect_failure bash "$helper" "$ROOT/scripts/release/release.sh" "$native" preflight "$source" "$previous" "$candidate"
    rg -F 'unstaged: Cargo.lock' target/rejection.log >/dev/null
    cmp "$native/Cargo.lock" target/native-lock
    cmp "$native/.git/index" target/native-index
    "$TEST_REAL_GIT" -C "$native" show HEAD:Cargo.lock > "$native/Cargo.lock"

    printf '\nstaged unrelated edit\n' >> "$native/README.md"
    "$TEST_REAL_GIT" -C "$native" add -- README.md
    "$TEST_REAL_GIT" -C "$native" show HEAD:README.md > "$native/README.md"
    printf '\nworking edit\n' >> "$native/AGENTS.md"
    local unusual=$'untracked\nname.txt'
    printf 'retained evidence\n' > "$native/$unusual"
    cp "$native/.git/index" target/native-index
    cp "$native/AGENTS.md" target/native-working
    expect_failure bash "$helper" "$ROOT/scripts/release/release.sh" "$native" paths "$source"
    for expected in 'staged: README.md' 'unstaged: README.md' 'unstaged: AGENTS.md'; do
        rg -F "$expected" target/rejection.log >/dev/null
    done
    printf '  untracked: %q\n' "$unusual" > target/native-expected
    rg -Fx -f target/native-expected target/rejection.log >/dev/null
    cmp "$native/.git/index" target/native-index
    cmp "$native/AGENTS.md" target/native-working
    [[ "$(cat "$native/$unusual")" == 'retained evidence' ]]

    TEST_NATIVE_GIT_FAIL=status expect_failure bash "$helper" "$ROOT/scripts/release/release.sh" "$native" paths "$source"
    rg -F 'injected native Git observation failure' target/rejection.log >/dev/null
    rg -F 'cannot inspect release-source status' target/rejection.log >/dev/null
    if rg -F 'uncommitted paths' target/rejection.log >/dev/null; then exit 1; fi
    cmp "$native/.git/index" target/native-index

    "$TEST_REAL_GIT" -C "$native" restore --staged -- README.md
    "$TEST_REAL_GIT" -C "$native" show HEAD:AGENTS.md > "$native/AGENTS.md"
    rm "$native/$unusual"
    bash "$helper" "$ROOT/scripts/release/release.sh" "$native" clean "$source"
    printf '\n- Permitted pending notes.\n' >> "$native/CHANGELOG.md"
    expect_failure bash "$helper" "$ROOT/scripts/release/release.sh" "$native" clean "$source"
    rg -F 'unstaged: CHANGELOG.md' target/rejection.log >/dev/null
    [[ "$("$TEST_REAL_GIT" -C "$native" rev-parse HEAD)" == "$source" ]]
}

if [[ "$metadata_only" == true ]]; then
    for outcome in success failure; do
        run_case "dependency-$outcome" test_dependency_bootstrap "$outcome"
    done
    run_case validation-source-identity test_validation_source_identity
    run_case real-index-boundaries test_real_index_boundaries
    echo 'release metadata real-Git and validation-retention tests passed'
    exit 0
fi

for outcome in success failure; do
    run_case "dependency-$outcome" test_dependency_bootstrap "$outcome"
    run_case "nested-dependency-$outcome" test_nested_dependency_bootstrap "$outcome"
done
run_case validation-source-identity test_validation_source_identity
run_case versions test_versions
for mode in comments selected duplicate failed-output empty failed-parser; do run_case "version-reader-$mode" test_version_reader "$mode"; done
for invalid in conflict duplicate dated historical-duplicate competing same-date unnumbered; do run_case "notes-$invalid" test_invalid_changelog "$invalid"; done
for mode in empty missing imported bumped failed-output empty-output; do run_case "shared-notes-$mode" test_shared_changelog "$mode"; done
run_case preparation test_preparation
run_case multi-package-preparation test_multi_package_preparation
for failure in mismatch duplicate missing failed-output; do run_case "lockfile-$failure" test_lockfile_rejection "$failure"; done
for failure in TEST_FETCH_FAIL TEST_GATE_FAIL TEST_METADATA_FAIL TEST_DIRTY TEST_TAG_EXISTS TEST_GATE_DIRTY TEST_GATE_HEAD TEST_PREPARED_FORMAT_FAIL; do run_case "reject-$failure" test_rejected_preparation "$failure"; done
run_case staging test_staging
for state in no-release tagged; do run_case "publish-$state" test_publish "$state"; done
run_case publish-failure test_publish_failure
for drift in notes tag head; do run_case "completed-$drift" test_invalid_release "$drift"; done
for custody in missing member; do run_case "validation-$custody" test_validation_custody "$custody"; done
run_case early-plan-retry-retention test_early_plan_retry
run_case prepared-normal-retry test_prepared_normal_retry
for mode in explicit patch minor gate-failure; do run_case "older-release-$mode" test_older_release_recovery "$mode"; done
for reason in missing identity remote; do run_case "selected-proof-$reason" test_selected_proof_rejection "$reason"; done
run_case completed-evidence-replay test_completed_evidence_replay
run_case dependency-failure-conditional-status test_dependency_failure_conditional_status
run_case conditional-substitute-rejections test_conditional_substitute_rejections
run_case real-index-boundaries test_real_index_boundaries
printf 'Release adapters: PASS (Make/gate/recovery command substitutes and real private-index boundaries; no new Git commits or live publication).\n'
