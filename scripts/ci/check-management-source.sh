#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
[[ $# -le 1 ]] || exit 2
source_root="${1:-$root/crates/ic-platform/src/management_canister_types/maintained}"
manifest="$root/crates/ic-platform/src/management_canister_types/maintained/provenance.json"
mkdir -p "$root/target"
scratch="$(mktemp -d "$root/target/management-source.XXXXXX")"
trap 'rm -rf "$scratch"' EXIT
perl -I "$root/scripts/ci" -MSourceFingerprint=normalize -0777 -e '
    use strict; use warnings;
    local $/; my $source = <>;
    $source =~ s/use crate::candid_codec::\{Nat, Reserved\};\s*use crate::principal::Principal;/use candid::{CandidType, Nat, Principal, Reserved};/ == 1 or die "unexpected management carrier imports\n";
    $source =~ s/\nmod codec;\n/\n/ == 1 or die "unexpected codec module\n";
    $source =~ s/#\[derive\(/#[derive(CandidType, /g == 49 or die "unexpected management derives\n";
    $source =~ s/pub status: Nat,/pub status: candid::Nat,/ == 1 or die "unexpected HTTP status carrier\n";
    my $callback = qq{mod transform_func {
    #![cfg_attr(
        target_family = "wasm",
        expect(
            missing_docs,
            reason = "The upstream function-reference macro emits undocumented members."
        )
    )]
    use super::{HttpRequestResult, TransformArgs};
    candid::define_function!(pub TransformFunc : (TransformArgs) -> (HttpRequestResult) query);
}};
    $source =~ s/mod transform_func;/$callback/ == 1 or die "unexpected transform owner\n";
    my $http_lint = <<'LINT';
#[cfg_attr(
    all(test, not(target_family = "wasm")),
    expect(
        clippy::upper_case_acronyms,
        reason = "Native private schema tests retain the external HTTP method names."
    )
)]
LINT
    $source =~ s/pub enum HttpMethod/$http_lint . "pub enum HttpMethod"/e == 1 or die "unexpected HTTP method declaration\n";
    my $normalized = normalize($source);
    $normalized =~ s/(#\[derive\([^)]*),\)\]/$1)]/g;
    print $normalized;
' "$source_root/mod.rs" > "$scratch/tokens"
checksum="$("$root/.tools/host/bin/jq" -er '.codec_base_fingerprint' "$manifest")"
bash "$root/scripts/ci/verify-file-checksum.sh" sha256 "$checksum" "$scratch/tokens"
checksum="$("$root/.tools/host/bin/jq" -er '.license_sha256' "$manifest")"
bash "$root/scripts/ci/verify-file-checksum.sh" sha256 "$checksum" "$source_root/LICENSE-APACHE"
printf 'Management DTOs reconstruct the reviewed 0.11.0 declarations; Apache license matches.\n'

bash "$root/scripts/ci/check-management-native-source.sh"
