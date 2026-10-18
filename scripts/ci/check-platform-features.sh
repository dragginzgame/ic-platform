#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
mkdir -p target/feature-graphs
# These are production package graphs, including build dependencies but not dev
# dependencies. Cargo tree prints canonical package names even for renamed deps.
for target in "$(rustc -vV | sed -n 's/^host: //p')" wasm32-unknown-unknown; do
    for selection in none candid executor candid,executor; do
        graph="target/feature-graphs/$target-$selection.txt"
        args=(tree -p ic-platform --locked --no-default-features --target "$target" --edges normal,build --prefix none)
        if [[ "$selection" != none ]]; then args+=(--features "$selection"); fi
        cargo "${args[@]}" > "$graph"
        if grep -Eq '^(candid_parser|pocket-ic|ic-testkit|ic-testkit-candid) ' "$graph"; then
            echo "Production graph contains host parsing or Testkit transport: $graph" >&2; exit 1
        fi
        if [[ "$selection" == none && "$(wc -l < "$graph")" -ne 1 ]]; then
            echo "Default Platform graph must have no dependencies: $graph" >&2; exit 1
        fi
        if [[ "$selection" == executor ]] && grep -Eq '^(candid|candid_derive) ' "$graph"; then
            echo "Executor graph must remain Candid-free: $graph" >&2; exit 1
        fi
    done
done
cargo check -p ic-platform --lib --locked --no-default-features
cargo check -p ic-platform --lib --locked --no-default-features --features executor --target wasm32-unknown-unknown
