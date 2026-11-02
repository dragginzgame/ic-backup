#!/usr/bin/env bash
# Shared companions: scripts/ci/check-rust-toolchain.sh scripts/ci/prepare-script-fixture.sh
set -euo pipefail
ROOT="${BASH_SOURCE[0]}"
[[ "$ROOT" == /* ]] || ROOT="$PWD/$ROOT"
ROOT="$(cd -P "${ROOT%/*}/../.." && printf '%s/.' "$PWD")"
ROOT="${ROOT%/.}"
export PATH="$ROOT/.tools/host/bin:$PATH"
mkdir -p "$ROOT/target"
fixture="$(mktemp -d "$ROOT/target/compiler-tests.XXXXXX")"
printf 'Compiler policy fixtures retained: %s\n' "$fixture"
mkdir -p "$fixture/source/scripts/ci" "$fixture/bin"
bash "$ROOT/scripts/ci/prepare-script-fixture.sh" "$ROOT" "$fixture/source" \
    scripts/ci/check-rust-toolchain.sh Cargo.toml rust-toolchain.toml
export COMPILER_FIXTURE_VERSION COMPILER_FIXTURE_SOURCE="$fixture/source"
COMPILER_FIXTURE_VERSION="$(yq -p toml -o json '.toolchain.channel' "$ROOT/rust-toolchain.toml" | jq -er '.')"
for tool in cargo rustc; do
    cat > "$fixture/bin/$tool" <<'TOOL'
#!/usr/bin/env bash
set -euo pipefail
name="${0##*/}"
printf '%s\n' "$name" >> "$COMPILER_FIXTURE_LOG"
if [[ "$name" == rustc && "${COMPILER_FIXTURE_CHANGE:-0}" == 1 ]]; then
    printf '[toolchain]\nchannel = "stable"\n' > "$COMPILER_FIXTURE_SOURCE/rust-toolchain.toml"
fi
printf '%s %s\nrelease: %s\n' "$name" "${COMPILER_FIXTURE_REPORTED:-$COMPILER_FIXTURE_VERSION}" \
    "${COMPILER_FIXTURE_REPORTED:-$COMPILER_FIXTURE_VERSION}"
[[ "${COMPILER_FIXTURE_FAIL:-}" != "$name" ]] || exit 9
TOOL
    chmod +x "$fixture/bin/$tool"
done
export PATH="$fixture/bin:$PATH" RUSTC="$fixture/bin/rustc"
for fault in valid mismatch rustc-failure cargo-failure pin moving changed newline parser-failure; do
    cp "$ROOT/Cargo.toml" "$fixture/source/Cargo.toml"
    cp "$ROOT/rust-toolchain.toml" "$fixture/source/rust-toolchain.toml"
    unset COMPILER_FIXTURE_REPORTED COMPILER_FIXTURE_FAIL COMPILER_FIXTURE_CHANGE YQ
    export COMPILER_FIXTURE_LOG="$fixture/$fault.commands"
    : > "$COMPILER_FIXTURE_LOG"
    case "$fault" in
        mismatch) export COMPILER_FIXTURE_REPORTED=0.0.0 ;;
        rustc-failure) export COMPILER_FIXTURE_FAIL=rustc ;;
        cargo-failure) export COMPILER_FIXTURE_FAIL=cargo ;;
        pin) printf '[toolchain]\nchannel = "0.0.0"\n' > "$fixture/source/rust-toolchain.toml" ;;
        moving) printf '[toolchain]\nchannel = "stable"\n' > "$fixture/source/rust-toolchain.toml" ;;
        changed) export COMPILER_FIXTURE_CHANGE=1 ;;
        newline)
            printf '%s\n' '[workspace.package]' "rust-version = \"$COMPILER_FIXTURE_VERSION\\n\"" > "$fixture/source/Cargo.toml" ;;
        parser-failure)
            cat > "$fixture/bin/failed-parser" <<'PARSER'
#!/usr/bin/env bash
printf '{"workspace":{"package":{"rust-version":"%s"}},"toolchain":{"channel":"%s"}}\n' \
    "$COMPILER_FIXTURE_VERSION" "$COMPILER_FIXTURE_VERSION"
exit 9
PARSER
            chmod +x "$fixture/bin/failed-parser"
            export YQ="$fixture/bin/failed-parser" ;;
    esac
    status=0
    bash "$fixture/source/scripts/ci/check-rust-toolchain.sh" \
        > "$fixture/$fault.stdout" 2> "$fixture/$fault.stderr" || status=$?
    if [[ "$fault" == valid ]]; then
        [[ "$status" == 0 && "$(cat "$fixture/$fault.stdout")" == "$COMPILER_FIXTURE_VERSION" ]] || exit 1
        printf 'rustc\ncargo\n' > "$fixture/expected-commands"
        cmp "$COMPILER_FIXTURE_LOG" "$fixture/expected-commands"
    else
        [[ "$status" != 0 && ! -s "$fixture/$fault.stdout" ]] || exit 1
        grep -F 'compiler admission refused' "$fixture/$fault.stderr" >/dev/null
    fi
    printf '%s\n' "$fault" >> "$fixture/cases.txt"
done
echo 'Compiler policy and command identity admission: PASS.'
