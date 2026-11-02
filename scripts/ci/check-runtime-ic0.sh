#!/usr/bin/env bash
set -euo pipefail
# Real external ownership and public API proof, with no executor or application types.
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
published=""
if [[ "${1:-}" == --published ]]; then
    [[ $# == 2 && "$2" =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]] || exit 2
    published="$2"
elif [[ $# != 0 ]]; then
    echo 'usage: check-runtime-ic0.sh [--published X.Y.Z]' >&2; exit 2
fi
source scripts/ci/consumer-fixture.sh
consumer_fixture "$root" runtime-ic0 src
trap 'printf "IC0-only consumer evidence: %s\n" "$fixture"' EXIT
selection='path = "../../crates/ic-platform-runtime"'
if [[ -n "$published" ]]; then selection="version = \"=$published\""; fi
msrv="$(bash scripts/ci/read-platform-msrv.sh)"
cat > "$fixture/Cargo.toml" <<MANIFEST
[workspace]
[package]
name = "runtime-ic0-consumer"
version = "0.0.0"
edition = "2024"
rust-version = "$msrv"
publish = false
[dependencies]
ic-platform-runtime = { $selection, default-features = false, features = ["ic0"] }
MANIFEST
cp scripts/fixtures/ic0-consumer/lib.rs "$fixture/src/lib.rs"
bash scripts/ci/check-platform-compiler.sh > "$fixture/compiler.txt"
consumer_metadata "$root"
if [[ -n "$published" ]]; then
    jq -e --arg version "$published" '
        [.packages[] | select(.name == "ic-platform-runtime")] |
        length == 1 and .[0].version == $version and (.[0].source | startswith("registry+"))
    ' "$fixture/metadata.json" > /dev/null
fi
for graph_target in "$(rustc -vV | sed -n 's/^host: //p')" wasm32-unknown-unknown; do
    graph="$fixture/$graph_target.graph"
    cargo tree --manifest-path "$fixture/Cargo.toml" --frozen --target "$graph_target" \
        --edges normal,build --prefix none > "$graph"
    # The owner has zero active dependencies under ic0. Refuse any hidden addition.
    printf '%s\n' ic-platform-runtime runtime-ic0-consumer > "$fixture/expected"
    awk '{print $1}' "$graph" | LC_ALL=C sort -u > "$fixture/actual"
    diff -u "$fixture/expected" "$fixture/actual"
    CARGO_TARGET_DIR="$root/target/consumer-build" cargo clippy \
        --manifest-path "$fixture/Cargo.toml" --frozen --lib --target "$graph_target" -- -D warnings
done
printf '%s\n' 'Public stable IO compiles on native/Wasm with only the dependency-free IC0 owner.'
