#!/usr/bin/env bash
set -euo pipefail

# Consumer integration checks; no Git mutations, network calls or Rust builds.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
mkdir -p "$ROOT/target"
FIXTURE="$(mktemp -d "$ROOT/target/shared-tooling-tests.XXXXXX")"
CONSUMER="$FIXTURE/consumer"

finish() {
    local status=$?
    if [[ "$status" -eq 0 ]]; then
        rm -rf "$FIXTURE"
    else
        printf 'Shared-tooling tests failed; fixture/logs retained: %s\n' "$FIXTURE" >&2
    fi
}
trap finish EXIT

# Run the unchanged upstream cases against the actual vendored runner, while
# retaining its output if qualification fails.
GITHUB_STEP_SUMMARY="$FIXTURE/runner-summary.md" \
    bash "$ROOT/scripts/ci/test-validation-target-runner.sh" >"$FIXTURE/runner.log" 2>&1
cat "$FIXTURE/runner.log"

# Qualify failure retention through the actual adopted scripts. Intercept only
# their first child helper after original fixture inputs have been written.
# Use the selected Bash directly; generating its shebang avoids BSD sed's
# replacement-newline behavior and an env-bash recursion through this wrapper.
retention="$FIXTURE/retention"
mkdir -p "$retention/bin"
{
    printf '#!%s\n' "$BASH"
    cat <<'BASH'
if [[ "${1:-}" == */scripts/ci/"$RETENTION_FAILED_HELPER" ]]; then
    printf 'injected child-helper status: %s\n' "$RETENTION_CHILD_STATUS" >&2
    exit "$RETENTION_CHILD_STATUS"
fi
exec "$RETENTION_REAL_BASH" "$@"
BASH
} > "$retention/bin/bash"
chmod +x "$retention/bin/bash"
for owner in dependency-pins validation-target-runner release-runner; do
    case "$owner" in
        dependency-pins)
            helper=check-dependency-pins.sh; child_status=23; expected_status=1
            prefix=dependency-pins-test; original=consumer/Cargo.toml ;;
        validation-target-runner)
            helper=run-validation-targets.sh; child_status=31; expected_status=31
            prefix=validation-runner-test; original=Makefile ;;
        release-runner)
            helper=run-release.sh; child_status=33; expected_status=33
            prefix=release-runner-test; original=patch/version ;;
    esac
    mkdir "$retention/$owner"
    status=0
    TMPDIR="$retention/$owner" PATH="$retention/bin:$PATH" \
        RETENTION_REAL_BASH="$BASH" RETENTION_FAILED_HELPER="$helper" \
        RETENTION_CHILD_STATUS="$child_status" \
        "$BASH" "$ROOT/scripts/ci/test-$owner.sh" > "$retention/$owner.log" 2>&1 || status=$?
    [[ "$status" == "$expected_status" ]] || {
        cat "$retention/$owner.log" >&2
        echo "unexpected $owner failure status: $status" >&2
        exit 1
    }
    retained=("$retention/$owner/$prefix".*)
    [[ "${#retained[@]}" == 1 && -f "${retained[0]}/$original" ]]
    rg -F "${retained[0]}" "$retention/$owner.log" >/dev/null
    captured="$retention/$owner.log"
    case "$owner" in
        dependency-pins) rg -Fx 'version = "0.1.0"' "${retained[0]}/$original" >/dev/null ;;
        validation-target-runner)
            captured="${retained[0]}/passing-tests.log"
            rg -F 'first-failure-marker' "${retained[0]}/$original" >/dev/null ;;
        release-runner) [[ "$(cat "${retained[0]}/$original")" == 0.1.0 ]] ;;
    esac
    # A missing/broken launcher must not accidentally satisfy a refusal case.
    rg -Fx "injected child-helper status: $child_status" "$captured" >/dev/null
    printf 'Fixture retention: PASS [%s] (status %s; original input retained).\n' "$owner" "$status"
done

reset_consumer() {
    mkdir -p "$CONSUMER"
    cp "$ROOT/.shared-tooling.snapshot" "$CONSUMER/"
    while IFS=$'\t' read -r record _digest _mode path; do
        [[ "$record" == file ]] || continue
        mkdir -p "$CONSUMER/$(dirname "$path")"
        # Remove only this test's destination (which a prior case may symlink).
        rm -f "$CONSUMER/$path"
        cp -p "$ROOT/$path" "$CONSUMER/$path"
    done <"$ROOT/.shared-tooling.snapshot"
}

verify() {
    bash "$ROOT/scripts/ci/verify-shared-tooling-snapshot.sh" --consumer "$CONSUMER"
}

expect_rejection() {
    local status=0
    verify >"$FIXTURE/$1.log" 2>&1 || status=$?
    [[ "$status" -eq 1 ]] || {
        printf 'Expected snapshot rejection in %s; got %s\n' "$1" "$status" >&2
        exit 1
    }
    [[ "$(cat "$CONSUMER/target/evidence")" == 'retained evidence' ]]
}

reset_consumer
mkdir -p "$CONSUMER/target"
printf 'retained evidence\n' >"$CONSUMER/target/evidence"
verify >"$FIXTURE/valid.log" 2>&1

printf '\nchanged bytes\n' >>"$CONSUMER/docs/principles/README.md"
expect_rejection changed-bytes
reset_consumer

chmod +x "$CONSUMER/docs/principles/README.md"
expect_rejection changed-mode
reset_consumer

chmod -x "$CONSUMER/scripts/ci/run-validation-targets.sh"
expect_rejection lost-executable-mode
reset_consumer

rm "$CONSUMER/docs/principles/README.md"
expect_rejection missing-file
reset_consumer

rm "$CONSUMER/docs/principles/README.md"
ln -s "$ROOT/docs/principles/README.md" "$CONSUMER/docs/principles/README.md"
expect_rejection symlink
reset_consumer

sed -n '/^file/{p;q;}' "$ROOT/.shared-tooling.snapshot" >>"$CONSUMER/.shared-tooling.snapshot"
expect_rejection duplicate-path
reset_consumer

printf 'file\t%064d\t-\t../outside\n' 0 >>"$CONSUMER/.shared-tooling.snapshot"
expect_rejection escaping-path
reset_consumer
verify >"$FIXTURE/recovered.log" 2>&1

echo 'Shared-tooling consumer checks: PASS (exact bytes/modes, rejection and evidence retention).'
