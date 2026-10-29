#!/usr/bin/env bash
set -euo pipefail
# Reconstruct the imported value/hash source; admit packaging changes only.
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
source_root="${1:-$root/crates/ic-platform/src/icrc_ledger_types}"
[[ $# -le 1 ]] || exit 2
scratch="$(mktemp -d "${TMPDIR:-/tmp}/platform-icrc-source.XXXXXX")"
trap 'rm -rf "$scratch"' EXIT
perl -0777 -e '
    use strict; use warnings;
    my $file = shift; open my $input, "<", $file or die $!;
    local $/; my $source = <$input>;
    $source =~ s/#!\[allow\(\s*missing_docs,\s*reason = "Retained upstream declarations and documentation"\s*\)\]//;
    $source =~ s/use super::account::Account;/use crate::icrc1::account::Account;/;
    $source =~ s/#\[cfg\(any\(\)\)\]\s*pub fn arb_value/#[cfg(test)]\npub fn arb_value/;
    $source =~ s/#\[test\]\s*#\[cfg\(any\(\)\)\]\s*fn test_value_to_icrc3value_roundtrip/#[test]\nfn test_value_to_icrc3value_roundtrip/;
    my @strings;
    $source =~ s{"(?:\\.|[^"\\])*"}{push(@strings, $&); "\x01" . $#strings . "\x02"}gse;
    $source =~ s{//[^\n]*}{}g;
    $source =~ s{/\*.*?\*/}{}gs;
    $source =~ s/\s+//g;
    $source =~ s/\x01(\d+)\x02/$strings[$1]/ge;
    print $source;
' "$source_root/value/mod.rs" > "$scratch/value"
expected="$(jq -er '.value_source_fingerprint' "$source_root/provenance.json")"
bash "$root/scripts/ci/verify-file-checksum.sh" sha256 "$expected" "$scratch/value"
expected="$(jq -er '.license_sha256' "$source_root/provenance.json")"
bash "$root/scripts/ci/verify-file-checksum.sh" sha256 "$expected" "$source_root/LICENSE-APACHE"
expected="$(jq -er '.reference_schema_sha256' "$source_root/provenance.json")"
bash "$root/scripts/ci/verify-file-checksum.sh" sha256 "$expected" "$root/crates/ic-platform/tests/fixtures/icrc-reference.did"
printf 'Imported ICRC value/hash bodies and vectors reconstruct upstream 0.2.0; license matches.\n'
