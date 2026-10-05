#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
export CARGO_TARGET_DIR="$ROOT/target"
DATA="$ROOT/scripts/release/release-data.pl"
RELEASE_FILES=(Cargo.toml Cargo.lock CHANGELOG.md docs/release.json)

fail() {
    echo "error: $*" >&2
    exit 1
}

version() { perl "$DATA" version; }

ensure_clean() {
    git rev-parse --verify HEAD >/dev/null 2>&1 ||
        fail "repository has no commit; the maintainer must commit the source first"
    [[ -z "$(git status --porcelain --untracked-files=all)" ]] ||
        fail "worktree must be clean, including untracked files"
}

tag_absent() {
    if git rev-parse --verify --quiet "refs/tags/v$1" >/dev/null; then
        fail "tag v$1 already exists"
    fi
}

# Serialize release mutations in this repository. Normal build ownership must
# still be observed by the maintainer before starting a release.
lock_release() {
    mkdir -p target
    exec 9>target/release.lock
    flock -n 9 || fail "another release command is active"
}

allowed_changes() {
    local base="$1" path
    while IFS= read -r path; do
        case "$path" in
            Cargo.toml | Cargo.lock | CHANGELOG.md | docs/release.json) ;;
            *) fail "non-release path changed since validation: $path" ;;
        esac
    done < <(git diff --name-only "$base" --)
    while IFS= read -r path; do
        [[ "$path" == docs/release.json ]] ||
            fail "untracked file must be reviewed before release: $path"
    done < <(git ls-files --others --exclude-standard)
}

verify_prepared() {
    perl "$DATA" verify
    local source head
    source="$(perl "$DATA" source)"
    head="$(git rev-parse HEAD)"
    git merge-base --is-ancestor "$source" "$head" ||
        fail "validated source is not an ancestor of HEAD"
    allowed_changes "$source"
    cargo metadata --offline --locked --no-deps --format-version 1 >/dev/null
}

bump() {
    local requested="${1:-}" next previous source release_date
    RELEASE_HAD_RECEIPT=0
    next="$(perl "$DATA" next "$requested")"
    previous="$(version)"
    ensure_clean
    tag_absent "$next"
    # Reusing the initial unpublished scaffold version is supported only before
    # any version tag or completed local release receipt exists.
    if [[ "$next" == "$previous" ]]; then
        [[ ! -f docs/release.json && -z "$(git tag --list 'v*')" ]] ||
            fail "an existing release requires a strictly greater version"
    fi
    release_date="$(date -u +%F)"
    perl "$DATA" changelog-check "$next" "$release_date"
    echo "Preparing $previous -> $next; the current draft will be labeled $next."
    source="$(git rev-parse HEAD)"
    make --no-print-directory release-verify
    ensure_clean
    [[ "$(git rev-parse HEAD)" == "$source" ]] ||
        fail "HEAD changed during validation"
    tag_absent "$next"

    RELEASE_BACKUP_DIR="$(mktemp -d "$CARGO_TARGET_DIR/release-backup.XXXXXX")"
    cp Cargo.toml Cargo.lock CHANGELOG.md "$RELEASE_BACKUP_DIR/"
    if [[ -f docs/release.json ]]; then
        cp docs/release.json "$RELEASE_BACKUP_DIR/release.json"
        RELEASE_HAD_RECEIPT=1
    fi
    # Failed version mutation restores the exact clean input. No staging or
    # external effect occurs in this transaction.
    trap 'cp "$RELEASE_BACKUP_DIR/Cargo.toml" Cargo.toml
          cp "$RELEASE_BACKUP_DIR/Cargo.lock" Cargo.lock
          cp "$RELEASE_BACKUP_DIR/CHANGELOG.md" CHANGELOG.md
          if [[ "$RELEASE_HAD_RECEIPT" == 1 ]]; then
              cp "$RELEASE_BACKUP_DIR/release.json" docs/release.json
          else
              rm -f docs/release.json
          fi
          rm -rf "$RELEASE_BACKUP_DIR"' EXIT
    # Resolve the draft while the manifest still identifies the original release;
    # this distinguishes a provisional future label from imported undated history.
    # The same rollback restores the exact draft if a later mutation fails.
    perl "$DATA" finalize "$next" "$release_date"
    perl "$DATA" set-version "$next"
    cargo update --offline -p ic-backup
    cargo metadata --offline --locked --no-deps --format-version 1 >/dev/null
    perl "$DATA" receipt "$source" "$release_date"
    perl "$DATA" verify
    trap - EXIT
    rm -rf "$RELEASE_BACKUP_DIR"
    echo "Prepared $previous -> $next; review the release-file diff."
}

