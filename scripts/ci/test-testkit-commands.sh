#!/usr/bin/env bash
# Shared companions: scripts/ci/fixture-lifecycle.sh scripts/ci/prepare-script-fixture.sh
# Qualify local Make routing; provisioning and receipt admission stay with their owners.
set -euo pipefail
ROOT="${BASH_SOURCE[0]}"
[[ "$ROOT" == /* ]] || ROOT="$PWD/$ROOT"
ROOT="$(cd -P "${ROOT%/*}/../.." && printf '%s/.' "$PWD")"
ROOT="${ROOT%/.}"
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$ROOT/scripts/ci/fixture-lifecycle.sh"
mkdir -p "$ROOT/target"
fixture="$(fixture_create "${TMPDIR:-$ROOT/target}/testkit-commands.XXXXXX")"
fixture_complete=false
finish() {
    fixture_finish "$?" "$fixture" 'Testkit command fixture retained'
}
trap finish EXIT
mkdir -p "$fixture/consumer/make" "$fixture/consumer/scripts/ci" "$fixture/bin" "$fixture/cli"
bash "$ROOT/scripts/ci/prepare-script-fixture.sh" "$ROOT" "$fixture/consumer" \
    Makefile make/consumer.mk
cat > "$fixture/consumer/Cargo.lock" <<'LOCK'
version = 4

[[package]]
name = "ic-testkit"
version = "0.33.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
LOCK
cat > "$fixture/bin/cargo" <<'CARGO'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${TESTKIT_COMMAND_JOBSERVER:-0}" == 1 ]]; then
    if [[ "${MAKEFLAGS:-}" =~ --jobserver-(auth|fds)=([0-9]+),([0-9]+) ]]; then
        reader="${BASH_REMATCH[2]}"; writer="${BASH_REMATCH[3]}"
        : <&"$reader"
        : >&"$writer"
    elif [[ "${MAKEFLAGS:-}" =~ --jobserver-auth=fifo:([^[:space:]]+) ]]; then
        [[ -p "${BASH_REMATCH[1]}" ]] || exit 1
    else
        echo 'missing usable GNU Make jobserver' >&2
        exit 1
    fi
fi
case "$*" in
    'fetch --locked') printf 'fetch\n' >> "$TESTKIT_COMMAND_ORDER" ;;
    'check --offline --locked -p ic-backup -p ic-backup-agent --all-targets --all-features'|'clippy --offline --locked -p ic-backup -p ic-backup-agent --all-targets --all-features -- -D warnings')
        printf 'build %s\n' "$1" >> "$TESTKIT_COMMAND_ORDER" ;;
    *) exit 97 ;;
esac
CARGO
{
    printf '#!%s\n' "$BASH"
    cat <<'BASH'
set -euo pipefail
if [[ "${1:-}" == scripts/dev/install-rust-tools.sh ]]; then
    shift
    printf '%s\n' "$*" >> "$TESTKIT_COMMAND_INSTALL"
    [[ " $* " == *' --features cli '* ]] || exit 27
    [[ ${TESTKIT_COMMAND_INSTALL_FAILURE:-0} != 75 ]] || exit 75
    [[ ${TESTKIT_COMMAND_INSTALL_FAILURE:-0} == 0 ]] || exit 17
    printf '%s\n' "$TESTKIT_COMMAND_CLI"
else
    case "${1:-}" in
        */scripts/dev/install-host-tools.sh|*/scripts/dev/install-ic-tools.sh|*/scripts/dev/install-rust-tools.sh)
            name="${1##*/}"
            if [[ "$name" == install-ic-tools.sh ]]; then
                printf '%s\0' "${@:2}" > "$TESTKIT_COMMAND_OPTIONAL_ARGS"
                [[ " $* " == *' --tool '* ]] || {
                    echo 'Explicit IC executable selection is required' >&2
                    exit 28
                }
            fi
            if [[ "$name" == install-ic-tools.sh && "${TESTKIT_COMMAND_OPTIONAL_IC:-0}" != 1 ]]; then
                echo 'Optional IC tools are unavailable in this fixture' >&2
                exit 26
            fi
            if [[ " $* " == *' --package ic-build '* ]]; then
                [[ " $* " == *' --features cli '* ]] || exit 27
                mode=setup
                [[ "${!#}" != --check ]] || mode=check
                printf 'editor %s\n' "$mode" >> "$TESTKIT_COMMAND_ORDER"
                [[ "${TESTKIT_COMMAND_EDITOR_FAILURE:-0}" == 0 ]] || exit 25
                exit 0
            fi
            if [[ "${!#}" == --preflight ]]; then
                [[ "${TESTKIT_COMMAND_PREFLIGHT_FAILURE:-}" != "$name" ]] || exit 24
                exit 0
            fi
            printf 'common %s\n' "$name" >> "$TESTKIT_COMMAND_ORDER"
            [[ "${TESTKIT_COMMAND_COMMON_FAILURE:-}" != "$name" ]] || exit 18
            exit 0 ;;
        *) exec "$TESTKIT_COMMAND_BASH" "$@" ;;
    esac
