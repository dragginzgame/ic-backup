#!/usr/bin/env bash
set -euo pipefail
ROOT="$0"
[[ "$ROOT" == /* ]] || ROOT="$PWD/$ROOT"
ROOT="$(cd -P "${ROOT%/*}/../.." && printf '%s/.' "$PWD")"
ROOT="${ROOT%/.}"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/pocketic-checks.XXXXXX")"
fixture="$(cd "$fixture" && pwd -P)"
trap 'if [[ $? == 0 ]]; then rm -rf "$fixture"; else printf "PocketIC checks fixture retained: %s\n" "$fixture" >&2; fi' EXIT
mkdir -p "$fixture/workspace/src" "$fixture/bin" "$fixture/tmp"
export TMPDIR="$fixture/tmp"
pins="$ROOT/ci/ic-tools.tsv"
version="$(awk -v tool=pocket-ic -f "$ROOT/scripts/ci/ic-tool-pins.awk" "$pins")"
alignment="$ROOT/scripts/ci/check-pocketic-alignment.sh"
binary="$ROOT/scripts/ci/check-pocketic-binary.sh"
manifest="$fixture/workspace/Cargo.toml"
cat > "$manifest" <<TOML
[workspace]
[package]
name = "pocket-ic"
version = "$version"
edition = "2021"
TOML
: > "$fixture/workspace/src/lib.rs"
# Real Cargo owns manifest/lock selection, including its refusal to update a lock.
# This local synthetic package needs neither registry sources nor compilation.
(
    cd "$fixture/workspace"
    CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 cargo generate-lockfile --offline --manifest-path "$manifest"
)
cp "$fixture/workspace/Cargo.lock" "$fixture/original.lock"
[[ "$(bash "$alignment" --manifest "$manifest" --pins "$pins")" == "$version" ]]
cmp "$fixture/original.lock" "$fixture/workspace/Cargo.lock"

# Real offline Cargo must select the same workspace under CDPATH and preserve
# literal directory bytes, including a leading dash, spaces and a final newline.
for directory in workspace '-workspace with spaces' $'workspace\n'; do
    if [[ "$directory" != workspace ]]; then
        cp -R "$fixture/workspace" "$fixture/$directory"
    fi
    selected="$(cd "$fixture" && CDPATH="$fixture" bash "$alignment" \
        --manifest "$directory/Cargo.toml" --pins "$pins")"
    [[ "$selected" == "$version" ]]
    cmp "$fixture/original.lock" "$fixture/$directory/Cargo.lock"
done
expect_failure() {
    if "$@" > "$fixture/refusal.out" 2> "$fixture/refusal.err"; then
        echo 'PocketIC check accepted invalid input' >&2; exit 1
    fi
    [[ ! -s "$fixture/refusal.out" ]]
}
sed 's/name = "pocket-ic"/name = "changed-client"/' "$manifest" > "$fixture/changed.toml"
cp "$fixture/changed.toml" "$manifest"
expect_failure bash "$alignment" --manifest "$manifest" --pins "$pins"
cmp "$fixture/original.lock" "$fixture/workspace/Cargo.lock"
rg -F 'PocketIC alignment evidence retained:' "$fixture/refusal.err" > /dev/null

# Controlled metadata covers unsupported identities and failed producers without
# creating or downloading multiple real registry versions.
export POCKETIC_CHECK_FIXTURE="$fixture"
cat > "$fixture/bin/cargo" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
echo called >> "$POCKETIC_CHECK_FIXTURE/cargo-calls"
[[ "$CARGO_NET_OFFLINE" == true && "$RUSTUP_AUTO_INSTALL" == 0 ]]
[[ "$PWD" == "$POCKETIC_CHECK_FIXTURE/workspace" ]]
[[ $# == 7 && "$1" == metadata && "$2" == --locked && "$3" == --offline && "$4" == --format-version && "$5" == 1 && "$6" == --manifest-path && "$7" == "$PWD/Cargo.toml" ]]
cat "$POCKETIC_CHECK_FIXTURE/metadata.json"
exit "${POCKETIC_CHECK_CARGO_STATUS:-0}"
SCRIPT
chmod +x "$fixture/bin/cargo"
export PATH="$fixture/bin:$PATH"
printf '{"version":1,"packages":[{"name":"pocket-ic","version":"%s"}]}\n' "$version" > "$fixture/good.json"
cp "$fixture/good.json" "$fixture/metadata.json"
[[ "$(bash "$alignment" --manifest "$manifest" --pins "$pins")" == "$version" ]]

# Losing the selected directory after normalization must stop before Cargo;
# otherwise metadata could accidentally run in the invoking repository.
POCKETIC_CHECK_REAL_MKTEMP="$(command -v mktemp)"
export POCKETIC_CHECK_REAL_MKTEMP
cat > "$fixture/bin/mktemp" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
"$POCKETIC_CHECK_REAL_MKTEMP" "$@"
mv "$POCKETIC_CHECK_FIXTURE/workspace" "$POCKETIC_CHECK_FIXTURE/unavailable"
SCRIPT
chmod +x "$fixture/bin/mktemp"
before="$(wc -l < "$fixture/cargo-calls")"
expect_failure bash "$alignment" --manifest "$manifest" --pins "$pins"
[[ "$(wc -l < "$fixture/cargo-calls")" == "$before" ]]
mv "$fixture/unavailable" "$fixture/workspace"
rm "$fixture/bin/mktemp"

POCKETIC_CHECK_CARGO_STATUS=7 expect_failure bash "$alignment" --manifest "$manifest" --pins "$pins"
for transform in \
    '.packages = []' \
    '.packages += .packages' \
    '.packages[0].version = "999.0.0"' \
    '.packages[0].version = "016.0.0"' \
    '.packages[0].version += "-rc.1"' \
    '.packages[0].version += "\n"' \
    '.packages[0].version = 16' \
    '.version = 2'; do
    jq "$transform" "$fixture/good.json" > "$fixture/metadata.json"
    expect_failure bash "$alignment" --manifest "$manifest" --pins "$pins"
done
cat "$fixture/good.json" "$fixture/good.json" > "$fixture/metadata.json"
expect_failure bash "$alignment" --manifest "$manifest" --pins "$pins"
printf 'malformed json\n' > "$fixture/metadata.json"
expect_failure bash "$alignment" --manifest "$manifest" --pins "$pins"
cp "$fixture/good.json" "$fixture/metadata.json"
before="$(wc -l < "$fixture/cargo-calls")"
for fault in missing duplicate inconsistent digest; do
    case "$fault" in
        missing) awk -F '\t' '$1 != "pocket-ic" || $3 != "darwin-arm64"' "$pins" ;;
        duplicate) cat "$pins"; awk -F '\t' '$1 == "pocket-ic" && $3 == "linux-x86_64"' "$pins" ;;
        inconsistent) awk -F '\t' 'BEGIN { OFS="\t" } $1 == "pocket-ic" && $3 == "darwin-arm64" { $2="999.0.0" } { print }' "$pins" ;;
        digest) awk -F '\t' 'BEGIN { OFS="\t" } $1 == "quill" { $4="invalid" } { print }' "$pins" ;;
    esac > "$fixture/bad-pins.tsv"
    expect_failure bash "$alignment" --manifest "$manifest" --pins "$fixture/bad-pins.tsv"
done
[[ "$(wc -l < "$fixture/cargo-calls")" == "$before" ]]

# Authenticate override bytes before execution, regardless of version output.
cat > "$fixture/-pocket ic" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail
[[ $# == 1 && "$1" == --version ]]
echo executed >> "$POCKETIC_CHECK_FIXTURE/executions"
printf 'pocket-ic-server %s\n' "$POCKETIC_CHECK_VERSION"
exit "${POCKETIC_CHECK_PROBE_STATUS:-0}"
SCRIPT
chmod +x "$fixture/-pocket ic"
export POCKETIC_CHECK_VERSION="$version"
digest="$(bash "$ROOT/scripts/ci/verify-file-checksum.sh" --print sha256 "$fixture/-pocket ic")"
expect_failure bash "$binary" "$version" ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff "$fixture/-pocket ic"
[[ ! -e "$fixture/executions" ]]
expect_failure bash "$binary" "$version" invalid "$fixture/-pocket ic"
[[ ! -e "$fixture/executions" ]]
(
    cd "$fixture"
    bash "$binary" "$version" "$digest" '-pocket ic'
)
ln -s "$fixture/-pocket ic" "$fixture/server-link"
bash "$binary" "$version" "$digest" "$fixture/server-link"
[[ "$(bash "$alignment" --manifest "$manifest" --pins "$pins" --bin "$fixture/server-link" --sha256 "$digest")" == "$version" ]]
POCKETIC_CHECK_PROBE_STATUS=9 expect_failure bash "$binary" "$version" "$digest" "$fixture/server-link"
POCKETIC_CHECK_VERSION=999.0.0 expect_failure bash "$binary" "$version" "$digest" "$fixture/server-link"
before="$(wc -l < "$fixture/executions")"
expect_failure bash "$alignment" --manifest "$manifest" --pins "$pins" --bin "$fixture/server-link"
expect_failure bash "$alignment" --manifest "$manifest" --pins "$pins" --sha256 "$digest"
expect_failure bash "$binary" "$version" "$digest" "$fixture/missing"
chmod -x "$fixture/-pocket ic"
expect_failure bash "$binary" "$version" "$digest" "$fixture/server-link"
[[ "$(wc -l < "$fixture/executions")" == "$before" ]]
echo 'PocketIC locked alignment and authenticated binary checks passed'
