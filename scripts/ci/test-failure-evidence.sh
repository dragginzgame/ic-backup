#!/usr/bin/env bash
# Qualify this consumer's actual workflow step; Shared owns selector mechanics.
set -euo pipefail
ROOT="${BASH_SOURCE[0]}"
[[ "$ROOT" == /* ]] || ROOT="$PWD/$ROOT"
ROOT="$(cd -P "${ROOT%/*}/../.." && printf '%s/.' "$PWD")"
ROOT="${ROOT%/.}"
export PATH="$ROOT/.tools/host/bin:$PATH"
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$ROOT/scripts/ci/fixture-lifecycle.sh"
fixture="$(fixture_create "${TMPDIR:-$ROOT/target}/failure-evidence.XXXXXX")"
fixture_complete=false
trap 'fixture_finish "$?" "$fixture" "CI evidence fixture retained"' EXIT
yq -r '.jobs.library.steps[] | select(.id == "validation-archive") | .run' \
    "$ROOT/.github/workflows/ci.yml" > "$fixture/step.sh"
[[ -s "$fixture/step.sh" ]]
for mode in complete partial redirected-target redirected-tools redirected-rust; do
    selected="$fixture/$mode"
    repo="$selected/repository"
    mkdir -p "$repo/scripts/ci" "$repo/target/validation-failures" \
        "$repo/target/ci-fixtures" "$selected/runner/formatting.fixture" "$selected/outside/ci-fixtures"
    cp "$ROOT/scripts/ci/collect-tool-evidence.sh" "$ROOT/scripts/ci/archive-evidence.sh" "$repo/scripts/ci/"
    printf 'original product failure\n' > "$repo/target/validation-failures/error.log"
    printf 'independent formatter failure\n' > "$selected/runner/formatting.fixture/error.log"
    printf 'admitted shared tool\n' > "$selected/shared-tool"
    printf 'must never be traversed\n' > "$selected/outside/canary"
    cat > "$repo/scripts/ci/select-tool-evidence.sh" <<'SELECT'
#!/usr/bin/env bash
printf '%s\0%s\0' "$CI_EVIDENCE_SELECTED" shared-tool
exit "$CI_EVIDENCE_SELECTOR_STATUS"
SELECT
    selector_status=0
    expected_status=0
    case "$mode" in
        partial) selector_status=17; expected_status=1 ;;
        redirected-target)
            mv "$repo/target" "$selected/original-target"
            ln -s "$selected/outside" "$repo/target"
            expected_status=1 ;;
        redirected-tools)
            ln -s "$selected/outside" "$repo/.tools"
            expected_status=1 ;;
        redirected-rust)
            mkdir "$repo/.tools"
            ln -s "$selected/outside" "$repo/.tools/rust"
            expected_status=1 ;;
    esac
    status=0
    (cd "$repo"; GITHUB_WORKSPACE="$repo" RUNNER_TEMP="$selected/runner" \
        TMPDIR="$repo/target/ci-fixtures" GITHUB_OUTPUT="$selected/output" \
        CI_EVIDENCE_SELECTED="$selected" CI_EVIDENCE_SELECTOR_STATUS="$selector_status" \
        bash "$fixture/step.sh") > "$selected/step.log" 2>&1 || status=$?
    [[ "$status" == "$expected_status" ]]
    archive_root="$(sed -n 's/^path=//p' "$selected/output")"
    [[ -d "$archive_root" && -f "$archive_root/evidence.tar.gz" ]]
    [[ "$(cat "$archive_root/diagnostics/selection-status")" == "$selector_status" ]]
    tar -tzf "$archive_root/evidence.tar.gz" > "$selected/members"
    grep -Fx './formatting.fixture/error.log' "$selected/members"
    if [[ "$mode" == partial ]]; then
        if grep -Fx './shared-tool' "$selected/members"; then exit 1; fi
        grep -F '/selections.nul' "$selected/members"
    else
        grep -Fx './shared-tool' "$selected/members"
    fi
    if [[ "$mode" == redirected-target ]]; then
        grep -Fx './target' "$selected/members"
        if grep -F 'canary' "$selected/members"; then exit 1; fi
        cmp "$selected/outside/canary" <(printf 'must never be traversed\n')
    else
        grep -Fx './target/validation-failures/error.log' "$selected/members"
        tar -xOzf "$archive_root/evidence.tar.gz" ./target/validation-failures/error.log \
            > "$selected/retained-error"
        cmp "$repo/target/validation-failures/error.log" "$selected/retained-error"
    fi
    if [[ "$mode" == redirected-tools ]]; then grep -Fx './.tools' "$selected/members"; fi
    if [[ "$mode" == redirected-rust ]]; then grep -Fx './.tools/rust' "$selected/members"; fi
    if grep -F 'canary' "$selected/members"; then exit 1; fi
    if [[ "$mode" == redirected-* ]]; then
        [[ "$(cat "$archive_root/diagnostics/local-selection-status")" == 1 ]]
        [[ -s "$archive_root/diagnostics/local-selection.log" ]]
    fi
    printf 'CI failure evidence: PASS [%s]\n' "$mode"
done
fixture_complete=true
