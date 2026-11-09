#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
[[ $# -le 1 ]] || exit 2
source_root="${1:-$root/crates/ic-platform/src/management_canister_types/native}"
mkdir -p "$root/target"
scratch="$(mktemp -d "$root/target/management-native-source.XXXXXX")"
trap 'rm -rf "$scratch"' EXIT
perl -I "$root/scripts/ci" -MSourceFingerprint=normalize -0777 -e '
    use strict; use warnings;
    local $/; my $source = <>;
    $source =~ s/use super::maintained::\{CanisterId, ChunkHash, SnapshotId, SnapshotSource\};\s*// == 1 or die "unexpected native management imports\n";
    $source =~ s/use crate::candid_codec::Nat;\s*// == 1 or die "unexpected Nat import\n";
    $source =~ s/use serde::\{Deserialize, Serialize\};\s*// == 1 or die "unexpected Serde imports\n";
    $source =~ s/mod codec;\s*// == 1 or die "unexpected codec module\n";
    $source =~ s/mod client;\s*// == 1 or die "unexpected native client module\n";
    $source =~ s/pub use client::\{[^}]*\};\s*// == 1 or die "unexpected native client exports\n";
    $source =~ s/#\[derive\(/#[derive(CandidType, /g == 14 or die "unexpected native management derives\n";
    $source =~ s/#\[cfg\(test\)\]\s*mod tests;\s*// == 1 or die "unexpected native tests module\n";
    my $normalized = normalize($source);
    $normalized =~ s/(#\[derive\([^)]*),\)\]/$1)]/g;
    print $normalized;
' "$source_root/mod.rs" > "$scratch/tokens"
checksum="$(jq -er '.selected_source_fingerprint' "$root/crates/ic-platform/src/management_canister_types/native/provenance.json")"
bash "$root/scripts/ci/verify-file-checksum.sh" sha256 "$checksum" "$scratch/tokens"
echo 'Native management snapshot declarations reconstruct upstream 0.11.0.'
perl -I "$root/scripts/ci" -MSourceFingerprint=normalize -0777 -e '
    use strict; use warnings;
    local $/; my $source = <>;
    $source =~ s/use super::super::maintained::\{CanisterId, CanisterInstallMode, CanisterSettings\};\s*// == 1 or die "unexpected native client imports\n";
    $source =~ s/use crate::candid_codec::Nat;\s*// == 1 or die "unexpected native client Nat\n";
    $source =~ s/use serde::\{Deserialize, Serialize\};\s*// == 1 or die "unexpected native client Serde\n";
    $source =~ s/pub wasm_module: Vec<u8>/pub wasm_module: WasmModule/ == 1 or die "unexpected native client Wasm\n";
    $source =~ s/#\[derive\(/#[derive(CandidType, /g == 4 or die "unexpected native client derives\n";
    my $normalized = normalize($source);
    $normalized =~ s/(#\[derive\([^)]*),\)\]/$1)]/g;
    print $normalized;
' "$source_root/client.rs" > "$scratch/client-tokens"
checksum="$(jq -er '.selected_source_fingerprint' "$root/crates/ic-platform/src/management_canister_types/native/client-provenance.json")"
bash "$root/scripts/ci/verify-file-checksum.sh" sha256 "$checksum" "$scratch/client-tokens"
echo 'Native install/create/log declarations reconstruct upstream 0.11.0.'
