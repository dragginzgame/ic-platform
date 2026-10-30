#!/usr/bin/env bash
# Reproduce the measured Candid 0.10.39 reader experiment; production sources stay untouched.
# Requires the repository's prepared Rust/Node tools and cached Cargo dependencies.
# Optional installed proof: set POCKET_IC_BIN to the prepared 16.1.0 server.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd -P)"
root="$(cd "$here/../../../.." && pwd -P)"
cd "$root"
mkdir -p target
out="$(mktemp -d "$root/target/candid-reader.XXXXXX")"
trap 'printf "Candid reader evidence retained: %s\n" "$out"' EXIT
cargo metadata --locked --offline --all-features --format-version 1 > "$out/metadata.json"
source_manifest="$(node -e 'const m=require(process.argv[1]); const p=m.packages.find(p=>p.name==="candid"&&p.version==="0.10.39"); if(!p)throw Error("Prepared candid 0.10.39 is required");process.stdout.write(p.manifest_path)' "$out/metadata.json")"
cp -R "$(dirname "$source_manifest")" "$out/candid-baseline"
printf '\n[workspace]\n' >> "$out/candid-baseline/Cargo.toml"
node - "$here/upstream.json" "$out/candid-baseline" <<'JS'
const fs=require('fs'),crypto=require('crypto');for(const [p,h] of Object.entries(require(process.argv[2]).files)) {const a=crypto.createHash('sha256').update(fs.readFileSync(`${process.argv[3]}/${p}`)).digest('hex');if(a!==h)throw Error(`Upstream source mismatch: ${p}`);}
JS
cp -R "$out/candid-baseline" "$out/candid-reader"
patch -s -d "$out/candid-reader" -p1 < "$here/reader.patch"
cp -R "$out/candid-reader" "$out/candid-compact"
patch -s -d "$out/candid-compact" -p1 < "$here/byteorder.patch"
mkdir "$out/platform"
cp Cargo.toml Cargo.lock rust-toolchain.toml "$out/platform/"
cp -R crates "$out/platform/"
make clippy > "$out/owner-clippy.log" 2>&1
node "$here/corpus.cjs" "$out" "$root/crates/ic-platform/tests/fixtures"
for variant in baseline reader compact; do
    harness="$out/$variant-harness"
    cp -R "$here/harness" "$harness"
    sed "s|../candid-reader\"|../candid-$variant\"|" "$here/harness/Cargo.toml.in" > "$harness/Cargo.toml"
    mkdir -p "$harness/src/tests/fixtures"
    cp crates/ic-platform/tests/management_consumer.rs "$harness/src/tests/mod.rs"
    cp crates/ic-platform/tests/fixtures/management-*.hex "$harness/src/tests/fixtures/"
    printf '\n#[cfg(test)]\nmod tests;\n' >> "$harness/src/lib.rs"
    features=()
    if [[ "$variant" == baseline ]]; then features=(--features baseline-reader); fi
    export CARGO_TARGET_DIR="$root/target/candid-reader-native-build"
    cargo clippy --manifest-path "$harness/Cargo.toml" --offline --all-targets "${features[@]}" -- -D warnings > "$out/$variant-clippy.log" 2>&1
    cargo test --manifest-path "$harness/Cargo.toml" --offline --lib > "$out/$variant-tests.log" 2>&1
    cargo build --manifest-path "$harness/Cargo.toml" --offline --bin differential "${features[@]}" > "$out/$variant-build.log" 2>&1
    cp "$CARGO_TARGET_DIR/debug/differential" "$out/$variant-differential"
    "$out/$variant-differential" < "$out/corpus.txt" > "$out/$variant-results.txt"
    if [[ "$variant" == baseline ]]; then
        "$out/baseline-differential" generate > "$out/typed-seeds.txt"
        node "$here/corpus.cjs" "$out" "$root/crates/ic-platform/tests/fixtures" "$out/typed-seeds.txt"
    fi
    "$out/$variant-differential" < "$out/typed-corpus.txt" > "$out/$variant-typed-results.txt"
    export CARGO_TARGET_DIR="$root/target/candid-reader-wasm-build"
    cargo build --manifest-path "$harness/Cargo.toml" --offline --release --target wasm32-unknown-unknown --lib > "$out/$variant-wasm.log" 2>&1
    cp "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/candid_reader_probe.wasm" "$out/$variant.wasm"
    cargo tree --manifest-path "$harness/Cargo.toml" --locked --offline --target wasm32-unknown-unknown -e normal,build --prefix none > "$out/$variant.graph"
done
node "$here/analyze.cjs" "$out"
if [[ -n "${POCKET_IC_BIN:-}" ]]; then
    cp -R "$here/driver" "$out/driver"
    mv "$out/driver/Cargo.toml.in" "$out/driver/Cargo.toml"
    export CARGO_TARGET_DIR="$root/target/candid-reader-driver-build"
    cargo clippy --manifest-path "$out/driver/Cargo.toml" --offline --all-targets -- -D warnings > "$out/driver-clippy.log" 2>&1
    cargo build --manifest-path "$out/driver/Cargo.toml" --offline > "$out/driver-build.log" 2>&1
    "$CARGO_TARGET_DIR/debug/candid-reader-installed" "$out" > "$out/installed.log" 2>&1
fi
