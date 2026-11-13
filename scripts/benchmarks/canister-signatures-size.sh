#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
[[ $# -le 1 ]] || { echo 'usage: canister-signatures-size.sh [NEW_OUTPUT_DIRECTORY]' >&2; exit 2; }
mkdir -p "$root/target/evidence"
output="${1:-$(mktemp -d "$root/target/evidence/signature-size.XXXXXX")}"
mkdir -p "$output/src"
output="$(cd "$output" && pwd -P)"
[[ ! -e "$output/Cargo.toml" ]] || { echo 'Choose a new output directory' >&2; exit 2; }
cp "$root/scripts/fixtures/signature-size/lib.rs" "$output/src/lib.rs"
for pair in 'signature-range-sharded-valid:signature' 'bls-public:root'; do
    input="${pair%%:*}"; name="${pair##*:}"
    perl -0777 -ne 's/\s+//g; print pack("H*", $_)' "$root/crates/ic-platform/tests/fixtures/$input.hex" > "$output/src/$name.bin"
done
# Protocol data: issuer [1; 10], seed "seed", DER short-form envelope.
printf '\x30\x20\x30\x0c\x06\x0a\x2b\x06\x01\x04\x01\x83\xb8\x43\x01\x02\x03\x10\x00\x0a\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01seed' > "$output/src/public.bin"
quoted_source="$(perl -MJSON::PP -e 'print encode_json($ARGV[0])' "$root/crates/ic-platform")"
# shellcheck source=scripts/ci/workspace-dependencies.sh
source "$root/scripts/ci/workspace-dependencies.sh"
platform_dependencies_read "$root"
creation="$(platform_dependency ic-canister-sig-creation)"
verification="$(platform_dependency ic-signature-verification)"
cat > "$output/Cargo.toml" <<MANIFEST
[package]
name = "platform-signature-size-probe"
version = "0.0.0"
edition = "2024"
publish = false
[package.metadata.toko-tooling.loc]
category = "benchmarks"
[workspace]
[lib]
crate-type = ["cdylib"]
[features]
owned-creation = ["dep:ic-platform", "ic-platform/canister-sig-creation"]
reference-creation = ["dep:ic-canister-sig-creation"]
owned-verification = ["dep:ic-platform", "ic-platform/signature-verification"]
reference-verification = ["dep:ic-signature-verification"]
[dependencies]
ic-platform = { path = $quoted_source, optional = true, default-features = false }
ic-canister-sig-creation = { $creation, optional = true }
ic-signature-verification = { $verification, optional = true }
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = "debuginfo"
MANIFEST
cp "$root/Cargo.lock" "$output/Cargo.lock"
platform_fixture_policy "$root" "$output"
cargo metadata --offline --manifest-path "$output/Cargo.toml" --format-version 1 > "$output/metadata.json"
rustc -vV > "$output/compiler.txt"
ic-wasm --version > "$output/optimizer.txt"
sha256sum "$root/Cargo.lock" "$root/crates/ic-platform/Cargo.toml" \
    "$root/crates/ic-platform/src/canister_sig_creation/mod.rs" \
    "$root/crates/ic-platform/src/signature_verification/mod.rs" \
    "$output/src/"* "$output/Cargo.toml" "$output/Cargo.lock" > "$output/inputs.sha256"
printf 'profile\traw_total\traw_code\traw_custom\toptimized_total\toptimized_code\toptimized_custom\n' > "$output/sizes.tsv"
for profile in owned-creation reference-creation owned-verification reference-verification; do
    cargo tree --offline --locked --manifest-path "$output/Cargo.toml" --target wasm32-unknown-unknown \
        --no-default-features --features "$profile" --edges normal,build --prefix none > "$output/$profile.graph"
    if [[ "$profile" == owned-* ]] && grep -Eq '^(candid|ic_principal|ic-canister-sig-creation|ic-signature-verification|ic-verify-bls-signature|ic0|ic-platform-runtime) v' "$output/$profile.graph"; then
        echo 'Owned signature probe acquired a forbidden production dependency' >&2; exit 1
    fi
    cargo clippy --offline --locked --manifest-path "$output/Cargo.toml" --target wasm32-unknown-unknown \
        --no-default-features --features "$profile" --lib -- -D warnings > "$output/$profile.clippy.log" 2>&1
    cargo build --offline --locked --release --manifest-path "$output/Cargo.toml" --target wasm32-unknown-unknown \
        --no-default-features --features "$profile" > "$output/$profile.build.log" 2>&1
    cp "$output/target/wasm32-unknown-unknown/release/platform_signature_size_probe.wasm" "$output/$profile.raw.wasm"
    ic-wasm "$output/$profile.raw.wasm" -o "$output/$profile.optimized.wasm" shrink > "$output/$profile.optimizer.log" 2>&1
    raw="$(perl "$root/scripts/benchmarks/wasm-sections.pl" "$output/$profile.raw.wasm")"
    optimized="$(perl "$root/scripts/benchmarks/wasm-sections.pl" "$output/$profile.optimized.wasm")"
    printf '%s\t%s\t%s\n' "$profile" "$raw" "$optimized" >> "$output/sizes.tsv"
done
cat "$output/sizes.tsv"
printf 'Evidence: %s\n' "$output"
