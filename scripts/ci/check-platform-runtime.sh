#!/usr/bin/env bash
set -euo pipefail
# Requires an explicitly prepared Testkit server or server URL; never installs it.
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
mode="${1:-run}"
[[ $# -le 1 ]] || { echo "Expected at most one mode" >&2; exit 2; }
case "$mode" in run|--check-graph|--build-only|--refresh-lock) ;; *) echo "Usage: $0 [--check-graph|--build-only|--refresh-lock]" >&2; exit 2 ;; esac
if [[ "$mode" == run && -z "${POCKET_IC_BIN:-}" && -z "${IC_TESTKIT_POCKET_IC_URL:-}" ]]; then
    echo 'Set POCKET_IC_BIN (Testkit-admitted binary) or IC_TESTKIT_POCKET_IC_URL first' >&2
    exit 1
fi
mkdir -p target
fixture="$(mktemp -d "$root/target/cdk-runtime.XXXXXX")"
mkdir -p "$fixture/crates/canister/src/fixtures" "$fixture/apps/driver/src"
# Inherit the reviewed test dependency selections from the root catalog.
testkit="$("$root/.tools/host/bin/yq" -p toml -o yaml -r '.workspace.dependencies.ic-testkit' Cargo.toml)"
timers="$("$root/.tools/host/bin/yq" -p toml -o yaml -r '.workspace.dependencies.ic-timers' Cargo.toml)"
cbor="$("$root/.tools/host/bin/yq" -p toml -o yaml -r '.workspace.dependencies.serde_cbor' Cargo.toml)"
cat > "$fixture/Cargo.toml" <<MANIFEST
[workspace]
members = ["crates/canister", "apps/driver"]
resolver = "3"
[workspace.package]
version = "0.0.0"
edition = "2024"
publish = false
[workspace.dependencies]
ic-platform = { path = "../../crates/ic-platform", features = ["cdk"] }
ic-testkit = "$testkit"
ic-timers = "$timers"
serde_cbor = "$cbor"
[patch.crates-io]
ic-platform = { path = "../../crates/ic-platform" }
MANIFEST
cat > "$fixture/crates/canister/Cargo.toml" <<'MANIFEST'
[package]
name = "platform-runtime-canister"
version.workspace = true
edition.workspace = true
publish.workspace = true
rust-version = "1.91"
[lib]
crate-type = ["cdylib"]
[dependencies]
ic-platform.workspace = true
ic-timers.workspace = true
MANIFEST
cat > "$fixture/apps/driver/Cargo.toml" <<'MANIFEST'
[package]
name = "platform-runtime-driver"
version.workspace = true
edition.workspace = true
publish.workspace = true
rust-version = "1.91"
[dependencies]
ic-platform = { workspace = true, features = ["certification"] }
ic-testkit.workspace = true
serde_cbor.workspace = true
MANIFEST
sed '/VALUE.set(value);/a\
    ic_timers::initialize_runtime().unwrap();' crates/ic-platform/tests/cdk_consumer.rs > "$fixture/crates/canister/src/lib.rs"
cat scripts/fixtures/cdk-runtime/timers.rs >> "$fixture/crates/canister/src/lib.rs"
cp crates/ic-platform/tests/fixtures/cdk-service.did "$fixture/crates/canister/src/fixtures/"
cp scripts/fixtures/cdk-runtime/driver.rs "$fixture/apps/driver/src/main.rs"
lock="$root/scripts/fixtures/cdk-runtime/Cargo.lock"
if [[ "$mode" == --refresh-lock ]]; then
    # Explicitly rebase on the production lock, resolving the test-only additions.
    # Ordinary validation never resolves or rewrites the maintained selection.
    cp Cargo.lock "$fixture/Cargo.lock"
    cargo metadata --manifest-path "$fixture/Cargo.toml" --filter-platform "$(rustc -vV | sed -n 's/^host: //p')" --format-version 1 > "$fixture/metadata.json"
    cp "$fixture/Cargo.lock" "$lock"
    printf 'Review the refreshed qualification lock: %s\n' "$lock"
    exit 0
fi
cp "$lock" "$fixture/Cargo.lock"
cargo metadata --manifest-path "$fixture/Cargo.toml" --locked --offline --filter-platform "$(rustc -vV | sed -n 's/^host: //p')" --format-version 1 > "$fixture/metadata.json"
for graph_target in "$(rustc -vV | sed -n 's/^host: //p')" wasm32-unknown-unknown; do
    cargo tree --manifest-path "$fixture/Cargo.toml" --locked -p platform-runtime-canister --target "$graph_target" --edges normal,build --prefix none > "$fixture/$graph_target.graph"
    bash scripts/ci/check-platform-runtime-graph.sh "$fixture/$graph_target.graph"
done
if [[ "$mode" == --check-graph ]]; then
    printf 'Runtime graph evidence retained at %s\n' "$fixture"
    exit 0
fi
CARGO_TARGET_DIR="$fixture/build/host" cargo +1.91.0 test --manifest-path "$fixture/Cargo.toml" --locked -p platform-runtime-driver
CARGO_TARGET_DIR="$fixture/build/host" cargo +1.91.0 build --manifest-path "$fixture/Cargo.toml" --locked -p platform-runtime-driver
for toolchain in 1.99.0 1.91.0; do
    CARGO_TARGET_DIR="$fixture/build/$toolchain" cargo +"$toolchain" build --manifest-path "$fixture/Cargo.toml" --locked -p platform-runtime-canister --release --target wasm32-unknown-unknown
    if [[ "$mode" == run ]]; then
        "$fixture/build/host/debug/platform-runtime-driver" "$fixture/build/$toolchain/wasm32-unknown-unknown/release/platform_runtime_canister.wasm"
    fi
done
printf 'Runtime sources, lock and metadata retained at %s\n' "$fixture"
