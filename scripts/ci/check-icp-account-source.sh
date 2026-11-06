#!/usr/bin/env bash
# Guard the selected supplier constructor without compiling the supplier's CDK.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
source_root="${1:-$root/crates/ic-platform/src/icp_ledger_types/account_identifier}"
[[ $# -le 1 ]] || exit 2
mkdir -p "$root/target"
scratch="$(mktemp -d "$root/target/icp-account-source.XXXXXX")"
trap 'rm -rf "$scratch"' EXIT
perl -0777 -ne '
    my ($body) = /(    pub fn new\(owner: &Principal, subaccount: &Subaccount\) -> Self \{.*?\n    \})/s;
    defined $body or die "missing upstream ICP constructor\n";
    $body =~ s/\s+//g;
    print $body;
' "$source_root/mod.rs" > "$scratch/body"
expected="$(jq -er '.new_body_fingerprint' "$source_root/provenance.json")"
bash "$root/scripts/ci/verify-file-checksum.sh" sha256 "$expected" "$scratch/body"
expected="$(jq -er '.license_sha256' "$source_root/provenance.json")"
bash "$root/scripts/ci/verify-file-checksum.sh" sha256 "$expected" "$source_root/LICENSE-APACHE"
echo 'Selected ICP account identifier constructor and license match upstream 0.16.0.'