fi
BASH
} > "$fixture/bin/bash"
cat > "$fixture/cli/ic-testkit-server" <<'CLI'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >> "$TESTKIT_COMMAND_CALLS"
printf 'server %s\n' "$1" >> "$TESTKIT_COMMAND_ORDER"
[[ ${TESTKIT_COMMAND_SERVER_FAILURE:-0} == 0 ]] || exit 23
printf '%s\n' "$TESTKIT_COMMAND_SERVER"
CLI
chmod +x "$fixture/bin/cargo" "$fixture/bin/bash" "$fixture/cli/ic-testkit-server"
export TESTKIT_COMMAND_INSTALL="$fixture/install.log"
export TESTKIT_COMMAND_CALLS="$fixture/calls.log"
export TESTKIT_COMMAND_ORDER="$fixture/order.log"
export TESTKIT_COMMAND_OPTIONAL_ARGS="$fixture/optional-args"
export TESTKIT_COMMAND_CLI="$fixture/cli/ic-testkit-server"
export TESTKIT_COMMAND_SERVER="$fixture/admitted-server"
export TESTKIT_COMMAND_BASH="$BASH"
export PATH="$fixture/bin:$ROOT/.tools/host/bin:$PATH"
printf '%s\n' 'retained original evidence' > "$fixture/retained"
# Each invocation owns its Make context, including under a parent CI runner.
run_make() {
    env -u MAKEFLAGS -u MFLAGS -u MAKELEVEL -u MAKEFILES \
        make --silent --no-print-directory -C "$fixture/consumer" "$@"
}
run_make testkit-server-check > "$fixture/path"
printf '%s\n' "$TESTKIT_COMMAND_SERVER" > "$fixture/expected-path"
cmp "$fixture/path" "$fixture/expected-path"
printf '%s\n' "--consumer $fixture/consumer --package ic-testkit --lockfile Cargo.lock --bin ic-testkit-server --profile release --features cli --check" > "$fixture/expected-install"
cmp "$TESTKIT_COMMAND_INSTALL" "$fixture/expected-install"
printf '%s\n' "check --directory $fixture/consumer/.tools/testkit-server" > "$fixture/expected-calls"
cmp "$TESTKIT_COMMAND_CALLS" "$fixture/expected-calls"
run_make install-testkit-server > "$fixture/setup-path"
printf '%s\n' "--consumer $fixture/consumer --package ic-testkit --lockfile Cargo.lock --bin ic-testkit-server --profile release --features cli" >> "$fixture/expected-install"
cmp "$TESTKIT_COMMAND_INSTALL" "$fixture/expected-install"
printf '%s\n' "setup --directory $fixture/consumer/.tools/testkit-server" >> "$fixture/expected-calls"
cmp "$TESTKIT_COMMAND_CALLS" "$fixture/expected-calls"
# Missing/changed CLI admission must stop before invoking the server owner.
if TESTKIT_COMMAND_INSTALL_FAILURE=1 run_make testkit-server-check > "$fixture/missing.log" 2>&1; then exit 1; fi
grep -F 'Locked Testkit CLI' "$fixture/missing.log" >/dev/null
grep -F 'make install-testkit-server' "$fixture/missing.log" >/dev/null
cmp "$TESTKIT_COMMAND_CALLS" "$fixture/expected-calls"
# Busy shared-slot admission is distinct from missing installation, with no setup.
if TESTKIT_COMMAND_INSTALL_FAILURE=75 run_make testkit-server-check > "$fixture/busy.log" 2>&1; then exit 1; fi
grep -F 'Locked Testkit CLI is busy' "$fixture/busy.log" >/dev/null
grep -F 'Error 75' "$fixture/busy.log" >/dev/null
if grep -F 'make install-testkit-server' "$fixture/busy.log"; then exit 1; fi
cmp "$TESTKIT_COMMAND_CALLS" "$fixture/expected-calls"
# Server admission failure propagates without a setup or alternate server attempt.
if TESTKIT_COMMAND_SERVER_FAILURE=1 run_make testkit-server-check > "$fixture/changed.log" 2>&1; then exit 1; fi
printf '%s\n' "check --directory $fixture/consumer/.tools/testkit-server" >> "$fixture/expected-calls"
cmp "$TESTKIT_COMMAND_CALLS" "$fixture/expected-calls"
# A jobserver declaration alone is insufficient: absent/non-FIFO paths and
# missing authentication must refuse before the mock compiler records a build.
for auth in '' "--jobserver-auth=fifo:$fixture/missing-fifo" "--jobserver-auth=fifo:$fixture/retained"; do
    : > "$TESTKIT_COMMAND_ORDER"
    if TESTKIT_COMMAND_JOBSERVER=1 MAKEFLAGS="$auth" "$fixture/bin/cargo" check --offline --locked -p ic-backup -p ic-backup-agent --all-targets --all-features > "$fixture/invalid-jobserver.log" 2>&1; then exit 1; fi
    [[ ! -s "$TESTKIT_COMMAND_ORDER" ]] || exit 1
