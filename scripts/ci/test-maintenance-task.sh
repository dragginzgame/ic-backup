#!/usr/bin/env bash
# Shared companions: scripts/dev/run-maintenance.sh scripts/ci/fixture-lifecycle.sh
set -euo pipefail

ROOT="$0"
[[ "$ROOT" == /* ]] || ROOT="$PWD/$ROOT"
ROOT="$(cd -P "${ROOT%/*}/../.." && printf '%s/.' "$PWD")"
ROOT="${ROOT%/.}"
FIXTURE="$(mktemp -d "${TMPDIR:-/tmp}/maintenance-task-test.XXXXXX")"
FIXTURE="$(CDPATH="" cd -P -- "$FIXTURE" && printf '%s/.' "$PWD")"
FIXTURE="${FIXTURE%/.}"
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$ROOT/scripts/ci/fixture-lifecycle.sh"
fixture_complete=false
trap 'fixture_finish "$?" "$FIXTURE" "Maintenance fixture retained"' EXIT
mkdir -p "$FIXTURE/projects with spaces" "$FIXTURE/state with spaces"
mkdir "$FIXTURE/bin"
MAINTENANCE_REAL_DATE="$(command -v date)"
export MAINTENANCE_REAL_DATE MAINTENANCE_TEST_NOW=2000000000
cat > "$FIXTURE/bin/date" <<'CLOCK'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$*" == '-u +%s' ]]; then printf '%s\n' "$MAINTENANCE_TEST_NOW";
else exec "$MAINTENANCE_REAL_DATE" "$@"; fi
CLOCK
chmod +x "$FIXTURE/bin/date"
export PATH="$FIXTURE/bin:$PATH"
export MAINTENANCE_TEST_TRACE="$FIXTURE/trace"
export MAINTENANCE_TEST_CALLS="$FIXTURE/calls"
cat > "$FIXTURE/codex" <<'CLI'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >> "$MAINTENANCE_TEST_CALLS"
case "$*" in
    --version) echo 'codex-cli fixture'; exit 0 ;;
    'exec --help') echo '--approve-for-me'; exit 0 ;;
    'login status') exit 0 ;;
esac
[[ $# == 8 && "$1" == exec && "$2" == --approve-for-me && "$3" == --cd && -d "$4" && "$5" == --json && "$6" == --output-last-message && "$8" == - ]] || exit 1
printf 'dispatch\n' >> "$MAINTENANCE_TEST_TRACE"
cat > "${7%/*}/received-prompt"
[[ -s "${7%/*}/received-prompt" ]] || exit 1
case "${MAINTENANCE_TEST_RESULT:-success}" in
    failure) echo 'retained failure' >&2; exit 17 ;;
    empty) exit 0 ;;
    success) printf 'Fixture report\n' > "$7" ;;
esac
CLI
chmod +x "$FIXTURE/codex"
export CODEX_BIN="$FIXTURE/codex"
runner="$ROOT/scripts/dev/run-maintenance.sh"
projects="$FIXTURE/projects with spaces"
state="$FIXTURE/state with spaces"
bash "$runner" --check "$projects" "$state" > "$FIXTURE/check.log"
[[ ! -e "$MAINTENANCE_TEST_TRACE" && ! -e "$state/runs" ]] || exit 1
# Bad literal or physical directory names must not probe the CLI, consume an
# interval or create state, even when a future state root does not exist yet.
cp "$MAINTENANCE_TEST_CALLS" "$FIXTURE/expected-calls"
for suffix in $'\n' $'\r'; do
    bad="$FIXTURE/bad$suffix"
    mkdir "$bad"
    printf 'preserved timestamp\n' > "$bad/last-attempt"
    printf 'preserved report\n' > "$bad/report.md"
    ln -s "$bad" "$FIXTURE/alias"
    for mode in --check --run --if-due; do
        for input in "$bad" "$FIXTURE/alias"; do
            if bash "$runner" "$mode" "$input" "$state" > "$FIXTURE/path.log" 2>&1; then exit 1; fi
            grep -F 'contains CR/LF' "$FIXTURE/path.log"
            for target in "$input" "$input/new/nested"; do
                if bash "$runner" "$mode" "$projects" "$target" > "$FIXTURE/path.log" 2>&1; then exit 1; fi
                grep -F 'contains CR/LF' "$FIXTURE/path.log"
            done
        done
    done
    # Also reject an invalid runner checkout reached through an ordinary alias.
    mkdir -p "$bad/scripts/dev"
    cp "$runner" "$bad/scripts/dev/run-maintenance.sh"
    if bash "$FIXTURE/alias/scripts/dev/run-maintenance.sh" --check "$projects" "$state" > "$FIXTURE/path.log" 2>&1; then exit 1; fi
    grep -F 'contains CR/LF' "$FIXTURE/path.log"
    cmp "$MAINTENANCE_TEST_CALLS" "$FIXTURE/expected-calls"
    [[ ! -e "$state/runs" && ! -e "$state/last-attempt" && ! -e "$bad/runs" && ! -e "$bad/new" &&
        "$(cat "$bad/last-attempt")" == 'preserved timestamp' && "$(cat "$bad/report.md")" == 'preserved report' ]] || exit 1
    rm "$FIXTURE/alias"
done
# Normal aliases and spaces remain supported, including new state descendants.
ln -s "$projects" "$FIXTURE/projects-alias"
ln -s "$state" "$FIXTURE/state-alias"
bash "$runner" --check "$FIXTURE/projects-alias" "$FIXTURE/state-alias/new/nested" > "$FIXTURE/alias-check.log"
[[ ! -e "$state/new" ]] || exit 1
bash "$runner" --if-due "$projects" "$state" > "$FIXTURE/first.log"
set -- "$state"/runs/*
first="$1"
[[ $# == 1 && "$(cat "$first/exit-status")" == 0 ]] || exit 1
cmp "$first/prompt.txt" "$first/received-prompt"
[[ "$(cat "$first/report.md")" == 'Fixture report' ]] || exit 1
bash "$runner" --if-due "$projects" "$state" > "$FIXTURE/not-due.log"
[[ "$(wc -l < "$MAINTENANCE_TEST_TRACE" | tr -d ' ')" == 1 ]] || exit 1
# A third-morning wakeup one second earlier still dispatches, rather than
# slipping to day four. Failure keeps logs and consumes the interval.
export MAINTENANCE_TEST_NOW=2000259199
status=0
MAINTENANCE_TEST_RESULT=failure bash "$runner" --if-due "$projects" "$state" > "$FIXTURE/failed.log" 2>&1 || status=$?
[[ "$status" == 17 ]] || exit 1
failure_count=0
for attempt in "$state"/runs/*; do
    if [[ "$(cat "$attempt/exit-status")" == 17 ]]; then
        [[ "$(cat "$attempt/stderr.log")" == 'retained failure' ]] || exit 1
        failure_count=$((failure_count + 1))
    fi
done
[[ "$failure_count" == 1 ]] || exit 1
bash "$runner" --if-due "$projects" "$state" > "$FIXTURE/failed-not-due.log"
[[ "$(wc -l < "$MAINTENANCE_TEST_TRACE" | tr -d ' ')" == 2 ]] || exit 1
# A manual retry bypasses the interval but still rejects an empty final report.
if MAINTENANCE_TEST_RESULT=empty bash "$runner" --run "$projects" "$state" > "$FIXTURE/empty.log" 2>&1; then exit 1; fi
[[ -s "$first/report.md" && "$(wc -l < "$MAINTENANCE_TEST_TRACE" | tr -d ' ')" == 3 ]] || exit 1
status=0
CODEX_BIN="$FIXTURE/missing-codex" bash "$runner" --run "$projects" "$state" > "$FIXTURE/missing-cli.log" 2>&1 || status=$?
[[ "$status" != 0 ]] || exit 1
bash "$runner" --if-due "$projects" "$state" > "$FIXTURE/missing-cli-not-due.log"
[[ "$(wc -l < "$MAINTENANCE_TEST_TRACE" | tr -d ' ')" == 3 ]] || exit 1
printf 'corrupt\n' > "$state/last-attempt"
if bash "$runner" --if-due "$projects" "$state" > "$FIXTURE/corrupt.log" 2>&1; then exit 1; fi
printf '9999999999\n' > "$state/last-attempt"
if bash "$runner" --if-due "$projects" "$state" > "$FIXTURE/future.log" 2>&1; then exit 1; fi
[[ "$(wc -l < "$MAINTENANCE_TEST_TRACE" | tr -d ' ')" == 3 ]] || exit 1
echo 'Maintenance task fixture passed (substitute CLI; no live agent or schedule).'
fixture_complete=true
