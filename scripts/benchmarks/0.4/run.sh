#!/usr/bin/env bash
set -euo pipefail
bench="${BENCH_ROOT:?Set BENCH_ROOT to a prepared benchmark directory}"
printf 'case\trevision\trun\tseconds\tpeak_rss_kb\twasm_bytes\tcode_bytes\tcustom_bytes\tgzip_bytes\tpackages\n' > "$bench/results/results.tsv"
for run in 1 2 3; do
 revisions=(base candidate)
 if [[ "$run" == 2 ]]; then revisions=(candidate base); fi
 for case in wire external-candid creation verification; do
  for revision in "${revisions[@]}"; do
   selection="$case"
   features=()
   if [[ "$case" == wire || "$case" == external-candid ]]; then selection=runtime; fi
   if [[ "$case" == external-candid ]]; then features=(--features external-candid); fi
   fixture="$bench/fixtures/$revision-$selection"
   label="$case-$revision-$run"
   target="$bench/targets/$label"
   printf 'START %s\n' "$label" >> "$bench/results/progress.log"
   CARGO_BUILD_JOBS=4 CARGO_TARGET_DIR="$target" RUSTFLAGS="--remap-path-prefix=$bench/$revision=/src/platform --remap-path-prefix=$fixture=/src/fixture" \
    /usr/bin/time -f '%e\t%M' -o "$bench/results/$label.time" \
    cargo +1.99.1 build --manifest-path "$fixture/Cargo.toml" --frozen --release --target wasm32-unknown-unknown "${features[@]}" > "$bench/results/$label.build.log" 2>&1
   wasm="$target/wasm32-unknown-unknown/release/platform_benchmark.wasm"
   cp "$wasm" "$bench/results/$label.wasm"
   cargo +1.99.1 tree --manifest-path "$fixture/Cargo.toml" --frozen --target wasm32-unknown-unknown --edges normal,build --prefix none "${features[@]}" > "$bench/results/$label.graph"
   count=$(awk '$1 != "platform-benchmark" {print $1, $2}' "$bench/results/$label.graph" | LC_ALL=C sort -u | wc -l)
   printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$case" "$revision" "$run" "$(cat "$bench/results/$label.time")" "$(perl "$bench/sections.pl" "$wasm")" "$(gzip -cn < "$wasm" | wc -c)" "$count" >> "$bench/results/results.tsv"
   printf 'DONE %s\n' "$label" >> "$bench/results/progress.log"
  done
 done
done
