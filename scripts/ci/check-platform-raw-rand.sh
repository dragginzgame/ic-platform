#!/usr/bin/env bash
# One focused raw_rand consumer proof; no sibling edits, installs or root lock writes.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
mode="${1:-run}"
[[ $# -le 1 ]] || exit 2
case "$mode" in run|--check-graph|--build-only) ;; *) echo 'Expected [--check-graph|--build-only]' >&2; exit 2 ;; esac
if [[ "$mode" == run && -z "${POCKET_IC_BIN:-}" && -z "${IC_TESTKIT_POCKET_IC_URL:-}" ]]; then
    echo 'Set an explicitly prepared POCKET_IC_BIN or IC_TESTKIT_POCKET_IC_URL first' >&2
    exit 1
fi
mkdir -p target/evidence
output="$(mktemp -d "$root/target/evidence/raw-rand.XXXXXX")"
finish() {
    local result=$?
    trap - EXIT
    perl "$root/scripts/ci/runtime-inputs.pl" verify "$root" "$output" Cargo.lock --raw-rand > "$output/source-validation.log" 2>&1 || result=1
    printf '%s\n' "$result" > "$output/status"
    printf 'raw_rand evidence: %s\n' "$output"
    exit "$result"
}
trap finish EXIT
perl "$root/scripts/ci/runtime-inputs.pl" capture "$root" "$output" Cargo.lock --raw-rand
input_root="$output/inputs"
mkdir -p "$input_root/.tools"
ln -s "$root/.tools/host" "$input_root/.tools/host"
compiler="$(bash "$input_root/scripts/ci/read-platform-msrv.sh" "$input_root/Cargo.toml")"
# shellcheck source=scripts/ci/workspace-dependencies.sh
source "$input_root/scripts/ci/workspace-dependencies.sh"
platform_dependencies_read "$input_root"
testkit="$(platform_dependency ic-testkit)"
tokio="$(platform_dependency tokio)"
mkdir -p "$output/consumer/canister/src" "$output/consumer/driver/src"
cp "$input_root/scripts/fixtures/raw-rand/canister/mod.rs" "$output/consumer/canister/src/lib.rs"
cp "$input_root/scripts/fixtures/cdk-runtime/raw_rand_driver.rs" "$output/consumer/driver/src/main.rs"
cp -R "$input_root/scripts/fixtures/cdk-runtime/pocketic_support" "$output/consumer/driver/src/"
cp "$input_root/Cargo.lock" "$output/consumer/Cargo.lock"
cat > "$output/consumer/Cargo.toml" <<MANIFEST
[workspace]
members = ["canister", "driver"]
resolver = "3"
[workspace.package]
edition = "2024"
rust-version = "$compiler"
[workspace.dependencies]
ic-platform = { path = "$input_root/crates/ic-platform", default-features = false }
ic-platform-runtime = { path = "$input_root/crates/ic-platform-runtime", default-features = false }
ic-testkit = { $testkit }
tokio = { $tokio }
MANIFEST
cat > "$output/consumer/canister/Cargo.toml" <<'MANIFEST'
[package]
name = "platform-raw-rand-canister"
version = "0.0.0"
edition.workspace = true
rust-version.workspace = true
[package.metadata.toko-tooling.loc]
category = "tests"
[lib]
crate-type = ["cdylib"]
[features]
reference-codec = ["ic-platform/candid-codec"]
[dependencies]
ic-platform = { workspace = true, features = ["raw-rand"] }
ic-platform-runtime = { workspace = true, features = ["runtime"] }
MANIFEST
cat > "$output/consumer/driver/Cargo.toml" <<'MANIFEST'
[package]
name = "platform-raw-rand-driver"
version = "0.0.0"
edition.workspace = true
rust-version.workspace = true
[package.metadata.toko-tooling.loc]
category = "tests"
[dependencies]
ic-platform = { workspace = true, features = ["raw-rand", "candid-codec", "principal"] }
ic-testkit.workspace = true
tokio.workspace = true
MANIFEST
cargo +"$compiler" metadata --manifest-path "$output/consumer/Cargo.toml" --offline --format-version 1 > "$output/metadata.json" 2> "$output/metadata.log"
for package in platform-raw-rand-canister platform-raw-rand-driver; do
    required=ic-platform,ic-platform-runtime
    if [[ "$package" == platform-raw-rand-driver ]]; then required=ic-platform,ic-testkit; fi
    bash "$input_root/scripts/ci/check-qualification-identities.sh" "$output/metadata.json" "$input_root" \
        "$package" "$required" > "$output/$package.identities.json"
