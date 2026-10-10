#!/usr/bin/env bash
set -euo pipefail

# Qualify this repository's actual simulator Wasm with explicitly prepared tools.
# Fixture owners retain both inputs and outputs; this helper never installs tools.
[[ $# == 1 && "$1" == /* && -d "$1" && -f "$1/original.wasm" && ! -e "$1/optimized.wasm" ]] || {
    echo 'usage: optimize-test-wasm.sh <absolute-existing-fixture-root>' >&2
    exit 2
}
ROOT="${BASH_SOURCE[0]}"
[[ "$ROOT" == /* ]] || ROOT="$PWD/$ROOT"
ROOT="$(cd -P "${ROOT%/*}/../.." && printf '%s/.' "$PWD")"
ROOT="${ROOT%/.}"
bin="$(bash "$ROOT/scripts/dev/install-ic-tools.sh" --check)"
umask 077
"$bin/wasm-opt" --version > "$1/optimizer.version"
bash "$ROOT/scripts/ci/verify-file-checksum.sh" --print sha256 "$bin/wasm-opt" > "$1/optimizer.sha256"
"$bin/wasm-opt" "$1/original.wasm" -O2 --all-features -o "$1/optimized.wasm" \
    > "$1/optimizer.stdout" 2> "$1/optimizer.stderr"
