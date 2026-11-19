#!/usr/bin/env bash
# Explicit setup only. Testkit owns archive/hash/decompression/publication admission.
set -euo pipefail
umask 077
directory="$1"
runner=(cargo run --manifest-path qualification/Cargo.toml -p ic-backup-qualification --example server --locked --quiet --)
if [[ -e "$directory/pocket-ic" || -L "$directory/pocket-ic" ]]; then
    "${runner[@]}" check "$directory"
    exit
fi
mkdir -p qualification/target/testkit-setup
attempt="$(mktemp -d qualification/target/testkit-setup/attempt.XXXXXX)"
printf 'Testkit preparation evidence: %s\n' "$attempt"
url="$("${runner[@]}" asset)"
curl --fail --show-error --location --proto '=https' --proto-redir '=https' \
    --max-time 120 --max-filesize 67108864 --output "$attempt/archive.gz" \
    "$url" 2> "$attempt/download.stderr" || {
    status=$?
    perl -Ivendor/toko-tooling/scripts/ci -MSetupDiagnostics=excerpt \
        -e 'print STDERR excerpt(\*STDIN, 8192)' < "$attempt/download.stderr" || true
    exit "$status"
}
mkdir -p "$directory"
gzip_path="$(command -v gzip)"
[[ "$gzip_path" == /* ]] || { echo 'prepare an absolute gzip executable on PATH' >&2; exit 1; }
"${runner[@]}" prepare "$attempt/archive.gz" "$directory" "$gzip_path"
