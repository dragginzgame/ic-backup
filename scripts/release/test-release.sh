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
# Historical fixture ownership is recorded in docs/release-fixture-ownership.json.
# Current recovery behavior belongs to docs/releasing.md. The canonical shared
# suite owns increment/phase and generic lost-reply matrices.
unset RELEASE_SOURCE RELEASE_PREVIOUS RELEASE_VERSION RELEASE_DATE RELEASE_KIND RELEASE_COMMIT \
    RELEASE_REMOTE RELEASE_BRANCH RELEASE_MAKE RELEASE_DELIVERY VERSION \
    MAKEFLAGS MAKEOVERRIDES MFLAGS MAKELEVEL GNUMAKEFLAGS MAKEFILES
unset VALIDATION_REPOSITORY_ROOT VALIDATION_RUNNER_SNAPSHOT_PATH
metadata_only=false
receipts_only=false
preparation_only=false
if [[ "$#" == 1 && "$1" == --metadata-only ]]; then
    metadata_only=true
elif [[ "$#" == 1 && "$1" == --receipts-only ]]; then
    receipts_only=true
elif [[ "$#" == 1 && "$1" == --preparation-only ]]; then
    preparation_only=true
elif [[ "$#" != 0 ]]; then
    echo 'usage: test-release.sh [--metadata-only|--preparation-only|--receipts-only]' >&2
    exit 2
fi
TEST_REAL_MAKE="$(command -v make)"
TEST_REAL_GIT="$(command -v git)"
TEST_REAL_CARGO="$(command -v cargo)"
export TEST_REAL_MAKE TEST_REAL_GIT TEST_REAL_CARGO
TEST_RELEASE_EDITOR="$("$TEST_REAL_MAKE" --no-print-directory -s -C "$ROOT" release-editor-check)"
export TEST_RELEASE_EDITOR CARGO_RELEASE_METADATA_EDITOR="$TEST_RELEASE_EDITOR"
export CARGO_RELEASE_METADATA_MAX_BYTES=1048576
export PATH="$ROOT/.tools/host/bin:$PATH"
mkdir -p "$ROOT/target"
TEMPORARY="$(mktemp -d "$ROOT/target/shared-release-tests.XXXXXX")"
mkdir -p "$TEMPORARY/bin"
CASE_NAME=setup
CASE_LOG=/dev/null
fixture_complete=false
export TEST_SOURCE=1111111111111111111111111111111111111111
finish() {
    local status=$?
    [[ "$fixture_complete" == true || "$status" != 0 ]] || status=1
    if [[ "$status" != 0 ]]; then
        printf 'Release adapter tests failed [%s]; retained: %s\n' "$CASE_NAME" "$TEMPORARY" >&2
        tail -n 80 "$CASE_LOG" >&2
    else
        printf 'Release adapter fixtures and command traces retained: %s\n' "$TEMPORARY"
    fi
    exit "$status"
}
trap finish EXIT
run_case() {
    CASE_NAME="$1"; shift
    CASE_LOG="$TEMPORARY/$CASE_NAME.log"
    printf '%s\n' "$CASE_NAME" >> "$TEMPORARY/cases.txt"
    export FIXTURE="$TEMPORARY/$CASE_NAME"
    (
        case_complete=false
        trap 'status=$?; [[ "$case_complete" == true || "$status" != 0 ]] || status=1; exit "$status"' EXIT
        create_fixture
        "$@"
        case_complete=true
    ) > "$CASE_LOG" 2>&1
}
create_fixture() {
    mkdir -p "$FIXTURE/scripts/release" "$FIXTURE/scripts/ci" "$FIXTURE/crates/ic-backup/src" "$FIXTURE/docs" "$FIXTURE/target/debug"
    mkdir -p "$FIXTURE/make"
    bash "$ROOT/scripts/ci/prepare-script-fixture.sh" "$ROOT" "$FIXTURE" \
        Makefile make/consumer.mk scripts/release/release.sh scripts/release/release-data.pl \
        scripts/ci/check-release-source.sh scripts/ci/check-release-tag.sh \
        scripts/ci/next-release-version.sh scripts/ci/validate-release-inputs.pl \
        scripts/ci/read-cargo-workspace-version.sh scripts/ci/finalize-release-changelog.awk \
        scripts/ci/cargo-release-metadata.sh scripts/ci/cargo-workspace-members.jq \
        scripts/ci/compare-stable-versions.pl
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
    printf 'version = 4\n\n[[package]]\nname = "ic-backup"\nversion = "0.1.0"\n\n[[package]]\nname = "retained-dependency"\nversion = "9.8.7"\nsource = "registry+https://github.com/rust-lang/crates.io-index"\n' > Cargo.lock
    printf '\n[[package]]\nname = "ic-testkit"\nversion = "0.28.1"\nsource = "registry+https://github.com/rust-lang/crates.io-index"\n' >> Cargo.lock
    printf '# Changelog\n\n## [0.1.1]\n\n- Completed notes.\n\n## [0.1.0] - 2026-10-01\n\n- Retained history.\n\n## [0.0.9]\n\n- Imported undated history.\n' > CHANGELOG.md
    sed -n '/^## \[0.1.0\]/,$p' CHANGELOG.md > target/history
    export TEST_LOG="$FIXTURE/target/commands.log" TEST_EFFECTS="$FIXTURE/target/effects.log"
    : > "$TEST_LOG"; : > "$TEST_EFFECTS"
    mkdir -p scripts/dev target/mock-tools/0.28.1
    printf 'original receipt\n' > target/mock-tools/0.28.1/receipt
    : > target/tool-effects.log
    cat > target/release-editor <<'EDITOR'
#!/usr/bin/env bash
exec "$TEST_RELEASE_EDITOR" "$@"
EDITOR
    chmod +x target/release-editor
    export CARGO_RELEASE_METADATA_EDITOR="$FIXTURE/target/release-editor"
    # Substitutes qualify consumer ordering only. Canonical installer tests own
    # receipt/lock integrity and real Testkit owns server asset admission.
    cat > scripts/dev/install-host-tools.sh <<'TOOL'
#!/usr/bin/env bash
set -euo pipefail
target="${0##*/}"; target="${target%.sh}"
if [[ "${!#}" == --preflight ]]; then
    printf 'preflight %s\n' "$target" >> "$TEST_LOG"
    [[ "${TEST_COMMON_PREFLIGHT_FAIL:-}" != "$target" ]] || exit 24
    exit 0
fi
printf 'common %s\n' "$target" >> "$TEST_LOG"
[[ "${TEST_COMMON_TOOL_FAIL:-}" != "$target" ]] || exit 8
TOOL
    cat > scripts/dev/install-rust-tools.sh <<'TOOL'
#!/usr/bin/env bash
set -euo pipefail
printf 'installer %s\n' "$*" >> "$TEST_LOG"
version=''; package=''; features=''; check=false; preflight=false
while [[ $# -gt 0 ]]; do
    case "$1" in
        --package) package="$2"; shift 2 ;;
        --features) features="$2"; shift 2 ;;
        --lockfile)
            # Project this fixture's known package. Strict source/ambiguity/drift
            # admission belongs to the real shared installer's own regressions.
            version="$(yq -p toml -o json -I 0 '.' "$2" | jq -er '.package[] | select(.name == "ic-testkit") | .version')"
            shift 2 ;;
        --check) check=true; shift ;;
        --preflight) preflight=true; shift ;;
        *) shift ;;
    esac
