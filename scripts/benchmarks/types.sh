#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
mkdir -p "$root/target/evidence"
fixture="$(mktemp -d "$root/target/evidence/type-benchmark.XXXXXX")"
mkdir -p "$fixture/src"
cp "$root/scripts/fixtures/type-benchmark/main.rs" "$fixture/src/main.rs"
cat > "$fixture/Cargo.toml" <<'MANIFEST'
[package]
name = "platform-type-benchmark"
version = "0.0.0"
edition = "2024"

[package.metadata.toko-tooling.loc]
category = "benchmarks"
[workspace]
[dependencies]
ic-platform = { path = "../../../crates/ic-platform", default-features = false, features = ["icrc-ledger-types"] }
[profile.release]
opt-level = 3
lto = false
codegen-units = 16
panic = "abort"
MANIFEST
cp "$root/Cargo.lock" "$fixture/Cargo.lock"
# shellcheck source=scripts/ci/workspace-dependencies.sh
source "$root/scripts/ci/workspace-dependencies.sh"
platform_fixture_policy "$root" "$fixture"
cargo metadata --manifest-path "$fixture/Cargo.toml" --offline --format-version 1 > "$fixture/metadata.json"
bash "$root/vendor/toko-tooling/scripts/ci/check-cargo-dependency-boundaries.sh" --consumer "$fixture" --external-ic
CARGO_BUILD_JOBS=4 cargo run --manifest-path "$fixture/Cargo.toml" --release --frozen > "$fixture/results.tsv"
printf 'Type size and Account timing evidence: %s\n' "$fixture"
