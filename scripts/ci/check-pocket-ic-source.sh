#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
source_root="${1:-$root/crates/ic-platform-pocket-ic}"
[[ $# -le 1 ]] || exit 2
scratch="$(mktemp -d "${TMPDIR:-/tmp}/pocket-ic-source.XXXXXX")"
trap 'rm -rf "$scratch"' EXIT
while read -r checksum file; do
    [[ -n "$checksum" && -n "$file" ]] || exit 1
    checked="$source_root/$file"
    case "$file" in
        src/lib.rs|src/nonblocking.rs)
            # Reconstruct only the reviewed management dependency import rename
            # and rustfmt's corresponding import ordering. Original bodies stay exact.
            perl -0777 -pe '
                s{((?:pub )?use ic_management_canister_types::\{.*?\n\};\n)((?:(?:pub )?use ic_transport_types[^\n]+\n)+)}{
                    my ($management, $transport) = ($1, $2);
                    $management =~ s/ic_management_canister_types/pocket_ic_management_canister_types/;
                    $transport . $management;
                }se == 1 or die "management import adaptation changed\n";
            ' "$checked" > "$scratch/reconstructed"
            checked="$scratch/reconstructed" ;;
    esac
    bash "$root/scripts/ci/verify-file-checksum.sh" sha256 "$checksum" "$checked"
done < "$source_root/upstream/maintained-source.sha256"
echo 'Owned PocketIC client source and license fingerprints passed'
