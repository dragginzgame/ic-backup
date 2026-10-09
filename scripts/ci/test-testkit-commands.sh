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
cp "$ROOT/scripts/ci/check-make-execution.sh" "$fixture/consumer/scripts/ci/"
cat > "$fixture/bin/cargo" <<'CARGO'
#!/usr/bin/env bash
set -euo pipefail
[[ "$*" == 'metadata --offline --locked --format-version 1' ]]
cat "$TESTKIT_COMMAND_METADATA"
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
    exec "$TESTKIT_COMMAND_BASH" "$@"
fi
BASH
} > "$fixture/bin/bash"
cat > "$fixture/cli/ic-testkit-server" <<'CLI'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >> "$TESTKIT_COMMAND_CALLS"
[[ ${TESTKIT_COMMAND_SERVER_FAILURE:-0} == 0 ]] || exit 23
printf '%s\n' "$TESTKIT_COMMAND_SERVER"
CLI
chmod +x "$fixture/bin/cargo" "$fixture/bin/bash" "$fixture/cli/ic-testkit-server"
export TESTKIT_COMMAND_METADATA="$fixture/metadata.json"
export TESTKIT_COMMAND_INSTALL="$fixture/install.log"
export TESTKIT_COMMAND_CALLS="$fixture/calls.log"
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
[[ "$(cat "$fixture/retained")" == 'retained original evidence' ]]
printf '%s\n' 'Testkit setup/check routing, original selection and refusal fixtures passed'
