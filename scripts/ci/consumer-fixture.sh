#!/usr/bin/env bash
# Common offline graph preparation for independent Cargo consumer fixtures.
# Callers own their temporary directory, manifest, API probe and extra admission.

consumer_fixture_cargo() {
    local rust_version="$1"
    shift
    if [[ -n "$rust_version" ]]; then
        cargo "+$rust_version" "$@"
    else
        cargo "$@"
    fi
}

consumer_fixture_snapshot() {
    local root="$1" fixture="$2" rust_version="$3"
    cp "$root/Cargo.lock" "$fixture/original.lock"
    consumer_fixture_cargo "$rust_version" metadata --offline --locked \
        --manifest-path "$root/Cargo.toml" --format-version 1 > "$fixture/original.json"
}

consumer_fixture_prepare() {
    local root="$1" fixture="$2" selected="$3" consumer="$4" rust_version="$5"
    cp "$fixture/original.lock" "$selected/Cargo.lock"
    # Preparation may prune unused packages and add only the fixture root.
    consumer_fixture_cargo "$rust_version" metadata --offline \
        --manifest-path "$selected/Cargo.toml" --format-version 1 \
        > "$selected/metadata.json" 2> "$selected/preparation.log"
    jq -L "$root/scripts/ci" -e --arg consumer "$consumer" \
        --slurpfile original "$fixture/original.json" '
        include "consumer-graph";
        original_graph_admitted($original[0]; $consumer)
        or error("Independent consumer changed selected packages or retained test/tooling dependencies")
    ' "$selected/metadata.json" >/dev/null
}
