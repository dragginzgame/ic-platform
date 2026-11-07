#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
[[ -v POCKET_IC_BIN || -v IC_TESTKIT_POCKET_IC_URL ]] || {
    echo 'Set a Testkit-admitted POCKET_IC_BIN or IC_TESTKIT_POCKET_IC_URL' >&2; exit 1;
}
fixture="$(mktemp -d "$root/target/agent-installed.XXXXXX")"
trap 'printf "Agent installed evidence: %s\n" "$fixture"' EXIT
mkdir "$fixture/src"
cp "$root/scripts/fixtures/agent-installed/main.rs" "$fixture/src/"
cp "$root/scripts/fixtures/agent-installed/"{prime256v1,secp256k1}.pem "$fixture/src/"
testkit="$("$root/.tools/host/bin/yq" -p toml -o json '.workspace.dependencies.ic-testkit' "$root/Cargo.toml" | "$root/.tools/host/bin/jq" -r 'to_entries | map("\(.key) = \(.value | tojson)") | join(", ")')"
tokio="$("$root/.tools/host/bin/yq" -p toml -o json '.workspace.dependencies.tokio' "$root/Cargo.toml" | "$root/.tools/host/bin/jq" -r 'to_entries | map("\(.key) = \(.value | tojson)") | join(", ")')"
cat > "$fixture/Cargo.toml" <<MANIFEST
[workspace]
[package]
name = "agent-installed-proof"
version = "0.0.0"
edition = "2024"
rust-version = "1.99.1"
publish = false

[package.metadata.shared-tooling.loc]
category = "tests"
[dependencies]
ic-platform = { path = "$root/crates/ic-platform", default-features = false, features = ["agent"] }
ic-testkit = { $testkit }
tokio = { $tokio }
wat = "=1.263.0"
MANIFEST
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$root/target/agent-installed-build}"
cargo clippy --offline --manifest-path "$fixture/Cargo.toml" --all-targets -- -D warnings
cargo run --offline --locked --manifest-path "$fixture/Cargo.toml"