done
case "$package" in
    ic-build|ic-testkit) [[ "$features" == cli ]] || exit 27 ;;
esac
if [[ "$package" == ic-build ]]; then
    printf '%s/target/release-editor\n' "$PWD"
    exit 0
fi
if [[ "$preflight" == true ]]; then
    printf 'preflight install-rust-tools\n' >> "$TEST_LOG"
    [[ "${TEST_COMMON_PREFLIGHT_FAIL:-}" != install-rust-tools ]] || exit 24
    exit 0
fi
if [[ -z "$version" ]]; then
    printf 'common install-rust-tools\n' >> "$TEST_LOG"
    [[ "${TEST_COMMON_TOOL_FAIL:-}" != install-rust-tools ]] || exit 8
    exit 0
fi
selection="target/mock-tools/$version"
[[ ! -e "$selection/invalid" && ! -d "$selection/lock" ]] || exit 8
if [[ ! -f "$selection/receipt" ]]; then
    [[ "$check" == false && "${CARGO_NET_OFFLINE:-false}" != true && "${TEST_TOOL_FAIL:-0}" != 1 ]] || exit 8
    mkdir -p "$selection"
    printf 'selected receipt\n' > "$selection/receipt"
    printf 'build %s\n' "$version" >> target/tool-effects.log
fi
printf '%s/target/mock-testkit-cli\n' "$PWD"
TOOL
    cat > target/mock-testkit-cli <<'TOOL'
#!/usr/bin/env bash
set -euo pipefail
printf 'cli %s\n' "$*" >> "$TEST_LOG"
printf '%s\n' "$1" >> target/tool-effects.log
[[ "${TEST_SERVER_FAIL:-0}" != 1 ]] || exit 8
printf '%s/mock-server\n' "$PWD"
TOOL
    chmod +x target/mock-testkit-cli
    printf 'retained build artifact\n' > target/debug/cache-sentinel
    export VALIDATION_REPOSITORY_ROOT="$FIXTURE" VALIDATION_FAILURE_LOG_DIR="$FIXTURE/target/validation-failures" GITHUB_STEP_SUMMARY="$FIXTURE/target/summary.md"
    unset YQ
    unset TEST_CLIPPY_FAIL TEST_CLIPPY_ORDER TEST_DIRTY TEST_TAG_EXISTS TEST_GATE_FAIL TEST_METADATA_FAIL TEST_PUSH_FAIL TEST_FETCH_FAIL TEST_PUBLISH_FAIL TEST_LOST_EFFECT TEST_GATE_DIRTY TEST_GATE_HEAD TEST_DESTINATION TEST_PREPARED_FORMAT_FAIL TEST_REMOTE_FAIL TEST_TOOL_FAIL TEST_SERVER_FAIL TEST_COMMON_TOOL_FAIL CARGO_NET_OFFLINE
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
assert_unchanged() { [[ "$(fingerprint)" == "$before" && ! -e docs/release.json ]] || exit 1; }
assert_cache_retained() { [[ "$(cat target/debug/cache-sentinel)" == 'retained build artifact' ]] || exit 1; }

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
        [[ -e "$file" || -e "target/commits/$current/files/$file" ]] || continue
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
            *'^{commit}')
                name="${value%\^\{commit\}}"
                [[ "$name" =~ ^[0-9a-f]{40}$ && -d "target/commits/$name" ]] || exit 1
                echo "$name" ;;
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
                [[ -e "$file" || -e "target/commits/$(head)/files/$file" ]] || continue
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
    cat-file)
        if [[ "$3" =~ ^[0-9a-f]{40}$ && -d "target/commits/$3" ]]; then echo commit
        else [[ -f target/mock-tag && -f "target/tags/${3#refs/tags/}.object" ]] || exit 1; echo tag; fi ;;
    archive)
        selected="${*: -1}"
        [[ "$selected" =~ ^[0-9a-f]{40}$ && -d "target/commits/$selected/files" ]] || exit 1
        tar -cf - -C "target/commits/$selected/files" . ;;
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
# The actual release-verify/CI recipe retains its target roster and sequencing.
# Substitute preparation and expensive stages; run the real Clippy recipe with
# the exact Cargo substitute below, including under parallel parent Make.
if [[ "${TEST_CLIPPY_ORDER:-0}" == 1 && -n "$target" && "$target" != clippy ]]; then
    printf '%s\n' "$target" >> "$TEST_EFFECTS"
    exit 0
