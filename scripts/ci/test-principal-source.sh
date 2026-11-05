#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
mkdir -p "$root/target"
fixture="$(mktemp -d "$root/target/platform-principal-source.XXXXXX")"
trap 'printf "Principal source mutation evidence: %s\n" "$fixture"' EXIT
for mutation in derivation tag capacity vector license provider; do
    selected="$fixture/$mutation"
    cp -R "$root/crates/ic-platform-types/src/principal" "$selected"
    case "$mutation" in
        derivation) perl -pi -e 's/Sha224::digest\(public_key\)/Sha224::digest([])/' "$selected/self_authenticating/mod.rs" ;;
        tag) perl -pi -e 's/TAG: u8 = 2/TAG: u8 = 3/' "$selected/self_authenticating/mod.rs" ;;
        capacity) perl -pi -e 's/BYTES: usize = 29/BYTES: usize = 28/' "$selected/mod.rs" ;;
        vector) perl -0777 -pi -e 's/209, 74, 2/208, 74, 2/ == 1 or die "missing vector mutation target\n"' "$selected/tests/mod.rs" ;;
        license) printf '\nchanged\n' >> "$selected/LICENSE-APACHE" ;;
        provider) sed -i.bak '/^use sha2_11 as sha2;$/d' "$selected/self_authenticating/mod.rs" ;;
    esac
    if diff -qr "$root/crates/ic-platform-types/src/principal" "$selected" > "$fixture/$mutation.diff"; then
        echo "Principal mutation did not change source: $mutation" >&2; exit 1
    fi
    if bash "$root/scripts/ci/check-principal-source.sh" "$selected" > "$fixture/$mutation.log" 2>&1; then
        echo "Principal guard admitted $mutation mutation" >&2; exit 1
    fi
done
echo 'Principal source guard rejected derivation, tag, capacity, vector, license and provider mutations.'
