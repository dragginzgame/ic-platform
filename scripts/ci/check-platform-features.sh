#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
mkdir -p target
cargo tree -p ic-platform --locked --no-default-features --edges normal --prefix none > target/default-graph.txt
[[ "$(wc -l < target/default-graph.txt)" -eq 1 ]] || {
    echo 'Default Platform graph must have no dependencies' >&2; exit 1;
}
cargo tree -p ic-platform --locked --no-default-features --features executor --edges normal --prefix none > target/executor-graph.txt
if grep -Eq '^(candid|candid_derive|candid_parser) ' target/executor-graph.txt; then
    echo 'Executor graph must remain Candid-free' >&2; exit 1
fi
cargo check -p ic-platform --lib --locked --no-default-features
cargo check -p ic-platform --lib --locked --no-default-features --features executor --target wasm32-unknown-unknown
