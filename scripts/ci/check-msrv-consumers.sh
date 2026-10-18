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
source "$ROOT/scripts/ci/consumer-fixture.sh"
mkdir -p "$ROOT/target"
fixture="$(mktemp -d "${TMPDIR:-$ROOT/target}/msrv-consumers.XXXXXX")"
finish() {
    local status=$?
    printf 'MSRV consumer evidence retained: %s (status %s)\n' "$fixture" "$status"
    exit "$status"
}
trap finish EXIT
export RUSTUP_AUTO_INSTALL=0
consumer_fixture_snapshot "$ROOT" "$fixture" "$msrv"
jq -e --arg floor "$msrv" '
    . as $metadata | [.packages[] | select(.id as $id | $metadata.workspace_members | index($id))]
    | all(.rust_version == $floor) or error("MSRV selection does not match the workspace packages")
' "$fixture/original.json" >/dev/null
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
        host_fs_requirement="$(jq -er '
            .packages[] | select(.name == "ic-backup") | .dependencies[]
            | select(.name == "ic-host-fs" and .kind == null) | .req | @json
        ' "$fixture/original.json")"
        printf 'ic-host-fs = { version = %s, default-features = false }\n' \
            "$host_fs_requirement" >> "$selected/Cargo.toml"
        cat > "$selected/src/main.rs" <<'RUST'
use ic_backup::{
    model::{
        ic_mutation::{IcMutationAcknowledgement, IcMutationAcknowledgementError},
        ic_snapshot_transfer_read::{IcSnapshotTransferReadError, IcSnapshotTransferReadResponse},
        ic_update_response::IcUpdateResponseInput,
    },
    ops::persistence::PersistenceError,
};
use ic_host_fs::durable::NamedWriteError;
use std::io;

fn main() {
    let _: fn(IcUpdateResponseInput) -> Result<IcMutationAcknowledgement, IcMutationAcknowledgementError>
        = IcMutationAcknowledgement::new;
    let _: fn(IcUpdateResponseInput) -> Result<IcSnapshotTransferReadResponse, IcSnapshotTransferReadError>
        = IcSnapshotTransferReadResponse::new;
    let publication: NamedWriteError<io::Error> = NamedWriteError::AfterPublication {
        source: io::Error::other("completion uncertain"),
    };
    let error = PersistenceError::from(publication);
    match error {
        PersistenceError::Publication(publication) => {
            let _: NamedWriteError<io::Error> = publication;
        }
        _ => panic!("publication errors retain Host's canonical type"),
    }
}
RUST
    else
        printf 'use %s as _;\nfn main() {}\n' "${package//-/_}" > "$selected/src/main.rs"
    fi
    consumer_fixture_prepare "$ROOT" "$fixture" "$selected" "$consumer" "$msrv"
    if [[ "$package" == ic-backup ]]; then
        jq -e 'all(.packages[]; .name != "ic-agent")
            or error("Core consumer unexpectedly enables the native Agent boundary")' \
            "$selected/metadata.json" > /dev/null
    fi
    CARGO_TARGET_DIR="$ROOT/target" cargo "+$msrv" check --offline --locked \
        --manifest-path "$selected/Cargo.toml" --all-features > "$selected/check.log" 2>&1
    cat "$selected/check.log"
    cmp "$ROOT/Cargo.lock" "$fixture/original.lock"
    printf 'Independent MSRV consumer passed: %s on Rust %s\n' "$package" "$msrv"
done
