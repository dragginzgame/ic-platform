#!/usr/bin/env bash
set -euo pipefail
bench="${BENCH_ROOT:?Set BENCH_ROOT to a prepared benchmark directory}"
mkdir -p "$bench/pre-principal"
for revision in pre-principal candidate; do
 fixture="$bench/fixtures/$revision-principal"
 mkdir -p "$fixture/src"
 cp "$bench/principal-source/main.rs" "$fixture/src/main.rs"
 cat > "$fixture/Cargo.toml" <<MANIFEST
[package]
name = "principal-benchmark"
version = "0.0.0"
edition = "2024"
[workspace]
[dependencies]
ic-platform = { path = "$bench/$revision/crates/ic-platform", default-features = false }
[profile.release]
opt-level = 3
lto = false
codegen-units = 16
panic = "abort"
MANIFEST
 cp "$bench/$revision/Cargo.lock" "$fixture/Cargo.lock"
 cargo +1.99.1 metadata --manifest-path "$fixture/Cargo.toml" --offline --filter-platform x86_64-unknown-linux-gnu --format-version 1 > "$fixture/metadata.json"
 CARGO_BUILD_JOBS=4 CARGO_TARGET_DIR="$bench/targets/$revision-principal" cargo +1.99.1 build --manifest-path "$fixture/Cargo.toml" --frozen --release > "$bench/results/$revision-principal.build.log" 2>&1
done
printf 'revision\trun\tlength\toperation\titerations\tnanoseconds\n' > "$bench/results/principal.tsv"
for run in 1 2 3 4 5; do
 revisions=(pre-principal candidate)
 if ((run % 2 == 0)); then revisions=(candidate pre-principal); fi
 for revision in "${revisions[@]}"; do
  "$bench/targets/$revision-principal/release/principal-benchmark" | awk -v revision="$revision" -v run="$run" 'BEGIN {OFS="\t"} {print revision,run,$0}' >> "$bench/results/principal.tsv"
 done
done