fi
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
    'sort --workspace --check --grouped') exec "$TEST_REAL_CARGO" "$@" ;;
    'metadata --locked --offline --format-version 1')
        version="$(yq -p toml -o json '.workspace.package.version' Cargo.toml | jq -r .)"
        jq -n --arg version "$version" '{workspace_members:["fixture"],packages:[{id:"fixture",name:"ic-backup",version:$version,source:null}]}' ;;
    'fetch --locked') echo fetch >> "$TEST_EFFECTS"; [[ "${TEST_FETCH_FAIL:-0}" != 1 ]] || exit 7; touch target/mock-cache ;;
    'clippy --offline --locked -p ic-backup -p ic-backup-agent --all-targets --all-features -- -D warnings')
        echo clippy >> "$TEST_EFFECTS"; [[ "${TEST_CLIPPY_FAIL:-0}" != 1 ]] || exit 23 ;;
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
        [[ "$(cat "$TEST_EFFECTS")" == fetch ]] || exit 1
        [[ -s target/validation-failures/latest.log && -s target/validation-failures/latest-errors.log ]] || exit 1
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
    [[ "$(cat target/parent/summary)" == 'parent evidence' ]] || exit 1
}
test_validation_source_identity() {
    before="$(fingerprint)"
    # Explicit per-call identity still reaches the real production guard after
    # the suite has excluded enclosing release/Make context at its entry.
    expect_failure "$TEST_REAL_MAKE" --no-print-directory release-verify \
        'CI_TARGETS=deps check' RELEASE_SOURCE=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
        RELEASE_PREVIOUS=0.1.0 RELEASE_VERSION=0.1.1 RELEASE_DATE=2026-10-05
    assert_unchanged
    [[ ! -e target/release-state/0.1.1.validation.json ]] || exit 1
    "$TEST_REAL_MAKE" --no-print-directory release-verify 'CI_TARGETS=deps check' \
        "RELEASE_SOURCE=$TEST_SOURCE" RELEASE_PREVIOUS=0.1.0 \
        RELEASE_VERSION=0.1.1 RELEASE_DATE=2026-10-05
    perl scripts/release/release-data.pl validation-check \
        target/release-state/0.1.1.validation.json "$TEST_SOURCE" 2026-10-05 0.1.0 0.1.1 original
    assert_unchanged
    [[ ! -e target/release-state/0.1.1.plan && ! -e target/mock-staged && ! -e target/mock-tag && ! -e target/remote-head ]] || exit 1
    assert_cache_retained
}
test_versions() {
    local requested expected value before
    before="$(fingerprint)"
    # Invoke the actual consumer entry point under a noisy directory-search environment.
    [[ "$(CDPATH="$PWD" bash scripts/release/release.sh version)" == 0.1.0 ]] || exit 1
    while read -r requested expected; do
        CDPATH="$PWD" "$TEST_REAL_MAKE" --no-print-directory release-plan "VERSION=$requested" > target/plan
        rg -Fx 'Current: 0.1.0' target/plan
        rg -Fx "Target:  $expected" target/plan
        assert_unchanged
    done <<'VERSIONS'
patch 0.1.1
minor 0.2.0
major 1.0.0
0.1.0 0.1.0
0.1.1 0.1.1
0.10.0 0.10.0
1.0.0 1.0.0
999999999.999999999.999999999 999999999.999999999.999999999
VERSIONS
    expect_failure "$TEST_REAL_MAKE" --no-print-directory release-plan VERSION=0.0.9
    rg -F 'target version must not decrease' target/rejection.log
    assert_unchanged
    for value in 01.2.3 0.01.1 0.1.01 v0.1.1 0.1 0.1.1.0 0.1.1-rc.1 0.1.1+build \
        1000000000.0.0 0.1000000000.0 0.1.1000000000; do
        expect_failure "$TEST_REAL_MAKE" --no-print-directory release-plan "VERSION=$value"
        rg -F 'expected canonical x.y.z with components below 1 billion' target/rejection.log
        assert_unchanged
    done
    expect_failure perl scripts/release/release-data.pl next ''
    assert_unchanged
    [[ ! -s "$TEST_EFFECTS" ]] || exit 1
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
            [[ "${#candidates[@]}" == 1 && -f "${candidates[0]}" && ! -s "${candidates[0]}" ]] || exit 1
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
        [[ "$(perl scripts/release/release-data.pl version)" == 0.1.0 ]] || exit 1
        assert_unchanged
    elif [[ "$mode" == render ]]; then
        sed 's/version = "0.1.0"/version = "0.1.0" # retained version comment/' Cargo.toml > target/commented.toml
        cp target/commented.toml Cargo.toml
        cp Cargo.lock target/original.lock
        cp crates/ic-backup/Cargo.toml target/original-member.toml
        perl scripts/release/release-data.pl prepare-version 0.1.1
        rg -Fx 'version = "0.1.1" # retained version comment' Cargo.toml
        sed 's/version = "0.1.0"/version = "0.1.1"/' target/original.lock > target/expected.lock
        cmp Cargo.lock target/expected.lock
        cmp crates/ic-backup/Cargo.toml target/original-member.toml
        [[ ! -s "$TEST_EFFECTS" ]] || exit 1
    elif [[ "$mode" == selected ]]; then
        mkdir -p "target/commits/$TEST_SOURCE/files/crates/ic-backup/src"
        cp Cargo.toml "target/commits/$TEST_SOURCE/files/"
        cp crates/ic-backup/Cargo.toml "target/commits/$TEST_SOURCE/files/crates/ic-backup/"
        cp crates/ic-backup/src/lib.rs "target/commits/$TEST_SOURCE/files/crates/ic-backup/src/"
        printf 'invalid working TOML\n' >> Cargo.toml
        printf 'invalid working member TOML\n' >> crates/ic-backup/Cargo.toml
        before="$(fingerprint)"
        [[ "$(perl scripts/release/release-data.pl version --commit "$TEST_SOURCE")" == 0.1.0 ]] || exit 1
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
        [[ ! -s "$TEST_EFFECTS" ]] || exit 1
        if perl scripts/release/release-data.pl version > target/version-output 2> target/version-error; then
            echo 'invalid version read was accepted' >&2; exit 1
        fi
        [[ ! -s target/version-output ]] || exit 1
    fi
    assert_cache_retained
}

