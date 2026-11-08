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
rg --files crates | LC_ALL=C sort > "$output/source-files.txt"
printf '%s\n' Cargo.toml Cargo.lock rust-toolchain.toml .cargo/config.toml \
    scripts/ci/check-platform-raw-rand.sh scripts/ci/workspace-dependencies.sh \
    scripts/ci/retain-build-artifact.sh scripts/ci/check-platform-runtime-graph.sh \
    scripts/ci/platform-graph-policy.awk scripts/ci/read-platform-msrv.sh \
    scripts/benchmarks/wasm-sections.pl scripts/fixtures/raw-rand/canister/mod.rs \
    scripts/fixtures/cdk-runtime/raw_rand_driver.rs scripts/fixtures/cdk-runtime/pocketic_support/mod.rs \
    >> "$output/source-files.txt"
while IFS= read -r file; do shasum -a 256 "$file"; done < "$output/source-files.txt" > "$output/source.sha256"
finish() {
    local result=$?
    trap - EXIT
    shasum -a 256 --check "$output/source.sha256" > "$output/source-validation.log" || result=1
    printf '%s\n' "$result" > "$output/status"
    printf 'raw_rand evidence: %s\n' "$output"
    exit "$result"
}
trap finish EXIT
compiler="$(bash scripts/ci/read-platform-msrv.sh Cargo.toml)"
# shellcheck source=scripts/ci/workspace-dependencies.sh
source scripts/ci/workspace-dependencies.sh
platform_dependencies_read "$root"
testkit="$(platform_dependency ic-testkit)"
tokio="$(platform_dependency tokio)"
mkdir -p "$output/canister/src" "$output/driver/src"
cp scripts/fixtures/raw-rand/canister/mod.rs "$output/canister/src/lib.rs"
cp scripts/fixtures/cdk-runtime/raw_rand_driver.rs "$output/driver/src/main.rs"
cp -R scripts/fixtures/cdk-runtime/pocketic_support "$output/driver/src/"
cp Cargo.lock "$output/Cargo.lock"
cat > "$output/Cargo.toml" <<MANIFEST
[workspace]
members = ["canister", "driver"]
resolver = "3"
[workspace.package]
edition = "2024"
rust-version = "$compiler"
[workspace.dependencies]
ic-platform = { path = "$root/crates/ic-platform", default-features = false }
ic-platform-runtime = { path = "$root/crates/ic-platform-runtime", default-features = false }
ic-testkit = { $testkit }
tokio = { $tokio }
MANIFEST
cat > "$output/canister/Cargo.toml" <<'MANIFEST'
[package]
name = "platform-raw-rand-canister"
version = "0.0.0"
edition.workspace = true
rust-version.workspace = true
[package.metadata.shared-tooling.loc]
category = "tests"
[lib]
crate-type = ["cdylib"]
[features]
reference-candid = ["ic-platform/candid"]
[dependencies]
ic-platform = { workspace = true, features = ["raw-rand"] }
ic-platform-runtime = { workspace = true, features = ["runtime"] }
MANIFEST
cat > "$output/driver/Cargo.toml" <<'MANIFEST'
[package]
name = "platform-raw-rand-driver"
version = "0.0.0"
edition.workspace = true
rust-version.workspace = true
[package.metadata.shared-tooling.loc]
category = "tests"
[dependencies]
ic-platform = { workspace = true, features = ["raw-rand", "candid"] }
ic-testkit.workspace = true
tokio.workspace = true
MANIFEST
cargo +"$compiler" metadata --manifest-path "$output/Cargo.toml" --offline --format-version 1 > "$output/metadata.json" 2> "$output/metadata.log"
for target in "$(rustc +"$compiler" -vV | sed -n 's/^host: //p')" wasm32-unknown-unknown; do
    cargo +"$compiler" tree --manifest-path "$output/Cargo.toml" --frozen --offline -p platform-raw-rand-canister \
        --no-default-features --target "$target" --edges normal,build --prefix none > "$output/$target.graph"
    bash scripts/ci/check-platform-runtime-graph.sh "$output/$target.graph"
    if grep -Eq '^(candid(_derive|_parser)?|ic-management-canister-types|ic-agent) ' "$output/$target.graph"; then
        echo 'raw_rand consumer acquired a general Candid/native helper' >&2; exit 1
    fi
done
[[ "$mode" != --check-graph ]] || exit 0
cache="${CARGO_TARGET_DIR:-$root/target/runtime-build}"
rustc +"$compiler" -vV > "$output/compiler.txt"
ic-wasm --version > "$output/optimizer-version.txt"
# shellcheck source=scripts/ci/retain-build-artifact.sh
source scripts/ci/retain-build-artifact.sh
printf 'codec\traw_total\traw_code\traw_custom\toptimized_total\toptimized_code\toptimized_custom\n' > "$output/sizes.tsv"
for codec in reference narrow; do
    features=()
    if [[ "$codec" == reference ]]; then features=(--features reference-candid); fi
    CARGO_TARGET_DIR="$cache/raw-rand-wasm" cargo +"$compiler" clippy --manifest-path "$output/Cargo.toml" \
        --frozen --offline -p platform-raw-rand-canister --target wasm32-unknown-unknown "${features[@]}" -- -D warnings > "$output/$codec.clippy.log" 2>&1
    CARGO_PROFILE_RELEASE_OPT_LEVEL=3 CARGO_PROFILE_RELEASE_LTO=fat CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 \
    CARGO_PROFILE_RELEASE_PANIC=abort CARGO_PROFILE_RELEASE_STRIP=symbols \
        retain_build_artifact "$cache/raw-rand-wasm" wasm32-unknown-unknown/release/platform_raw_rand_canister.wasm \
        "$output/$codec.raw.wasm" cargo +"$compiler" build --manifest-path "$output/Cargo.toml" --frozen --offline \
        -p platform-raw-rand-canister --target wasm32-unknown-unknown --release "${features[@]}" > "$output/$codec.build.log" 2>&1
    ic-wasm "$output/$codec.raw.wasm" -o "$output/$codec.wasm" shrink > "$output/$codec.optimizer.log" 2>&1
    {
        printf '%s\t' "$codec"
        perl scripts/benchmarks/wasm-sections.pl "$output/$codec.raw.wasm"
        printf '\t'
        perl scripts/benchmarks/wasm-sections.pl "$output/$codec.wasm"
        printf '\n'
    } >> "$output/sizes.tsv"
done
cat "$output/sizes.tsv"
[[ "$mode" != --build-only ]] || exit 0
CARGO_TARGET_DIR="$cache/host-$compiler-owner" cargo +"$compiler" clippy --manifest-path "$output/Cargo.toml" \
    --frozen --offline -p platform-raw-rand-driver -- -D warnings > "$output/driver.clippy.log" 2>&1
retain_build_artifact "$cache/host-$compiler-owner" debug/platform-raw-rand-driver "$output/driver-bin" \
    cargo +"$compiler" build --manifest-path "$output/Cargo.toml" --frozen --offline -p platform-raw-rand-driver > "$output/driver.build.log" 2>&1
"$output/driver-bin" "$output/reference.wasm" "$output/narrow.wasm" > "$output/installed.log" 2>&1
cat "$output/installed.log"
