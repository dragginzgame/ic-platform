#!/usr/bin/env bash
# Matched published-baseline/current-Platform qualification; never changes product sources.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
[[ $# == 0 ]] || { echo 'usage: bls-wrapper.sh (requires prepared POCKET_IC_BIN or IC_TESTKIT_POCKET_IC_URL)' >&2; exit 2; }
[[ -n "${POCKET_IC_BIN:-}${IC_TESTKIT_POCKET_IC_URL:-}" ]] || {
    echo 'Select an explicitly prepared simulator first' >&2; exit 2;
}
mkdir -p "$root/target/evidence"
output="$(mktemp -d "$root/target/evidence/bls-wrapper.XXXXXX")"
printf 'BLS wrapper evidence: %s\n' "$output"
manifest="$root/qualification/Cargo.toml"
compiler="$(bash "$root/scripts/ci/read-platform-msrv.sh")"
export CARGO_TARGET_DIR="$root/target/bls-wrapper-probe"
export CARGO_BUILD_JOBS=4
export CARGO_PROFILE_RELEASE_OPT_LEVEL=3 CARGO_PROFILE_RELEASE_LTO=fat
export CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_PROFILE_RELEASE_PANIC=abort
export CARGO_PROFILE_RELEASE_STRIP=symbols
CARGO_ENCODED_RUSTFLAGS="--remap-path-prefix=$root=/platform"
export CARGO_ENCODED_RUSTFLAGS
rustc +"$compiler" -vV > "$output/rustc.txt"
cargo +"$compiler" --version > "$output/cargo.txt"
ic-wasm --version > "$output/optimizer.txt"
printf '%s\n' 'wasm32-unknown-unknown; opt=3; lto=fat; units=1; panic=abort; strip=symbols; cpubits=auto; ic-wasm shrink' > "$output/recipe.txt"
find "$root/qualification" "$root/crates" -type f \( -name '*.rs' -o -name Cargo.toml -o -name Cargo.lock \) -print0 | xargs -0 shasum -a 256 > "$output/source-sha256.txt"
shasum -a 256 "$root/Cargo.toml" "$root/Cargo.lock" "$0" >> "$output/source-sha256.txt"
printf 'variant\traw_total\traw_code\traw_custom\toptimized_total\toptimized_code\toptimized_custom\n' > "$output/sizes.tsv"
for variant in baseline candidate; do
    cargo +"$compiler" tree --manifest-path "$manifest" --locked --offline -p bls-wrapper-probe \
        --no-default-features --features "$variant" --target wasm32-unknown-unknown \
        --edges normal,build --prefix none --format '{p} {f}' > "$output/$variant.graph.txt"
    cargo +"$compiler" clippy --manifest-path "$manifest" --locked --offline -p bls-wrapper-probe \
        --lib --no-default-features --features "$variant" --target wasm32-unknown-unknown \
        -- -D warnings > "$output/$variant.clippy.log" 2>&1
    cargo +"$compiler" build --manifest-path "$manifest" --locked --offline -p bls-wrapper-probe \
        --lib --no-default-features --features "$variant" --target wasm32-unknown-unknown \
        --release > "$output/$variant.build.log" 2>&1
    cp "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/bls_wrapper_probe.wasm" "$output/$variant.raw.wasm"
    ic-wasm "$output/$variant.raw.wasm" -o "$output/$variant.optimized.wasm" shrink > "$output/$variant.optimizer.log" 2>&1
    {
        printf '%s\t' "$variant"
        perl "$root/scripts/benchmarks/wasm-sections.pl" "$output/$variant.raw.wasm"
        printf '\t'
        perl "$root/scripts/benchmarks/wasm-sections.pl" "$output/$variant.optimized.wasm"
        printf '\n'
    } >> "$output/sizes.tsv"
done
if rg '^(ic-verify-bls-signature |sha2 v0\.10\.|hex |lazy_static )' "$output/candidate.graph.txt"; then
    echo 'Retired wrapper dependencies remain in candidate Wasm graph' >&2; exit 1
fi
cargo +"$compiler" clippy --manifest-path "$manifest" --locked --offline -p bls-wrapper-probe \
    --example measure --no-default-features -- -D warnings > "$output/driver.clippy.log" 2>&1
cargo +"$compiler" run --manifest-path "$manifest" --locked --offline -p bls-wrapper-probe \
    --example measure --no-default-features -- "$output/baseline.optimized.wasm" "$output/candidate.optimized.wasm" \
    > "$output/instructions.tsv" 2> "$output/driver.log"
shasum -a 256 --check "$output/source-sha256.txt" > "$output/source-validation.log"
shasum -a 256 "$output/"*.wasm > "$output/artifact-sha256.txt"
cat "$output/sizes.tsv"
printf 'BLS wrapper qualification retained: %s\n' "$output"