test_preparation() {
    export TEST_LOST_EFFECT=commit
    expect_failure "$TEST_REAL_MAKE" --no-print-directory release-patch
    perl scripts/release/release-data.pl verify
    [[ -f target/release-state/0.1.1.validation.json ]] || exit 1
    rg -q '^name = "retained-dependency"$' Cargo.lock
    rg -q '^version = "9.8.7"$' Cargo.lock
    cmp target/history <(sed -n '/^## \[0.1.0\]/,$p' CHANGELOG.md)
    local backups=(target/release-backup.*)
    [[ -d "${backups[0]}" ]] || exit 1
    assert_cache_retained
}
test_multi_package_preparation() {
    local form="${1:-inline}" dependency=ic-backup expected=0.2
    perl -pi -e 's/members = \["crates\/ic-backup"\]/members = ["crates\/ic-backup", "crates\/ic-backup-agent"]/' Cargo.toml
    case "$form" in
        inline) printf '\n[workspace.dependencies]\nic-backup = { version = "0.1", path = "crates/ic-backup" }\n' >> Cargo.toml ;;
        comments)
            sed 's/version = "0.1.0"/version = "0.1.0" # retained version comment/' Cargo.toml > target/commented.toml
            cp target/commented.toml Cargo.toml
            printf '\n[workspace.dependencies]\n"ic-backup" = { path = "crates/ic-backup", version = "0.1" } # retained dependency comment\n' >> Cargo.toml ;;
        table)
            printf '\n[workspace.dependencies.ic-backup]\npath = "crates/ic-backup"\nversion = "=0.1.0" # exact member\n' >> Cargo.toml
            expected='=0.2.0' ;;
        alias)
            dependency=backup-core
            printf '\n[workspace.dependencies]\nbackup-core = { package = "ic-backup", path = "crates/ic-backup", version = "^0.1" }\n' >> Cargo.toml
            expected='^0.2' ;;
        *) echo 'unknown manifest fixture' >&2; exit 1 ;;
    esac
    mkdir -p crates/ic-backup-agent/src
    printf '[package]\nname = "ic-backup-agent"\nversion.workspace = true\nedition.workspace = true\n[dependencies]\n%s.workspace = true\n' "$dependency" > crates/ic-backup-agent/Cargo.toml
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
    yq -p toml -o json '.' Cargo.toml | jq -e --arg dependency "$dependency" --arg expected "$expected" \
        '.workspace.dependencies[$dependency].version == $expected'
    if [[ "$form" == comments ]]; then
        rg -Fx 'version = "0.2.0" # retained version comment' Cargo.toml
        rg -Fx '"ic-backup" = { path = "crates/ic-backup", version = "0.2" } # retained dependency comment' Cargo.toml
    fi
    [[ "$(rg -c 'version = "0.2.0"' Cargo.lock)" == 2 ]] || exit 1
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
        failed-output|failed-renderer|empty-renderer)
            cat > target/release-editor <<'EDITOR'
#!/usr/bin/env bash
if [[ "$TEST_EDITOR_FAILURE" == failed-output && "$3" == lock ]]; then
    printf 'version = 4\n'; exit 13
elif [[ "$TEST_EDITOR_FAILURE" == failed-renderer && "$3" == manifest ]]; then
    printf '[workspace.package]\nversion = "0.1.1"\n'; exit 13
elif [[ "$TEST_EDITOR_FAILURE" == empty-renderer && "$3" == manifest ]]; then
    exit 0
