#!/usr/bin/env bash
# Consumer policy proof: the independent oracle remains admitted, new application
# declarations and stale allowances refuse. Parser fixtures are not compiled.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
mkdir -p "$root/target"
fixture="$(mktemp -d "$root/target/platform-candid-boundary.XXXXXX")"
trap 'printf "Candid policy evidence: %s\n" "$fixture"' EXIT
source_path=crates/ic-platform/src/tests/reference/mod.rs
mkdir -p "$fixture/$(dirname "$source_path")"
cp "$root/$source_path" "$fixture/$source_path"
jq --arg path "$source_path" '{schema,allowances:[.allowances[] | select(.path==$path)]}' \
    "$root/ci/candid-boundary.json" > "$fixture/policy.json"
check=(perl "$root/scripts/ci/check-candid-boundary.pl" "$fixture" policy.json "$source_path")
"${check[@]}"
printf '\n#[derive(CandidType)] struct ApplicationCanary;\n' >> "$fixture/$source_path"
if "${check[@]}" > "$fixture/unreviewed.log" 2>&1; then
    echo 'Unreviewed application Candid declaration was admitted' >&2; exit 1
fi
cp "$root/$source_path" "$fixture/$source_path"
sed 's/fn decode_one/fn renamed_decode_one/' "$fixture/$source_path" > "$fixture/changed.rs"
cp "$fixture/changed.rs" "$fixture/$source_path"
if "${check[@]}" > "$fixture/stale.log" 2>&1; then
    echo 'Stale external allowance was admitted' >&2; exit 1
fi
echo 'Platform Candid policy admits the test oracle and refuses new/stale declarations.'

# Check the consumer's real codec allowance, then prove another codec and a
# stale exception cannot hide behind that file-level review.
codec_path=crates/ic-platform/src/certification/cbor/mod.rs
mkdir -p "$fixture/$(dirname "$codec_path")"
cp "$root/$codec_path" "$fixture/$codec_path"
jq --arg path "$codec_path" '{schema,allowances:[],cbor_allowances:[.cbor_allowances[] | select(.path==$path)]}' \
    "$root/ci/candid-boundary.json" > "$fixture/cbor-policy.json"
cbor_check=(perl "$root/scripts/ci/check-candid-boundary.pl" --cbor "$fixture" cbor-policy.json "$codec_path")
"${cbor_check[@]}"
printf '\nuse minicbor as UnreviewedCodec;\n' >> "$fixture/$codec_path"
if "${cbor_check[@]}" > "$fixture/cbor-new.log" 2>&1; then
    echo 'Unreviewed CBOR codec was admitted' >&2; exit 1
fi
printf '// Codec removed; its allowance must also be removed.\n' > "$fixture/$codec_path"
if "${cbor_check[@]}" > "$fixture/cbor-stale.log" 2>&1; then
    echo 'Stale CBOR allowance was admitted' >&2; exit 1
fi
echo 'Platform CBOR policy admits its codec owner and refuses new/stale codec allowances.'
