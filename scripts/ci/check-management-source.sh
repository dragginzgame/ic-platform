#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
[[ $# -le 1 ]] || exit 2
source_root="${1:-$root/crates/ic-platform/src/management_canister_types/maintained}"
manifest="$root/crates/ic-platform/src/management_canister_types/maintained/provenance.json"
for entry in 'maintained_source_sha256 mod.rs' 'license_sha256 LICENSE-APACHE'; do
    read -r field file <<< "$entry"
    checksum="$("$root/.tools/host/bin/jq" -er ".$field" "$manifest")"
    bash "$root/scripts/ci/verify-file-checksum.sh" sha256 "$checksum" "$source_root/$file"
done
printf 'Selected management DTO source and Apache license match the reviewed 0.11.0 import.\n'