stage() {
    verify_prepared
    [[ "$(git rev-parse HEAD)" == "$(perl "$DATA" source)" ]] ||
        fail "stage must start from the exact validated source"
    git add -- "${RELEASE_FILES[@]}"
}

commit_release() {
    verify_prepared
    local current source
    current="$(version)"
    source="$(perl "$DATA" source)"
    [[ "$(git rev-parse HEAD)" == "$source" ]] ||
        fail "release commit must directly follow the validated source"
    git diff --quiet || fail "unstaged changes remain"
    git diff --cached --quiet && fail "no staged release changes"
    tag_absent "$current"
    git commit -m "Release $current"
    git tag -a "v$current" -m "Release $current"
}

tag_check() {
    ensure_clean
    verify_prepared
    local current head tagged source parent
    current="$(version)"
    head="$(git rev-parse HEAD)"
    tagged="$(git rev-parse --verify "refs/tags/v$current^{commit}")"
    [[ "$tagged" == "$head" ]] || fail "current version tag is not at HEAD"
    [[ "$(git cat-file -t "refs/tags/v$current")" == tag ]] ||
        fail "release tag must be annotated"
    source="$(perl "$DATA" source)"
    parent="$(git rev-parse HEAD^)"
    [[ "$parent" == "$source" ]] ||
        fail "release commit does not directly follow the validated source"
}

remote_preflight() {
    [[ "$(git symbolic-ref --quiet --short HEAD)" == main ]] ||
        fail "release push requires branch main"
    git remote get-url origin >/dev/null ||
        fail "configure the intended origin before a one-shot release"
}

push_release() {
    remote_preflight
    tag_check
    # Push exactly this branch and release tag; never unrelated local tags.
    git push --atomic origin HEAD:refs/heads/main "refs/tags/v$(version)"
}

publish() {
    case "${1:-}" in "" | --dry-run) ;; *) fail "expected publish [--dry-run]" ;; esac
    # Publish the current package through Cargo. Release receipts and tags own
    # the repository release transaction, not registry publication admission.
    cargo publish --locked --registry crates-io -p ic-backup ${1:+"$1"}
}

command="${1:-}"
shift || true
case "$command" in
    version) version ;;
    plan)
        next="$(perl "$DATA" next "${1:-patch}")"
        echo "Current: $(version)"
        echo "Target:  $next"
        echo "Bump: clean source -> release-verify -> Cargo/lockfile/changelog/receipt"
        echo "One-shot: bump -> stage -> commit -> annotated tag -> atomic push"
        echo "Registry publication is a separate command; no effects performed."
        ;;
    ensure-clean) ensure_clean ;;
    bump) lock_release; bump "${1:-}" ;;
    stage) lock_release; stage ;;
    commit) lock_release; commit_release ;;
    tag-check) tag_check ;;
    push) lock_release; push_release ;;
    publish) lock_release; publish "${1:-}" ;;
    release)
        lock_release
        remote_preflight
        bump "${1:-}"
        stage
        commit_release
        push_release
        ;;
    *) fail "expected version, plan, ensure-clean, bump, stage, commit, tag-check, push, publish, or release" ;;
esac
