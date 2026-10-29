#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
msrv="$(bash "$root/scripts/ci/read-platform-msrv.sh")"
mkdir -p target
fixture="$(mktemp -d "$root/target/ic-call-consumer.XXXXXX")"
mkdir -p "$fixture/src"
wire="$("$root/.tools/host/bin/yq" -p toml -o yaml -r '.workspace.dependencies.ic-wire' Cargo.toml)"
cat > "$fixture/Cargo.toml" <<MANIFEST
[workspace]
[package]
name = "ic-call-consumer-proof"
version = "0.0.0"
edition = "2024"
rust-version = "$msrv"
publish = false
[lints.rust]
warnings = "deny"
[features]
default = []
wire = ["dep:ic-wire"]
external-candid = ["dep:ic-platform"]
[dependencies]
ic-call = { path = "../../crates/ic-call", default-features = false }
ic-platform = { path = "../../crates/ic-platform", default-features = false, features = ["candid"], optional = true }
ic-wire = { version = "$wire", optional = true }
MANIFEST
cp scripts/fixtures/ic-call-consumer/lib.rs "$fixture/src/lib.rs"
cp Cargo.lock "$fixture/Cargo.lock"
# Only this disposable standalone package is resolved from prepared caches.
cargo metadata --manifest-path "$fixture/Cargo.toml" --offline --filter-platform "$(rustc -vV | sed -n 's/^host: //p')" --format-version 1 > "$fixture/metadata.json"
for profile in raw wire external-candid; do
    features=()
    admission=()
    if [[ "$profile" != raw ]]; then features=(--features "$profile"); fi
    if [[ "$profile" == external-candid ]]; then admission=(--external-candid); fi
    for target in "$(rustc -vV | sed -n 's/^host: //p')" wasm32-unknown-unknown; do
        cargo tree --manifest-path "$fixture/Cargo.toml" --frozen ${features[@]+"${features[@]}"} --target "$target" --edges normal,build --prefix none > "$fixture/$profile-$target.graph"
        bash scripts/ci/check-platform-runtime-graph.sh "$fixture/$profile-$target.graph" ${admission[@]+"${admission[@]}"}
        if grep -Eq '^(ic-timers|ic-relay|ic-platform-macros) ' "$fixture/$profile-$target.graph"; then
            echo 'Prepared calls acquired unrelated timer, relay or entrypoint machinery' >&2; exit 1
        fi
    done
    # Strict standalone Clippy precedes all tests of each actual selection.
    cargo clippy --manifest-path "$fixture/Cargo.toml" --frozen --all-targets ${features[@]+"${features[@]}"} -- -D warnings
    for compiler in "$("$root/.tools/host/bin/yq" -p toml -o yaml -r '.toolchain.channel' rust-toolchain.toml)" "$msrv"; do
        cargo +"$compiler" test --manifest-path "$fixture/Cargo.toml" --frozen ${features[@]+"${features[@]}"}
        cargo +"$compiler" check --manifest-path "$fixture/Cargo.toml" --frozen ${features[@]+"${features[@]}"} --target wasm32-unknown-unknown
    done
done
printf 'Standalone ic-call consumer evidence retained at %s\n' "$fixture"
