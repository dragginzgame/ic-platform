#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
jq="$root/.tools/host/bin/jq"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/platform-selections.XXXXXX")"
trap 'rm -rf "$fixture"' EXIT
cargo metadata --no-deps --format-version 1 --frozen |
    "$jq" '.packages[] | select(.name == "ic-platform") | .features | keys - ["default"]' > "$fixture/features.json"
"$jq" --arg mode routine --slurpfile policy scripts/ci/platform-feature-policy.json -f scripts/ci/platform-feature-selections.jq "$fixture/features.json" > "$fixture/routine.json"
# Prove powerset completeness on a bounded universe. Ordinary qualification must
# not materialize millions of production combinations before its routine matrix.
printf '%s\n' '["alpha","beta","gamma","delta"]' > "$fixture/small.json"
"$jq" --arg mode exhaustive --slurpfile policy scripts/ci/platform-feature-policy.json -f scripts/ci/platform-feature-selections.jq "$fixture/small.json" > "$fixture/exhaustive.json"
# shellcheck disable=SC2016 # The quoted expression contains jq variables.
"$jq" -en --slurpfile features "$fixture/features.json" --slurpfile small "$fixture/small.json" --slurpfile routine "$fixture/routine.json" --slurpfile exhaustive "$fixture/exhaustive.json" '
  $features[0] as $f | $small[0] as $s | $routine[0] as $r | $exhaustive[0] as $e |
  ($e | length) == (pow(2; $s | length) + 1) and
  ($e | unique | length) == ($e | length) and
  all($e[]; . == "none" or . == "default" or
    (split(",") as $selection | ($selection - $s | length) == 0 and ($selection | unique) == $selection)) and
  all($r[]; . == "none" or . == "default" or
    (split(",") as $selection | ($selection - $f | length) == 0 and ($selection | unique) == $selection)) and
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
if "$jq" --arg mode typo --slurpfile policy scripts/ci/platform-feature-policy.json -f scripts/ci/platform-feature-selections.jq "$fixture/features.json" >/dev/null 2>&1; then
    echo 'Unknown matrix mode was admitted' >&2; exit 1
fi
if printf '[]' | "$jq" --arg mode routine --slurpfile policy scripts/ci/platform-feature-policy.json -f scripts/ci/platform-feature-selections.jq >/dev/null 2>&1; then
    echo 'Missing critical feature was admitted' >&2; exit 1
fi
printf 'Feature selections: real singles/pairs/critical groups and bounded exhaustive completeness passed (%s routine, %s synthetic exhaustive).\n' \
    "$("$jq" length "$fixture/routine.json")" "$("$jq" length "$fixture/exhaustive.json")"
