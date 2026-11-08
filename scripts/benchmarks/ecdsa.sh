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
# shellcheck source=scripts/benchmarks/driver.sh
source "$root/scripts/benchmarks/driver.sh"
build_benchmark_driver "$root" "$output" "$compiler" ecdsa "${ECDSA_BENCH_HOST_TARGET_DIR:-$output/target-driver}"
for recipe in ${WASM_BENCH_RECIPES:-3-fat-1 z-fat-1}; do
    "$driver" auto "$output/auto/$recipe-ecdsa-instructions.optimized.wasm" \
        32 "$output/32/$recipe-ecdsa-instructions.optimized.wasm" \
        > "$output/$recipe-instructions.tsv"
done
printf 'ECDSA backend evidence retained at %s\n' "$output"
