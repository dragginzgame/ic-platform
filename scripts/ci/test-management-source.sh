#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
mkdir -p "$root/target"
fixture="$(mktemp -d "$root/target/management-source-test.XXXXXX")"
trap 'rm -rf "$fixture"' EXIT
for file in mod.rs LICENSE-APACHE; do
    destination="$fixture/management-${file//./-}"
    cp -R "$root/crates/ic-platform/src/management_canister_types/maintained" "$destination"
    printf '\nmutated\n' >> "$destination/$file"
    if bash "$root/scripts/ci/check-management-source.sh" "$destination" > "$fixture/log" 2>&1; then
        echo "Unexpectedly admitted management source/license mutation: $file" >&2; exit 1
    fi
done
destination="$fixture/management-lint"
cp -R "$root/crates/ic-platform/src/management_canister_types/maintained" "$destination"
perl -i -pe 's/pub enum HttpMethod/#[allow(clippy::upper_case_acronyms)]\npub enum HttpMethod/' "$destination/mod.rs"
if bash "$root/scripts/ci/check-management-source.sh" "$destination" > "$fixture/log" 2>&1; then
    echo 'Unexpectedly admitted management lint-normalization mutation' >&2; exit 1
fi
printf 'Management DTO source and license mutation refusals passed.\n'

for mutation in field serde extra; do
    destination="$fixture/native-$mutation"
    cp -R "$root/crates/ic-platform/src/management_canister_types/native" "$destination"
    case "$mutation" in
        field) perl -i -pe 's/pub wasm_module_size: u64/pub wasm_module_size: u32/' "$destination/mod.rs" ;;
        serde) perl -i -pe 's/rename = "active"/rename = "changed"/' "$destination/mod.rs" ;;
        extra) printf '\npub type Extra = u8;\n' >> "$destination/mod.rs" ;;
    esac
    if bash "$root/scripts/ci/check-management-native-source.sh" "$destination" > "$fixture/log" 2>&1; then
        echo "Native management source guard admitted $mutation" >&2; exit 1
    fi
done
echo 'Native management guards reject field, Serde and declaration changes.'
