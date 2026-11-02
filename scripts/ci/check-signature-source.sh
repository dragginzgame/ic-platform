#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
source_root="${1:-$root/crates/ic-platform/src/signature_upstream}"
[[ $# -le 1 ]] || exit 2
admitted_creation="$(mktemp)"
admitted_verification="$(mktemp)"
reference="$(mktemp)"
trap 'rm -f "$admitted_creation" "$admitted_verification" "$reference"' EXIT
# One authorized framing call at the exact parser entry; reconstruct every
# imported byte before checking the original hash. No cryptographic hash changes.
perl -0777 -e '
    my $source = <>;
    my $call = "    super::admission::canister_sig_der(pk_der)?; // Platform: checked DER admission.\n";
    my $entry = "pub fn extract_raw_canister_sig_pk_from_der(pk_der: &[u8]) -> Result<Vec<u8>, String> {\n";
    die "Missing DER admission at parser entry\n" unless index($source, $entry . $call) >= 0;
    my $count = $source =~ s/^\Q$call\E//mg;
    die "Expected exactly one DER admission call\n" unless $count == 1;
    ($source =~ s/crate::certification::upstream::/ic_certification::/g) == 1
        or die "Missing owned creation certification import\n";
    ($source =~ s/^use crate::principal::Principal;$/use ic_principal::Principal;/m) == 1
        or die "Missing owned Principal import\n";
    # #123 replaces only the eleven test assertions with MSRV-compatible std
    # macros. Restore the original assertions/import before fingerprinting.
    ($source =~ s/assert!\(\s*matches!\(&result, (Err\(e\) if e\.contains\("[^"\n]+"\))\),\s*"\{result:\?\}"\s*,?\s*\);/assert_matches!(result, $1);/g) == 10
        or die "Changed signature test assertions or diagnostics\n";
    ($source =~ s/\Qassert!(result.is_ok(), "{result:?}");\E/assert_matches!(result, Ok(_));/) == 1
        or die "Changed signature success assertion\n";
    ($source =~ s/(mod tests \{\n    use super::\*;\n)/$1    use assert_matches::assert_matches;\n/) == 1
        or die "Missing signature test module\n";
    print $source;
' "$source_root/creation/mod.rs" > "$admitted_creation"
# The BLS packaging move changes this import only, never verifier bodies.
perl -0777 -e '
    my $source = <>;
    ($source =~ s/^use crate::bls::verify_bls_signature;$/use ic_verify_bls_signature::verify_bls_signature;/m) == 1
        or die "Missing owned BLS import\n";
    ($source =~ s/crate::certification::upstream::/ic_certification::/g) == 2
        or die "Missing owned verification certification imports\n";
    ($source =~ s/^use crate::principal::Principal;$/use ic_principal::Principal;/m) == 1
        or die "Missing owned Principal import\n";
    print $source;
' "$source_root/verification/mod.rs" > "$admitted_verification"
# Exact imported bytes, including original tests: only the documented import/module
# edits, 0.5.1 cache deletion, 0.6 reference-only attributes and the checked
# admission call reconstructed above were admitted.
# Never refresh these hashes to approve an algorithm change.
while read -r hash file; do
    source="$source_root/$file"
    if [[ "$file" == creation/mod.rs ]]; then source="$admitted_creation"; fi
    if [[ "$file" == verification/mod.rs ]]; then source="$admitted_verification"; fi
    bash "$root/scripts/ci/verify-file-checksum.sh" sha256 "$hash" "$source"
done <<'HASHES'
874ec06ba0f460f3ae41514e20371d1388a84f494ea6122b1a9bb66585bfef31 creation/mod.rs
9b40b7b638b739421fcbdad5d860a789b5fffcfa939009f05896592757092dca verification/mod.rs
a7d816caf02fa4fbaced7723c2ef742f2b2c4b6e1347f42ecf3aba9692b0c3fc creation/LICENSE
6f44e3f228570e343cdb96bd929d1e8c6057ac33b411e47d77427f824e5a96ac verification/LICENSE
HASHES
# Strip only the eight reviewed packaging attributes and recover the exact 0.5.1 source.
sed '/^#\[cfg(test)\] \/\/ Platform: reference-only\.$/d; /^#\[allow(unused_imports)\] \/\/ Platform: reference-only HashTree\.$/d' \
    "$admitted_creation" > "$reference"
bash "$root/scripts/ci/verify-file-checksum.sh" sha256 \
    d042a45c3f89fa9c5fa43eccf29bb0b64297f032ef052c4f49f0daab1f2be106 "$reference"
# Recover the unchanged verifier after removing only the additive child module.
sed '/^pub(crate) mod evidence; \/\/ Platform: authenticated time adapter\.$/d' \
    "$admitted_verification" > "$reference"
bash "$root/scripts/ci/verify-file-checksum.sh" sha256 \
    d434da3f0db0f8212c0e107da27f2d50fc8916f2db6b94874d092d76c8c878a5 "$reference"
printf 'Imported signature bodies, vectors and licenses match the qualified source.\n'
