#!/usr/bin/env bash
# Explicit, serial comparison against a committed baseline; never changes source locks.
set -euo pipefail
[[ $# -ge 1 && $# -le 3 ]] || { echo 'usage: check-wasm-size-regression.sh BASE_REF [MAX_GROWTH_BYTES] [--allow-dependency-changes]' >&2; exit 2; }
mode="${3:-}"
[[ -z "$mode" || "$mode" == --allow-dependency-changes ]] || { echo 'Unknown dependency comparison mode' >&2; exit 2; }
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
allowance="${2:-128}"
node -e 'const value = process.argv[1]; if (!/^\d+$/.test(value) || !Number.isSafeInteger(Number(value))) { console.error("MAX_GROWTH_BYTES must be a nonnegative safe integer"); process.exit(2); }' -- "$allowance"
baseline="$(git -C "$root" rev-parse --verify --end-of-options "$1^{commit}")"
mkdir -p "$root/target/evidence"
output="$(mktemp -d "$root/target/evidence/wasm-regression.XXXXXX")"
mkdir "$output/source"
git -C "$root" archive "$baseline" crates Cargo.toml Cargo.lock | tar -x -C "$output/source"
# Strict comparisons share the current selection. Deployment comparisons retain
# each source's own lock; dependency effects are part of the measured change.
if [[ -z "$mode" ]]; then cp "$root/Cargo.lock" "$output/source/Cargo.lock"; fi
printf '%s\n' "$baseline" > "$output/baseline-commit"
export WASM_BENCH_PROFILES="${WASM_BENCH_PROFILES:-principal-binary principal-self-auth runtime verification certification-cbor}"
export WASM_BENCH_RECIPES="${WASM_BENCH_RECIPES:-3-fat-1}"
WASM_BENCH_TOOLCHAIN="$(bash "$root/scripts/ci/read-platform-msrv.sh")"
export WASM_BENCH_TOOLCHAIN
printf 'Wasm regression evidence: %s\n' "$output"
bash "$root/scripts/benchmarks/wasm-size.sh" "$output/source" "$output/before"
bash "$root/scripts/benchmarks/wasm-size.sh" "$root" "$output/after"
comparison_args=("$output/before" "$output/after" "$allowance")
if [[ -n "$mode" ]]; then comparison_args+=("$mode"); fi
node "$root/scripts/benchmarks/compare-wasm-sizes.mjs" "${comparison_args[@]}" | tee "$output/deltas.tsv"