done
for target in "$(rustc +"$compiler" -vV | sed -n 's/^host: //p')" wasm32-unknown-unknown; do
    cargo +"$compiler" tree --manifest-path "$output/consumer/Cargo.toml" --frozen --offline -p platform-raw-rand-canister \
        --no-default-features --target "$target" --edges normal,build --prefix none > "$output/$target.graph"
    bash "$input_root/scripts/ci/check-platform-runtime-graph.sh" "$output/$target.graph"
    if grep -Eq '^(candid(_derive|_parser)?|ic-management-canister-types|ic-agent) ' "$output/$target.graph"; then
        echo 'raw_rand consumer acquired a general Candid/native helper' >&2; exit 1
    fi
done
[[ "$mode" != --check-graph ]] || exit 0
cache="${CARGO_TARGET_DIR:-$root/target/runtime-build}"
rustc +"$compiler" -vV > "$output/compiler.txt"
ic-wasm --version > "$output/optimizer-version.txt"
# shellcheck source=scripts/ci/retain-build-artifact.sh
source "$input_root/scripts/ci/retain-build-artifact.sh"
printf 'codec\traw_total\traw_code\traw_custom\toptimized_total\toptimized_code\toptimized_custom\n' > "$output/sizes.tsv"
for codec in reference narrow; do
    features=()
    if [[ "$codec" == reference ]]; then features=(--features reference-codec); fi
    CARGO_TARGET_DIR="$cache/raw-rand-wasm" cargo +"$compiler" clippy --manifest-path "$output/consumer/Cargo.toml" \
        --frozen --offline -p platform-raw-rand-canister --target wasm32-unknown-unknown "${features[@]}" -- -D warnings > "$output/$codec.clippy.log" 2>&1
    CARGO_PROFILE_RELEASE_OPT_LEVEL=3 CARGO_PROFILE_RELEASE_LTO=fat CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 \
    CARGO_PROFILE_RELEASE_PANIC=abort CARGO_PROFILE_RELEASE_STRIP=symbols \
        retain_build_artifact "$cache/raw-rand-wasm" wasm32-unknown-unknown/release/platform_raw_rand_canister.wasm \
        "$output/$codec.raw.wasm" cargo +"$compiler" build --manifest-path "$output/consumer/Cargo.toml" --frozen --offline \
        -p platform-raw-rand-canister --target wasm32-unknown-unknown --release "${features[@]}" > "$output/$codec.build.log" 2>&1
    ic-wasm "$output/$codec.raw.wasm" -o "$output/$codec.wasm" shrink > "$output/$codec.optimizer.log" 2>&1
    {
        printf '%s\t' "$codec"
        perl "$input_root/scripts/benchmarks/wasm-sections.pl" "$output/$codec.raw.wasm"
        printf '\t'
        perl "$input_root/scripts/benchmarks/wasm-sections.pl" "$output/$codec.wasm"
        printf '\n'
    } >> "$output/sizes.tsv"
done
cat "$output/sizes.tsv"
[[ "$mode" != --build-only ]] || exit 0
CARGO_TARGET_DIR="$cache/host-$compiler-owner" cargo +"$compiler" clippy --manifest-path "$output/consumer/Cargo.toml" \
    --frozen --offline -p platform-raw-rand-driver -- -D warnings > "$output/driver.clippy.log" 2>&1
retain_build_artifact "$cache/host-$compiler-owner" debug/platform-raw-rand-driver "$output/driver-bin" \
    cargo +"$compiler" build --manifest-path "$output/consumer/Cargo.toml" --frozen --offline -p platform-raw-rand-driver > "$output/driver.build.log" 2>&1
"$output/driver-bin" "$output/reference.wasm" "$output/narrow.wasm" > "$output/installed.log" 2>&1
cat "$output/installed.log"
