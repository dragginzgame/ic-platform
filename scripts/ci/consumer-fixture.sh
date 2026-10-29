#!/usr/bin/env bash
# Sourced by independent consumer proofs; retain disposable sources and evidence.
consumer_fixture() {
    local consumer_root="$1"
    mkdir -p "$consumer_root/target"
    fixture="$(mktemp -d "$consumer_root/target/$2.XXXXXX")"
    mkdir -p "$fixture/$3"
}
consumer_metadata() {
    local consumer_root="$1"
    cp "$consumer_root/Cargo.lock" "$fixture/Cargo.lock"
    # Only the disposable lock resolves; foreign host sources need not be cached.
    cargo metadata --manifest-path "$fixture/Cargo.toml" --offline \
        --filter-platform "$(rustc -vV | sed -n 's/^host: //p')" \
        --format-version 1 > "$fixture/metadata.json"
}
