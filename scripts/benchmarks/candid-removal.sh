#!/usr/bin/env bash
# Compare the published upstream adapter with the owned codec on named shapes.
# Source archives and generated probes are evidence, never additional checkouts.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
baseline="$(git -C "$root" rev-parse --verify "${1:-97fe286}^{commit}")"
baseline_codec="${CANDID_BASELINE_CODEC:-upstream}"
case "$baseline_codec" in upstream|owned) ;; *) echo 'CANDID_BASELINE_CODEC must be upstream or owned' >&2; exit 2 ;; esac
profiles="${CANDID_COMPARISON_PROFILES:-management}"
measure="${CANDID_MEASURE_INSTRUCTIONS:-0}"
case "$measure" in 0|1) ;; *) echo 'CANDID_MEASURE_INSTRUCTIONS must be 0 or 1' >&2; exit 2 ;; esac
if [[ "$measure" == 1 ]]; then
    [[ -n "${POCKET_IC_BIN:-}${IC_TESTKIT_POCKET_IC_URL:-}" ]] || {
        echo 'Select an explicitly prepared Testkit server first' >&2; exit 1;
    }
    make -C "$root" toko-tooling-check
fi
for profile in $profiles; do
    case "$profile" in management|blob|words|options|map|records|bigints) ;;
        *) echo "Unknown comparison profile: $profile" >&2; exit 2 ;; esac
done
mkdir -p "$root/target/evidence"
output="$(mktemp -d "$root/target/evidence/candid-removal.XXXXXX")"
mkdir "$output/source"
git -C "$root" archive "$baseline" crates Cargo.toml Cargo.lock | tar -x -C "$output/source"
cp "$root/Cargo.lock" "$output/source/Cargo.lock"
printf '%s\n' "$baseline" > "$output/baseline-commit"
toolchain="$(bash "$root/scripts/ci/read-platform-msrv.sh")"
rustc "+$toolchain" -vV > "$output/compiler.txt"
ic-wasm --version > "$output/optimizer.txt"
printf 'arm\tprofile\traw_total\traw_code\traw_custom\toptimized_total\toptimized_code\toptimized_custom\n' > "$output/sizes.tsv"
printf 'Evidence: %s\n' "$output"
driver_args=()
for arm in before after; do
    source="$root"
    [[ "$arm" != before ]] || source="$output/source"
    for profile in $profiles; do
    probe="$output/$arm/$profile"
    features='"ic0", "management-canister-types"'
    if [[ "$profile" != management ]]; then
        features='"ic0", "candid-codec"'
        [[ "$arm" != before || "$baseline_codec" == owned ]] || features='"ic0", "candid"'
    fi
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
ic-platform = { path = "$source/crates/ic-platform", default-features = false, features = [$features] }
MANIFEST
    case "$profile" in
        management) value='ic_platform::management_canister_types::CanisterStatusResult' ;;
        blob) value='Vec<u8>' ;;
        words) value='Vec<u64>' ;;
        options) value='Option<Result<Vec<u64>, String>>' ;;
        map) value='std::collections::BTreeMap<String, u64>' ;;
        records) value='Vec<(u64, Option<String>)>' ;;
        bigints)
            owner=candid_codec
            [[ "$arm" != before || "$baseline_codec" == owned ]] || owner=candid::contract
            value="(ic_platform::$owner::Nat, ic_platform::$owner::Int)" ;;
    esac
    printf 'type Value = %s;\n' "$value" > "$probe/src/lib.rs"
    # Keep the same ingress/reply and logical DTO round trip in both arms.
    cat >> "$probe/src/lib.rs" <<'RUST'
