#!/usr/bin/env bash
set -euo pipefail
# Requires an explicitly prepared Testkit server or server URL; never installs it.
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
published=""
if [[ "${1:-}" == --published ]]; then
    [[ $# -ge 2 && "$2" =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]] || exit 2
    published="$2"
    shift 2
fi
mode="${1:-run}"
[[ $# -le 1 ]] || { echo "Expected [--published X.Y.Z] [mode]" >&2; exit 2; }
case "$mode" in run|--prepare|--check-graph|--build-only|--refresh-lock) ;; *) echo "Usage: $0 [--published X.Y.Z] [run|--prepare|--check-graph|--build-only|--refresh-lock]" >&2; exit 2 ;; esac
[[ -z "$published" || "$mode" != --refresh-lock ]] || {
    echo 'Published qualification cannot rewrite the maintained local lock' >&2; exit 2;
}
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
wire="$("$root/.tools/host/bin/yq" -p toml -o yaml -r '.workspace.dependencies.ic-wire' Cargo.toml)"
serde="$("$root/.tools/host/bin/yq" -p toml -o yaml -r '.workspace.dependencies.serde.version' Cargo.toml)"
cbor="$("$root/.tools/host/bin/yq" -p toml -o yaml -r '.workspace.dependencies.serde_cbor' Cargo.toml)"
selection='path = "../../crates/ic-platform"'
if [[ -n "$published" ]]; then selection="version = \"=$published\""; fi
cat > "$fixture/Cargo.toml" <<MANIFEST
[workspace]
members = ["crates/canister", "apps/driver"]
resolver = "3"
[workspace.package]
version = "0.0.0"
edition = "2024"
publish = false
[workspace.dependencies]
ic-platform = { $selection, default-features = false, features = ["cdk"] }
ic-testkit = "$testkit"
ic-timers = "$timers"
ic-wire = "$wire"
serde = { version = "$serde", default-features = false }
serde_cbor = "$cbor"
MANIFEST
if [[ -z "$published" ]]; then
    cat >> "$fixture/Cargo.toml" <<'MANIFEST'
[patch.crates-io]
ic-platform = { path = "../../crates/ic-platform" }
MANIFEST
fi
cat > "$fixture/crates/canister/Cargo.toml" <<'MANIFEST'
[package]
name = "platform-runtime-canister"
version.workspace = true
edition.workspace = true
publish.workspace = true
rust-version = "1.91"
[features]
external-candid = ["ic-platform/candid"]
[lib]
crate-type = ["cdylib"]
[dependencies]
ic-platform.workspace = true
ic-timers.workspace = true
ic-wire.workspace = true
serde.workspace = true
MANIFEST
cat > "$fixture/apps/driver/Cargo.toml" <<'MANIFEST'
[package]
name = "platform-runtime-driver"
version.workspace = true
edition.workspace = true
publish.workspace = true
rust-version = "1.91"
[dependencies]
ic-platform = { workspace = true, features = ["certification", "candid"] }
ic-testkit.workspace = true
serde_cbor.workspace = true
serde.workspace = true
ic-wire.workspace = true
MANIFEST
cp scripts/fixtures/cdk-runtime/canister.rs "$fixture/crates/canister/src/lib.rs"
cp -R scripts/fixtures/cdk-runtime/{wire_canister,wire_protocol,protocol,codec} "$fixture/crates/canister/src/"
cp scripts/fixtures/cdk-runtime/driver.rs "$fixture/apps/driver/src/main.rs"
cp -R scripts/fixtures/cdk-runtime/{wire_driver,wire_protocol,protocol} "$fixture/apps/driver/src/"
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
if [[ "$mode" == --prepare ]]; then
    [[ -z "$published" ]] || { echo "Prepare published caches explicitly before qualification" >&2; exit 2; }
    cargo fetch --manifest-path "$fixture/Cargo.toml" --locked
    exit 0
fi
if [[ -n "$published" ]]; then
    # Resolve only the disposable lock from explicitly prepared registry caches.
    cargo metadata --manifest-path "$fixture/Cargo.toml" --offline --filter-platform "$(rustc -vV | sed -n 's/^host: //p')" --format-version 1 > "$fixture/metadata.json"
    "$root/.tools/host/bin/jq" -e --arg version "$published" -f "$root/scripts/ci/published-runtime.jq" "$fixture/metadata.json" > /dev/null || {
        echo 'Published runtime requires the exact registry facade/generator and no local replacements' >&2
        exit 1
    }
fi
cargo metadata --manifest-path "$fixture/Cargo.toml" --locked --offline --filter-platform "$(rustc -vV | sed -n 's/^host: //p')" --format-version 1 > "$fixture/metadata.json"
for graph_target in "$(rustc -vV | sed -n 's/^host: //p')" wasm32-unknown-unknown; do
    for protocol in wire external-candid; do
        features=()
        admission=()
        if [[ "$protocol" == external-candid ]]; then features=(--features external-candid); admission=(--external-candid); fi
        cargo tree --manifest-path "$fixture/Cargo.toml" --locked -p platform-runtime-canister "${features[@]}" --target "$graph_target" --edges normal,build --prefix none > "$fixture/$graph_target-$protocol.graph"
        bash scripts/ci/check-platform-runtime-graph.sh "$fixture/$graph_target-$protocol.graph" "${admission[@]}"
    done
done
if [[ "$mode" == --check-graph ]]; then
    printf 'Runtime graph evidence retained at %s\n' "$fixture"
    exit 0
fi
CARGO_TARGET_DIR="$fixture/build/host" cargo +1.91.0 test --manifest-path "$fixture/Cargo.toml" --locked -p platform-runtime-driver
CARGO_TARGET_DIR="$fixture/build/host" cargo +1.91.0 build --manifest-path "$fixture/Cargo.toml" --locked -p platform-runtime-driver
for toolchain in 1.99.1 1.91.0; do
    for protocol in wire external-candid; do
        features=()
        if [[ "$protocol" == external-candid ]]; then features=(--features external-candid); fi
        CARGO_TARGET_DIR="$fixture/build/$toolchain" cargo +"$toolchain" build --manifest-path "$fixture/Cargo.toml" --locked -p platform-runtime-canister "${features[@]}" --release --target wasm32-unknown-unknown
        mkdir -p "$fixture/artifacts/$toolchain"
        cp "$fixture/build/$toolchain/wasm32-unknown-unknown/release/platform_runtime_canister.wasm" "$fixture/artifacts/$toolchain/$protocol.wasm"
        if [[ "$mode" == run ]]; then
            if [[ "$protocol" == external-candid ]]; then
                PLATFORM_EXTERNAL_CANDID=1 "$fixture/build/host/debug/platform-runtime-driver" "$fixture/artifacts/$toolchain/$protocol.wasm"
            else
                (
                    unset PLATFORM_EXTERNAL_CANDID
                    "$fixture/build/host/debug/platform-runtime-driver" "$fixture/artifacts/$toolchain/$protocol.wasm"
                )
            fi
        fi
    done
done
printf 'Runtime sources, lock and metadata retained at %s\n' "$fixture"
