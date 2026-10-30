#!/usr/bin/env bash
# Isolated packaging/size investigation. Never patches the consumer's registry.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd -P)"
root="$(cd "$here/../../../.." && pwd -P)"
cd "$root"
mkdir -p target
out="$(mktemp -d "$root/target/candid-packaging.XXXXXX")"
trap 'printf "Candid packaging evidence: %s\n" "$out"' EXIT
make clippy > "$out/owner-clippy.log" 2>&1
cargo metadata --locked --offline --all-features --format-version 1 > "$out/owner-metadata.json"
source_manifest="$(node -e 'const m=require(process.argv[1]);const p=m.packages.find(p=>p.name==="candid"&&p.version==="0.10.39");if(!p)throw Error("Expected prepared Candid 0.10.39");process.stdout.write(p.manifest_path)' "$out/owner-metadata.json")"
node - "$here/upstream.json" "${source_manifest%/Cargo.toml}" <<'JS'
const fs=require('fs'),crypto=require('crypto');for(const [p,h] of Object.entries(require(process.argv[2]).files)){if(crypto.createHash('sha256').update(fs.readFileSync(`${process.argv[3]}/${p}`)).digest('hex')!==h)throw Error(`Upstream source mismatch: ${p}`);}
JS
cp -R "${source_manifest%/Cargo.toml}" "$out/compact-source"
patch -s -d "$out/compact-source" -p1 < "$here/../candid-reader/reader.patch"
patch -s -d "$out/compact-source" -p1 < "$here/../candid-reader/byteorder.patch"
node "$here/prepare.cjs" "$out/compact-source" "$out/codec"
cp "$here/codec.Cargo.toml.in" "$out/codec/Cargo.toml"
cp Cargo.lock "$out/codec/Cargo.lock"
patch -s -d "$out/codec" -p1 < "$here/edition-lints.patch"
mkdir -p "$out/platform" "$out/consumer/src/fixtures"
cp Cargo.toml Cargo.lock "$out/platform/"
cp -R crates "$out/platform/"
cp "$here/consumer.Cargo.toml.in" "$out/consumer/Cargo.toml"
cp "$here/consumer/lib.rs" "$out/consumer/src/lib.rs"
cp crates/ic-platform/tests/fixtures/management-snapshot-status.hex "$out/consumer/src/fixtures/"
cp Cargo.lock "$out/consumer/Cargo.lock"
export CARGO_TARGET_DIR="$root/target/candid-packaging-build"
# Only disposable locks resolve, before frozen qualification.
cargo metadata --manifest-path "$out/consumer/Cargo.toml" --offline --features owned-codec,baseline-reader,mixed-codec --format-version 1 > "$out/metadata.json"
cargo fmt --manifest-path "$out/codec/Cargo.toml"
cargo fmt --manifest-path "$out/consumer/Cargo.toml"
cargo clippy --manifest-path "$out/codec/Cargo.toml" --offline --all-targets -- -D warnings > "$out/codec-clippy.log" 2>&1
cargo clippy --manifest-path "$out/consumer/Cargo.toml" --frozen --all-targets --all-features -- -D warnings > "$out/consumer-clippy.log" 2>&1
for variant in baseline owned; do
    features=baseline-reader
    if [[ "$variant" == owned ]]; then features=owned-codec; fi
    cargo test --manifest-path "$out/consumer/Cargo.toml" --frozen --lib --features "$features" > "$out/$variant-tests.log" 2>&1
    cargo build --manifest-path "$out/consumer/Cargo.toml" --frozen --bin differential --features "$features" > "$out/$variant-native.log" 2>&1
    cp "$CARGO_TARGET_DIR/debug/differential" "$out/$variant-differential"
done
"$out/baseline-differential" generate > "$out/typed-seeds.txt"
node "$here/../candid-reader/corpus.cjs" "$out" "$root/crates/ic-platform/tests/fixtures" "$out/typed-seeds.txt"
for corpus in corpus typed-corpus; do
    for variant in baseline owned; do
        "$out/$variant-differential" < "$out/$corpus.txt" > "$out/$variant-$corpus.txt"
    done
    cmp "$out/baseline-$corpus.txt" "$out/owned-$corpus.txt"
done
for variant in baseline owned baseline-two mixed; do
    features=()
    case "$variant" in
        owned) features=(--features owned-codec) ;;
        baseline-two) features=(--features mixed-codec) ;;
        mixed) features=(--features "owned-codec,mixed-codec") ;;
    esac
    cargo build --manifest-path "$out/consumer/Cargo.toml" --frozen --release --target wasm32-unknown-unknown --lib "${features[@]}" > "$out/$variant-wasm.log" 2>&1
    cp "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/candid_packaging_consumer.wasm" "$out/$variant.wasm"
done
cargo tree --manifest-path "$out/consumer/Cargo.toml" --frozen --features owned-codec --target wasm32-unknown-unknown --edges normal,build --prefix none > "$out/owned.graph"
cargo +1.91.0 check --manifest-path "$out/consumer/Cargo.toml" --frozen --features owned-codec --lib --target wasm32-unknown-unknown > "$out/msrv.log" 2>&1
# Prove the concrete upstream configuration and argument-trait limitations.
mkdir -p "$out/identity-refusal/src"
sed -e 's/candid-packaging-consumer/codec-api-identity-refusal/' -e '/\[lib\]/,+1d' "$here/consumer.Cargo.toml.in" > "$out/identity-refusal/Cargo.toml"
cp "$out/consumer/Cargo.lock" "$out/identity-refusal/Cargo.lock"
cp "$here/incompatible.rs.in" "$out/identity-refusal/src/main.rs"
if cargo check --manifest-path "$out/identity-refusal/Cargo.toml" --offline --features owned-codec > "$out/identity-refusal.log" 2>&1; then
    echo 'Expected concrete codec API identity mismatches' >&2; exit 1
fi
[[ "$(grep -c 'error\[E0308\]' "$out/identity-refusal.log")" == 3 ]]
node "$here/analyze.cjs" "$out"
