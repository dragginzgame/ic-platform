#!/usr/bin/env bash
set -euo pipefail
# Historical comparison. Requires cached registry sources and Rust 1.99.1.
root="$(cd "$(dirname "$0")/../../.." && pwd -P)"
export BENCH_ROOT
BENCH_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/platform-benchmark.XXXXXX")"
mkdir -p "$BENCH_ROOT/results"
for pair in base:390644f candidate:9a661f7 pre-principal:f105386; do
    name="${pair%%:*}"
    revision="${pair#*:}"
    mkdir -p "$BENCH_ROOT/$name"
    git -C "$root" archive "$revision" | tar -x -C "$BENCH_ROOT/$name"
done
cp "$root/scripts/benchmarks/0.4/sections.pl" "$BENCH_ROOT/"
cp -R "$root/scripts/benchmarks/0.4/principal-source" "$BENCH_ROOT/"
printf 'Benchmark evidence: %s\n' "$BENCH_ROOT"
bash "$root/scripts/benchmarks/0.4/prepare.sh"
bash "$root/scripts/benchmarks/0.4/run.sh"
bash "$root/scripts/benchmarks/0.4/principal.sh"
perl "$root/scripts/benchmarks/0.4/summarize.pl" > "$BENCH_ROOT/results/summary.json"
