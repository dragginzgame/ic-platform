#!/usr/bin/env bash
set -euo pipefail
# Historical registry feature experiment; production now uses the maintained wrapper.
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
compiler="$(bash "$root/scripts/ci/read-platform-msrv.sh")"
[[ $# == 1 && ! -e "$1" ]] || { echo 'usage: bls.sh NEW_EVIDENCE_DIRECTORY' >&2; exit 2; }
[[ -n "${POCKET_IC_BIN:-}${IC_TESTKIT_POCKET_IC_URL:-}" ]] || {
    echo 'Select an explicitly prepared PocketIC server through Testkit first' >&2; exit 1;
}
mkdir -p "$1"
output="$(cd "$1" && pwd -P)"
recipes="${WASM_BENCH_RECIPES:-3-fat-1 z-fat-1}"
mkdir "$output/source"
cp "$root/Cargo.toml" "$root/Cargo.lock" "$output/source/"
cp -R "$root/crates" "$output/source/"
for variant in alloc noalloc; do
    source="$output/$variant-source"
    mkdir "$source"
    cp "$output/source/Cargo.toml" "$output/source/Cargo.lock" "$source/"
    cp -R "$output/source/crates" "$source/"
    # Select the original registry implementation only in this disposable source.
    # Retain the production import and all protected source files untouched.
    BLS_EXPERIMENT_ALLOC="$variant" perl -0777 -i -pe '
        my $features = $ENV{BLS_EXPERIMENT_ALLOC} eq "alloc" ? q{, features = ["alloc"]} : "";
        s/^(ic_principal = )/ic-verify-bls-signature = { version = "=0.6.0", default-features = false$features }\n$1/m == 1
            or die "missing root dependency anchor\n";
    ' "$source/Cargo.toml"
    perl -0777 -i -pe '
        s/^bls = \[[^\n]+\]/bls = ["dep:ic-verify-bls-signature"]/m == 1 or die "missing BLS feature\n";
        s/^(ic_principal = )/ic-verify-bls-signature = { workspace = true, optional = true }\n$1/m == 1
            or die "missing member dependency anchor\n";
    ' "$source/crates/ic-platform/Cargo.toml"
    cat > "$source/crates/ic-platform/src/bls/mod.rs" <<'RUST'
//! Historical upstream allocation-feature experiment, not production source.
pub use ic_verify_bls_signature::{
    InvalidPrivateKey, InvalidPublicKey, InvalidSignature, PrivateKey, PublicKey, Signature,
    verify_bls_signature,
};
RUST
    WASM_BENCH_PROFILES='bls verification' WASM_BENCH_RECIPES="$recipes" \
        bash "$root/scripts/benchmarks/wasm-size.sh" "$source" "$output/$variant"
done
mkdir -p "$output/driver/src"
cp "$root/scripts/fixtures/bls-benchmark/main.rs" "$output/driver/src/main.rs"
cp "$root/Cargo.lock" "$output/driver/Cargo.lock"
platform="$(perl -MJSON::PP -e 'print encode_json($ARGV[0])' "$root/crates/ic-platform")"
testkit="$("$root/.tools/host/bin/yq" -p toml -r '.workspace.dependencies.ic-testkit' "$root/Cargo.toml")"
cat > "$output/driver/Cargo.toml" <<MANIFEST
[package]
name = "platform-bls-benchmark"
version = "0.0.0"
edition = "2024"
rust-version = "$compiler"
[workspace]
resolver = "3"
[dependencies]
ic-platform = { path = $platform, default-features = false, features = ["pocket-ic"] }
ic-testkit = "$testkit"
MANIFEST
cargo +"$compiler" metadata --manifest-path "$output/driver/Cargo.toml" --offline \
    --format-version 1 > "$output/driver/metadata.json"
CARGO_TARGET_DIR="${BLS_BENCH_HOST_TARGET_DIR:-$output/target-driver}" \
    cargo +"$compiler" build --manifest-path "$output/driver/Cargo.toml" --frozen \
    > "$output/driver/build.log" 2>&1
driver="${BLS_BENCH_HOST_TARGET_DIR:-$output/target-driver}/debug/platform-bls-benchmark"
shasum -a 256 "$driver" "$output/driver/src/main.rs" "$output/driver/Cargo.lock" > "$output/driver/sha256.txt"
for recipe in $recipes; do
    "$driver" "$root/crates/ic-platform/tests/fixtures" \
        alloc "$output/alloc/$recipe-bls.optimized.wasm" \
        noalloc "$output/noalloc/$recipe-bls.optimized.wasm" \
        > "$output/$recipe-instructions.tsv"
done
printf 'BLS size/instruction evidence retained at %s\n' "$output"
