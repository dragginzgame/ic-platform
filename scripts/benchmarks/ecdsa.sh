#!/usr/bin/env bash
set -euo pipefail
# Available upstream backends only. Never patch cryptographic source or locks.
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
compiler="$(bash "$root/scripts/ci/read-platform-msrv.sh")"
[[ $# == 1 && ! -e "$1" ]] || { echo 'usage: ecdsa.sh NEW_EVIDENCE_DIRECTORY' >&2; exit 2; }
[[ -n "${POCKET_IC_BIN:-}${IC_TESTKIT_POCKET_IC_URL:-}" ]] || {
    echo 'Select an explicitly prepared PocketIC server through Testkit first' >&2; exit 1;
}
mkdir -p "$1"
output="$(cd "$1" && pwd -P)"
mkdir "$output/source"
cp "$root/Cargo.toml" "$root/Cargo.lock" "$output/source/"
cp -R "$root/crates" "$output/source/"
for backend in auto 32; do
    WASM_BENCH_PROFILES='ecdsa ecdsa-instructions' \
        WASM_BENCH_RECIPES="${WASM_BENCH_RECIPES:-3-fat-1 z-fat-1}" \
        WASM_BENCH_CPUBITS="$backend" \
        bash "$root/scripts/benchmarks/wasm-size.sh" "$output/source" "$output/$backend"
done
mkdir -p "$output/driver/src"
cp "$root/scripts/fixtures/cdk-runtime/ecdsa_driver.rs" "$output/driver/src/main.rs"
cp -R "$root/scripts/fixtures/cdk-runtime/pocketic_support" "$output/driver/src/"
cp "$root/Cargo.lock" "$output/driver/Cargo.lock"
platform="$(perl -MJSON::PP -e 'print encode_json($ARGV[0])' "$root/crates/ic-platform")"
testkit="$("$root/.tools/host/bin/yq" -p toml -o json '.workspace.dependencies.ic-testkit' "$root/Cargo.toml" | "$root/.tools/host/bin/jq" -r 'to_entries | map("\(.key) = \(.value | tojson)") | join(", ")')"
tokio="$("$root/.tools/host/bin/yq" -p toml -o json '.workspace.dependencies.tokio' "$root/Cargo.toml" | "$root/.tools/host/bin/jq" -r 'to_entries | map("\(.key) = \(.value | tojson)") | join(", ")')"
cat > "$output/driver/Cargo.toml" <<MANIFEST
[package]
name = "platform-ecdsa-benchmark"
version = "0.0.0"
edition = "2024"
rust-version = "$compiler"

[package.metadata.shared-tooling.loc]
category = "benchmarks"
[workspace]
resolver = "3"
[dependencies]
ic-platform = { path = $platform, default-features = false, features = ["ecdsa", "principal-external"] }
ic-testkit = { $testkit }
tokio = { $tokio }
MANIFEST
cargo +"$compiler" metadata --manifest-path "$output/driver/Cargo.toml" --offline \
    --format-version 1 > "$output/driver/metadata.json"
CARGO_TARGET_DIR="${ECDSA_BENCH_HOST_TARGET_DIR:-$output/target-driver}" \
    cargo +"$compiler" build --manifest-path "$output/driver/Cargo.toml" --frozen \
    > "$output/driver/build.log" 2>&1
driver="${ECDSA_BENCH_HOST_TARGET_DIR:-$output/target-driver}/debug/platform-ecdsa-benchmark"
shasum -a 256 "$driver" "$output/driver/src/main.rs" "$output/driver/Cargo.lock" > "$output/driver/sha256.txt"
for recipe in ${WASM_BENCH_RECIPES:-3-fat-1 z-fat-1}; do
    "$driver" auto "$output/auto/$recipe-ecdsa-instructions.optimized.wasm" \
        32 "$output/32/$recipe-ecdsa-instructions.optimized.wasm" \
        > "$output/$recipe-instructions.tsv"
done
printf 'ECDSA backend evidence retained at %s\n' "$output"