fi
exec "$TEST_RELEASE_EDITOR" "$@"
EDITOR
            export TEST_EDITOR_FAILURE="$1" ;;
    esac
    before="$(fingerprint)"
    expect_failure perl scripts/release/release-data.pl prepare-version 0.1.1
    assert_unchanged
    [[ ! -s "$TEST_EFFECTS" ]] || exit 1
    assert_cache_retained
}
test_rejected_preparation() {
    before="$(fingerprint)"
    export "$1=1"
    expect_failure "$TEST_REAL_MAKE" --no-print-directory release-patch
    if [[ "$1" != TEST_GATE_DIRTY ]]; then assert_unchanged; fi
    [[ ! -f target/mock-tag && ! -f target/remote-head ]] || exit 1
    assert_cache_retained
}
test_staging() {
    before="$(fingerprint)"
    expect_failure "$TEST_REAL_MAKE" --no-print-directory release-patch release-minor
    assert_unchanged
    [[ ! -s "$TEST_EFFECTS" ]] || exit 1
    for mode in -i -n -t -q; do
        expect_failure "$TEST_REAL_MAKE" --no-print-directory "$mode" release-patch
        assert_unchanged
        [[ ! -s "$TEST_EFFECTS" && ! -e target/release-state ]] || exit 1
    done
    expect_failure "$TEST_REAL_MAKE" --no-print-directory release-stage
    for delivery in pr invalid; do
        expect_failure "$TEST_REAL_MAKE" --no-print-directory release-patch "RELEASE_DELIVERY=$delivery"
        expect_failure "$TEST_REAL_MAKE" --no-print-directory release-resume VERSION=0.1.1 "RELEASE_DELIVERY=$delivery"
        assert_unchanged
        [[ ! -s "$TEST_EFFECTS" && ! -e target/release-state ]] || exit 1
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
    [[ "$(cat "$TEST_EFFECTS")" == $'clippy\npublish' && "$(fingerprint)" == "$before" ]] || exit 1
    assert_cache_retained
}
test_publish_failure() {
    before="$(fingerprint)"; export TEST_PUBLISH_FAIL=1
    expect_failure "$TEST_REAL_MAKE" --no-print-directory publish-dry-run
    assert_unchanged
    [[ "$(cat "$TEST_EFFECTS")" == clippy ]] || exit 1
}
test_clippy_gate() {
    local entry="$1" jobs="$2" outcome="$3"
    before="$(fingerprint)"
    if [[ "$entry" == release-verify ]]; then
        cat >> Makefile <<'MAKE'
.PHONY: fixture-ci-targets
fixture-ci-targets:
	@printf '%s\n' $(CI_TARGETS)
MAKE
        "$TEST_REAL_MAKE" --no-print-directory -s fixture-ci-targets > target/expected-order
        sed -n '1,/^clippy$/p' target/expected-order > target/lint-prefix
        [[ "$(tail -n 1 target/lint-prefix)" == clippy ]] || exit 1
        if grep -Ex 'check|test|doc|check-msrv|package|tooling-check|release-check|hooks-check' target/lint-prefix; then
            echo 'expensive stage precedes Clippy' >&2; exit 1
        fi
        export TEST_CLIPPY_ORDER=1
    else
        printf '%s\n' clippy publish > target/expected-order
        printf '%s\n' clippy > target/lint-prefix
    fi
    if [[ "$outcome" == failure ]]; then
        export TEST_CLIPPY_FAIL=1
        expect_failure "$TEST_REAL_MAKE" --no-print-directory "-j$jobs" "$entry"
        cmp target/lint-prefix "$TEST_EFFECTS"
        if grep -F 'cargo publish ' "$TEST_LOG"; then exit 1; fi
    else
        "$TEST_REAL_MAKE" --no-print-directory "-j$jobs" "$entry"
        cmp target/expected-order "$TEST_EFFECTS"
    fi
    grep -Fx 'cargo clippy --offline --locked -p ic-backup -p ic-backup-agent --all-targets --all-features -- -D warnings' "$TEST_LOG" >/dev/null
    [[ ! -d target/release-state/lock ]] || exit 1
    assert_unchanged
    assert_cache_retained
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
    [[ ! -s "$TEST_EFFECTS" ]] || exit 1
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
    [[ ! -s "$TEST_EFFECTS" && ! -f target/mock-tag ]] || exit 1
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
    local proofs=(target/release-state/0.1.1.validation.*/validation.json)
    cmp target/earlier-proof "${proofs[0]}"
    [[ "$(tail -n 1 target/release-state/0.1.1.plan)" == complete ]] || exit 1
    [[ "$(perl scripts/release/release-data.pl source)" == "$TEST_SOURCE" ]] || exit 1
    assert_cache_retained
}
test_prepared_explicit_resume() {
    export TEST_LOST_EFFECT=commit
    expect_failure "$TEST_REAL_MAKE" --no-print-directory release-patch
    unset TEST_LOST_EFFECT
    before="$(fingerprint)"
    cp docs/release.json target/original-receipt.json
    cp target/release-state/0.1.1.validation.json target/original-validation.json
    perl scripts/release/release-data.pl verify
    : > "$TEST_EFFECTS"
    cp target/tool-effects.log target/tools-before-resume
    "$TEST_REAL_MAKE" --no-print-directory release-resume VERSION=0.1.1
    [[ "$(fingerprint)" == "$before" && "$(cat "$TEST_EFFECTS")" == $'tag\npush' ]] || exit 1
    cmp docs/release.json target/original-receipt.json
    cmp target/release-state/0.1.1.validation.json target/original-validation.json
    perl scripts/release/release-data.pl verify
    [[ "$(perl scripts/release/release-data.pl version)" == 0.1.1 ]] || exit 1
    [[ "$(tail -n 1 target/release-state/0.1.1.plan)" == complete ]] || exit 1
    cmp target/history <(sed -n '/^## \[0.1.0\]/,$p' CHANGELOG.md)
    [[ ! -e target/release-state/0.1.2.plan ]] || exit 1
    "$TEST_REAL_MAKE" --no-print-directory release-resume VERSION=0.1.1
    cmp target/tool-effects.log target/tools-before-resume
    assert_cache_retained
}

