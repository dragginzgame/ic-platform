#!/usr/bin/env bash
# Focused installed management qualification; Testkit owns the prepared server.
set -euo pipefail
[[ $# == 0 ]] || exit 2
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
[[ -n "${POCKET_IC_BIN:-}" || -n "${IC_TESTKIT_POCKET_IC_URL:-}" ]] || {
    echo 'Set POCKET_IC_BIN (Testkit-admitted binary) or IC_TESTKIT_POCKET_IC_URL first' >&2
    exit 1
}
mkdir -p "$root/target"
fixture="$(mktemp -d "$root/target/pocket-ic-client.XXXXXX")"
trap 'printf "PocketIC client evidence: %s\n" "$fixture"' EXIT
mkdir "$fixture/src"
testkit="$("$root/.tools/host/bin/yq" -p toml -o json -r '.workspace.dependencies.ic-testkit' "$root/Cargo.toml")"
cat > "$fixture/Cargo.toml" <<MANIFEST
[workspace]
[package]
name = "pocket-ic-owner-proof"
version = "0.0.0"
edition = "2024"
[dependencies]
ic-platform-pocket-ic = { path = "../../crates/ic-platform-pocket-ic" }
ic-testkit = "$testkit"
MANIFEST
cp "$root/scripts/fixtures/pocket-ic-client/main.rs" "$fixture/src/main.rs"
# Reuse prepared qualification selections, pruning only this disposable lock.
cp "$root/scripts/fixtures/cdk-runtime/Cargo.lock" "$fixture/Cargo.lock"
cargo metadata --offline --manifest-path "$fixture/Cargo.toml" --format-version 1 > "$fixture/metadata.json"
bash "$root/scripts/ci/check-platform-compiler.sh" > "$fixture/compiler.txt"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$root/target/pocket-ic-client-build}"
cargo clippy --offline --locked --manifest-path "$fixture/Cargo.toml" --all-targets -- -D warnings
cargo run --offline --locked --manifest-path "$fixture/Cargo.toml"
