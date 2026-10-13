#!/usr/bin/env bash
set -euo pipefail

# Qualify each public library independently, without dev-feature unification.
# Preparation and checks are offline; never install a compiler or alter root locks.
[[ $# == 1 && "$1" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || {
    echo 'usage: check-msrv-consumers.sh <qualified-msrv>' >&2; exit 2;
}
msrv="$1"
ROOT="${BASH_SOURCE[0]}"
[[ "$ROOT" == /* ]] || ROOT="$PWD/$ROOT"
ROOT="$(cd -P "${ROOT%/*}/../.." && printf '%s/.' "$PWD")"
ROOT="${ROOT%/.}"
mkdir -p "$ROOT/target"
fixture="$(mktemp -d "${TMPDIR:-$ROOT/target}/msrv-consumers.XXXXXX")"
finish() {
    local status=$?
    printf 'MSRV consumer evidence retained: %s (status %s)\n' "$fixture" "$status"
    exit "$status"
}
trap finish EXIT
cp "$ROOT/Cargo.lock" "$fixture/original.lock"
export RUSTUP_AUTO_INSTALL=0
cargo "+$msrv" metadata --offline --locked --manifest-path "$ROOT/Cargo.toml" \
    --format-version 1 > "$fixture/workspace.json"
jq -e --arg floor "$msrv" '
    . as $metadata | [.packages[] | select(.id as $id | $metadata.workspace_members | index($id))]
    | all(.rust_version == $floor) or error("MSRV selection does not match the workspace packages")
' "$fixture/workspace.json" >/dev/null
for package in ic-backup ic-backup-agent; do
    consumer="$package-msrv-consumer"
    selected="$fixture/$package"
    mkdir -p "$selected/src"
    dependency="$(jq -nr --arg path "$ROOT/crates/$package" '$path | @json')"
    cat > "$selected/Cargo.toml" <<TOML
[workspace]
[package]
name = "$consumer"
version = "0.0.0"
edition = "2024"
rust-version = "$msrv"
[dependencies]
$package = { path = $dependency }
TOML
    if [[ "$package" == ic-backup ]]; then
        cat > "$selected/src/main.rs" <<'RUST'
use ic_backup::ops::persistence::{
    IcSnapshotLocalMetrics, MeasurementAggregate, MeasurementHistogram, MeasurementSummary,
};

fn main() {
    let diagnostics = IcSnapshotLocalMetrics::default();
    let summary: MeasurementSummary = diagnostics.verification_success_ns();
    let aggregate: MeasurementAggregate = diagnostics.prepared_chunk_bytes();
    let histogram: MeasurementHistogram<4> = diagnostics.prepared_chunk_bytes_histogram();
    let _: u128 = summary.total();
    let _: Option<u128> = summary.mean().unwrap();
    let _: u128 = aggregate.total();
    let _: Option<u128> = aggregate.mean().unwrap();
    let _: u128 = histogram.aggregate().total();
    let _: u64 = summary.samples();
    let _: Option<u64> = summary.latest();
    let _: Option<u64> = aggregate.maximum();
    let _: &[u64; 4] = histogram.bucket_counts();
}
RUST
    else
        printf 'use %s as _;\nfn main() {}\n' "${package//-/_}" > "$selected/src/main.rs"
    fi
    cp "$fixture/original.lock" "$selected/Cargo.lock"
    # Initial fixture preparation may add its root and drop unused packages.
    # Cargo metadata preserves the copied selections; generate-lockfile would
    # reselect compatible versions from the cache. Admission below checks this.
    cargo "+$msrv" metadata --offline --manifest-path "$selected/Cargo.toml" \
        --format-version 1 > "$selected/metadata.json" 2> "$selected/preparation.log"
    # A standalone lock may drop unused packages and add this fixture only.
    # Refuse any selected package/version/source absent from the original graph.
    jq -L "$ROOT/scripts/ci" -e --arg consumer "$consumer" --slurpfile original "$fixture/workspace.json" '
        include "consumer-graph";
        original_graph_admitted($original[0]; $consumer)
        or error("Independent consumer changed selected packages or retained simulator dependencies")
    ' "$selected/metadata.json" >/dev/null
    CARGO_TARGET_DIR="$ROOT/target" cargo "+$msrv" check --offline --locked \
        --manifest-path "$selected/Cargo.toml" --all-features > "$selected/check.log" 2>&1
    cat "$selected/check.log"
    cmp "$ROOT/Cargo.lock" "$fixture/original.lock"
    printf 'Independent MSRV consumer passed: %s on Rust %s\n' "$package" "$msrv"
done