test_committed_preparation_retry() {
    local disposition="$1" fix=ffffffffffffffffffffffffffffffffffffffff next=0.1.1
    # The real adapter writes metadata, fails its format check, then rolls back.
    TEST_PREPARED_FORMAT_FAIL=1 expect_failure "$TEST_REAL_MAKE" --no-print-directory release-patch
    [[ "$(tail -n 1 target/release-state/0.1.1.plan)" == prepare ]] || exit 1
    [[ "$(perl scripts/release/release-data.pl version)" == 0.1.0 && ! -e docs/release.json ]] || exit 1
    cp target/release-state/0.1.1.plan target/original-plan
    cp target/release-state/0.1.1.validation.json target/original-validation.json
    local backups=(target/release-backup.*/Cargo.toml)
    [[ "${#backups[@]}" == 1 ]] || exit 1
    cp "${backups[0]}" target/original-backup
    if [[ "$disposition" == bumped ]]; then
        # A maintainer may instead commit the local metadata disposition.
        perl scripts/release/release-data.pl finalize 0.1.1 2026-10-11 0.1.0
        perl scripts/release/release-data.pl prepare-version 0.1.1
        perl scripts/release/release-data.pl receipt "$TEST_SOURCE" 2026-10-11
        { printf '# Changelog\n\n## [0.1.2]\n\n- Follow-up notes.\n\n'; sed -n '/^## \[0.1.1\]/,$p' CHANGELOG.md; } > target/new-notes
        cp target/new-notes CHANGELOG.md
        next=0.1.2
    fi
    mkdir -p "target/commits/$fix/files/crates/ic-backup" "target/commits/$fix/files/docs"
    printf '%s\n' "$TEST_SOURCE" > "target/commits/$fix/parent"
    cp Cargo.toml Cargo.lock CHANGELOG.md "target/commits/$fix/files/"
    cp crates/ic-backup/Cargo.toml "target/commits/$fix/files/crates/ic-backup/"
    if [[ -e docs/release.json ]]; then cp docs/release.json "target/commits/$fix/files/docs/"; fi
    printf '%s\n' "$fix" > target/mock-head
    before="$(fingerprint)"
    : > "$TEST_EFFECTS"
    if [[ "$disposition" == bumped ]]; then
        # A clean committed candidate may advance, preserving its unconfirmed intent.
        cp docs/release.json target/original-receipt.json
    fi
    TEST_GATE_FAIL=1 expect_failure "$TEST_REAL_MAKE" --no-print-directory release-patch
    [[ "$(fingerprint)" == "$before" && "$(cat "$TEST_EFFECTS")" == $'fetch\nvalidate' ]] || exit 1
    cmp target/original-validation.json target/release-state/0.1.1.validation.json
    cmp target/original-backup "${backups[0]}"
    if [[ "$disposition" == bumped ]]; then
        cmp target/original-receipt.json docs/release.json
        cmp target/original-plan target/release-state/0.1.1.plan
        [[ ! -e target/tags/v0.1.1.commit && ! -e target/tags/v0.1.1.object ]] || exit 1
    fi
    [[ ! -e "target/release-state/$next.plan" ]] || exit 1
    "$TEST_REAL_MAKE" --no-print-directory release-patch
    [[ "$(perl scripts/release/release-data.pl version)" == "$next" && "$(perl scripts/release/release-data.pl source)" == "$fix" ]] || exit 1
    [[ "$(tail -n 1 "target/release-state/$next.plan")" == complete ]] || exit 1
    if [[ "$disposition" == bumped ]]; then
        cmp target/original-plan target/release-state/0.1.1.plan
    else
        local proofs=(target/release-state/0.1.1.validation.*/validation.json)
        cmp target/original-validation.json "${proofs[0]}"
    fi
    cmp target/original-backup "${backups[0]}"
    assert_cache_retained
}

