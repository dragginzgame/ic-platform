#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
[[ -v POCKET_IC_BIN || -v IC_TESTKIT_POCKET_IC_URL ]] || {
    echo 'Set a Testkit-admitted POCKET_IC_BIN or IC_TESTKIT_POCKET_IC_URL' >&2; exit 1;
}
msrv="$(bash "$root/scripts/ci/read-platform-msrv.sh")"
mkdir -p "$root/target"
fixture="$(mktemp -d "$root/target/agent-installed.XXXXXX")"
trap 'printf "Agent installed evidence: %s\n" "$fixture"' EXIT
mkdir "$fixture/src"
cp "$root/scripts/fixtures/cdk-runtime/agent_driver.rs" "$fixture/src/main.rs"
cp -R "$root/scripts/fixtures/cdk-runtime/pocketic_support" "$fixture/src/"
cp "$root/scripts/fixtures/agent-installed/"{prime256v1,secp256k1}.pem "$fixture/src/"
# shellcheck source=scripts/ci/workspace-dependencies.sh
source "$root/scripts/ci/workspace-dependencies.sh"
platform_dependencies_read "$root"
testkit="$(platform_dependency ic-testkit)"
tokio="$(platform_dependency tokio)"
cat > "$fixture/Cargo.toml" <<MANIFEST
[workspace]
[package]
name = "agent-installed-proof"
version = "0.0.0"
edition = "2024"
rust-version = "$msrv"
publish = false

[package.metadata.toko-tooling.loc]
category = "tests"
[dependencies]
ic-platform = { path = "$root/crates/ic-platform", default-features = false, features = ["agent"] }
ic-testkit = { $testkit }
tokio = { $tokio }
wat = "=1.263.0"
MANIFEST
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$root/target/agent-installed-build}"
bash "$root/scripts/ci/check-platform-compiler.sh" > "$fixture/compiler.txt"
cargo +"$msrv" metadata --offline --manifest-path "$fixture/Cargo.toml" --format-version 1 > "$fixture/metadata.json"
bash "$root/scripts/ci/check-qualification-identities.sh" "$fixture/metadata.json" "$root" \
    agent-installed-proof ic-platform,ic-testkit > "$fixture/identities.json"
cargo +"$msrv" clippy --offline --locked --manifest-path "$fixture/Cargo.toml" --all-targets -- -D warnings
cargo +"$msrv" run --offline --locked --manifest-path "$fixture/Cargo.toml"
