#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
source_root="${1:-$root/crates/ic-platform/src/signature_upstream}"
[[ $# -le 1 ]] || exit 2
admitted_creation="$(mktemp)"
reference="$(mktemp)"
trap 'rm -f "$admitted_creation" "$reference"' EXIT
# One authorized framing call at the exact parser entry; reconstruct every
# imported byte before checking the original hash. No cryptographic hash changes.
perl -0777 -e '
    my $source = <>;
    my $call = "    super::admission::canister_sig_der(pk_der)?; // Platform: checked DER admission.\n";
    my $entry = "pub fn extract_raw_canister_sig_pk_from_der(pk_der: &[u8]) -> Result<Vec<u8>, String> {\n";
    die "Missing DER admission at parser entry\n" unless index($source, $entry . $call) >= 0;
    my $count = $source =~ s/^\Q$call\E//mg;
    die "Expected exactly one DER admission call\n" unless $count == 1;
    print $source;
' "$source_root/creation/mod.rs" > "$admitted_creation"
# Exact imported bytes, including original tests: only the documented import/module
# edits, 0.5.1 cache deletion, 0.6 reference-only attributes and the checked
# admission call reconstructed above were admitted.
# Never refresh these hashes to approve an algorithm change.
while read -r hash file; do
    source="$source_root/$file"
    if [[ "$file" == creation/mod.rs ]]; then source="$admitted_creation"; fi
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
    "$source_root/verification/mod.rs" > "$reference"
bash "$root/scripts/ci/verify-file-checksum.sh" sha256 \
    d434da3f0db0f8212c0e107da27f2d50fc8916f2db6b94874d092d76c8c878a5 "$reference"
printf 'Imported signature bodies, vectors and licenses match the qualified source.\n'