test_selected_tool_preflight() {
    local mode="$1"
    perl -pi -e 's/version = "0\.28\.1"/version = "0.28.2"/' Cargo.lock
    before="$(fingerprint)"
    case "$mode" in
        retry) export TEST_GATE_FAIL=1 ;;
        offline) export CARGO_NET_OFFLINE=true ;;
        network) export TEST_TOOL_FAIL=1 ;;
        common) export TEST_COMMON_TOOL_FAIL=install-rust-tools ;;
        prerequisite) export TEST_COMMON_PREFLIGHT_FAIL=install-rust-tools ;;
        invalid|lock)
            mkdir -p target/mock-tools/0.28.2
            if [[ "$mode" == invalid ]]; then touch target/mock-tools/0.28.2/invalid; else mkdir target/mock-tools/0.28.2/lock; fi ;;
    esac
    expect_failure "$TEST_REAL_MAKE" --no-print-directory release-patch
    assert_unchanged
    [[ "$(cat target/mock-tools/0.28.1/receipt)" == 'original receipt' ]] || exit 1
    if [[ "$mode" == retry ]]; then
        [[ "$(cat target/tool-effects.log)" == $'build 0.28.2\nsetup\ncheck' ]] || exit 1
        # First original-source preflight fetched before setup/admission/gate.
        awk '/^cargo fetch/{fetch=NR} /^cli setup/{setup=NR} /^cli check/{check=NR} /^make .*release-verify/{gate=NR} END {exit !(fetch < setup && setup < check && check < gate)}' "$TEST_LOG"
        awk '/^common install-host-tools$/{host=NR} /^common install-rust-tools$/{rust=NR} /^cargo fetch/{fetch=NR} /^cli setup/{exit !(host > 0 && host < rust && rust < fetch)}' "$TEST_LOG"
        unset TEST_GATE_FAIL
        "$TEST_REAL_MAKE" --no-print-directory release-patch
        [[ "$(grep -c '^build ' target/tool-effects.log)" == 1 ]] || exit 1
        [[ "$(grep -c '^setup$' target/tool-effects.log)" == 3 ]] || exit 1
        cp target/tool-effects.log target/tools-before-resume
        "$TEST_REAL_MAKE" --no-print-directory release-resume VERSION=0.1.1
        cmp target/tool-effects.log target/tools-before-resume
    else
        [[ ! -s target/tool-effects.log ]] || exit 1
        if grep -q '^validate$' "$TEST_EFFECTS"; then exit 1; fi
        if [[ "$mode" == prerequisite ]] && grep -q '^common ' "$TEST_LOG"; then exit 1; fi
    fi
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
    cp target/release-state/0.1.1.plan target/original-plan
    record_reviewed_fix "$next"
    if [[ "$mode" == explicit-unsorted ]]; then
        # Current HEAD is deliberately unsorted but clean in the Git substitute.
        # Resuming the earlier release must sort-check its selected original tree.
        printf '\n[workspace.dependencies]\n# Production dependencies\nzzz = "1"\naaa = "1"\n' >> Cargo.toml
        cp Cargo.toml "target/commits/ffffffffffffffffffffffffffffffffffffffff/files/Cargo.toml"
        if "$TEST_REAL_CARGO" sort --workspace --check --grouped > target/current-order.log 2>&1; then
            echo 'selected-source fixture must have an unsorted current manifest' >&2; exit 1
        fi
        cp Cargo.toml target/unsorted-current.toml
    fi
    expect_failure perl scripts/release/release-data.pl verify
    : > "$TEST_EFFECTS"
    if [[ "$mode" == explicit || "$mode" == explicit-unsorted ]]; then
        "$TEST_REAL_MAKE" --no-print-directory release-resume VERSION=0.1.1
        [[ "$(cat "$TEST_EFFECTS")" == push ]] || exit 1
        [[ "$(cat target/remote-head)" == bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb ]] || exit 1
        [[ "$(perl scripts/release/release-data.pl version)" == 0.1.1 ]] || exit 1
        [[ ! -e "target/release-state/$next.plan" ]] || exit 1
        if [[ "$mode" == explicit-unsorted ]]; then
            cmp Cargo.toml target/unsorted-current.toml
            rg -F 'cargo sort --workspace --check --grouped' "$TEST_LOG" > /dev/null
        fi
    else
        if [[ "$mode" == gate-failure ]]; then
            export TEST_GATE_FAIL=1
            expect_failure "$TEST_REAL_MAKE" --no-print-directory release-patch
            [[ "$(perl scripts/release/release-data.pl version)" == 0.1.1 ]] || exit 1
            [[ ! -e target/release-state/0.1.2.plan ]] || exit 1
            [[ ! -e target/remote-head ]] || exit 1
            cmp target/original-plan target/release-state/0.1.1.plan
            unset TEST_GATE_FAIL
            "$TEST_REAL_MAKE" --no-print-directory release-patch
        else
            "$TEST_REAL_MAKE" --no-print-directory "release-$mode"
        fi
        [[ "$(perl scripts/release/release-data.pl version)" == "$next" ]] || exit 1
        [[ "$(perl scripts/release/release-data.pl source)" == ffffffffffffffffffffffffffffffffffffffff ]] || exit 1
        [[ "$(tail -n 1 "target/release-state/$next.plan")" == complete ]] || exit 1
        [[ "$(cat "target/tags/v$next.commit")" == "$(cat target/mock-head)" ]] || exit 1
        rg -q 'release-verify.*RELEASE_SOURCE=ffffffffffffffffffffffffffffffffffffffff' "$TEST_LOG"
        perl scripts/release/release-data.pl verify
    fi
    [[ "$(cat target/tags/v0.1.1.commit)" == bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb ]] || exit 1
    if [[ "$mode" == explicit || "$mode" == explicit-unsorted ]]; then
        [[ "$(tail -n 1 target/release-state/0.1.1.plan)" == complete ]] || exit 1
        [[ "$(sed -n '2p' target/pushes.log)" == 'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb v0.1.1' ]] || exit 1
    else
        cmp target/original-plan target/release-state/0.1.1.plan
        [[ "$(wc -l < target/pushes.log)" -eq 2 ]] || exit 1
        [[ "$(sed -n '2p' target/pushes.log)" == "$(cat target/mock-head) v$next" ]] || exit 1
    fi
    cmp target/original-proof.json target/release-state/0.1.1.validation.json
    perl scripts/release/release-data.pl verify --commit bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
    assert_cache_retained
}
test_older_release_padded_count() {
    # macOS wc pads counts on stdin; keep its presentation independent of the ledger.
    TEST_REAL_WC="$(command -v wc)"
    export TEST_REAL_WC
    mkdir -p target/padded-wc
    cat > target/padded-wc/wc <<'WC'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$#" == 1 && "$1" == -l ]]; then
    count="$("$TEST_REAL_WC" "$@")"
    printf '%8d\n' "$count" > "$FIXTURE/target/padded-count"
    cat "$FIXTURE/target/padded-count"
else
    exec "$TEST_REAL_WC" "$@"
fi
WC
    chmod +x target/padded-wc/wc
    export PATH="$PWD/target/padded-wc:$PATH"
    test_older_release_recovery patch
    [[ -s target/padded-count ]] || exit 1
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
    expect_failure "$TEST_REAL_MAKE" --no-print-directory release-resume VERSION=0.1.1
    [[ ! -s "$TEST_EFFECTS" && ! -f target/remote-head ]] || exit 1
    [[ "$(tail -n 1 target/release-state/0.1.1.plan)" == push ]] || exit 1
    assert_cache_retained
}
test_completed_evidence_replay() {
    "$TEST_REAL_MAKE" --no-print-directory release-patch
    mv target/release-state/0.1.1.validation.json target/retained-proof.json
    : > "$TEST_EFFECTS"
    expect_failure "$TEST_REAL_MAKE" --no-print-directory release-resume VERSION=0.1.1
    [[ ! -s "$TEST_EFFECTS" ]] || exit 1
    # The new common resume input cannot disable Backup's mandatory receipt check.
    expect_failure "$TEST_REAL_MAKE" --no-print-directory release-resume VERSION=0.1.1 "RELEASE_RESUME_CHECK_COMMAND=:"
    [[ ! -s "$TEST_EFFECTS" ]] || exit 1
    mv target/retained-proof.json target/release-state/0.1.1.validation.json
    "$TEST_REAL_MAKE" --no-print-directory release-resume VERSION=0.1.1
    [[ ! -s "$TEST_EFFECTS" ]] || exit 1
    expect_failure perl scripts/release/release-data.pl prepare-version 9.9.9 --commit bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
    expect_failure perl scripts/release/release-data.pl verify --commit invalid
    expect_failure perl scripts/release/release-data.pl verify "$TEST_SOURCE" 1900-01-01 0.1.1 --commit bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
    [[ "$(perl scripts/release/release-data.pl version)" == 0.1.1 ]] || exit 1
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
    [[ ! -e target/mock-cache && "$(cat "$TEST_EFFECTS")" == fetch ]] || exit 1
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
    [[ ! -s "$TEST_EFFECTS" ]] || exit 1
    export TEST_PREPARED_FORMAT_FAIL=1
    expect_failure bash target/conditional-substitute.sh "$TEMPORARY/bin/make" fmt-check
    [[ "$(cat "$TEST_EFFECTS")" == prepared-format-check ]] || exit 1
    [[ ! -e target/mock-tag && ! -e target/mock-staged && ! -e target/mock-head ]] || exit 1
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
    [[ "$(cat target/native-staged.txt)" == README.md ]] || exit 1
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
    [[ ! -f "$native/target/commands.log" ]] || exit 1
    "$TEST_REAL_GIT" -C "$native" diff --cached --name-only > target/native-staged.txt
    [[ "$(cat target/native-staged.txt)" == Cargo.toml ]] || exit 1
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
    [[ "$(cat "$native/$unusual")" == 'retained evidence' ]] || exit 1

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
    [[ "$("$TEST_REAL_GIT" -C "$native" rev-parse HEAD)" == "$source" ]] || exit 1
}

