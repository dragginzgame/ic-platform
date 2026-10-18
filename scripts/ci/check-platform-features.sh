#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
mkdir -p target
cargo tree -p ic-platform --locked --no-default-features --edges normal,build --prefix none > target/default-graph.txt
[[ "$(wc -l < target/default-graph.txt)" -eq 1 ]] || {
    echo 'Default Platform graph must have no dependencies' >&2; exit 1;
}
cargo tree -p ic-platform --locked --no-default-features --features executor --edges normal,build --prefix none > target/executor-graph.txt
if grep -Eq '^(candid|candid_derive|candid_parser) ' target/executor-graph.txt; then
    echo 'Executor graph must remain Candid-free' >&2; exit 1
fi
cargo tree -p ic-platform --locked --no-default-features --features executor --target wasm32-unknown-unknown --edges normal,build --prefix none > target/executor-wasm-graph.txt
if grep -Eq '^(candid|candid_derive|candid_parser) ' target/executor-wasm-graph.txt; then
    echo 'Wasm executor graph must remain Candid-free' >&2; exit 1
fi
cargo tree -p ic-platform --locked --all-features --edges normal,build --prefix none > target/all-features-graph.txt
if grep -Eq '^candid_parser ' target/all-features-graph.txt; then
    echo 'Runtime features must not select the host parser' >&2; exit 1
fi
cargo check -p ic-platform --lib --locked --no-default-features
cargo check -p ic-platform --lib --locked --no-default-features --features executor --target wasm32-unknown-unknown
