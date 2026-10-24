#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
source_root="${1:-$root/crates/ic-platform/src/signature_upstream}"
[[ $# -le 1 ]] || exit 2
# Exact imported bytes, including original tests: only the documented import/module
# edits, 0.5.1 cache deletion and 0.6 reference-only cfg attributes were admitted.
# Never refresh these hashes to approve an algorithm change.
while read -r hash file; do
    bash "$root/scripts/ci/verify-file-checksum.sh" sha256 "$hash" "$source_root/$file"
done <<'HASHES'
874ec06ba0f460f3ae41514e20371d1388a84f494ea6122b1a9bb66585bfef31 creation/mod.rs
9b40b7b638b739421fcbdad5d860a789b5fffcfa939009f05896592757092dca verification/mod.rs
a7d816caf02fa4fbaced7723c2ef742f2b2c4b6e1347f42ecf3aba9692b0c3fc creation/LICENSE
6f44e3f228570e343cdb96bd929d1e8c6057ac33b411e47d77427f824e5a96ac verification/LICENSE
HASHES
# Strip only the eight reviewed packaging attributes and recover the exact 0.5.1 source.
reference="$(mktemp)"
trap 'rm -f "$reference"' EXIT
sed '/^#\[cfg(test)\] \/\/ Platform: reference-only\.$/d; /^#\[allow(unused_imports)\] \/\/ Platform: reference-only HashTree\.$/d' \
    "$source_root/creation/mod.rs" > "$reference"
bash "$root/scripts/ci/verify-file-checksum.sh" sha256 \
    d042a45c3f89fa9c5fa43eccf29bb0b64297f032ef052c4f49f0daab1f2be106 "$reference"
# Recover the unchanged verifier after removing only the additive child module.
sed '/^pub(crate) mod evidence; \/\/ Platform: authenticated time adapter\.$/d' \
    "$source_root/verification/mod.rs" > "$reference"
bash "$root/scripts/ci/verify-file-checksum.sh" sha256 \
    d434da3f0db0f8212c0e107da27f2d50fc8916f2db6b94874d092d76c8c878a5 "$reference"
printf 'Imported signature bodies, vectors and licenses match the qualified source.\n'
