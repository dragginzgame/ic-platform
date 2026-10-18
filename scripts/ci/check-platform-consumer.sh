#!/usr/bin/env bash
set -euo pipefail
# A separate Cargo package cannot accidentally use Platform's private dependencies.
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
mkdir -p target
for capability in candid certification candid-parser agent management ic0 icrc; do
features="\"$capability\""
if [[ "$capability" == agent ]]; then features='"agent", "certification"'; fi
if [[ "$capability" == management ]]; then features='"management-canister-types", "candid"'; fi
if [[ "$capability" == icrc ]]; then features='"icrc-ledger-types", "candid"'; fi
fixture="$(mktemp -d "$root/target/boundary-consumer.XXXXXX")"
mkdir -p "$fixture/crates/consumer/src"
cat > "$fixture/Cargo.toml" <<MANIFEST
[workspace]
members = ["crates/consumer"]
resolver = "3"
[workspace.package]
version = "0.1.0"
edition = "2024"
[workspace.dependencies]
ic-platform = { path = "../../crates/ic-platform", default-features = false, features = [$features] }
MANIFEST
cat > "$fixture/crates/consumer/Cargo.toml" <<'MANIFEST'
[package]
name = "platform-consumer-proof"
version.workspace = true
edition.workspace = true
publish = false
[dependencies]
ic-platform.workspace = true
MANIFEST
source="$capability"
if [[ "$capability" == candid-parser ]]; then source=parser; fi
cp "crates/ic-platform/tests/${source}_consumer.rs" "$fixture/crates/consumer/src/lib.rs"
if [[ "$capability" == management ]]; then
    mkdir -p "$fixture/crates/consumer/src/fixtures"
    cp crates/ic-platform/tests/fixtures/management-info-*.hex "$fixture/crates/consumer/src/fixtures/"
fi
if [[ "$capability" == icrc ]]; then
    mkdir -p "$fixture/crates/consumer/src/fixtures"
    cp crates/ic-platform/tests/fixtures/icrc-*.hex "$fixture/crates/consumer/src/fixtures/"
fi
cp Cargo.lock "$fixture/Cargo.lock"
# Only the disposable local package identity is added to an already selected graph.
# Filter metadata to the selected host: unfiltered offline metadata otherwise
# requires cached source for foreign targets (for example windows-link on macOS).
cargo metadata --manifest-path "$fixture/Cargo.toml" --offline --filter-platform "$(rustc -vV | sed -n 's/^host: //p')" --format-version 1 > "$fixture/metadata.json"
CARGO_TARGET_DIR="$root/target/consumer-build" cargo test --manifest-path "$fixture/Cargo.toml" --locked
CARGO_TARGET_DIR="$root/target/consumer-msrv" cargo +1.88.0 test --manifest-path "$fixture/Cargo.toml" --locked
if [[ "$capability" != candid-parser && "$capability" != agent ]]; then
    CARGO_TARGET_DIR="$root/target/consumer-build" cargo check --manifest-path "$fixture/Cargo.toml" --lib --target wasm32-unknown-unknown --locked
fi
printf 'Consumer evidence retained at %s\n' "$fixture"
done
