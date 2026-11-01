#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
source_root="${1:-$root/crates/ic-platform-pocket-ic}"
[[ $# -le 1 ]] || exit 2
while read -r checksum file; do
    [[ -n "$checksum" && -n "$file" ]] || exit 1
    bash "$root/scripts/ci/verify-file-checksum.sh" sha256 "$checksum" "$source_root/$file"
done < "$source_root/upstream/maintained-source.sha256"
echo 'Owned PocketIC client source and license fingerprints passed'
