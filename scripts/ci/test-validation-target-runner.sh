#!/usr/bin/env bash
set -euo pipefail

# Fixtures below supply their own Make selections and logger identities.
unset MAKEFLAGS MFLAGS MAKEOVERRIDES GNUMAKEFLAGS MAKEFILES
unset VALIDATION_REPOSITORY_ROOT VALIDATION_RUNNER_SNAPSHOT_PATH

ROOT="$0"
[[ "$ROOT" == /* ]] || ROOT="$PWD/$ROOT"
ROOT="$(cd -P "${ROOT%/*}/../.." && printf '%s/.' "$PWD")"
ROOT="${ROOT%/.}"
FIXTURE="$(mktemp -d "${TMPDIR:-/tmp}/validation-runner-test.XXXXXX")"
trap 'if [[ $? == 0 ]]; then rm -rf "$FIXTURE"; else printf "Failed validation-target-runner fixture retained: %s\n" "$FIXTURE" >&2; fi' EXIT

mkdir -p "$FIXTURE/scripts/ci" "$FIXTURE/failure-logs"
cp "$ROOT/scripts/ci/run-validation-targets.sh" "$FIXTURE/scripts/ci/"
cp "$ROOT/scripts/ci/check-make-execution.sh" "$FIXTURE/scripts/ci/"
printf '%s\n' \
    '.PHONY: pass passing-tests mutate-runner fail-one fail-two fail-after-caught-panic fail-with-test-context' \
    'pass:' \
    $'\t@echo pass-marker' \
    'passing-tests:' \
    $'\t@echo "test error::tests::passing ... ok"' \
    $'\t@echo "test error::tests::ignored ... ignored"' \
    $'\t@echo "test result: ok. 1 passed; 0 failed; 1 ignored"' \
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
    $'\t@exit 11' \
    'fail-with-test-context:' \
    $'\t@echo "test error::tests::context ... ok"' \
    $'\t@echo "error[E0308]: typed-error-marker"' \
    $'\t@echo "error:no-space-marker"' \
    $'\t@echo "error:"' \
    $'\t@echo "test error::tests::actual ... FAILED"' \
    $'\t@exit 13' >"$FIXTURE/Makefile"

# GNU Make owns option parsing, including compact flags and long aliases.
# Matching variable values and legitimate parallel controls must remain valid.
for variable in MAKEFLAGS MFLAGS GNUMAKEFLAGS; do
    for flags in i n q t v ksin --ignore-errors --dry-run --just-print --recon --question --touch --version; do
        if env "$variable=$flags" VALIDATION_REPOSITORY_ROOT="$FIXTURE" \
            bash "$FIXTURE/scripts/ci/run-validation-targets.sh" fail-one \
            > "$FIXTURE/$variable-$flags.log" 2>&1; then
            echo 'validation accepted an incompatible Make execution mode' >&2
            exit 1
        fi
        if [[ "$variable" == GNUMAKEFLAGS ]] && ! rg -F 'requires recipe execution and failure propagation' "$FIXTURE/$variable-$flags.log" >/dev/null; then
            # GNU Make before 4.0 ignores GNUMAKEFLAGS: the real failing target
            # must still execute and fail, rather than claiming skipped success.
            rg -F 'first-failure-marker' "$FIXTURE/$variable-$flags.log" >/dev/null
            rg -F 'VALIDATION FAILED' "$FIXTURE/$variable-$flags.log" >/dev/null
        else
            rg -F 'requires recipe execution and failure propagation' "$FIXTURE/$variable-$flags.log" >/dev/null
        fi
        if rg -F 'VALIDATION PASSED' "$FIXTURE/$variable-$flags.log" >/dev/null; then exit 1; fi
    done
done

# Namespaced successful and ignored tests must remain ordinary live output.
# Even a late invalid goal must refuse the whole request before dispatch.
for invalid in '' --dry-run --version --ignore-errors -n MAKEFLAGS=i 'VALUE=1' $'bad\ttarget' $'bad\ntarget'; do
    status=0
    VALIDATION_REPOSITORY_ROOT="$FIXTURE" VALIDATION_LOG_DIR="$FIXTURE/refused-logs" \
        bash "$FIXTURE/scripts/ci/run-validation-targets.sh" pass "$invalid" \
        > "$FIXTURE/refused-goal.log" 2>&1 || status=$?
    [[ "$status" == 2 && ! -e "$FIXTURE/refused-logs" ]]
    if rg 'pass-marker|VALIDATION PASSED' "$FIXTURE/refused-goal.log" >/dev/null; then
        echo 'invalid goal list dispatched a target or reported success' >&2; exit 1
    fi
done

VALIDATION_FAILURE_LOG_DIR="$FIXTURE/passing-logs" \
    VALIDATION_REPOSITORY_ROOT="$FIXTURE" \
    VALIDATION_RUNNER_DEPTH=0 VALIDATION_RUNNER_SNAPSHOT_PATH='' \
    bash "$FIXTURE/scripts/ci/run-validation-targets.sh" passing-tests \
    >"$FIXTURE/passing-tests.log" 2>&1
rg -Fx 'test error::tests::passing ... ok' "$FIXTURE/passing-tests.log" >/dev/null
rg -Fx 'test error::tests::ignored ... ignored' "$FIXTURE/passing-tests.log" >/dev/null
rg -F 'VALIDATION PASSED:' "$FIXTURE/passing-tests.log" >/dev/null
[[ ! -e "$FIXTURE/passing-logs" ]]

fail_fast_status=0
VALIDATION_FAILURE_LOG_DIR="$FIXTURE/failure-logs" \
    VALIDATION_REPOSITORY_ROOT="$FIXTURE" \
    VALIDATION_RUNNER_DEPTH=0 \
    VALIDATION_RUNNER_SNAPSHOT_PATH='' \
    bash "$FIXTURE/scripts/ci/run-validation-targets.sh" \
    --fail-fast fail-one fail-two \
    >"$FIXTURE/fail-fast.log" 2>&1 || fail_fast_status=$?

[[ "$fail_fast_status" -eq 2 ]] || {
    echo "validation target runner test failed: fail-fast status was $fail_fast_status" >&2
    exit 1
}
rg -F 'first-failure-marker' "$FIXTURE/fail-fast.log" >/dev/null
if rg -F 'second-failure-marker' "$FIXTURE/fail-fast.log" >/dev/null; then
    echo "validation target runner test failed: fail-fast ran the second target" >&2
    exit 1
fi
fast_combined="$(sed -n 's/^Combined failure log retained at: //p' "$FIXTURE/fail-fast.log")"
fast_raw=("$FIXTURE/failure-logs/"*-0-fail-one.log)
cmp "${fast_raw[${#fast_raw[@]}-1]}" "$fast_combined"

status=0
VALIDATION_FAILURE_LOG_DIR="$FIXTURE/failure-logs" \
    VALIDATION_REPOSITORY_ROOT="$FIXTURE" \
    VALIDATION_RUNNER_DEPTH=0 \
    VALIDATION_RUNNER_SNAPSHOT_PATH='' \
    bash "$FIXTURE/scripts/ci/run-validation-targets.sh" \
    pass mutate-runner fail-one fail-two fail-after-caught-panic fail-with-test-context \
    >"$FIXTURE/output.log" 2>&1 || status=$?

[[ "$status" -eq 2 ]] || {
    echo "validation target runner test failed: expected Make failure status 2, got $status" >&2
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
    '[fail-one] first-failure-marker' \
    '[ERR:fail-two] Target failed' \
    '[fail-two] second-failure-marker' \
    '[ERR:fail-after-caught-panic] Target failed' \
    '[ERR:summary] Latest highlighted errors:' \
    '[ERR:fail-with-test-context] error[E0308]: typed-error-marker' \
    '[ERR:fail-with-test-context] error:no-space-marker' \
    '[ERR:fail-with-test-context] error:' \
    '[ERR:fail-with-test-context] test error::tests::actual ... FAILED' \
    '[fail-with-test-context] test error::tests::context ... ok' \
    '[ERR:summary] VALIDATION FAILED: fail-one fail-two fail-after-caught-panic fail-with-test-context' \
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
combined="$(sed -n 's/^Combined failure log retained at: //p' "$FIXTURE/output.log")"
cat "$FIXTURE/failure-logs/"*-2-fail-one.log "$FIXTURE/failure-logs/"*-3-fail-two.log \
    "$FIXTURE/failure-logs/"*-4-fail-after-caught-panic.log "$FIXTURE/failure-logs/"*-5-fail-with-test-context.log \
    > "$FIXTURE/combined.expected"
cmp "$FIXTURE/combined.expected" "$combined"
cmp "$combined" "$FIXTURE/failure-logs/latest-combined.log"
[[ -f "$fast_combined" ]]
for expected in \
    '[fail-one] first-failure-marker' \
    '[fail-two] second-failure-marker' \
    '[fail-with-test-context] test error::tests::context ... ok' \
    '[ERR:fail-with-test-context] error[E0308]: typed-error-marker' \
    '[ERR:fail-with-test-context] test error::tests::actual ... FAILED' \
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
    [[ "$status" == 2 ]]
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
cp "$ROOT/scripts/ci/check-make-execution.sh" "$parent/scripts/ci/"
cp "$ROOT/scripts/ci/check-make-execution.sh" "$parent/child/scripts/ci/"
cat > "$parent/Makefile" <<'MAKE'
.PHONY: validate adoption same-checkout selection child-gate ci
validate:
	+bash scripts/ci/run-validation-targets.sh adoption same-checkout
adoption:
	@GNUMAKEFLAGS="$(FIXTURE_MAKE_FLAGS)" MAKEFILES="$(FIXTURE_MAKE_FILES)" bash "$(METADATA_FIXTURE)"
	+bash child/scripts/ci/run-validation-targets.sh child-gate
same-checkout:
	+bash scripts/ci/run-validation-targets.sh selection
selection:
	@test "$(RELEASE_VERSION)" = 9.8.7
	@test "$(RELEASE_COMMIT)" = parent-selected-commit
	@test "$(LABEL)" = 'night time'
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
# Only the independently configured adoption fixture receives these controls.
# Same-checkout nested validation must retain the parent's legitimate selections.
printf 'RELEASE_VERSION := 99.99.99\n' > "$parent/fixture-includes.mk"
for context in baseline includes gnu-flags; do
    fixture_flags=''; fixture_includes=''
    case "$context" in
        includes) fixture_includes="$parent/fixture-includes.mk" ;;
        gnu-flags) fixture_flags=--dry-run ;;
    esac
    context_log="$FIXTURE/nested-context-$context.log"
    if ! VALIDATION_RUNNER_DEPTH=0 VALIDATION_FAILURE_LOG_DIR="$parent/failure-logs" \
        make --no-print-directory -j2 -k -s -C "$parent" validate \
        RELEASE_VERSION=9.8.7 RELEASE_COMMIT=parent-selected-commit \
        'LABEL=night time' \
        "FIXTURE_MAKE_FLAGS=$fixture_flags" "FIXTURE_MAKE_FILES=$fixture_includes" \
        METADATA_FIXTURE="$ROOT/scripts/ci/test-release-metadata.sh" \
        > "$context_log" 2>&1; then
        cat "$context_log" >&2
        echo "validation target runner test failed: nested release context ($context)" >&2
        exit 1
    fi
    [[ ! -e "$parent/incorrect-route" ]] || {
        echo 'validation target runner test failed: executed the parent gate' >&2
        exit 1
    }
    if rg -i 'jobserver unavailable|jobserver.*forced' "$context_log" >/dev/null; then
        echo 'validation target runner lost the inherited Make jobserver' >&2
        exit 1
    fi
    for marker in child-gate-marker nested-selection-marker \
        'release metadata real-Git and validation-retention tests passed'; do
        rg -F "$marker" "$context_log" >/dev/null
    done
done

# A retained run includes successes, raw bytes and a timing row per completed
# target. Literal structured prefixes are shared mechanics, not product policy.
cp "$ROOT/scripts/ci/run-validation-targets.sh" "$FIXTURE/scripts/ci/"
cat >> "$FIXTURE/Makefile" <<'MAKE'
structured:
	@printf '[CONSUMER:E001] structured-marker\n'
	@printf 'test error::tests::passing ... ok\n'
	@printf '\033[32mcolored raw bytes\033[0m\n'
	@exit 7
nested-failure:
	+bash scripts/ci/run-validation-targets.sh fail-one
MAKE
status=0
VALIDATION_REPOSITORY_ROOT="$FIXTURE" VALIDATION_LOG_DIR="$FIXTURE/runs" \
    VALIDATION_FAILURE_LOG_DIR="$FIXTURE/failure-logs" VALIDATION_FAILURE_EVENT_PREFIX='[CONSUMER:E' \
    GITHUB_STEP_SUMMARY="$FIXTURE/summary.md" VALIDATION_RUNNER_DEPTH=0 \
    bash "$FIXTURE/scripts/ci/run-validation-targets.sh" pass structured \
    > "$FIXTURE/retained.log" 2>&1 || status=$?
[[ "$status" == 2 ]]
run="$(sed -n 's/^Validation logs and timings: //p' "$FIXTURE/retained.log")"
[[ -d "$run" ]]
printf 'pass-marker\n' > "$FIXTURE/pass.expected"
cmp "$FIXTURE/pass.expected" "$run/0.log"
awk -F '\t' -v run="$run" '
    NR == 1 { if ($0 != "target\tresult\tseconds\tlog") exit 1 }
    NR == 2 { if ($1 != "pass" || $2 != "PASS" || $3 !~ /^[0-9]+$/ || $4 != run "/0.log") exit 1 }
    NR == 3 { if ($1 != "structured" || $2 != "FAIL" || $3 !~ /^[0-9]+$/ || $4 != run "/1.log") exit 1 }
    END { if (NR != 3) exit 1 }
' "$run/timings.tsv"
rg -F '[ERR:structured] [CONSUMER:E001] structured-marker' "$FIXTURE/retained.log" >/dev/null
rg -F '[ERR:structured] [CONSUMER:E001] structured-marker' "$FIXTURE/failure-logs/latest-errors.log" >/dev/null
rg -F $'\033[32mcolored raw bytes\033[0m' "$run/1.log" >/dev/null
if rg -F '[ERR:' "$run/1.log" >/dev/null; then exit 1; fi
[[ "$(rg -c '^### Validation summary' "$FIXTURE/summary.md")" == 1 ]]
cp "$FIXTURE/failure-logs/latest-combined.log" "$FIXTURE/combined-before-pass"
VALIDATION_REPOSITORY_ROOT="$FIXTURE" VALIDATION_LOG_DIR="$FIXTURE/runs" \
    VALIDATION_FAILURE_LOG_DIR="$FIXTURE/failure-logs" \
    GITHUB_STEP_SUMMARY="$FIXTURE/summary.md" VALIDATION_RUNNER_DEPTH=1 \
    bash "$FIXTURE/scripts/ci/run-validation-targets.sh" pass > "$FIXTURE/retained-pass.log" 2>&1
pass_run="$(sed -n 's/^Validation logs and timings: //p' "$FIXTURE/retained-pass.log")"
[[ "$pass_run" != "$run" && -f "$pass_run/0.log" && -f "$run/1.log" ]]
[[ "$(rg -c '^### Validation summary' "$FIXTURE/summary.md")" == 1 ]]
cmp "$FIXTURE/combined-before-pass" "$FIXTURE/failure-logs/latest-combined.log"

# Each nesting level combines its own raw target streams, without rediscovering
# child files and duplicating them in the outer batch.
status=0
VALIDATION_REPOSITORY_ROOT="$FIXTURE" VALIDATION_LOG_DIR="$FIXTURE/runs" \
    VALIDATION_FAILURE_LOG_DIR="$FIXTURE/failure-logs" \
    bash "$FIXTURE/scripts/ci/run-validation-targets.sh" nested-failure fail-two \
    > "$FIXTURE/nested-failures.log" 2>&1 || status=$?
[[ "$status" == 2 ]]
outer_run="$(sed -n 's/^Validation logs and timings: //p' "$FIXTURE/nested-failures.log" | head -n 1)"
cat "$outer_run/0.log" "$outer_run/1.log" > "$FIXTURE/nested-combined.expected"
cmp "$FIXTURE/nested-combined.expected" "$FIXTURE/failure-logs/latest-combined.log"

# A failed aggregate write must preserve raw inputs and the prior complete
# latest batch, while returning the original target failure status.
mkdir "$FIXTURE/cat-bin" "$FIXTURE/aggregate-tmp"
REAL_CAT="$(command -v cat)"
export REAL_CAT
cat > "$FIXTURE/cat-bin/cat" <<'SCRIPT'
#!/usr/bin/env bash
case "${1:-}" in */0.log) printf 'partial aggregate\n'; exit 23 ;; esac
exec "$REAL_CAT" "$@"
SCRIPT
chmod +x "$FIXTURE/cat-bin/cat"
cp "$FIXTURE/failure-logs/latest-combined.log" "$FIXTURE/combined-before-failure"
status=0
PATH="$FIXTURE/cat-bin:$PATH" TMPDIR="$FIXTURE/aggregate-tmp" \
    VALIDATION_REPOSITORY_ROOT="$FIXTURE" VALIDATION_FAILURE_LOG_DIR="$FIXTURE/failure-logs" \
    bash "$FIXTURE/scripts/ci/run-validation-targets.sh" fail-one \
    > "$FIXTURE/aggregate-failure.log" 2>&1 || status=$?
[[ "$status" == 2 ]]
cmp "$FIXTURE/combined-before-failure" "$FIXTURE/failure-logs/latest-combined.log"
aggregate_raw=("$FIXTURE/aggregate-tmp/"validation.*/0.log)
rg -F first-failure-marker "${aggregate_raw[0]}" >/dev/null
partial="$(sed -n 's/^Incomplete combined failure log retained at: //p' "$FIXTURE/aggregate-failure.log")"
rg -Fx 'partial aggregate' "$partial" >/dev/null

# Simulate a signal to the executing logger after output, without signaling the
# test process group. Make's admission probe still uses the real executable.
real_make="$(command -v make)"
mkdir "$FIXTURE/signal-bin"
cat > "$FIXTURE/signal-bin/make" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$*" != *interrupt-fixture* ]]; then exec "$REAL_MAKE" "$@"; fi
printf 'partial raw output\n'
kill -TERM "$PPID"
SCRIPT
chmod +x "$FIXTURE/signal-bin/make"
status=0
REAL_MAKE="$real_make" PATH="$FIXTURE/signal-bin:$PATH" \
    VALIDATION_FAILURE_LOG_DIR="$FIXTURE/failure-logs" \
    VALIDATION_REPOSITORY_ROOT="$FIXTURE" VALIDATION_LOG_DIR="$FIXTURE/runs" \
    bash "$FIXTURE/scripts/ci/run-validation-targets.sh" interrupt-fixture \
    > "$FIXTURE/interrupted.log" 2>&1 || status=$?
[[ "$status" == 143 ]]
interrupted_run="$(sed -n 's/^Validation logs and timings: //p' "$FIXTURE/interrupted.log")"
rg -Fx 'partial raw output' "$interrupted_run/0.log" >/dev/null
[[ "$(wc -l < "$interrupted_run/timings.tsv" | tr -d ' ')" == 1 ]]
cmp "$FIXTURE/combined-before-failure" "$FIXTURE/failure-logs/latest-combined.log"
echo "validation target runner test passed"
