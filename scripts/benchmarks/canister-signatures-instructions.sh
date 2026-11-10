#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
[[ $# -le 1 ]] || { echo 'usage: canister-signatures-instructions.sh [NEW_OUTPUT_DIRECTORY]' >&2; exit 2; }
[[ -n "${POCKET_IC_BIN:-}${IC_TESTKIT_POCKET_IC_URL:-}" ]] || {
    echo 'Select an explicitly prepared Testkit server first' >&2; exit 1;
}
make -C "$root" toko-tooling-check
mkdir -p "$root/target/evidence"
output="${1:-$(mktemp -d "$root/target/evidence/signature-instructions.XXXXXX")}"
mkdir -p "$output/probe/src"
output="$(cd "$output" && pwd -P)"
[[ ! -e "$output/probe/Cargo.toml" ]] || { echo 'Choose a new output directory' >&2; exit 2; }
cp "$root/scripts/fixtures/signature-instructions/"*.rs "$output/probe/src/"
for pair in 'signature-time-valid:direct' 'signature-range-legacy-valid:legacy' 'signature-range-sharded-valid:sharded' 'bls-public:root'; do
    input="${pair%%:*}"; name="${pair##*:}"
    perl -0777 -ne 's/\s+//g; print pack("H*", $_)' "$root/crates/ic-platform/tests/fixtures/$input.hex" > "$output/probe/src/$name.bin"
done
printf '\x30\x20\x30\x0c\x06\x0a\x2b\x06\x01\x04\x01\x83\xb8\x43\x01\x02\x03\x10\x00\x0a\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01seed' > "$output/probe/src/public.bin"
platform="$(perl -MJSON::PP -e 'print encode_json($ARGV[0])' "$root/crates/ic-platform")"
runtime="$(perl -MJSON::PP -e 'print encode_json($ARGV[0])' "$root/crates/ic-platform-runtime")"
# shellcheck source=scripts/ci/workspace-dependencies.sh
source "$root/scripts/ci/workspace-dependencies.sh"
platform_dependencies_read "$root"
cat > "$output/probe/Cargo.toml" <<MANIFEST
[package]
name = "platform-signature-instructions-probe"
version = "0.0.0"
edition = "2024"
publish = false
[package.metadata.toko-tooling.loc]
category = "benchmarks"
[workspace]
[lib]
crate-type = ["cdylib"]
[features]
owned = ["dep:ic-platform", "ic-platform/signature-verification"]
reference = ["dep:ic-canister-sig-creation", "dep:ic-signature-verification"]
reference-admitted = ["dep:ic-canister-sig-creation", "dep:ic-signature-verification", "dep:ic-certification", "dep:ic_bls12_381", "dep:serde", "dep:serde_bytes", "dep:serde_cbor"]
[dependencies]
ic-platform = { path = $platform, optional = true, default-features = false }
ic-platform-runtime = { path = $runtime, default-features = false, features = ["ic0"] }
MANIFEST
for dependency in ic-canister-sig-creation ic-signature-verification ic-certification ic_bls12_381 serde serde_bytes serde_cbor; do
    printf '%s = { %s, optional = true }\n' "$dependency" "$(platform_dependency "$dependency")" >> "$output/probe/Cargo.toml"
done
cat >> "$output/probe/Cargo.toml" <<'MANIFEST'
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = "debuginfo"
MANIFEST
cp "$root/Cargo.lock" "$output/probe/Cargo.lock"
cargo metadata --offline --manifest-path "$output/probe/Cargo.toml" --format-version 1 > "$output/probe/metadata.json"
rustc -vV > "$output/compiler.txt"
ic-wasm --version > "$output/optimizer.txt"
if [[ -n "${POCKET_IC_BIN:-}" ]]; then
    "$POCKET_IC_BIN" --version > "$output/server-version.txt"
    sha256sum "$POCKET_IC_BIN" > "$output/server.sha256"
fi
for variant in owned reference reference-admitted; do
    cargo metadata --offline --locked --manifest-path "$output/probe/Cargo.toml" --features "$variant" --format-version 1 > "$output/$variant.metadata.json"
    required=ic-platform-runtime
    [[ "$variant" != owned ]] || required=ic-platform,ic-platform-runtime
    bash "$root/scripts/ci/check-qualification-identities.sh" "$output/$variant.metadata.json" "$root" \
        platform-signature-instructions-probe "$required" > "$output/$variant.identities.json"
    cargo tree --offline --locked --manifest-path "$output/probe/Cargo.toml" --target wasm32-unknown-unknown \
        --features "$variant" --edges normal,build --prefix none > "$output/$variant.graph"
    if [[ "$variant" == owned ]] && grep -Eq '^(candid|ic_principal|ic-canister-sig-creation|ic-signature-verification) v' "$output/$variant.graph"; then
        echo 'Owned benchmark acquired a retired signature dependency' >&2; exit 1
    fi
    cargo clippy --offline --locked --manifest-path "$output/probe/Cargo.toml" --target wasm32-unknown-unknown \
        --features "$variant" --lib -- -D warnings > "$output/$variant.clippy.log" 2>&1
    cargo build --offline --locked --release --manifest-path "$output/probe/Cargo.toml" --target wasm32-unknown-unknown \
        --features "$variant" > "$output/$variant.build.log" 2>&1
    cp "$output/probe/target/wasm32-unknown-unknown/release/platform_signature_instructions_probe.wasm" "$output/$variant.raw.wasm"
    ic-wasm "$output/$variant.raw.wasm" -o "$output/$variant.wasm" shrink > "$output/$variant.optimizer.log" 2>&1
    sha256sum "$output/$variant.wasm" >> "$output/artifacts.sha256"
done
# shellcheck source=scripts/benchmarks/driver.sh
source "$root/scripts/benchmarks/driver.sh"
compiler="$(bash "$root/scripts/ci/read-platform-msrv.sh")"
build_benchmark_driver "$root" "$output" "$compiler" signature "${SIGNATURE_BENCH_HOST_TARGET_DIR:-$root/target/signature-benchmark-driver}"
sha256sum "$root/Cargo.toml" "$root/Cargo.lock" "$root/crates/ic-platform/Cargo.toml" \
    "$root/crates/ic-platform/src/canister_sig_creation/mod.rs" \
    "$root/crates/ic-platform/src/signature_verification/mod.rs" \
    "$output/probe/src/"* "$output/probe/Cargo.toml" "$output/probe/Cargo.lock" > "$output/inputs.sha256"
"$driver" owned "$output/owned.wasm" reference "$output/reference.wasm" \
    reference-admitted "$output/reference-admitted.wasm" > "$output/instructions.tsv"
cat "$output/instructions.tsv"
printf 'Evidence: %s\n' "$output"
