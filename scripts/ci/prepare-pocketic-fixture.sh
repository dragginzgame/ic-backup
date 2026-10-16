#!/usr/bin/env bash
set -euo pipefail

# Prepare one caller-owned simulator fixture without installing tools or starting effects.
[[ $# == 1 && "$1" == /* && -d "$1" && -f "$1/original.wasm" && ! -e "$1/server.sha256" ]] || {
    echo 'usage: prepare-pocketic-fixture.sh <absolute-existing-fixture-root>' >&2
    exit 2
}
script_dir="$(cd -P "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -P "$script_dir/../.." && pwd)"
binary="$(make --silent --no-print-directory -C "$repo_root" testkit-server-check)"
[[ "$binary" == /* && -f "$binary" ]] || {
    echo 'Testkit must admit an absolute server path' >&2
    exit 1
}
umask 077
digest="$(bash "$repo_root/scripts/ci/verify-file-checksum.sh" --print sha256 "$binary")"
printf '%s' "$digest" > "$1/server.sha256"
printf '%s\n' "$binary"
