#!/usr/bin/env bash
# Shared native benchmark build plumbing; callers retain cases and measurements.
build_benchmark_driver() {
    local root="$1" output="$2" compiler="$3" kind="$4" cache="$5"
    local features='"principal-external"' platform testkit tokio
    case "$kind" in
        bls) ;;
        ecdsa) features='"ecdsa", "principal-external"' ;;
        *) echo 'Expected bls or ecdsa benchmark driver' >&2; return 2 ;;
    esac
    mkdir -p "$output/driver/src"
    cp "$root/scripts/fixtures/cdk-runtime/${kind}_driver.rs" "$output/driver/src/main.rs"
    cp -R "$root/scripts/fixtures/cdk-runtime/pocketic_support" "$output/driver/src/"
    cp "$root/Cargo.lock" "$output/driver/Cargo.lock"
    platform="$(perl -MJSON::PP -e 'print encode_json($ARGV[0])' "$root/crates/ic-platform")"
    testkit="$("$root/.tools/host/bin/yq" -p toml -o json '.workspace.dependencies.ic-testkit' "$root/Cargo.toml" | "$root/.tools/host/bin/jq" -r 'to_entries | map("\(.key) = \(.value | tojson)") | join(", ")')"
    tokio="$("$root/.tools/host/bin/yq" -p toml -o json '.workspace.dependencies.tokio' "$root/Cargo.toml" | "$root/.tools/host/bin/jq" -r 'to_entries | map("\(.key) = \(.value | tojson)") | join(", ")')"
    cat > "$output/driver/Cargo.toml" <<MANIFEST
[package]
name = "platform-$kind-benchmark"
version = "0.0.0"
edition = "2024"
rust-version = "$compiler"
publish = false
[package.metadata.shared-tooling.loc]
category = "benchmarks"
[workspace]
resolver = "3"
[dependencies]
ic-platform = { path = $platform, default-features = false, features = [$features] }
ic-testkit = { $testkit }
tokio = { $tokio }
MANIFEST
    cargo +"$compiler" metadata --manifest-path "$output/driver/Cargo.toml" --offline \
        --format-version 1 > "$output/driver/metadata.json"
    CARGO_TARGET_DIR="$cache" cargo +"$compiler" clippy --manifest-path "$output/driver/Cargo.toml" \
        --frozen --all-targets -- -D warnings > "$output/driver/clippy.log" 2>&1
    CARGO_TARGET_DIR="$cache" cargo +"$compiler" build --manifest-path "$output/driver/Cargo.toml" \
        --frozen > "$output/driver/build.log" 2>&1
    driver="$cache/debug/platform-$kind-benchmark"
    shasum -a 256 "$driver" "$output/driver/src/main.rs" "$output/driver/Cargo.lock" > "$output/driver/sha256.txt"
}