done
# Canonical lockfile identity, source and drift cases belong to Shared Tooling.
# This consumer fixture qualifies routing and fail-closed command ordering.
# Exercise real prerequisite edges under parallel Make. Refusal must precede
# every dependent compiler/test invocation without installing anything.
for failure in TESTKIT_COMMAND_INSTALL_FAILURE TESTKIT_COMMAND_SERVER_FAILURE; do
    : > "$TESTKIT_COMMAND_ORDER"
    if env -u MAKEFLAGS -u MFLAGS -u MAKELEVEL -u MAKEFILES "$failure=1" make --silent --no-print-directory -C "$fixture/consumer" -j4 check clippy test check-msrv > "$fixture/parallel-$failure.log" 2>&1; then exit 1; fi
    if grep -q '^build ' "$TESTKIT_COMMAND_ORDER"; then exit 1; fi
    tail -n 1 "$TESTKIT_COMMAND_INSTALL" | grep -F -- '--check' >/dev/null
done
: > "$TESTKIT_COMMAND_ORDER"
TESTKIT_COMMAND_JOBSERVER=1 run_make -j4 check clippy > "$fixture/parallel-success.log" 2>&1
[[ "$(head -n 1 "$TESTKIT_COMMAND_ORDER")" == 'server check' ]] || exit 1
[[ "$(grep -c '^server ' "$TESTKIT_COMMAND_ORDER")" == 1 ]] || exit 1
[[ "$(grep -c '^build ' "$TESTKIT_COMMAND_ORDER")" == 2 ]] || exit 1
# The actual consumer aggregate includes locked fetch before Testkit setup;
# parallel Make must preserve required host/Cargo order without optional IC tools.
: > "$TESTKIT_COMMAND_ORDER"
TESTKIT_COMMAND_JOBSERVER=1 run_make -j4 install-tools > "$fixture/aggregate-install.log" 2>&1
printf '%s\n' 'common install-host-tools.sh' 'common install-rust-tools.sh' fetch 'server setup' 'editor setup' > "$fixture/expected-order"
cmp "$TESTKIT_COMMAND_ORDER" "$fixture/expected-order"
: > "$TESTKIT_COMMAND_ORDER"
run_make -j4 tools-check > "$fixture/aggregate-check.log" 2>&1
printf '%s\n' 'common install-host-tools.sh' 'common install-rust-tools.sh' 'server check' 'editor check' > "$fixture/expected-order"
cmp "$TESTKIT_COMMAND_ORDER" "$fixture/expected-order"
for target in install-tools tools-check; do
    for tool in install-host-tools.sh install-rust-tools.sh; do
        : > "$TESTKIT_COMMAND_ORDER"
        if TESTKIT_COMMAND_COMMON_FAILURE="$tool" run_make -j4 "$target" > "$fixture/aggregate-$target-$tool-failure.log" 2>&1; then exit 1; fi
        printf '%s\n' 'common install-host-tools.sh' > "$fixture/expected-order"
        if [[ "$tool" == install-rust-tools.sh ]]; then
            printf '%s\n' 'common install-rust-tools.sh' >> "$fixture/expected-order"
        fi
        cmp "$TESTKIT_COMMAND_ORDER" "$fixture/expected-order"
    done
