#!/usr/bin/env bash
set -euo pipefail

# Qualify the actual archive pair before release versions change. Cargo's normal
# registry verification can select the already published core at the same version.
# Publication still uses Cargo's ordinary registry admission; this is local proof.
ROOT="${BASH_SOURCE[0]}"
[[ "$ROOT" == /* ]] || ROOT="$PWD/$ROOT"
ROOT="$(cd -P "${ROOT%/*}/../.." && printf '%s/.' "$PWD")"
ROOT="${ROOT%/.}"
source "$ROOT/scripts/ci/consumer-fixture.sh"
cd "$ROOT"
export CARGO_TARGET_DIR="$ROOT/target"
export PATH="$ROOT/.tools/host/bin:$PATH"
umask 077
mkdir -p "$ROOT/target"
fixture="$(mktemp -d "${TMPDIR:-$ROOT/target}/package-check.XXXXXX")"
complete=false
finish() {
    local status=$?
    [[ "$complete" == true || "$status" != 0 ]] || status=1
    printf 'Package archive/consumer evidence retained: %s (status %s)\n' "$fixture" "$status"
    exit "$status"
}
trap finish EXIT
cp Cargo.toml "$fixture/original.toml"
version="$(bash scripts/ci/read-cargo-workspace-version.sh Cargo.toml)"
consumer_fixture_snapshot "$ROOT" "$fixture" ""
# Cargo still performs archive/manifest/lockfile admission. Verification below
# is mandatory and compiles the unpacked archives, never the live source paths.
cargo package --offline --locked --allow-dirty --no-verify \
    --target-dir "$fixture/build" -p ic-backup -p ic-backup-agent
mkdir -p "$fixture/unpacked" "$fixture/consumer/src"
for package in ic-backup ic-backup-agent; do
    archive="$fixture/build/package/$package-$version.crate"
    digest="$(bash scripts/ci/verify-file-checksum.sh --print sha256 "$archive")"
    printf '%s  %s\n' "$digest" "$archive" >> "$fixture/archives.sha256"
    tar -xzf "$archive" -C "$fixture/unpacked"
done
find "$fixture/unpacked" -type f -print0 > "$fixture/unpacked-files"
while IFS= read -r -d '' file; do
    digest="$(bash scripts/ci/verify-file-checksum.sh --print sha256 "$file")"
    printf '%s  %s\n' "$digest" "$file" >> "$fixture/unpacked.sha256"
done < "$fixture/unpacked-files"
core="$(jq -nr --arg path "$fixture/unpacked/ic-backup-$version" '$path | @json')"
agent="$(jq -nr --arg path "$fixture/unpacked/ic-backup-agent-$version" '$path | @json')"
cat > "$fixture/consumer/Cargo.toml" <<TOML
[workspace]
[package]
name = "ic-backup-package-consumer"
version = "0.0.0"
edition = "2024"
[dependencies]
ic-backup = { path = $core }
ic-backup-agent = { path = $agent }
[patch.crates-io]
ic-backup = { path = $core }
TOML
printf 'use ic_backup as _;\nuse ic_backup_agent as _;\n' > "$fixture/consumer/src/lib.rs"
consumer_fixture_prepare "$ROOT" "$fixture" "$fixture/consumer" \
    ic-backup-package-consumer ""
jq -e --arg unpacked "$fixture/unpacked" --arg version "$version" '
    [.packages[] | select(.name == "ic-backup" or .name == "ic-backup-agent")]
    | length == 2 and all(.[];
        .source == null and .version == $version
        and .manifest_path == ($unpacked + "/" + .name + "-" + $version + "/Cargo.toml"))
    or error("Archive consumer changed the graph or selected a different package owner")
' "$fixture/consumer/metadata.json" >/dev/null
cargo build --offline --locked --manifest-path "$fixture/consumer/Cargo.toml" --all-features
bash scripts/ci/verify-evidence-checksums.sh "$fixture/archives.sha256" "$fixture/unpacked.sha256"
cmp Cargo.lock "$fixture/original.lock"
cmp Cargo.toml "$fixture/original.toml"
printf 'Both current package archives compile together on the original normal graph\n'
complete=true
