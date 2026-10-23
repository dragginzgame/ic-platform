#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
jq="$root/.tools/host/bin/jq"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/platform-selections.XXXXXX")"
trap 'rm -rf "$fixture"' EXIT
cargo metadata --no-deps --format-version 1 --frozen |
    "$jq" '.packages[] | select(.name == "ic-platform") | .features | keys - ["default"]' > "$fixture/features.json"
for mode in routine exhaustive; do
    "$jq" --arg mode "$mode" -f scripts/ci/platform-feature-selections.jq "$fixture/features.json" > "$fixture/$mode.json"
done
"$jq" -en --slurpfile features "$fixture/features.json" --slurpfile routine "$fixture/routine.json" --slurpfile exhaustive "$fixture/exhaustive.json" '
  $features[0] as $f | $routine[0] as $r | $exhaustive[0] as $e |
  ($e | length) == (pow(2; $f | length) + 1) and
  ($e | unique | length) == ($e | length) and
  all($e[]; . == "none" or . == "default" or
    (split(",") as $s | ($s - $f | length) == 0 and ($s | unique) == $s)) and
  ($r - $e | length) == 0 and
  ($r | unique | length) == ($r | length) and
  (["none", "default", ($f | join(","))] - $r | length) == 0 and
  all($f[]; . as $a | ($r | index($a)) != null and
    all($f[] | select(. != $a); . as $b | ($r | index([$a,$b] | sort | join(","))) != null)) and
  (["candid,cdk,executor,ic0",
    "bls,ecdsa,ed25519",
    "cdk,certification,icrc-ledger-types,management-canister-types",
    "agent,candid-parser,cdk,pocket-ic,secp256k1,wire-schema",
    "bls,canister-sig-creation,certification,ed25519,signature-verification",
    ($f - ["agent","candid-parser","pocket-ic","secp256k1","wire-schema"] | join(","))] - $r | length) == 0
' >/dev/null
if "$jq" --arg mode typo -f scripts/ci/platform-feature-selections.jq "$fixture/features.json" >/dev/null 2>&1; then
    echo 'Unknown matrix mode was admitted' >&2; exit 1
fi
if printf '[]' | "$jq" --arg mode routine -f scripts/ci/platform-feature-selections.jq >/dev/null 2>&1; then
    echo 'Missing critical feature was admitted' >&2; exit 1
fi
printf 'Feature selections: singles, all pairs, critical groups and exhaustive completeness passed (%s routine, %s exhaustive).\n' \
    "$("$jq" length "$fixture/routine.json")" "$("$jq" length "$fixture/exhaustive.json")"
