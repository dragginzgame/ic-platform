#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
fixture="$(mktemp -d "$root/target/type-benchmark.XXXXXX")"
mkdir -p "$fixture/src"
cp "$root/scripts/fixtures/type-benchmark/main.rs" "$fixture/src/main.rs"
cat > "$fixture/Cargo.toml" <<'MANIFEST'
[package]
name = "platform-type-benchmark"
version = "0.0.0"
edition = "2024"
[workspace]
[dependencies]
ic-platform = { path = "../../crates/ic-platform", default-features = false, features = ["icrc-ledger-types"] }
[profile.release]
opt-level = 3
lto = false
codegen-units = 16
panic = "abort"
MANIFEST
cp "$root/Cargo.lock" "$fixture/Cargo.lock"
cargo metadata --manifest-path "$fixture/Cargo.toml" --offline --format-version 1 > "$fixture/metadata.json"
CARGO_BUILD_JOBS=4 cargo run --manifest-path "$fixture/Cargo.toml" --release --frozen > "$fixture/results.tsv"
printf 'Type size and Account timing evidence: %s\n' "$fixture"
