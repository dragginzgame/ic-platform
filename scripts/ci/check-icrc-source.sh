#!/usr/bin/env bash
set -euo pipefail
# Reconstruct the imported value/hash source; admit packaging changes only.
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
source_root="${1:-$root/crates/ic-platform/src/icrc_ledger_types}"
[[ $# -le 1 ]] || exit 2
mkdir -p "$root/target"
scratch="$(mktemp -d "$root/target/platform-icrc-source.XXXXXX")"
trap 'rm -rf "$scratch"' EXIT
perl -I "$root/scripts/ci" -MSourceFingerprint=normalize -0777 -e '
    use strict; use warnings;
    my $file = shift; open my $input, "<", $file or die $!;
    local $/; my $source = <$input>;
    $source =~ s/#!\[expect\(\s*missing_docs,\s*reason = "Selected upstream ICRC declarations contain undocumented public fields and variants\."\s*\)\]// == 1
        or die "unexpected ICRC documentation lint scaffolding\n";
    $source =~ s/use super::account::Account;/use crate::icrc1::account::Account;/;
    $source =~ s/use crate::candid_codec::\{Int, Nat\};\s*use crate::principal::Principal;/use candid::{CandidType, Deserialize, Int, Nat, Principal};/ == 1
        or die "unexpected ICRC carrier imports\n";
    $source =~ s/use serde::Deserialize;\s*// == 1 or die "unexpected serde import\n";
    $source =~ s/#\[derive\(/#[derive(CandidType, /g == 2
        or die "unexpected ICRC conditional derives\n";
    $source =~ s/#\[cfg\(any\(\)\)\]\s*pub fn arb_value/#[cfg(test)]\npub fn arb_value/;
    $source =~ s/#\[test\]\s*#\[cfg\(any\(\)\)\]\s*fn test_value_to_icrc3value_roundtrip/#[test]\nfn test_value_to_icrc3value_roundtrip/;
    $source =~ s/^use sha2_11 as sha2;\n//m == 1 or die "missing SHA-2 provider alias\n";
    print normalize($source);
' "$source_root/value/mod.rs" > "$scratch/value"
expected="$(jq -er '.value_source_fingerprint' "$source_root/provenance.json")"
bash "$root/scripts/ci/verify-file-checksum.sh" sha256 "$expected" "$scratch/value"
expected="$(jq -er '.license_sha256' "$source_root/provenance.json")"
bash "$root/scripts/ci/verify-file-checksum.sh" sha256 "$expected" "$source_root/LICENSE-APACHE"
expected="$(jq -er '.reference_schema_sha256' "$source_root/provenance.json")"
bash "$root/scripts/ci/verify-file-checksum.sh" sha256 "$expected" "$root/crates/ic-platform/tests/fixtures/icrc-reference.did"
printf 'Imported ICRC value/hash bodies and vectors reconstruct upstream 0.2.0; license matches.\n'
perl -I "$root/scripts/ci" -MSourceFingerprint=normalize -0777 -e '
    use strict; use warnings;
    open my $input, "<", shift or die $!; local $/; my $source = <$input>;
    $source =~ /(pub\(crate\) fn encode<W>.*)\n\}/s or die "missing Nat encoder\n";
    my $body = $1;
    $body =~ s/pub\(crate\) fn/pub fn/;
    $body =~ s/io::Result<\(\)>/crate::Result<()>/ == 1 or die "unexpected encoder signature\n";
    print normalize($body);
' "$source_root/hash_encoding/mod.rs" > "$scratch/nat-encoder"
expected="$(jq -er '.nat_encoder_fingerprint' "$source_root/provenance.json")"
bash "$root/scripts/ci/verify-file-checksum.sh" sha256 "$expected" "$scratch/nat-encoder"
expected="$(jq -er '.nat_encoder_license_sha256' "$source_root/provenance.json")"
bash "$root/scripts/ci/verify-file-checksum.sh" sha256 "$expected" "$source_root/hash_encoding/LICENSE"
printf 'Hash-input Nat encoder reconstructs Candid 0.10.40; license matches.\n'
