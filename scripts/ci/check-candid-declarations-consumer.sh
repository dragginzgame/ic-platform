#!/usr/bin/env bash
# Standalone declaration API: portable codec only, without oracle dependencies.
set -euo pipefail
[[ $# == 0 ]] || { echo 'usage: check-candid-declarations-consumer.sh' >&2; exit 2; }
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
# shellcheck source=scripts/ci/consumer-fixture.sh
source "$root/scripts/ci/consumer-fixture.sh"
consumer_fixture "$root" candid-declarations-consumer src
cat > "$fixture/Cargo.toml" <<'MANIFEST'
[workspace]
[package]
name = "candid-declarations-consumer-proof"
version = "0.0.0"
edition = "2024"
publish = false
[package.metadata.toko-tooling.loc]
category = "tests"
[dependencies]
ic-platform = { path = "../../crates/ic-platform", default-features = false, features = ["candid-codec"] }
MANIFEST
cp crates/ic-platform/tests/candid_declarations_consumer.rs "$fixture/src/lib.rs"
consumer_metadata "$root"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$root/target/candid-declarations-consumer-build}"
for target in "$(rustc -vV | sed -n 's/^host: //p')" wasm32-unknown-unknown; do
    cargo tree --manifest-path "$fixture/Cargo.toml" --locked --offline \
        --target "$target" --edges normal,build --prefix none > "$fixture/$target.graph"
    if rg '^(candid|candid_derive|candid_parser|ic-agent|ic0|ic-platform-runtime|ic-platform-macros|sha2|ic_bls12_381) v' "$fixture/$target.graph"; then
        echo 'Portable declarations acquired an unrelated dependency' >&2; exit 1
    fi
    cargo clippy --manifest-path "$fixture/Cargo.toml" --locked --offline \
        --target "$target" --lib -- -D warnings
done
cargo test --manifest-path "$fixture/Cargo.toml" --locked --offline
printf 'Standalone declaration consumer evidence: %s\n' "$fixture"
