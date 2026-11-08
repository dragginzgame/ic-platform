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
    $source =~ s/#\[cfg\(any\(\)\)\]\s*pub fn arb_value/#[cfg(test)]\npub fn arb_value/;
    $source =~ s/#\[test\]\s*#\[cfg\(any\(\)\)\]\s*fn test_value_to_icrc3value_roundtrip/#[test]\nfn test_value_to_icrc3value_roundtrip/;
    print normalize($source);
' "$source_root/value/mod.rs" > "$scratch/value"
expected="$(jq -er '.value_source_fingerprint' "$source_root/provenance.json")"
bash "$root/scripts/ci/verify-file-checksum.sh" sha256 "$expected" "$scratch/value"
expected="$(jq -er '.license_sha256' "$source_root/provenance.json")"
bash "$root/scripts/ci/verify-file-checksum.sh" sha256 "$expected" "$source_root/LICENSE-APACHE"
expected="$(jq -er '.reference_schema_sha256' "$source_root/provenance.json")"
bash "$root/scripts/ci/verify-file-checksum.sh" sha256 "$expected" "$root/crates/ic-platform/tests/fixtures/icrc-reference.did"
printf 'Imported ICRC value/hash bodies and vectors reconstruct upstream 0.2.0; license matches.\n'
