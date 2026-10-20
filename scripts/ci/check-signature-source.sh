#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
source_root="${1:-$root/crates/ic-platform/src/signature_upstream}"
[[ $# -le 1 ]] || exit 2
# Exact imported bytes, including original tests: only the documented import/module
# edits were admitted. Never refresh these hashes to approve an algorithm change.
while read -r hash file; do
    bash "$root/scripts/ci/verify-file-checksum.sh" sha256 "$hash" "$source_root/$file"
done <<'HASHES'
1d1376e918ef0eb0c77e7b9cd8ab74b522606dbab41961e5362b1626e6495100 creation/mod.rs
d434da3f0db0f8212c0e107da27f2d50fc8916f2db6b94874d092d76c8c878a5 verification/mod.rs
a7d816caf02fa4fbaced7723c2ef742f2b2c4b6e1347f42ecf3aba9692b0c3fc creation/LICENSE
6f44e3f228570e343cdb96bd929d1e8c6057ac33b411e47d77427f824e5a96ac verification/LICENSE
HASHES
printf 'Imported signature bodies, vectors and licenses match the qualified source.\n'
