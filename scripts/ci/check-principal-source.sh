#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
[[ $# -le 1 ]] || exit 2
source_root="${1:-$root/crates/ic-platform-types/src/principal}"
# Exact admitted module, including the original ic_principal 0.1.5 method body
# and tag. SHA-224 remains the upstream sha2 implementation.
reference="$(mktemp)"
trap 'rm -f "$reference"' EXIT
# Reconstruct the prior module after the provider alias and obsolete lint removal.
perl -0777 -pe '
    s/^use sha2_11 as sha2;\n//m == 1 or die "Missing SHA-2 provider alias\n";
    my $lint = "    #[expect(\n        deprecated,\n        reason = \"Preserve the exact upstream cryptographic derivation body.\"\n    )]\n";
    s/^(    pub fn self_authenticating)/$lint$1/m == 1 or die "Missing derivation\n";
' "$source_root/self_authenticating/mod.rs" > "$reference"
while read -r hash file; do
    source="$source_root/$file"
    if [[ "$file" == self_authenticating/mod.rs ]]; then source="$reference"; fi
    bash "$root/scripts/ci/verify-file-checksum.sh" sha256 "$hash" "$source"
done <<'HASHES'
50a285820fd9e9764ab8e1863a2f4f290fca7d127e7511c8ee7f43fb363efb6a  self_authenticating/mod.rs
166bd8e8cf7790087d1fd18a9fa4d060cc0d0b3e5ab30689aa5f3a59a93386bf  LICENSE-APACHE
HASHES
# The body also depends on the unchanged 29-byte representation limit.
grep -Fx '    pub const MAX_LENGTH_IN_BYTES: usize = 29;' "$source_root/mod.rs" > /dev/null
perl -0777 -ne 'm{(fn self_authentication_keeps_upstream_vectors\(\).*\z)}s or die "missing vectors\n"; print $1' \
    "$source_root/tests/mod.rs" > "$reference"
bash "$root/scripts/ci/verify-file-checksum.sh" sha256 \
    089a01f08234b58a84cb8a2cd333241db6f078cfa6cdd56f6be9cea87faf4cc4 "$reference"
echo 'Principal self-authentication derivation, vectors, capacity and Apache license match the qualified import.'
