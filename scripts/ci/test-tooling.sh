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
bash "$ROOT/scripts/ci/test-validation-target-runner.sh" >"$FIXTURE/runner.log" 2>&1
cat "$FIXTURE/runner.log"

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
