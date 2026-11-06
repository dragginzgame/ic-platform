#!/usr/bin/env bash
# Explicit, serial comparison against a committed baseline; never changes source locks.
set -euo pipefail
[[ $# -ge 1 && $# -le 2 ]] || { echo 'usage: check-wasm-size-regression.sh BASE_REF [MAX_GROWTH_BYTES]' >&2; exit 2; }
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
allowance="${2:-128}"
node -e 'const value = process.argv[1]; if (!/^\d+$/.test(value) || !Number.isSafeInteger(Number(value))) { console.error("MAX_GROWTH_BYTES must be a nonnegative safe integer"); process.exit(2); }' -- "$allowance"
baseline="$(git -C "$root" rev-parse --verify --end-of-options "$1^{commit}")"
mkdir -p "$root/target/evidence"
output="$(mktemp -d "$root/target/evidence/wasm-regression.XXXXXX")"
mkdir "$output/source"
git -C "$root" archive "$baseline" crates Cargo.toml Cargo.lock | tar -x -C "$output/source"
# Isolate implementation effects from the user's pending dependency resolution.
cp "$root/Cargo.lock" "$output/source/Cargo.lock"
printf '%s\n' "$baseline" > "$output/baseline-commit"
export WASM_BENCH_PROFILES="${WASM_BENCH_PROFILES:-principal-binary runtime external-candid verification certification-cbor}"
export WASM_BENCH_RECIPES="${WASM_BENCH_RECIPES:-3-fat-1}"
export WASM_BENCH_TOOLCHAIN="$(bash "$root/scripts/ci/read-platform-msrv.sh")"
printf 'Wasm regression evidence: %s\n' "$output"
bash "$root/scripts/benchmarks/wasm-size.sh" "$output/source" "$output/before"
bash "$root/scripts/benchmarks/wasm-size.sh" "$root" "$output/after"
node "$root/scripts/benchmarks/compare-wasm-sizes.mjs" "$output/before" "$output/after" "$allowance" | tee "$output/deltas.tsv"
