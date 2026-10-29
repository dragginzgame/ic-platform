#!/usr/bin/env bash
set -euo pipefail
# Manual, serial size qualification. No installs, sibling edits or root lock writes.
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
[[ $# -le 2 ]] || { echo 'usage: wasm-size.sh [PLATFORM_SOURCE [NEW_OUTPUT_DIRECTORY]]' >&2; exit 2; }
source_root="$(cd "${1:-$root}" && pwd -P)"
output="${2:-$(mktemp -d "${TMPDIR:-/tmp}/platform-wasm-size.XXXXXX")}"
mkdir -p "$output"
output="$(cd "$output" && pwd -P)"
[[ ! -e "$output/Cargo.toml" ]] || { echo 'Size evidence already exists; select a new output directory' >&2; exit 2; }
toolchain="${WASM_BENCH_TOOLCHAIN:-1.91.0}"
build_target="${WASM_BENCH_TARGET_DIR:-$output/target}"
profiles="${WASM_BENCH_PROFILES:-principal principal-binary principal-self-auth runtime external-candid verification}"
recipes="${WASM_BENCH_RECIPES:-3-none-16 3-thin-1 3-fat-1 s-fat-1 z-fat-1}"
for profile in $profiles; do
    case "$profile" in principal|principal-binary|principal-self-auth|runtime|external-candid|candid-exact|candid-sequence|candid-sequence-one|verification) ;; *) echo "Unknown size profile: $profile" >&2; exit 2 ;; esac
done
for recipe in $recipes; do
    case "$recipe" in 3-none-16|3-thin-1|3-fat-1|s-fat-1|z-fat-1) ;; *) echo "Unknown size recipe: $recipe" >&2; exit 2 ;; esac
done
ic-wasm --version > "$output/optimizer-version.txt"
cargo "+$toolchain" --version > "$output/cargo-version.txt"
rustc "+$toolchain" -vV > "$output/rustc-version.txt"
printf '%s\n' "--remap-path-prefix=$source_root=/platform-source" \
    "--remap-path-prefix=$output=/probe" > "$output/rustflags.txt"
encoded_flags="$(printf '%s\037%s' "--remap-path-prefix=$source_root=/platform-source" "--remap-path-prefix=$output=/probe")"
if [[ "$(git -C "$source_root" rev-parse --show-toplevel 2>/dev/null || true)" == "$source_root" ]] &&
    git -C "$source_root" rev-parse HEAD > "$output/source-head.txt" 2>/dev/null; then
    git -C "$source_root" diff --binary -- crates Cargo.toml Cargo.lock > "$output/source.patch"
else
    printf 'Archive source; use source-sha256.txt for identity.\n' > "$output/source-head.txt"
fi
find "$source_root/crates" -type f -print0 | xargs -0 shasum -a 256 > "$output/source-sha256.txt"
shasum -a 256 "$source_root/Cargo.toml" "$source_root/Cargo.lock" >> "$output/source-sha256.txt"
mkdir "$output/src"
cp "$root/scripts/fixtures/wasm-size/lib.rs" "$output/src/lib.rs"
cp "$source_root/Cargo.lock" "$output/Cargo.lock"
quoted_source="$(perl -MJSON::PP -e 'print encode_json($ARGV[0])' "$source_root/crates/ic-platform")"
cat > "$output/Cargo.toml" <<MANIFEST
[package]
name = "platform-wasm-size-probe"
version = "0.0.0"
edition = "2024"
[workspace]
[lib]
crate-type = ["cdylib"]
[features]
principal = ["ic-platform/principal", "dep:serde_json"]
principal-binary = ["ic-platform/principal"]
principal-self-auth = ["ic-platform/principal-self-authenticating"]
runtime = ["ic-platform/runtime"]
external-candid = ["ic-platform/candid", "ic-platform/management-canister-types"]
candid-exact = ["ic-platform/candid", "ic-platform/management-canister-types"]
candid-sequence = ["ic-platform/candid"]
candid-sequence-one = ["candid-sequence"]
verification = ["ic-platform/signature-verification"]
[dependencies]
ic-platform = { path = $quoted_source, default-features = false, features = ["ic0"] }
serde_json = { version = "1.0", optional = true }
MANIFEST
cargo "+$toolchain" metadata --manifest-path "$output/Cargo.toml" --offline --format-version 1 > "$output/metadata.json" 2> "$output/metadata.log"
shasum -a 256 "$output/src/lib.rs" "$output/Cargo.toml" "$output/Cargo.lock" > "$output/probe-sha256.txt"
printf 'recipe\tcapability\traw_bytes\tcode_bytes\tcustom_bytes\toptimized_bytes\toptimized_code_bytes\toptimized_custom_bytes\n' > "$output/sizes.tsv"
for recipe in $recipes; do
    IFS=- read -r optimization lto units <<< "$recipe"
    [[ "$lto" != none ]] || lto=false
    for profile in $profiles; do
        printf 'Wasm size: %s / %s\n' "$recipe" "$profile"
        stem="$output/$recipe-$profile"
        CARGO_TARGET_DIR="$build_target/$recipe" CARGO_BUILD_JOBS=4 \
            CARGO_ENCODED_RUSTFLAGS="$encoded_flags" \
            CARGO_PROFILE_RELEASE_OPT_LEVEL="$optimization" CARGO_PROFILE_RELEASE_LTO="$lto" \
            CARGO_PROFILE_RELEASE_CODEGEN_UNITS="$units" CARGO_PROFILE_RELEASE_PANIC=abort \
            CARGO_PROFILE_RELEASE_STRIP=symbols \
            cargo "+$toolchain" build --manifest-path "$output/Cargo.toml" --release --frozen \
            --target wasm32-unknown-unknown --no-default-features --features "$profile" > "$stem.build.log" 2>&1
        cp "$build_target/$recipe/wasm32-unknown-unknown/release/platform_wasm_size_probe.wasm" "$stem.raw.wasm"
        ic-wasm "$stem.raw.wasm" -o "$stem.optimized.wasm" shrink > "$stem.optimizer.log" 2>&1
        {
            printf '%s\t%s\t' "$recipe" "$profile"
            perl "$root/scripts/benchmarks/0.4/sections.pl" "$stem.raw.wasm"
            printf '\t'
            perl "$root/scripts/benchmarks/0.4/sections.pl" "$stem.optimized.wasm"
            printf '\n'
        } >> "$output/sizes.tsv"
        shasum -a 256 "$stem.raw.wasm" "$stem.optimized.wasm" >> "$output/artifact-sha256.txt"
    done
done
shasum -a 256 --check "$output/source-sha256.txt" > "$output/source-validation.log"
shasum -a 256 --check "$output/probe-sha256.txt" > "$output/probe-validation.log"
printf 'Size evidence retained at %s\n' "$output"
