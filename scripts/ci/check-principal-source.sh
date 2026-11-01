#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
[[ $# -le 1 ]] || exit 2
source_root="${1:-$root/crates/ic-platform-types/src/principal}"
# Exact admitted module, including the original ic_principal 0.1.5 method body
# and tag. SHA-224 remains the upstream sha2 implementation.
while read -r hash file; do
    bash "$root/scripts/ci/verify-file-checksum.sh" sha256 "$hash" "$source_root/$file"
done <<'HASHES'
50a285820fd9e9764ab8e1863a2f4f290fca7d127e7511c8ee7f43fb363efb6a  self_authenticating/mod.rs
166bd8e8cf7790087d1fd18a9fa4d060cc0d0b3e5ab30689aa5f3a59a93386bf  LICENSE-APACHE
HASHES
# The body also depends on the unchanged 29-byte representation limit.
grep -Fx '    pub const MAX_LENGTH_IN_BYTES: usize = 29;' "$source_root/mod.rs" > /dev/null
vector_hash="$(perl -0777 -ne 'm{(fn self_authentication_keeps_upstream_vectors\(\).*\z)}s or die "missing vectors\n"; print $1' "$source_root/tests/mod.rs" | shasum -a 256)"
[[ "${vector_hash%% *}" == "089a01f08234b58a84cb8a2cd333241db6f078cfa6cdd56f6be9cea87faf4cc4" ]]
echo 'Principal self-authentication derivation, vectors, capacity and Apache license match the qualified import.'
