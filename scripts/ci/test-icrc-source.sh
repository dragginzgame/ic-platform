#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
mkdir -p "$root/target"
fixture="$(mktemp -d "$root/target/platform-icrc-guard-test.XXXXXX")"
trap 'rm -rf "$fixture"' EXIT
mkdir -p "$fixture/value" "$fixture/hash_encoding"
source_root="$root/crates/ic-platform/src/icrc_ledger_types"
cp "$source_root/provenance.json" "$source_root/LICENSE-APACHE" "$fixture/"
for mutation in provider hash vector license lint encoder encoder-license carrier; do
    cp "$source_root/value/mod.rs" "$fixture/value/mod.rs"
    cp "$source_root/LICENSE-APACHE" "$fixture/LICENSE-APACHE"
    cp "$source_root/hash_encoding/mod.rs" "$source_root/hash_encoding/LICENSE" "$fixture/hash_encoding/"
    case "$mutation" in
        provider) sed -i '/use sha2_11 as sha2;/d' "$fixture/value/mod.rs" ;;
        encoder) sed -i 's/0x80u8/0x40u8/' "$fixture/hash_encoding/mod.rs" ;;
        encoder-license) printf 'changed' >> "$fixture/hash_encoding/LICENSE" ;;
        carrier) sed -i 's/{Int, Nat}/{Nat, Int}/' "$fixture/value/mod.rs" ;;
        hash) perl -pi -e 's/Value::Text\(text\) => Sha256::digest\(text.as_bytes\(\)\)/Value::Text(text) => Sha256::digest([])/' "$fixture/value/mod.rs" ;;
        vector) perl -pi -e 's/684888c0ebb17f374298b65ee2807526c066094c701bcc7ebbe1c1095f494fc1/084888c0ebb17f374298b65ee2807526c066094c701bcc7ebbe1c1095f494fc1/' "$fixture/value/mod.rs" ;;
        lint) perl -pi -e 's/Selected upstream ICRC declarations/Changed upstream ICRC declarations/' "$fixture/value/mod.rs" ;;
        license) printf '\nchanged\n' >> "$fixture/LICENSE-APACHE" ;;
    esac
    if bash "$root/scripts/ci/check-icrc-source.sh" "$fixture" > "$fixture/log" 2>&1; then
        echo "ICRC source guard admitted $mutation mutation" >&2; exit 1
    fi
done
echo 'ICRC source guard refuses hash-body, vector, license and exact lint-normalization mutations.'
