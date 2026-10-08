#!/usr/bin/env bash
# Shared companions: scripts/dev/run-maintenance.sh
set -euo pipefail

ROOT="$0"
[[ "$ROOT" == /* ]] || ROOT="$PWD/$ROOT"
ROOT="$(cd -P "${ROOT%/*}/../.." && printf '%s/.' "$PWD")"
ROOT="${ROOT%/.}"
FIXTURE="$(mktemp -d "${TMPDIR:-/tmp}/maintenance-task-test.XXXXXX")"
trap 'if [[ $? == 0 ]]; then rm -rf "$FIXTURE"; else printf "Maintenance fixture retained: %s\n" "$FIXTURE" >&2; fi' EXIT
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
cat > "$FIXTURE/codex" <<'CLI'
#!/usr/bin/env bash
set -euo pipefail
case "$*" in
    --version) echo 'codex-cli fixture'; exit 0 ;;
    'exec --help') echo '--approve-for-me'; exit 0 ;;
    'login status') exit 0 ;;
esac
[[ $# == 8 && "$1" == exec && "$2" == --approve-for-me && "$3" == --cd && -d "$4" && "$5" == --json && "$6" == --output-last-message && "$8" == - ]]
printf 'dispatch\n' >> "$MAINTENANCE_TEST_TRACE"
cat > "${7%/*}/received-prompt"
[[ -s "${7%/*}/received-prompt" ]]
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
[[ ! -e "$MAINTENANCE_TEST_TRACE" && ! -e "$state/runs" ]]
bash "$runner" --if-due "$projects" "$state" > "$FIXTURE/first.log"
set -- "$state"/runs/*
first="$1"
[[ $# == 1 && "$(cat "$first/exit-status")" == 0 ]]
cmp "$first/prompt.txt" "$first/received-prompt"
[[ "$(cat "$first/report.md")" == 'Fixture report' ]]
bash "$runner" --if-due "$projects" "$state" > "$FIXTURE/not-due.log"
[[ "$(wc -l < "$MAINTENANCE_TEST_TRACE" | tr -d ' ')" == 1 ]]
# A third-morning wakeup one second earlier still dispatches, rather than
# slipping to day four. Failure keeps logs and consumes the interval.
export MAINTENANCE_TEST_NOW=2000259199
status=0
MAINTENANCE_TEST_RESULT=failure bash "$runner" --if-due "$projects" "$state" > "$FIXTURE/failed.log" 2>&1 || status=$?
[[ "$status" == 17 ]]
failure_count=0
for attempt in "$state"/runs/*; do
    if [[ "$(cat "$attempt/exit-status")" == 17 ]]; then
        [[ "$(cat "$attempt/stderr.log")" == 'retained failure' ]]
        failure_count=$((failure_count + 1))
    fi
done
[[ "$failure_count" == 1 ]]
bash "$runner" --if-due "$projects" "$state" > "$FIXTURE/failed-not-due.log"
[[ "$(wc -l < "$MAINTENANCE_TEST_TRACE" | tr -d ' ')" == 2 ]]
# A manual retry bypasses the interval but still rejects an empty final report.
if MAINTENANCE_TEST_RESULT=empty bash "$runner" --run "$projects" "$state" > "$FIXTURE/empty.log" 2>&1; then exit 1; fi
[[ -s "$first/report.md" && "$(wc -l < "$MAINTENANCE_TEST_TRACE" | tr -d ' ')" == 3 ]]
status=0
CODEX_BIN="$FIXTURE/missing-codex" bash "$runner" --run "$projects" "$state" > "$FIXTURE/missing-cli.log" 2>&1 || status=$?
[[ "$status" != 0 ]]
bash "$runner" --if-due "$projects" "$state" > "$FIXTURE/missing-cli-not-due.log"
[[ "$(wc -l < "$MAINTENANCE_TEST_TRACE" | tr -d ' ')" == 3 ]]
printf 'corrupt\n' > "$state/last-attempt"
if bash "$runner" --if-due "$projects" "$state" > "$FIXTURE/corrupt.log" 2>&1; then exit 1; fi
printf '9999999999\n' > "$state/last-attempt"
if bash "$runner" --if-due "$projects" "$state" > "$FIXTURE/future.log" 2>&1; then exit 1; fi
[[ "$(wc -l < "$MAINTENANCE_TEST_TRACE" | tr -d ' ')" == 3 ]]
echo 'Maintenance task fixture passed (substitute CLI; no live agent or schedule).'