run_preparation_cases() {
    run_case preparation test_preparation
    for form in inline comments table alias; do run_case "multi-package-$form" test_multi_package_preparation "$form"; done
    run_case 'multi-package-unicode-雪' test_multi_package_preparation
    for failure in mismatch duplicate missing failed-output failed-renderer empty-renderer; do run_case "lockfile-$failure" test_lockfile_rejection "$failure"; done
}

if [[ "$preparation_only" == true ]]; then
    run_preparation_cases
    echo 'release manifest and lockfile preparation checks passed'
    fixture_complete=true
    exit 0
fi

run_older_release_cases() {
    for mode in explicit explicit-unsorted patch minor gate-failure; do
        run_case "older-release-$mode" test_older_release_recovery "$mode"
    done
    run_case older-release-padded-count test_older_release_padded_count
}

if [[ "$receipts_only" == true ]]; then
    run_case validation-source-identity test_validation_source_identity
    for custody in missing member; do run_case "validation-$custody" test_validation_custody "$custody"; done
    run_older_release_cases
    for reason in missing identity remote; do run_case "selected-proof-$reason" test_selected_proof_rejection "$reason"; done
    run_case completed-evidence-replay test_completed_evidence_replay
    echo 'release receipt selection and exact resume checks passed'
    fixture_complete=true
    exit 0
fi

run_version_reader_cases() {
    for mode in comments render selected duplicate failed-output empty failed-parser; do run_case "version-reader-$mode" test_version_reader "$mode"; done
}

if [[ "$metadata_only" == true ]]; then
    run_case versions test_versions
    run_version_reader_cases
    for outcome in success failure; do
        run_case "dependency-$outcome" test_dependency_bootstrap "$outcome"
    done
    run_case validation-source-identity test_validation_source_identity
    run_case real-index-boundaries test_real_index_boundaries
    echo 'release version planning, metadata real-Git and validation-retention tests passed'
    fixture_complete=true
    exit 0
fi

for outcome in success failure; do
    run_case "dependency-$outcome" test_dependency_bootstrap "$outcome"
    run_case "nested-dependency-$outcome" test_nested_dependency_bootstrap "$outcome"
done
run_case validation-source-identity test_validation_source_identity
run_case versions test_versions
run_version_reader_cases
for invalid in conflict duplicate dated historical-duplicate competing same-date unnumbered; do run_case "notes-$invalid" test_invalid_changelog "$invalid"; done
for mode in empty missing imported bumped failed-output empty-output; do run_case "shared-notes-$mode" test_shared_changelog "$mode"; done
run_preparation_cases
for failure in TEST_FETCH_FAIL TEST_GATE_FAIL TEST_METADATA_FAIL TEST_DIRTY TEST_TAG_EXISTS TEST_GATE_DIRTY TEST_GATE_HEAD TEST_PREPARED_FORMAT_FAIL; do run_case "reject-$failure" test_rejected_preparation "$failure"; done
run_case staging test_staging
for state in no-release tagged; do run_case "publish-$state" test_publish "$state"; done
run_case publish-failure test_publish_failure
for entry in release-verify publish publish-dry-run; do
    for jobs in 1 4; do
        for outcome in success failure; do
            run_case "clippy-$entry-$jobs-$outcome" test_clippy_gate "$entry" "$jobs" "$outcome"
        done
    done
done
for drift in notes tag head; do run_case "completed-$drift" test_invalid_release "$drift"; done
for custody in missing member; do run_case "validation-$custody" test_validation_custody "$custody"; done
run_case early-plan-retry-retention test_early_plan_retry
run_case prepared-explicit-resume test_prepared_explicit_resume
for disposition in unchanged bumped; do run_case "committed-preparation-$disposition" test_committed_preparation_retry "$disposition"; done
for mode in retry offline network invalid lock common prerequisite; do run_case "selected-tool-$mode" test_selected_tool_preflight "$mode"; done
run_older_release_cases
for reason in missing identity remote; do run_case "selected-proof-$reason" test_selected_proof_rejection "$reason"; done
run_case completed-evidence-replay test_completed_evidence_replay
run_case dependency-failure-conditional-status test_dependency_failure_conditional_status
run_case conditional-substitute-rejections test_conditional_substitute_rejections
run_case real-index-boundaries test_real_index_boundaries
printf 'Release adapters: PASS (Make/gate/recovery command substitutes and real private-index boundaries; no new Git commits or live publication).\n'
fixture_complete=true