#[unsafe(export_name = "canister_query codec")]
pub extern "C" fn codec() {
    let mut bytes = vec![0; ic_platform::ic0::msg_arg_data_size()];
    ic_platform::ic0::msg_arg_data_copy(&mut bytes, 0);
    let output = roundtrip(&bytes);
    ic_platform::ic0::msg_reply_data_append(&output);
    ic_platform::ic0::msg_reply();
}
RUST
    if [[ "$arm" == before && "$baseline_codec" == upstream ]]; then
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
    export CARGO_TARGET_DIR="$output/$arm/target" CARGO_BUILD_JOBS=4
    export CARGO_PROFILE_RELEASE_OPT_LEVEL=3 CARGO_PROFILE_RELEASE_LTO=fat
    export CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 CARGO_PROFILE_RELEASE_PANIC=abort
    export CARGO_PROFILE_RELEASE_STRIP=symbols
    CARGO_ENCODED_RUSTFLAGS="$(printf '%s\037%s' "--remap-path-prefix=$source=/platform-source" "--remap-path-prefix=$output/$arm=/probe")"
    export CARGO_ENCODED_RUSTFLAGS
    cargo "+$toolchain" clippy --offline --locked --release --manifest-path "$probe/Cargo.toml" --target wasm32-unknown-unknown -- -D warnings > "$probe/clippy.log" 2>&1
    cargo "+$toolchain" build --frozen --release --manifest-path "$probe/Cargo.toml" --target wasm32-unknown-unknown > "$probe/build.log" 2>&1
    cp "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/management_codec_size_probe.wasm" "$probe/raw.wasm"
    ic-wasm "$probe/raw.wasm" -o "$probe/optimized.wasm" shrink > "$probe/optimizer.log" 2>&1
    { printf '%s\t%s\t' "$arm" "$profile"; perl "$root/scripts/benchmarks/wasm-sections.pl" "$probe/raw.wasm"; printf '\t'; perl "$root/scripts/benchmarks/wasm-sections.pl" "$probe/optimized.wasm"; printf '\n'; } >> "$output/sizes.tsv"
    sha256sum --check "$probe/source-sha256.txt" > "$probe/source-validation.log"
    cp "$probe/src/lib.rs" "$probe/uninstrumented.rs"
    sha256sum "$probe/"{Cargo.toml,Cargo.lock,uninstrumented.rs,raw.wasm,optimized.wasm} > "$probe/artifact-sha256.txt"
    if [[ "$measure" == 1 && "$profile" != management ]]; then
        # Instrument a separate artifact; retain the uninstrumented size above.
        cat >> "$probe/src/lib.rs" <<'RUST'
#[unsafe(export_name = "canister_update measure")]
pub extern "C" fn measure() {
    let mut bytes = vec![0; ic_platform::ic0::msg_arg_data_size()];
    ic_platform::ic0::msg_arg_data_copy(&mut bytes, 0);
    let start = ic_platform::ic0::performance_counter(0);
    let output = roundtrip(std::hint::black_box(&bytes));
    let instructions = ic_platform::ic0::performance_counter(0) - start;
    ic_platform::ic0::msg_reply_data_append(&instructions.to_le_bytes());
    ic_platform::ic0::msg_reply_data_append(&output);
    ic_platform::ic0::msg_reply();
}
RUST
        cargo "+$toolchain" clippy --offline --locked --release --manifest-path "$probe/Cargo.toml" --target wasm32-unknown-unknown -- -D warnings > "$probe/measured-clippy.log" 2>&1
        cargo "+$toolchain" build --frozen --release --manifest-path "$probe/Cargo.toml" --target wasm32-unknown-unknown > "$probe/measured-build.log" 2>&1
        cp "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/management_codec_size_probe.wasm" "$probe/measured.raw.wasm"
        ic-wasm "$probe/measured.raw.wasm" -o "$probe/measured.wasm" shrink > "$probe/measured-optimizer.log" 2>&1
        sha256sum "$probe/src/lib.rs" "$probe/measured.wasm" > "$probe/measured-sha256.txt"
        driver_args+=("$arm" "$profile" "$probe/measured.wasm")
    fi
    done
done
cat "$output/sizes.tsv"
if [[ "$measure" == 1 ]]; then
    unset CARGO_ENCODED_RUSTFLAGS CARGO_PROFILE_RELEASE_PANIC CARGO_PROFILE_RELEASE_STRIP
    # shellcheck source=scripts/benchmarks/driver.sh
    source "$root/scripts/benchmarks/driver.sh"
    build_benchmark_driver "$root" "$output" "$toolchain" codec "$root/target/codec-benchmark-driver"
    if [[ -n "${POCKET_IC_BIN:-}" ]]; then
        "$POCKET_IC_BIN" --version > "$output/server-version.txt"
        sha256sum "$POCKET_IC_BIN" > "$output/server.sha256"
    fi
    "$driver" "${driver_args[@]}" > "$output/instructions.tsv"
    cat "$output/instructions.tsv"
fi
