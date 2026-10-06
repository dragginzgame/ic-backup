#!/usr/bin/env bash
set -euo pipefail

# Fixtures below supply their own Make selections and logger identities.
unset MAKEFLAGS MFLAGS MAKEOVERRIDES
unset VALIDATION_REPOSITORY_ROOT VALIDATION_RUNNER_SNAPSHOT_PATH

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
FIXTURE="$(mktemp -d "${TMPDIR:-/tmp}/validation-runner-test.XXXXXX")"
trap 'rm -rf "$FIXTURE"' EXIT

mkdir -p "$FIXTURE/scripts/ci" "$FIXTURE/failure-logs"
cp "$ROOT/scripts/ci/run-validation-targets.sh" "$FIXTURE/scripts/ci/"
printf '%s\n' \
    '.PHONY: pass mutate-runner fail-one fail-two fail-after-caught-panic' \
    'pass:' \
    $'\t@echo pass-marker' \
    'mutate-runner:' \
    $'\t@printf "for broken do\\n" > scripts/ci/run-validation-targets.sh' \
    $'\t@echo mutation-marker' \
    'fail-one:' \
    $'\t@echo "error: first-live-error-marker"' \
    $'\t@echo first-failure-marker' \
    $'\t@exit 7' \
    'fail-two:' \
    $'\t@echo "error: second-live-error-marker"' \
    $'\t@echo second-failure-marker' \
    $'\t@exit 9' \
    'fail-after-caught-panic:' \
    $'\t@echo "thread '\''caught-test'\'' panicked at fake.rs:1:1:"' \
    $'\t@echo "caught panic marker"' \
    $'\t@echo "test caught-test ... ok"' \
    $'\t@echo ordinary-context-one' \
    $'\t@echo ordinary-context-two' \
    $'\t@echo ordinary-context-three' \
    $'\t@echo ordinary-context-four' \
    $'\t@echo ordinary-context-five' \
    $'\t@echo "test actual-test ... FAILED"' \
    $'\t@exit 11' >"$FIXTURE/Makefile"

fail_fast_status=0
VALIDATION_FAILURE_LOG_DIR="$FIXTURE/failure-logs" \
    VALIDATION_REPOSITORY_ROOT="$FIXTURE" \
    VALIDATION_RUNNER_DEPTH=0 \
    VALIDATION_RUNNER_SNAPSHOT_PATH='' \
    bash "$FIXTURE/scripts/ci/run-validation-targets.sh" \
    --fail-fast fail-one fail-two \
    >"$FIXTURE/fail-fast.log" 2>&1 || fail_fast_status=$?

[[ "$fail_fast_status" -eq 1 ]] || {
    echo "validation target runner test failed: fail-fast status was $fail_fast_status" >&2
    exit 1
}
rg -F 'first-failure-marker' "$FIXTURE/fail-fast.log" >/dev/null
if rg -F 'second-failure-marker' "$FIXTURE/fail-fast.log" >/dev/null; then
    echo "validation target runner test failed: fail-fast ran the second target" >&2
    exit 1
fi

status=0
VALIDATION_FAILURE_LOG_DIR="$FIXTURE/failure-logs" \
    VALIDATION_REPOSITORY_ROOT="$FIXTURE" \
    VALIDATION_RUNNER_DEPTH=0 \
    VALIDATION_RUNNER_SNAPSHOT_PATH='' \
    bash "$FIXTURE/scripts/ci/run-validation-targets.sh" \
    pass mutate-runner fail-one fail-two fail-after-caught-panic \
    >"$FIXTURE/output.log" 2>&1 || status=$?

[[ "$status" -eq 1 ]] || {
    echo "validation target runner test failed: expected status 1, got $status" >&2
    exit 1
}
for expected in \
    'pass-marker' \
    'mutation-marker' \
    'first-failure-marker' \
    'second-failure-marker' \
    "thread 'caught-test' panicked at fake.rs:1:1:" \
    'test caught-test ... ok' \
    '[ERR:fail-one] error: first-live-error-marker' \
    '[ERR:fail-two] error: second-live-error-marker' \
    '[ERR:fail-after-caught-panic] test actual-test ... FAILED' \
    '[ERR:fail-one] Target failed' \
    '[ERR:fail-one] first-failure-marker' \
    '[ERR:fail-two] Target failed' \
    '[ERR:fail-two] second-failure-marker' \
    '[ERR:fail-after-caught-panic] Target failed' \
    '[ERR:summary] Latest highlighted errors:' \
    '[ERR:summary] VALIDATION FAILED: fail-one fail-two fail-after-caught-panic' \
    'PASS' \
    'FAIL' \
    'VALIDATION FAILED: fail-one fail-two fail-after-caught-panic'; do
    rg -F "$expected" "$FIXTURE/output.log" >/dev/null || {
        echo "validation target runner test failed: missing output: $expected" >&2
        exit 1
    }
done
if rg -F "[ERR:fail-after-caught-panic] thread 'caught-test' panicked at" \
    "$FIXTURE/output.log" >/dev/null; then
    echo "validation target runner test failed: caught panic was highlighted" >&2
    exit 1
fi
for expected in \
    '[ERR:fail-one] error: first-live-error-marker' \
    '[ERR:fail-two] error: second-live-error-marker'; do
    [[ "$(rg -F -c "$expected" "$FIXTURE/output.log")" -ge 3 ]] || {
        echo "validation target runner test failed: live error was not highlighted before both summaries" >&2
        exit 1
    }
