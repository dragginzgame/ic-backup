#!/usr/bin/env bash
# Qualify local Make routing; provisioning and receipt admission stay with their owners.
set -euo pipefail
ROOT="${BASH_SOURCE[0]}"
[[ "$ROOT" == /* ]] || ROOT="$PWD/$ROOT"
ROOT="$(cd -P "${ROOT%/*}/../.." && printf '%s/.' "$PWD")"
ROOT="${ROOT%/.}"
fixture="$(mktemp -d "$ROOT/target/testkit-commands.XXXXXX")"
finish() {
    local status=$?
    if [[ $status == 0 ]]; then
        rm -rf "$fixture"
    else
        printf 'Testkit command fixture retained: %s\n' "$fixture" >&2
    fi
}
trap finish EXIT
mkdir -p "$fixture/consumer/make" "$fixture/consumer/scripts/ci" "$fixture/bin" "$fixture/cli"
cp "$ROOT/Makefile" "$fixture/consumer/Makefile"
cp "$ROOT/make/tools.mk" "$ROOT/make/rust-format.mk" "$ROOT/make/execution.mk" "$fixture/consumer/make/"
cp "$ROOT/scripts/ci/check-make-execution.sh" "$ROOT/scripts/ci/run-formatting.sh" "$fixture/consumer/scripts/ci/"
cat > "$fixture/bin/cargo" <<'CARGO'
#!/usr/bin/env bash
set -euo pipefail
case "$*" in
    'fetch --locked') printf 'fetch\n' >> "$TESTKIT_COMMAND_ORDER" ;;
    'metadata --offline --locked --format-version 1') cat "$TESTKIT_COMMAND_METADATA" ;;
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
    [[ ${TESTKIT_COMMAND_INSTALL_FAILURE:-0} == 0 ]] || exit 17
    printf '%s\n' "$TESTKIT_COMMAND_CLI"
else
    case "${1:-}" in
        */scripts/dev/install-host-tools.sh|*/scripts/dev/install-ic-tools.sh|*/scripts/dev/install-rust-tools.sh)
            name="${1##*/}"
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
export TESTKIT_COMMAND_METADATA="$fixture/metadata.json"
export TESTKIT_COMMAND_INSTALL="$fixture/install.log"
export TESTKIT_COMMAND_CALLS="$fixture/calls.log"
export TESTKIT_COMMAND_ORDER="$fixture/order.log"
export TESTKIT_COMMAND_CLI="$fixture/cli/ic-testkit-server"
export TESTKIT_COMMAND_SERVER="$fixture/admitted-server"
export TESTKIT_COMMAND_BASH="$BASH"
export PATH="$fixture/bin:$ROOT/.tools/host/bin:$PATH"
printf '%s\n' '{"packages":[{"name":"ic-testkit","version":"0.25.5"}]}' > "$TESTKIT_COMMAND_METADATA"
printf '%s\n' 'retained original evidence' > "$fixture/retained"
# Each invocation owns its Make context, including under a parent CI runner.
run_make() {
    env -u MAKEFLAGS -u MFLAGS -u MAKELEVEL -u MAKEFILES \
        make --silent --no-print-directory -C "$fixture/consumer" "$@"
}
run_make testkit-server-check > "$fixture/path"
printf '%s\n' "$TESTKIT_COMMAND_SERVER" > "$fixture/expected-path"
cmp "$fixture/path" "$fixture/expected-path"
printf '%s\n' "--consumer $fixture/consumer --package ic-testkit --version 0.25.5 --bin ic-testkit-server --profile release --check" > "$fixture/expected-install"
cmp "$TESTKIT_COMMAND_INSTALL" "$fixture/expected-install"
printf '%s\n' "check --directory $fixture/consumer/.tools/testkit-server" > "$fixture/expected-calls"
cmp "$TESTKIT_COMMAND_CALLS" "$fixture/expected-calls"
run_make install-testkit-server > "$fixture/setup-path"
printf '%s\n' "--consumer $fixture/consumer --package ic-testkit --version 0.25.5 --bin ic-testkit-server --profile release" >> "$fixture/expected-install"
cmp "$TESTKIT_COMMAND_INSTALL" "$fixture/expected-install"
printf '%s\n' "setup --directory $fixture/consumer/.tools/testkit-server" >> "$fixture/expected-calls"
cmp "$TESTKIT_COMMAND_CALLS" "$fixture/expected-calls"
# Missing/changed CLI admission must stop before invoking the server owner.
if TESTKIT_COMMAND_INSTALL_FAILURE=1 run_make testkit-server-check > "$fixture/missing.log" 2>&1; then exit 1; fi
grep -F 'Testkit 0.25.5' "$fixture/missing.log" >/dev/null
grep -F 'make install-testkit-server' "$fixture/missing.log" >/dev/null
cmp "$TESTKIT_COMMAND_CALLS" "$fixture/expected-calls"
# Server admission failure propagates without a setup or alternate server attempt.
if TESTKIT_COMMAND_SERVER_FAILURE=1 run_make testkit-server-check > "$fixture/changed.log" 2>&1; then exit 1; fi
printf '%s\n' "check --directory $fixture/consumer/.tools/testkit-server" >> "$fixture/expected-calls"
cmp "$TESTKIT_COMMAND_CALLS" "$fixture/expected-calls"
printf '%s\n' '{"packages":[{"name":"ic-testkit","version":"0.26.0"}]}' > "$TESTKIT_COMMAND_METADATA"
if TESTKIT_COMMAND_INSTALL_FAILURE=1 run_make testkit-server-check > "$fixture/updated.log" 2>&1; then exit 1; fi
grep -F 'Testkit 0.26.0' "$fixture/updated.log" >/dev/null
tail -n 1 "$TESTKIT_COMMAND_INSTALL" | grep -F -- '--version 0.26.0' >/dev/null
cmp "$TESTKIT_COMMAND_CALLS" "$fixture/expected-calls"
cp "$TESTKIT_COMMAND_INSTALL" "$fixture/before-install"
for metadata in '{"packages":[]}' '{"packages":[{"name":"ic-testkit","version":"0.25.4"},{"name":"ic-testkit","version":"0.25.5"}]}'; do
    printf '%s\n' "$metadata" > "$TESTKIT_COMMAND_METADATA"
    if run_make testkit-server-check > "$fixture/ambiguous.log" 2>&1; then exit 1; fi
    cmp "$TESTKIT_COMMAND_INSTALL" "$fixture/before-install"
    cmp "$TESTKIT_COMMAND_CALLS" "$fixture/expected-calls"
done
# Exercise real prerequisite edges under parallel Make. Refusal must precede
# every dependent compiler/test invocation without installing anything.
printf '%s\n' '{"packages":[{"name":"ic-testkit","version":"0.26.0"}]}' > "$TESTKIT_COMMAND_METADATA"
for failure in TESTKIT_COMMAND_INSTALL_FAILURE TESTKIT_COMMAND_SERVER_FAILURE; do
    : > "$TESTKIT_COMMAND_ORDER"
    if env -u MAKEFLAGS -u MFLAGS -u MAKELEVEL -u MAKEFILES "$failure=1" make --silent --no-print-directory -C "$fixture/consumer" -j4 check clippy test check-msrv > "$fixture/parallel-$failure.log" 2>&1; then exit 1; fi
    if grep -q '^build ' "$TESTKIT_COMMAND_ORDER"; then exit 1; fi
    tail -n 1 "$TESTKIT_COMMAND_INSTALL" | grep -F -- '--check' >/dev/null
done
: > "$TESTKIT_COMMAND_ORDER"
run_make -j4 check clippy > "$fixture/parallel-success.log" 2>&1
[[ "$(head -n 1 "$TESTKIT_COMMAND_ORDER")" == 'server check' ]]
[[ "$(grep -c '^server ' "$TESTKIT_COMMAND_ORDER")" == 1 ]]
[[ "$(grep -c '^build ' "$TESTKIT_COMMAND_ORDER")" == 2 ]]
# The actual consumer aggregate includes locked fetch before Testkit setup;
# parallel Make must preserve common host/IC/Cargo order and stop on failure.
: > "$TESTKIT_COMMAND_ORDER"
run_make -j4 install-tools > "$fixture/aggregate-install.log" 2>&1
printf '%s\n' 'common install-host-tools.sh' 'common install-ic-tools.sh' 'common install-rust-tools.sh' fetch 'server setup' > "$fixture/expected-order"
cmp "$TESTKIT_COMMAND_ORDER" "$fixture/expected-order"
: > "$TESTKIT_COMMAND_ORDER"
run_make -j4 tools-check > "$fixture/aggregate-check.log" 2>&1
printf '%s\n' 'common install-host-tools.sh' 'common install-ic-tools.sh' 'common install-rust-tools.sh' 'server check' > "$fixture/expected-order"
cmp "$TESTKIT_COMMAND_ORDER" "$fixture/expected-order"
for target in install-tools tools-check; do
    : > "$TESTKIT_COMMAND_ORDER"
    if TESTKIT_COMMAND_COMMON_FAILURE=install-rust-tools.sh run_make -j4 "$target" > "$fixture/aggregate-$target-failure.log" 2>&1; then exit 1; fi
    printf '%s\n' 'common install-host-tools.sh' 'common install-ic-tools.sh' 'common install-rust-tools.sh' > "$fixture/expected-order"
    cmp "$TESTKIT_COMMAND_ORDER" "$fixture/expected-order"
done
[[ "$(cat "$fixture/retained")" == 'retained original evidence' ]]
printf '%s\n' 'Testkit setup/check routing, original selection and refusal fixtures passed'
