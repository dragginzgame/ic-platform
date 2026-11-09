#!/usr/bin/env bash
# Compare the published upstream codec with the owned management codec.
# Source archives and generated probes are evidence, never additional checkouts.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
baseline="$(git -C "$root" rev-parse --verify "${1:-97fe286}^{commit}")"
mkdir -p "$root/target/evidence"
output="$(mktemp -d "$root/target/evidence/candid-removal.XXXXXX")"
mkdir "$output/source"
git -C "$root" archive "$baseline" crates Cargo.toml Cargo.lock | tar -x -C "$output/source"
cp "$root/Cargo.lock" "$output/source/Cargo.lock"
printf '%s\n' "$baseline" > "$output/baseline-commit"
toolchain="$(bash "$root/scripts/ci/read-platform-msrv.sh")"
rustc "+$toolchain" -vV > "$output/compiler.txt"
ic-wasm --version > "$output/optimizer.txt"
printf 'arm\traw_total\traw_code\traw_custom\toptimized_total\toptimized_code\toptimized_custom\n' > "$output/sizes.tsv"
printf 'Evidence: %s\n' "$output"
for arm in before after; do
    source="$root"
    [[ "$arm" != before ]] || source="$output/source"
    probe="$output/$arm"
    mkdir -p "$probe/src"
    cp "$root/Cargo.lock" "$probe/Cargo.lock"
    cat > "$probe/Cargo.toml" <<MANIFEST
[package]
name = "management-codec-size-probe"
version = "0.0.0"
edition = "2024"
[workspace]
[lib]
crate-type = ["cdylib"]
[dependencies]
ic-platform = { path = "$source/crates/ic-platform", default-features = false, features = ["ic0", "management-canister-types"] }
MANIFEST
    # Keep the same ingress/reply and logical DTO round trip in both arms.
    cat > "$probe/src/lib.rs" <<'RUST'
use ic_platform::management_canister_types::CanisterStatusResult as Value;
#[unsafe(export_name = "canister_query codec")]
pub extern "C" fn codec() {
    let mut bytes = vec![0; ic_platform::ic0::msg_arg_data_size()];
    ic_platform::ic0::msg_arg_data_copy(&mut bytes, 0);
    let output = roundtrip(&bytes);
    ic_platform::ic0::msg_reply_data_append(&output);
    ic_platform::ic0::msg_reply();
}
RUST
    if [[ "$arm" == before ]]; then
        cat >> "$probe/src/lib.rs" <<'RUST'
fn roundtrip(bytes: &[u8]) -> Vec<u8> {
    use ic_platform::candid::{decode_exactly_one, DecodeLimits, contract::encode_one};
    let value: Value = decode_exactly_one(bytes, DecodeLimits {
        reply_bytes: 1_048_576, header_bytes: 65_536, type_entries: 256,
        decoding_quota: 10_000_000, skipping_quota: 10_000_000,
    }).unwrap();
    encode_one(value).unwrap()
}
RUST
    else
        cat >> "$probe/src/lib.rs" <<'RUST'
fn roundtrip(bytes: &[u8]) -> Vec<u8> {
    use ic_platform::candid_codec::{decoder::{decode_one, Limits}, encode_one};
    let value: Value = decode_one(bytes, Limits {
        bytes: 1_048_576, header: 65_536, types: 256,
        work: 10_000_000, allocation: 4_194_304,
    }).unwrap();
    encode_one(&value, 1_048_576).unwrap()
}
RUST
    fi
    cargo "+$toolchain" metadata --offline --manifest-path "$probe/Cargo.toml" --format-version 1 > "$probe/metadata.json" 2> "$probe/metadata.log"
    find "$source/crates" -type f -print0 | xargs -0 sha256sum > "$probe/source-sha256.txt"
    export CARGO_TARGET_DIR="$probe/target" CARGO_BUILD_JOBS=4
    export CARGO_PROFILE_RELEASE_OPT_LEVEL=3 CARGO_PROFILE_RELEASE_LTO=fat
    export CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_PROFILE_RELEASE_PANIC=abort
    export CARGO_PROFILE_RELEASE_STRIP=symbols
    CARGO_ENCODED_RUSTFLAGS="$(printf '%s\037%s' "--remap-path-prefix=$source=/platform-source" "--remap-path-prefix=$probe=/probe")"
    export CARGO_ENCODED_RUSTFLAGS
    cargo "+$toolchain" clippy --offline --locked --release --manifest-path "$probe/Cargo.toml" --target wasm32-unknown-unknown -- -D warnings > "$probe/clippy.log" 2>&1
    cargo "+$toolchain" build --frozen --release --manifest-path "$probe/Cargo.toml" --target wasm32-unknown-unknown > "$probe/build.log" 2>&1
    cp "$probe/target/wasm32-unknown-unknown/release/management_codec_size_probe.wasm" "$probe/raw.wasm"
    ic-wasm "$probe/raw.wasm" -o "$probe/optimized.wasm" shrink > "$probe/optimizer.log" 2>&1
    { printf '%s\t' "$arm"; perl "$root/scripts/benchmarks/wasm-sections.pl" "$probe/raw.wasm"; printf '\t'; perl "$root/scripts/benchmarks/wasm-sections.pl" "$probe/optimized.wasm"; printf '\n'; } >> "$output/sizes.tsv"
    sha256sum --check "$probe/source-sha256.txt" > "$probe/source-validation.log"
    sha256sum "$probe/"{Cargo.toml,Cargo.lock,src/lib.rs,raw.wasm,optimized.wasm} > "$probe/artifact-sha256.txt"
done
cat "$output/sizes.tsv"
