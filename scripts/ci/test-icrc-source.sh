#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/platform-icrc-guard-test.XXXXXX")"
trap 'rm -rf "$fixture"' EXIT
mkdir -p "$fixture/value"
source_root="$root/crates/ic-platform/src/icrc_ledger_types"
cp "$source_root/provenance.json" "$source_root/LICENSE-APACHE" "$fixture/"
for mutation in hash vector license; do
    cp "$source_root/value/mod.rs" "$fixture/value/mod.rs"
    cp "$source_root/LICENSE-APACHE" "$fixture/LICENSE-APACHE"
    case "$mutation" in
        hash) perl -pi -e 's/Value::Text\(text\) => Sha256::digest\(text.as_bytes\(\)\)/Value::Text(text) => Sha256::digest([])/' "$fixture/value/mod.rs" ;;
        vector) perl -pi -e 's/684888c0ebb17f374298b65ee2807526c066094c701bcc7ebbe1c1095f494fc1/084888c0ebb17f374298b65ee2807526c066094c701bcc7ebbe1c1095f494fc1/' "$fixture/value/mod.rs" ;;
        license) printf '\nchanged\n' >> "$fixture/LICENSE-APACHE" ;;
    esac
    if bash "$root/scripts/ci/check-icrc-source.sh" "$fixture" > "$fixture/log" 2>&1; then
        echo "ICRC source guard admitted $mutation mutation" >&2; exit 1
    fi
done
echo 'ICRC source guard refuses hash-body, vector and license mutations.'