done
: > "$TESTKIT_COMMAND_ORDER"
if TESTKIT_COMMAND_PREFLIGHT_FAILURE=install-rust-tools.sh run_make -j4 install-tools > "$fixture/preflight-rust.log" 2>&1; then exit 1; fi
[[ ! -s "$TESTKIT_COMMAND_ORDER" ]] || exit 1
: > "$TESTKIT_COMMAND_ORDER"
if TESTKIT_COMMAND_EDITOR_FAILURE=1 run_make tools-check > "$fixture/editor-refusal.log" 2>&1; then exit 1; fi
printf '%s\n' 'common install-host-tools.sh' 'common install-rust-tools.sh' 'server check' 'editor check' > "$fixture/expected-order"
cmp "$TESTKIT_COMMAND_ORDER" "$fixture/expected-order"
# Manual setup/check remain available, with their own admission failures.
for target in install-ic-tools ic-tools-check; do
    : > "$TESTKIT_COMMAND_ORDER"
    if TESTKIT_COMMAND_OPTIONAL_IC=1 run_make "$target" IC_TOOLS= > "$fixture/optional-$target-unselected.log" 2>&1; then exit 1; fi
    grep -F 'Explicit IC executable selection is required' "$fixture/optional-$target-unselected.log" >/dev/null || exit 1
    [[ ! -s "$TESTKIT_COMMAND_ORDER" ]] || exit 1
    if run_make "$target" IC_TOOLS=wasm-opt > "$fixture/optional-$target-unavailable.log" 2>&1; then exit 1; fi
    [[ ! -s "$TESTKIT_COMMAND_ORDER" ]] || exit 1
    TESTKIT_COMMAND_OPTIONAL_IC=1 run_make "$target" IC_TOOLS=wasm-opt > "$fixture/optional-$target.log" 2>&1
    printf '%s\0' --consumer "$fixture/consumer" --pins "$fixture/consumer/ci/ic-tools.tsv" --tool wasm-opt > "$fixture/expected-optional-args"
    if [[ "$target" == ic-tools-check ]]; then printf '%s\0' --check >> "$fixture/expected-optional-args"; fi
    cmp "$TESTKIT_COMMAND_OPTIONAL_ARGS" "$fixture/expected-optional-args"
    printf '%s\n' 'common install-ic-tools.sh' > "$fixture/expected-order"
    cmp "$TESTKIT_COMMAND_ORDER" "$fixture/expected-order"
    : > "$TESTKIT_COMMAND_ORDER"
    TESTKIT_COMMAND_OPTIONAL_IC=1 run_make "$target" 'IC_TOOLS=wasm-opt icp' > "$fixture/optional-$target-multiple.log" 2>&1
    printf '%s\0' --consumer "$fixture/consumer" --pins "$fixture/consumer/ci/ic-tools.tsv" --tool wasm-opt --tool icp > "$fixture/expected-optional-args"
    if [[ "$target" == ic-tools-check ]]; then printf '%s\0' --check >> "$fixture/expected-optional-args"; fi
    cmp "$TESTKIT_COMMAND_OPTIONAL_ARGS" "$fixture/expected-optional-args"
    cmp "$TESTKIT_COMMAND_ORDER" "$fixture/expected-order"
done
[[ "$(cat "$fixture/retained")" == 'retained original evidence' ]] || exit 1
printf '%s\n' 'Testkit setup/check routing, original selection and refusal fixtures passed'
fixture_complete=true
