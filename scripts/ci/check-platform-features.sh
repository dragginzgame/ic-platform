#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
mkdir -p target/feature-graphs
cargo metadata --no-deps --format-version 1 --locked > target/feature-graphs/metadata.json
"$root/.tools/host/bin/jq" -e '.packages[] | select(.name == "ic-platform") | .features.default == []' \
    target/feature-graphs/metadata.json > /dev/null || {
    echo 'Platform must declare empty default features' >&2; exit 1;
}
# These are production package graphs, including build dependencies but not dev
# dependencies. Cargo tree prints canonical package names even for renamed deps.
for target in "$(rustc -vV | sed -n 's/^host: //p')" wasm32-unknown-unknown; do
    for selection in default none candid executor certification \
        candid,executor candid,certification executor,certification candid,executor,certification \
        candid-parser candid,candid-parser executor,candid-parser certification,candid-parser \
        candid,executor,candid-parser candid,certification,candid-parser \
        executor,certification,candid-parser candid,executor,certification,candid-parser; do
        graph="target/feature-graphs/$target-$selection.txt"
        args=(tree -p ic-platform --locked --target "$target" --edges normal,build --prefix none)
        if [[ "$selection" != default ]]; then args+=(--no-default-features); fi
        if [[ "$selection" != none && "$selection" != default ]]; then args+=(--features "$selection"); fi
        cargo "${args[@]}" > "$graph"
        parser=false
        if [[ "$target" != wasm* && "$selection" == *candid-parser* ]]; then parser=true; fi
        if grep -Eq '^(pocket-ic|ic-testkit|ic-testkit-candid) ' "$graph"; then
            echo "Production graph contains Testkit transport: $graph" >&2; exit 1
        fi
        if [[ "$parser" == false ]] && grep -Eq '^candid_parser ' "$graph"; then
            echo "Parser must remain native host opt-in only: $graph" >&2; exit 1
        fi
        if [[ "$selection" != *executor* ]] && grep -Eq '^ic-cdk[^ ]* ' "$graph"; then
            echo "Graph without executor must remain CDK-free: $graph" >&2; exit 1
        fi
        # The host parser itself enables upstream Candid's full type/value support.
        if [[ ",$selection," != *,candid,* && "$parser" == false ]] && grep -Eq '^(candid|candid_derive) ' "$graph"; then
            echo "Graph without candid must remain Candid-free: $graph" >&2; exit 1
        fi
        if [[ "$selection" != *certification* ]] && grep -Eq '^ic-certification ' "$graph"; then
            echo "Certification must remain optional: $graph" >&2; exit 1
        fi
    done
done
cargo check -p ic-platform --lib --locked --no-default-features
cargo check -p ic-platform --lib --locked --no-default-features --features executor --target wasm32-unknown-unknown