done
[[ -s "$FIXTURE/failure-logs/latest.log" ]] || {
    echo "validation target runner test failed: latest failure log was not retained" >&2
    exit 1
}
if rg -F '[ERR:' "$FIXTURE/failure-logs/latest.log" >/dev/null; then
    echo "validation target runner test failed: raw failure log was decorated" >&2
    exit 1
fi
[[ -s "$FIXTURE/failure-logs/latest-errors.log" ]] || {
    echo "validation target runner test failed: highlighted failure log was not retained" >&2
    exit 1
}
for expected in \
    '[ERR:fail-one] first-failure-marker' \
    '[ERR:fail-two] second-failure-marker' \
    '[ERR:fail-after-caught-panic] test actual-test ... FAILED'; do
    rg -F "$expected" "$FIXTURE/failure-logs/latest-errors.log" >/dev/null || {
        echo "validation target runner test failed: highlighted log omits: $expected" >&2
        exit 1
    }
done
if rg -F "[ERR:fail-after-caught-panic] thread 'caught-test' panicked at" \
    "$FIXTURE/failure-logs/latest-errors.log" >/dev/null; then
    echo "validation target runner test failed: highlighted log includes caught panic" >&2
    exit 1
fi

# Retention failures must keep the original raw log in the temporary directory.
cp "$ROOT/scripts/ci/run-validation-targets.sh" "$FIXTURE/scripts/ci/"
mkdir -p "$FIXTURE/bin"
REAL_CP="$(command -v cp)"
export REAL_CP
cat > "$FIXTURE/bin/cp" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$2" == "$VALIDATION_FAILURE_LOG_DIR/"* ]]; then exit 1; fi
exec "$REAL_CP" "$@"
SCRIPT
chmod +x "$FIXTURE/bin/cp"
for failure in mkdir copy; do
    fallback_tmp="$FIXTURE/fallback-$failure"
    failure_root="$FIXTURE/blocked-$failure"
    mkdir -p "$fallback_tmp"
    if [[ "$failure" == mkdir ]]; then
        printf 'not a directory\n' > "$failure_root"
    else
        mkdir -p "$failure_root"
    fi
    status=0
    TMPDIR="$fallback_tmp" PATH="$FIXTURE/bin:$PATH" \
        VALIDATION_FAILURE_LOG_DIR="$failure_root" \
        VALIDATION_REPOSITORY_ROOT="$FIXTURE" \
        VALIDATION_RUNNER_DEPTH=0 VALIDATION_RUNNER_SNAPSHOT_PATH='' \
        bash "$FIXTURE/scripts/ci/run-validation-targets.sh" fail-one \
        > "$FIXTURE/fallback-$failure.log" 2>&1 || status=$?
    [[ "$status" == 1 ]]
    fallback_logs=("$fallback_tmp"/validation.*/0.log)
    [[ "${#fallback_logs[@]}" == 1 && -f "${fallback_logs[0]}" ]]
    rg -F first-failure-marker "${fallback_logs[0]}" >/dev/null
    rg -F "${fallback_logs[0]}" "$FIXTURE/fallback-$failure.log" >/dev/null
done

# Exercise real Make overrides and logger dispatch across checkout boundaries.
parent="$FIXTURE/parent"
mkdir -p "$parent/scripts/ci" "$parent/child/scripts/ci"
cp "$ROOT/scripts/ci/run-validation-targets.sh" "$parent/scripts/ci/"
cp "$ROOT/scripts/ci/run-validation-targets.sh" "$parent/child/scripts/ci/"
cat > "$parent/Makefile" <<'MAKE'
.PHONY: validate adoption same-checkout selection child-gate ci
validate:
	+bash scripts/ci/run-validation-targets.sh adoption same-checkout
adoption:
	@bash "$(METADATA_FIXTURE)"
	+bash child/scripts/ci/run-validation-targets.sh child-gate
same-checkout:
	+bash scripts/ci/run-validation-targets.sh selection
selection:
	@test "$(RELEASE_VERSION)" = 9.8.7
	@test "$(RELEASE_COMMIT)" = parent-selected-commit
	@test "$$VALIDATION_RUNNER_DEPTH" = 2
	@echo nested-selection-marker
child-gate ci:
	@echo incorrect-parent-gate >> incorrect-route
	@exit 9
MAKE
cat > "$parent/child/Makefile" <<'MAKE'
child-gate:
	@echo child-gate-marker
MAKE
if ! VALIDATION_RUNNER_DEPTH=0 VALIDATION_FAILURE_LOG_DIR="$parent/failure-logs" \
    make --no-print-directory -C "$parent" validate \
    RELEASE_VERSION=9.8.7 RELEASE_COMMIT=parent-selected-commit \
    METADATA_FIXTURE="$ROOT/scripts/ci/test-release-metadata.sh" \
    > "$FIXTURE/nested-context.log" 2>&1; then
    cat "$FIXTURE/nested-context.log" >&2
    echo 'validation target runner test failed: nested release context' >&2
    exit 1
fi
[[ ! -e "$parent/incorrect-route" ]] || {
    echo 'validation target runner test failed: executed the parent gate' >&2
    exit 1
}
for marker in child-gate-marker nested-selection-marker \
    'release metadata real-Git and validation-retention tests passed'; do
    rg -F "$marker" "$FIXTURE/nested-context.log" >/dev/null
done

echo "validation target runner test passed"
