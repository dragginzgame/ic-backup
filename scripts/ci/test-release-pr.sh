#!/usr/bin/env bash
# Shared companions: scripts/ci/release-pr.sh scripts/ci/run-release.sh
set -euo pipefail

# Synthetic Git histories and bare destinations under one disposable root;
# GitHub is a strict command substitute. No repository commit or network effect.
unset MAKEFLAGS MFLAGS MAKEOVERRIDES GNUMAKEFLAGS MAKEFILES
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_COMMON_DIR GIT_OBJECT_DIRECTORY GIT_ALTERNATE_OBJECT_DIRECTORIES
ROOT="$0"
[[ "$ROOT" == /* ]] || ROOT="$PWD/$ROOT"
ROOT="$(cd -P "${ROOT%/*}/../.." && printf '%s/.' "$PWD")"
ROOT="${ROOT%/.}"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/release-pr-test.XXXXXX")"
fixture="$(cd "$fixture" && pwd -P)"
# Bash 3.2 can enter EXIT with status zero after nounset; require completion too.
fixture_complete=false
finish() {
    local status=$?
    [[ "$fixture_complete" == true || "$status" != 0 ]] || status=1
    if [[ "$status" == 0 ]]; then rm -rf -- "$fixture"
    else echo "PR release fixtures retained: $fixture" >&2; fi
    exit "$status"
}
trap finish EXIT
mkdir "$fixture/bin" "$fixture/templates"
export PR_TEST_GIT
PR_TEST_GIT="$(command -v git)"
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_TEMPLATE_DIR="$fixture/templates"
export GIT_AUTHOR_NAME=Fixture GIT_COMMITTER_NAME=Fixture
export GIT_AUTHOR_EMAIL=fixture@example.invalid GIT_COMMITTER_EMAIL=fixture@example.invalid
export RELEASE_DELIVERY=pr

cat > "$fixture/bin/git" <<'GIT'
#!/usr/bin/env bash
set -euo pipefail
arguments=()
for argument in "$@"; do
    if [[ "$argument" == https://github.com/example/project.git ]]; then argument="$PR_TEST_REMOTE"; fi
    arguments[${#arguments[@]}]="$argument"
done
if [[ "$1" == push ]]; then
    case "${*: -1}" in
        *:refs/heads/release/*) effect=branch ;;
        refs/tags/*:refs/tags/*) effect=tag ;;
        *) echo 'unexpected push scope' >&2; exit 88 ;;
    esac
    [[ "$2" == --no-follow-tags && "$3" == --atomic && $# == 6 ]] || exit 1
    echo "$effect-push" >> "$PR_TEST_EVENTS"
    "$PR_TEST_GIT" "${arguments[@]}"
    if [[ "${PR_TEST_LOST_PUSH:-}" == "$effect" && ! -f "$PR_TEST_DATA/lost-$effect" ]]; then
        touch "$PR_TEST_DATA/lost-$effect"
        exit 23
    fi
    exit 0
fi
exec "$PR_TEST_GIT" "${arguments[@]}"
GIT
cat > "$fixture/bin/gh" <<'GH'
#!/usr/bin/env bash
set -euo pipefail
[[ "${PR_TEST_API_FAIL:-}" != yes ]] || exit 29
[[ "$1" == api && "$2" == --hostname && "$3" == github.com ]] || exit 1
shift 3
method=GET; input=''; endpoint=''; selection=''; pages=no
while [[ $# -gt 0 ]]; do
    case "$1" in
        --method) method="$2"; shift 2 ;;
        --input) input="$2"; shift 2 ;;
        --jq) selection="$2"; shift 2 ;;
        --paginate) pages=yes; shift ;;
        --include) shift ;;
        -f) case "$2" in state=all|head=example:release/v0.1.1|base=main) ;; *) exit 86 ;; esac; shift 2 ;;
        repos/*) [[ -z "$endpoint" ]] || exit 1; endpoint="$1"; shift ;;
        *) exit 87 ;;
    esac
done
case "$method:$endpoint" in
    GET:repos/example/project)
        [[ "$selection" == .full_name ]] || exit 1; echo example/project ;;
    GET:repos/example/project/pulls)
        [[ "$pages" == yes ]] || exit 1
        case "${PR_TEST_QUERY_RESPONSE:-valid}" in
            empty) exit 0 ;;
            malformed) printf '[\n'; exit 0 ;;
            nonarray) echo '{}'; exit 0 ;;
            partial) echo '[]'; exit 29 ;;
        esac
        if [[ -f "$PR_TEST_DATA/pr.json" ]]; then
            jq '[.]' "$PR_TEST_DATA/pr.json"
            # Multiple matching PRs remain a conflict even on separate pages.
            if [[ "${PR_TEST_DUPLICATE:-}" == yes ]]; then jq '[.]' "$PR_TEST_DATA/pr.json"; fi
            if [[ "${PR_TEST_PAGINATED:-}" == yes ]]; then echo '[]'; fi
        else echo '[]'; fi
        ;;
    GET:repos/example/project/pulls/1) cat "$PR_TEST_DATA/pr.json" ;;
    POST:repos/example/project/pulls)
        [[ ! -e "$PR_TEST_DATA/pr.json" ]] || exit 1
        [[ "${PR_TEST_UNCERTAIN_CREATE:-}" != yes ]] || exit 23
        if [[ "${PR_TEST_REJECT_CREATE:-}" == yes ]]; then
            printf 'HTTP/2.0 403 Forbidden\n\n{}\n'
            exit 1
        fi
        jq -e '.head == "release/v0.1.1" and .base == "main" and .title == "Release 0.1.1"' "$input" >/dev/null
        commit="$("$PR_TEST_GIT" --git-dir "$PR_TEST_REMOTE" rev-parse refs/heads/release/v0.1.1)"
        echo create >> "$PR_TEST_EVENTS"
        jq -n --arg commit "$commit" '{number:1,state:"open",merged:false,merge_commit_sha:null,
          base:{ref:"main",repo:{full_name:"example/project"}},
          head:{ref:"release/v0.1.1",sha:$commit,repo:{full_name:"example/project"}}}' > "$PR_TEST_DATA/pr.json"
        [[ "${PR_TEST_LOST_CREATE:-}" != yes ]] || exit 23
        cat "$PR_TEST_DATA/pr.json"
        ;;
    *) echo "unexpected GitHub request: $method $endpoint" >&2; exit 85 ;;
esac
GH
chmod +x "$fixture/bin/"*
export PATH="$fixture/bin:$PATH"

new_fixture() {
    export PR_TEST_DATA="$fixture/$1" PR_TEST_REMOTE="$fixture/$1/remote.git" PR_TEST_EVENTS="$fixture/$1/events"
    mkdir "$PR_TEST_DATA"
    : > "$PR_TEST_EVENTS"
    "$PR_TEST_GIT" init -q --bare "$PR_TEST_REMOTE"
    "$PR_TEST_GIT" init -q -b main "$PR_TEST_DATA/repository"
    cd "$PR_TEST_DATA/repository"
    printf '0.1.0\n' > VERSION
    cat > Makefile <<'MAKE'
.PHONY: release-version release-preflight release-verify release-prepare-version release-prepared-check release-files release-commit-check release-committed-check release-tagged-check release-push-check release-merged-preflight
release-version:
	@cat VERSION
release-files:
	@printf 'VERSION\0'
release-preflight release-verify release-prepare-version release-prepared-check release-commit-check release-committed-check release-tagged-check release-push-check release-merged-preflight:
	@bash adapter.sh $@
MAKE
    cat > adapter.sh <<'ADAPTER'
#!/usr/bin/env bash
set -euo pipefail
case "$1" in
    release-preflight) [[ "$(cat VERSION)" == "$RELEASE_PREVIOUS" ]] || exit 1 ;;
    release-prepare-version) echo "$RELEASE_VERSION" > VERSION ;;
    release-verify)
        printf 'gate %s %s\n' "$RELEASE_SOURCE" "${RELEASE_COMMIT:-initial}" >> "$PR_TEST_EVENTS"
        if [[ "${PR_TEST_FAIL_GATE:-}" == initial && -z "$RELEASE_COMMIT" ]] ||
           [[ "${PR_TEST_FAIL_GATE:-}" == merged && -n "$RELEASE_COMMIT" ]]; then exit 31; fi
        printf '%s\n' "$RELEASE_SOURCE" > "$(git rev-parse --git-path qualified-source)"
        ;;
    release-prepared-check|release-commit-check)
        [[ "${PR_TEST_STOP_COMMIT:-}" != yes || "$1" != release-commit-check ]] || exit 32
        [[ "$(cat VERSION)" == "$RELEASE_VERSION" ]] || exit 1
        ;;
    release-merged-preflight)
        [[ "$(cat VERSION)" == "$RELEASE_VERSION" && "$(git rev-parse HEAD)" == "$RELEASE_SOURCE" && "$RELEASE_COMMIT" == "$RELEASE_SOURCE" ]] || exit 1
        echo merged-preflight >> "$PR_TEST_EVENTS"
        ;;
    release-committed-check|release-tagged-check|release-push-check)
        [[ "$(cat VERSION)" == "$RELEASE_VERSION" ]] || exit 1
        [[ "$(cat "$(git rev-parse --git-path qualified-source)")" == "$RELEASE_SOURCE" ]] || exit 1
        [[ "$(git show "$RELEASE_COMMIT:VERSION")" == "$RELEASE_VERSION" ]] || exit 1
        if [[ "${PR_TEST_DIRTY_PUSH:-}" == yes && "$1" == release-push-check ]]; then echo changed >> VERSION; fi
        if [[ "${PR_TEST_RETAG_PUSH:-}" == yes && "$1" == release-push-check ]]; then
            git tag -f -a "v$RELEASE_VERSION" "$RELEASE_COMMIT" -m 'Changed annotation after checking'
        fi
        ;;
    *) exit 84 ;;
esac
ADAPTER
    "$PR_TEST_GIT" add VERSION Makefile adapter.sh
    "$PR_TEST_GIT" commit -qm 'Synthetic fixture base'
    source_commit="$(git rev-parse HEAD)"
    "$PR_TEST_GIT" remote add origin https://github.com/example/project.git
    "$PR_TEST_GIT" push -q "$PR_TEST_REMOTE" main
}

run_release() {
    local expected="$1" status=0
    shift
    bash "$ROOT/scripts/ci/run-release.sh" "${@:-patch}" origin main > "$PR_TEST_DATA/run.log" 2>&1 || status=$?
    [[ "$status" == "$expected" ]] || { cat "$PR_TEST_DATA/run.log" >&2; echo "expected $expected, got $status" >&2; exit 1; }
}

merge_pr() {
    local shape="$1" tree base parent_args=()
    prepared="$(git rev-parse refs/heads/release/v0.1.1)"
    tree="$(git rev-parse "$prepared^{tree}")"
    base="$("$PR_TEST_GIT" --git-dir "$PR_TEST_REMOTE" rev-parse main)"
    if [[ "$shape" == rebase ]]; then
        base="$(printf 'Synthetic base advancement\n' | "$PR_TEST_GIT" --git-dir "$PR_TEST_REMOTE" commit-tree "$source_commit^{tree}" -p "$base")"
    fi
    parent_args=(-p "$base")
    if [[ "$shape" == merge ]]; then parent_args[2]=-p; parent_args[3]="$prepared"; fi
    merged="$(printf 'Synthetic %s integration\n' "$shape" | "$PR_TEST_GIT" --git-dir "$PR_TEST_REMOTE" commit-tree "$tree" "${parent_args[@]}")"
    "$PR_TEST_GIT" --git-dir "$PR_TEST_REMOTE" update-ref refs/heads/main "$merged"
    jq --arg merged "$merged" '.merged=true | .state="closed" | .merge_commit_sha=$merged' "$PR_TEST_DATA/pr.json" > "$PR_TEST_DATA/merged.json"
    mv "$PR_TEST_DATA/merged.json" "$PR_TEST_DATA/pr.json"
}

for shape in merge squash rebase; do
    new_fixture "$shape"
    run_release 75
    [[ "$(cat VERSION)" == 0.1.1 && "$(tail -n 1 .git/release-state/0.1.1.plan)" == pr-review ]] || exit 1
    [[ "$("$PR_TEST_GIT" --git-dir "$PR_TEST_REMOTE" rev-parse main)" == "$source_commit" && -z "$(git tag)" ]] || exit 1
    cp "$PR_TEST_EVENTS" "$PR_TEST_DATA/before-wait"
    run_release 75
    cmp "$PR_TEST_EVENTS" "$PR_TEST_DATA/before-wait"
    merge_pr "$shape"
    # GitHub can delete a merged release branch automatically.
    "$PR_TEST_GIT" --git-dir "$PR_TEST_REMOTE" update-ref -d refs/heads/release/v0.1.1
    git switch -q main
    run_release 0
    [[ "$(git rev-parse 'v0.1.1^{commit}')" == "$merged" && "$(git cat-file -t v0.1.1)" == tag ]] || exit 1
    [[ "$("$PR_TEST_GIT" --git-dir "$PR_TEST_REMOTE" rev-parse 'v0.1.1^{commit}')" == "$merged" ]] || exit 1
    [[ "$(grep -c '^gate ' "$PR_TEST_EVENTS")" == 2 && "$(grep -c '^create$' "$PR_TEST_EVENTS")" == 1 ]] || exit 1
    grep -Fx "gate $merged $merged" "$PR_TEST_EVENTS" >/dev/null
    [[ -d .git/release-state/0.1.1.merged && "$(tail -n 1 .git/release-state/0.1.1.plan)" == complete ]] || exit 1
    cp "$PR_TEST_EVENTS" "$PR_TEST_DATA/before-completed-resume"
    run_release 0 resume 0.1.1
    cmp "$PR_TEST_EVENTS" "$PR_TEST_DATA/before-completed-resume"
    "$PR_TEST_GIT" --git-dir "$PR_TEST_REMOTE" update-ref -d refs/tags/v0.1.1
    run_release 1 resume 0.1.1
    cmp "$PR_TEST_EVENTS" "$PR_TEST_DATA/before-completed-resume"
done

new_fixture wrong-commit-branch
PR_TEST_STOP_COMMIT=yes run_release 2
git switch -q main
cp "$PR_TEST_EVENTS" "$PR_TEST_DATA/before-branch-refusal"
run_release 1
cmp "$PR_TEST_EVENTS" "$PR_TEST_DATA/before-branch-refusal"
[[ "$(git rev-parse main)" == "$source_commit" ]] || exit 1
git switch -q release/v0.1.1
run_release 75

new_fixture inherited-git-context
GIT_INDEX_FILE="$PR_TEST_DATA/foreign-index" run_release 1
[[ ! -d .git/release-state && ! -s "$PR_TEST_EVENTS" ]] || exit 1

new_fixture linked-worktree-lock
mkdir -p .git/release-state/lock
git worktree add -q -b topic "$PR_TEST_DATA/linked" HEAD
cd "$PR_TEST_DATA/linked"
run_release 1
[[ ! -s "$PR_TEST_EVENTS" ]] || exit 1

new_fixture failed-gates
PR_TEST_FAIL_GATE=initial run_release 2
[[ ! -f .git/release-state/0.1.1.plan && "$(cat VERSION)" == 0.1.0 ]] || exit 1
run_release 75
merge_pr squash
PR_TEST_FAIL_GATE=merged run_release 2
[[ -z "$(git tag)" && "$(tail -n 1 .git/release-state/0.1.1.plan)" == pr-validate ]] || exit 1
run_release 0
[[ "$(grep -c '^gate ' "$PR_TEST_EVENTS")" == 4 ]] || exit 1

for effect in branch create tag; do
    new_fixture "lost-$effect"
    case "$effect" in
        branch) PR_TEST_LOST_PUSH=branch run_release 23 ;;
        create) PR_TEST_LOST_CREATE=yes run_release 23 ;;
        tag) run_release 75; merge_pr rebase; PR_TEST_LOST_PUSH=tag run_release 23 ;;
    esac
    if [[ "$effect" != tag ]]; then run_release 75; merge_pr squash; fi
    run_release 0
    [[ "$(grep -c '^branch-push$' "$PR_TEST_EVENTS")" == 1 && "$(grep -c '^create$' "$PR_TEST_EVENTS")" == 1 && "$(grep -c '^tag-push$' "$PR_TEST_EVENTS")" == 1 ]] || exit 1
done

new_fixture lost-create-merged
PR_TEST_LOST_CREATE=yes run_release 23
merge_pr squash
"$PR_TEST_GIT" --git-dir "$PR_TEST_REMOTE" update-ref -d refs/heads/release/v0.1.1
run_release 0
[[ "$(grep -c '^branch-push$' "$PR_TEST_EVENTS")" == 1 && "$(grep -c '^create$' "$PR_TEST_EVENTS")" == 1 ]] || exit 1

new_fixture rejected-creation
PR_TEST_REJECT_CREATE=yes run_release 1
[[ "$(jq -r .attempted .git/release-state/0.1.1.plan.pr.json)" == false ]] || exit 1
run_release 75
[[ "$(grep -c '^branch-push$' "$PR_TEST_EVENTS")" == 1 && "$(grep -c '^create$' "$PR_TEST_EVENTS")" == 1 ]] || exit 1

new_fixture uncertain-creation
PR_TEST_UNCERTAIN_CREATE=yes run_release 23
cp "$PR_TEST_EVENTS" "$PR_TEST_DATA/before-uncertain"
run_release 1
cmp "$PR_TEST_EVENTS" "$PR_TEST_DATA/before-uncertain"
[[ "$(jq -r .attempted .git/release-state/0.1.1.plan.pr.json)" == true ]] || exit 1

new_fixture failed-query
# Stop before the first branch push or PR creation, even if an earlier page was
# received successfully. Retrying the retained preparation must not repeat it.
PR_TEST_QUERY_RESPONSE=partial run_release 29
[[ "$(tail -n 1 .git/release-state/0.1.1.plan)" == pr-publish ]] || exit 1
[[ "$(grep -c '^gate ' "$PR_TEST_EVENTS")" == 1 && ! -f "$PR_TEST_DATA/pr.json" ]] || exit 1
[[ "$(grep -c -- '-push$' "$PR_TEST_EVENTS" || true)" == 0 ]] || exit 1
cp "$PR_TEST_EVENTS" "$PR_TEST_DATA/before-query"
PR_TEST_API_FAIL=yes run_release 29
cmp "$PR_TEST_EVENTS" "$PR_TEST_DATA/before-query"
for response in empty malformed nonarray partial; do
    expected=1
    [[ "$response" != partial ]] || expected=29
    PR_TEST_QUERY_RESPONSE="$response" run_release "$expected"
    cmp "$PR_TEST_EVENTS" "$PR_TEST_DATA/before-query"
done
PR_TEST_PAGINATED=yes run_release 75
[[ "$(grep -c '^gate ' "$PR_TEST_EVENTS")" == 1 && "$(grep -c '^branch-push$' "$PR_TEST_EVENTS")" == 1 && "$(grep -c '^create$' "$PR_TEST_EVENTS")" == 1 ]] || exit 1
cp "$PR_TEST_EVENTS" "$PR_TEST_DATA/before-paginated-query"
PR_TEST_PAGINATED=yes run_release 75
cmp "$PR_TEST_EVENTS" "$PR_TEST_DATA/before-paginated-query"

new_fixture changed-push-input
run_release 75
merge_pr squash
PR_TEST_DIRTY_PUSH=yes run_release 1
[[ "$(grep -c '^tag-push$' "$PR_TEST_EVENTS" || true)" == 0 ]] || exit 1
[[ -f .git/release-state/0.1.1.merged/VERSION ]] || exit 1

new_fixture changed-push-tag-object
run_release 75
merge_pr squash
PR_TEST_RETAG_PUSH=yes run_release 1
[[ "$(grep -c '^tag-push$' "$PR_TEST_EVENTS" || true)" == 0 ]] || exit 1
[[ "$(git rev-parse 'v0.1.1^{commit}')" == "$merged" && "$(git cat-file -t v0.1.1)" == tag ]] || exit 1
[[ "$(tail -n 1 .git/release-state/0.1.1.plan)" == pr-push ]] || exit 1

for conflict in closed duplicate head base destination index delivery tree lock local-tag remote-tag; do
    new_fixture "conflict-$conflict"
    run_release 75
    case "$conflict" in
        closed) jq '.state="closed"' "$PR_TEST_DATA/pr.json" > "$PR_TEST_DATA/change.json"; mv "$PR_TEST_DATA/change.json" "$PR_TEST_DATA/pr.json" ;;
        duplicate) export PR_TEST_DUPLICATE=yes ;;
        head) jq '.head.sha="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"' "$PR_TEST_DATA/pr.json" > "$PR_TEST_DATA/change.json"; mv "$PR_TEST_DATA/change.json" "$PR_TEST_DATA/pr.json" ;;
        base) jq '.base.ref="other"' "$PR_TEST_DATA/pr.json" > "$PR_TEST_DATA/change.json"; mv "$PR_TEST_DATA/change.json" "$PR_TEST_DATA/pr.json" ;;
        destination) git remote set-url origin https://github.com/example/other.git ;;
        index) printf 'unrelated\n' > extra; git add extra; rm extra ;;
        delivery) export RELEASE_DELIVERY=direct ;;
        lock) mkdir .git/release-state/lock ;;
        local-tag|remote-tag)
            merge_pr squash
            git tag -a v0.1.1 "$source_commit" -m 'Conflicting synthetic tag'
            if [[ "$conflict" == remote-tag ]]; then
                "$PR_TEST_GIT" push -q "$PR_TEST_REMOTE" refs/tags/v0.1.1
                git tag -d v0.1.1 >/dev/null
            fi
            ;;
        tree)
            merge_pr squash
            other="$(printf 'Changed merge payload\n' | "$PR_TEST_GIT" --git-dir "$PR_TEST_REMOTE" commit-tree "$source_commit^{tree}" -p "$merged")"
            "$PR_TEST_GIT" --git-dir "$PR_TEST_REMOTE" update-ref refs/heads/main "$other"
            jq --arg other "$other" '.merge_commit_sha=$other' "$PR_TEST_DATA/pr.json" > "$PR_TEST_DATA/change.json"
            mv "$PR_TEST_DATA/change.json" "$PR_TEST_DATA/pr.json"
            ;;
    esac
    cp "$PR_TEST_EVENTS" "$PR_TEST_DATA/before-conflict"
    run_release 1
    case "$conflict" in
        local-tag|remote-tag) [[ "$(grep -c '^tag-push$' "$PR_TEST_EVENTS" || true)" == 0 ]] || exit 1 ;;
        *) [[ -z "$(git tag)" ]] || exit 1; cmp "$PR_TEST_EVENTS" "$PR_TEST_DATA/before-conflict" ;;
    esac
    unset PR_TEST_DUPLICATE
    export RELEASE_DELIVERY=pr
done
echo 'PR release review, merge/squash/rebase validation, recovery and conflict checks passed'
fixture_complete=true
