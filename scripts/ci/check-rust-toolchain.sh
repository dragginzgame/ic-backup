#!/usr/bin/env bash
set -euo pipefail

# Local compiler policy and actual command admission; never installs a toolchain.
[[ $# == 0 ]] || { echo 'usage: check-rust-toolchain.sh' >&2; exit 2; }
ROOT="${BASH_SOURCE[0]}"
[[ "$ROOT" == /* ]] || ROOT="$PWD/$ROOT"
ROOT="$(cd -P "${ROOT%/*}/../.." && printf '%s/.' "$PWD")"
ROOT="${ROOT%/.}"
export PATH="$ROOT/.tools/host/bin:$PATH"
export RUSTUP_AUTO_INSTALL=0
fail() { printf 'compiler admission refused: %s\n' "$*" >&2; exit 1; }
mkdir -p "$ROOT/target"
evidence="$(mktemp -d "$ROOT/target/compiler-admission.XXXXXX")"
printf 'Compiler admission evidence retained: %s\n' "$evidence" >&2
for file in Cargo.toml rust-toolchain.toml; do
    [[ -f "$ROOT/$file" && ! -L "$ROOT/$file" ]] || fail "expected regular $file"
    cp "$ROOT/$file" "$evidence/$file"
done
read_policy() {
    local document
    document="$("${YQ:-yq}" -p toml -o json '.' "$evidence/$1")" || fail "cannot parse $1"
    printf '%s\n' "$document" | jq -ers \
        "if length == 1 then .[0] | $2 | select(type == \"string\") | select((contains(\"\\n\") or contains(\"\\r\")) | not) else error(\"expected one policy document\") end"
}
minimum="$(read_policy Cargo.toml '.workspace.package."rust-version"')" || fail 'missing workspace Rust floor'
pin="$(read_policy rust-toolchain.toml '.toolchain.channel')" || fail 'missing compiler pin'
[[ "$minimum" =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ && "$pin" == "$minimum" ]] ||
    fail 'workspace rust-version and exact development pin must match'
printf '%s\n' "$pin" > "$evidence/selection"
observe() {
    local name="$1" status=0 observed
    shift
    "$@" > "$evidence/$name.stdout" 2> "$evidence/$name.stderr" || status=$?
    printf '%s\n' "$status" > "$evidence/$name.status"
    cat "$evidence/$name.stdout" "$evidence/$name.stderr" >&2
    [[ "$status" == 0 ]] || fail "$name identity observation failed"
    observed="$(sed -n 's/^release: //p' "$evidence/$name.stdout")"
    [[ "$observed" == "$pin" ]] || fail "$name reports $observed; expected $pin"
}
observe rustc "${RUSTC:-rustc}" --version --verbose
observe cargo cargo --version --verbose
for file in Cargo.toml rust-toolchain.toml; do
    cmp "$ROOT/$file" "$evidence/$file" >&2 || fail 'compiler policy changed during admission'
done
printf 'admitted\n' > "$evidence/result"
printf '%s\n' "$pin"
