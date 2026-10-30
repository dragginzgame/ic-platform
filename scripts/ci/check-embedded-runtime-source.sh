#!/usr/bin/env bash
set -euo pipefail
# Preserve the imported ABI, reviewed executor changes and complete license terms.
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
[[ $# -le 1 ]] || exit 2
source_root="${1:-$root/crates/ic-platform/src}"
jq="$root/.tools/host/bin/jq"
for module in ic0/upstream executor/runtime; do
    manifest="$root/crates/ic-platform/src/$module/provenance.json"
    while IFS=$'\t' read -r file checksum; do
        bash "$root/scripts/ci/verify-file-checksum.sh" sha256 "$checksum" "$source_root/$module/$file"
    done < <("$jq" -r '.maintained_sources | to_entries[] | [.key, .value] | @tsv' "$manifest")
    license="$("$jq" -er '.license_sha256' "$manifest")"
    bash "$root/scripts/ci/verify-file-checksum.sh" sha256 "$license" "$source_root/$module/LICENSE-APACHE"
done
echo 'Embedded IC0 ABI, maintained executor and licenses match reviewed sources.'
