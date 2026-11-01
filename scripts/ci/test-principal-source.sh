#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/platform-principal-source.XXXXXX")"
trap 'printf "Principal source mutation evidence: %s\n" "$fixture"' EXIT
for mutation in derivation tag capacity vector license; do
    selected="$fixture/$mutation"
    cp -R "$root/crates/ic-platform-types/src/principal" "$selected"
    case "$mutation" in
        derivation) perl -pi -e 's/Sha224::digest\(public_key\)/Sha224::digest([])/' "$selected/self_authenticating/mod.rs" ;;
        tag) perl -pi -e 's/TAG: u8 = 2/TAG: u8 = 3/' "$selected/self_authenticating/mod.rs" ;;
        capacity) perl -pi -e 's/BYTES: usize = 29/BYTES: usize = 28/' "$selected/mod.rs" ;;
        vector) perl -pi -e 's/209, 74, 2/208, 74, 2/' "$selected/tests/mod.rs" ;;
        license) printf '\nchanged\n' >> "$selected/LICENSE-APACHE" ;;
    esac
    if bash "$root/scripts/ci/check-principal-source.sh" "$selected" > "$fixture/$mutation.log" 2>&1; then
        echo "Principal guard admitted $mutation mutation" >&2; exit 1
    fi
done
echo 'Principal source guard rejected derivation, tag, capacity, vector and license mutations.'
