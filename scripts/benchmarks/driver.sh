#!/usr/bin/env bash
# Shared native benchmark build plumbing; callers retain cases and measurements.
build_benchmark_driver() {
    local root="$1" output="$2" compiler="$3" kind="$4" cache="$5"
    local features='"ecdsa"' platform testkit tokio dependency='' required=ic-testkit
    case "$kind" in
        bls|signature|codec) ;;
        ecdsa) required=ic-platform,ic-testkit ;;
        *) echo 'Expected bls, ecdsa, codec or signature benchmark driver' >&2; return 2 ;;
    esac
    mkdir -p "$output/driver/src"
    cp "$root/scripts/fixtures/cdk-runtime/${kind}_driver.rs" "$output/driver/src/main.rs"
    cp -R "$root/scripts/fixtures/cdk-runtime/pocketic_support" "$output/driver/src/"
    cp "$root/Cargo.lock" "$output/driver/Cargo.lock"
    platform="$(perl -MJSON::PP -e 'print encode_json($ARGV[0])' "$root/crates/ic-platform")"
    # shellcheck source=scripts/ci/workspace-dependencies.sh
    source "$root/scripts/ci/workspace-dependencies.sh"
    platform_dependencies_read "$root"
    if [[ "$kind" == ecdsa ]]; then
        dependency="ic-platform = { path = $platform, default-features = false, features = [$features] }"
    elif [[ "$kind" == codec ]]; then
        dependency="candid = { $(platform_dependency candid) }"
    fi
    testkit="$(platform_dependency ic-testkit)"
    tokio="$(platform_dependency tokio)"
    cat > "$output/driver/Cargo.toml" <<MANIFEST
[package]
name = "platform-$kind-benchmark"
version = "0.0.0"
edition = "2024"
rust-version = "$compiler"
publish = false
[package.metadata.toko-tooling.loc]
category = "benchmarks"
[workspace]
resolver = "3"
[dependencies]
$dependency
ic-testkit = { $testkit }
tokio = { $tokio }
MANIFEST
    platform_fixture_policy "$root" "$output/driver"
    cargo +"$compiler" metadata --manifest-path "$output/driver/Cargo.toml" --offline \
        --format-version 1 > "$output/driver/metadata.json"
    bash "$root/scripts/ci/check-qualification-identities.sh" "$output/driver/metadata.json" "$root" \
        "platform-$kind-benchmark" "$required" > "$output/driver/identities.json"
    CARGO_TARGET_DIR="$cache" cargo +"$compiler" clippy --manifest-path "$output/driver/Cargo.toml" \
        --frozen --all-targets -- -D warnings > "$output/driver/clippy.log" 2>&1
    # shellcheck source=scripts/ci/retain-build-artifact.sh
    source "$root/scripts/ci/retain-build-artifact.sh"
    driver="$output/driver/platform-$kind-benchmark"
    retain_build_artifact "$cache" "debug/platform-$kind-benchmark" "$driver" \
        cargo +"$compiler" build --manifest-path "$output/driver/Cargo.toml" \
        --frozen > "$output/driver/build.log" 2>&1
    shasum -a 256 "$driver" "$output/driver/src/main.rs" "$output/driver/Cargo.lock" > "$output/driver/sha256.txt"
}
