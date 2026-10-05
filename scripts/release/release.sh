#!/usr/bin/env bash
set -euo pipefail

# Consumer metadata, validation evidence and read-only inspection. All Git
# staging/commit/tag/push and release recovery belong to the vendored runner.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
cd "$ROOT"
export CARGO_TARGET_DIR="$ROOT/target"
DATA="$ROOT/scripts/release/release-data.pl"
RELEASE_FILES=(Cargo.toml Cargo.lock CHANGELOG.md docs/release.json)
fail() { echo "error: $*" >&2; exit 1; }
version() { perl "$DATA" version; }
ensure_clean() {
    git rev-parse --verify HEAD >/dev/null
    [[ -z "$(git status --porcelain --untracked-files=all)" ]] || fail 'worktree must be clean'
}
allowed_changes() {
    local base="$1" path paths denied=''
    paths="$(mktemp "$CARGO_TARGET_DIR/release-paths.XXXXXX")"
    git diff --name-only -z "$base" -- > "$paths"
    git ls-files --others --exclude-standard -z >> "$paths"
    while IFS= read -r -d '' path; do
        case "$path" in Cargo.toml|Cargo.lock|CHANGELOG.md|docs/release.json) ;;
            *) denied="$path"; break ;; esac
    done < "$paths"
    rm -f "$paths"
    [[ -z "$denied" ]] || fail "unrelated release path: $denied"
}
context() {
    : "${RELEASE_SOURCE:?}" "${RELEASE_PREVIOUS:?}" "${RELEASE_VERSION:?}" "${RELEASE_DATE:?}"
    [[ "$RELEASE_SOURCE" =~ ^[0-9a-f]{40,64}$ && "$RELEASE_DATE" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]] || fail 'invalid source/date'
    [[ "$RELEASE_PREVIOUS" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ && "$RELEASE_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || fail 'invalid release version'
    state_root="$(git rev-parse --git-path release-state)"
    [[ ! -L "$state_root" ]] || fail 'release state is symlinked'
    mkdir -p "$state_root" "$CARGO_TARGET_DIR"
    proof="$state_root/$RELEASE_VERSION.validation.json"
    [[ ! -L "$proof" ]] || fail 'validation evidence is symlinked'
}
validation_check() {
    perl "$DATA" validation-check "$proof" "$RELEASE_SOURCE" "$RELEASE_DATE" \
        "$RELEASE_PREVIOUS" "$RELEASE_VERSION" "$1"
}
preflight() {
    context
    [[ "$(git rev-parse HEAD)" == "$RELEASE_SOURCE" && "$(version)" == "$RELEASE_PREVIOUS" ]] || fail 'original source/version changed'
    allowed_changes "$RELEASE_SOURCE"
    # Pending notes may be dirty; package/lock/previous receipt must still match
    # the selected source so a partial prior bump cannot become a new base.
    git diff --quiet "$RELEASE_SOURCE" -- Cargo.toml Cargo.lock docs/release.json || fail 'original release metadata changed'
    perl "$DATA" changelog-check "$RELEASE_VERSION" "$RELEASE_DATE"
    make --no-print-directory shared-tooling-check
    cargo fetch --locked
}
prepare() {
    context
    [[ "$(git rev-parse HEAD)" == "$RELEASE_SOURCE" && "$(version)" == "$RELEASE_PREVIOUS" ]] || fail 'original source/version changed'
    allowed_changes "$RELEASE_SOURCE"
    validation_check original
    perl "$DATA" changelog-check "$RELEASE_VERSION" "$RELEASE_DATE"
    # Retain exact original metadata for manual recovery from SIGKILL or an
    # interrupted multi-file replacement; ordinary failures restore it on exit.
    RELEASE_BACKUP_DIR="$(mktemp -d "$CARGO_TARGET_DIR/release-backup.XXXXXX")"
    cp Cargo.toml Cargo.lock CHANGELOG.md "$RELEASE_BACKUP_DIR/"
    RELEASE_HAD_RECEIPT=0
    if [[ -f docs/release.json ]]; then cp docs/release.json "$RELEASE_BACKUP_DIR/release.json"; RELEASE_HAD_RECEIPT=1; fi
    printf 'Original release metadata retained: %s\n' "$RELEASE_BACKUP_DIR"
    trap 'cp "$RELEASE_BACKUP_DIR/Cargo.toml" Cargo.toml
          cp "$RELEASE_BACKUP_DIR/Cargo.lock" Cargo.lock
          cp "$RELEASE_BACKUP_DIR/CHANGELOG.md" CHANGELOG.md
          if [[ "$RELEASE_HAD_RECEIPT" == 1 ]]; then cp "$RELEASE_BACKUP_DIR/release.json" docs/release.json; else rm -f docs/release.json; fi' EXIT
    perl "$DATA" finalize "$RELEASE_VERSION" "$RELEASE_DATE"
    perl "$DATA" prepare-version "$RELEASE_VERSION"
    cargo metadata --offline --locked --no-deps --format-version 1 >/dev/null
    make --no-print-directory fmt-check
    perl "$DATA" receipt "$RELEASE_SOURCE" "$RELEASE_DATE"
    verify_prepared
    trap - EXIT
}
verify_prepared() {
    context
    [[ "$(version)" == "$RELEASE_VERSION" ]] || fail 'prepared version differs from plan'
    validation_check prepared
    perl "$DATA" verify
    [[ "$(perl "$DATA" source)" == "$RELEASE_SOURCE" ]] || fail 'prepared receipt source differs from plan'
    allowed_changes "$RELEASE_SOURCE"
    cargo metadata --offline --locked --no-deps --format-version 1 >/dev/null
}
commit_check() {
    verify_prepared
    [[ "$(git rev-parse HEAD)" == "$RELEASE_SOURCE" ]] || fail 'commit must follow exact validated source'
    git diff --quiet || fail 'unstaged changes remain'
    if git diff --cached --quiet; then fail 'release index is empty'; fi
}
committed_check() {
    verify_prepared
    [[ "$(git rev-parse HEAD^)" == "$RELEASE_SOURCE" ]] || fail 'release commit parent differs'
    ensure_clean
}
tag_check() {
    # Standalone tag inspection remains read-only and can inspect an earlier
    # released receipt without requiring the new runner's validation sidecar.
    ensure_clean
    perl "$DATA" verify
    local current
    current="$(version)"
    [[ "$(git cat-file -t "refs/tags/v$current")" == tag ]] || fail 'release tag must be annotated'
    [[ "$(git rev-parse "refs/tags/v$current^{commit}")" == "$(git rev-parse HEAD)" ]] || fail 'tag differs from release commit'
    [[ "$(git rev-parse HEAD^)" == "$(perl "$DATA" source)" ]] || fail 'release parent differs from receipt'
}
lock_release() {
    # Standalone publication shares the runner's exclusion; no stale-lock stealing.
    PUBLICATION_STATE="$(git rev-parse --git-path release-state)"
    [[ ! -L "$PUBLICATION_STATE" ]] || fail 'release state is symlinked'
    mkdir -p "$PUBLICATION_STATE"
    mkdir "$PUBLICATION_STATE/lock" 2>/dev/null || fail 'release lock is occupied; inspect its owner'
    printf '%s\n' "$$" > "$PUBLICATION_STATE/lock/owner"
    trap 'rm -f "$PUBLICATION_STATE/lock/owner"; rmdir "$PUBLICATION_STATE/lock"' EXIT
}
publish() {
    case "${1:-}" in ''|--dry-run) ;; *) fail 'expected publish [--dry-run]' ;; esac
    cargo publish --locked --registry crates-io -p ic-backup ${1:+"$1"}
}
command="${1:-}"
shift || true
case "$command" in
    version) version ;;
    plan)
        next="$(perl "$DATA" next "${1:-patch}")"
        printf 'Current: %s\nTarget:  %s\n' "$(version)" "$next"
        echo 'Maintainer workflow: preflight -> validate -> prepare -> stage -> commit/tag -> atomic push'
        echo 'Interrupted releases use release-resume at the exact saved version; no effects performed.' ;;
    ensure-clean) ensure_clean ;;
    preflight) preflight ;;
    prepare) prepare ;;
    prepared-check) verify_prepared ;;
    commit-check) commit_check ;;
    committed-check) committed_check ;;
    tagged-check|push-check) committed_check; tag_check ;;
    validation-record)
        context
        allowed_changes "$RELEASE_SOURCE"
        [[ "$(git rev-parse HEAD)" == "$RELEASE_SOURCE" ]] || fail 'validated source changed'
        if [[ -e "$proof" ]]; then
            archive="$(mktemp -d "$state_root/$RELEASE_VERSION.validation.XXXXXX")"
            cp -p "$proof" "$archive/validation.json"
            printf 'Earlier validation retained: %s\n' "$archive/validation.json"
        fi
        perl "$DATA" validation-receipt "$proof" "$RELEASE_SOURCE" "$RELEASE_DATE" "$RELEASE_PREVIOUS" "$RELEASE_VERSION" ;;
    files) printf '%s\0' "${RELEASE_FILES[@]}" ;;
    tag-check) tag_check ;;
    resume-check)
        requested="${1:-}"
        [[ "$requested" =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]] || fail 'expected exact saved release version'
        if [[ "$(version)" == "$requested" ]]; then
            state="$(git rev-parse --git-path release-state)"
            [[ ! -L "$state" && ! -L "$state/$requested.validation.json" ]] || fail 'resumed validation is symlinked'
            perl "$DATA" resume-validation-check "$state/$requested.validation.json" "$requested"
        fi ;;
    publish) lock_release; publish "${1:-}" ;;
    *) fail 'expected version, plan, ensure-clean, preflight, prepare, prepared-check, commit-check, committed-check, tagged-check, push-check, validation-record, files, tag-check, resume-check, or publish' ;;
esac
