#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
jq="$root/.tools/host/bin/jq"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/platform-selections.XXXXXX")"
trap 'rm -rf "$fixture"' EXIT
cargo metadata --no-deps --format-version 1 --frozen > "$fixture/metadata.json"
"$jq" '.packages[] | select(.name == "ic-platform") | .features | keys - ["default"]' "$fixture/metadata.json" > "$fixture/features.json"
contract() {
    "$jq" -e --slurpfile policy scripts/ci/platform-feature-policy.json \
        -f scripts/ci/platform-feature-contract.jq "$1" >/dev/null
}
contract "$fixture/metadata.json"
for selection in 'map(select(.name != "ic-platform"))' '. + map(select(.name == "ic-platform"))'; do
    "$jq" ".packages |= ($selection)" "$fixture/metadata.json" > "$fixture/invalid.json"
    if contract "$fixture/invalid.json" 2> "$fixture/error"; then
        echo 'Missing or ambiguous Platform package was admitted' >&2; exit 1
    fi
    grep -q 'expected exactly one ic-platform package' "$fixture/error"
done
# Mutate actual Cargo observations, retaining each policy's refusal evidence.
while IFS= read -r mutation; do
    "$jq" "(.packages[] | select(.name == \"ic-platform\")) |= ($mutation)" \
        "$fixture/metadata.json" > "$fixture/invalid.json"
    if contract "$fixture/invalid.json" 2> "$fixture/error"; then
        echo "Feature contract admitted mutation: $mutation" >&2; exit 1
    fi
    grep -q 'jq: error' "$fixture/error"
done <<'MUTATIONS'
.features.candid = []
.features.extra = []
.features.default = ["runtime"]
.features.executor = []
.features.cdk = []
.features.secp256k1 = []
.features.ic0 = ["dep:ic0"]
(.dependencies[] | select(.name == "ic-management-canister-types")).req = "^0.11"
(.dependencies[] | select(.name == "ic-management-canister-types")).target = null
.features["icrc-ledger-types"] += ["dep:icrc-ledger-types"]
.features.principal += ["dep:ic_principal"]
(.dependencies[] | select(.name == "ic_principal")).features += ["self_authenticating"]
(.dependencies[] | select(.name == "ic-verify-bls-signature")).uses_default_features = true
(.dependencies[] | select(.name == "ed25519-dalek")).uses_default_features = true
(.dependencies[] | select(.name == "ic-stable-structures")).req = "^0.7"
(.dependencies[] | select(.name == "k256")).uses_default_features = true
.links = null
.dependencies += [{"name":"ic0"}]
MUTATIONS
"$jq" --arg mode routine --slurpfile policy scripts/ci/platform-feature-policy.json -f scripts/ci/platform-feature-selections.jq "$fixture/features.json" > "$fixture/routine.json"
# Prove powerset completeness on a bounded universe. Ordinary qualification must
# not materialize millions of production combinations before its routine matrix.
printf '%s\n' '["alpha","beta","gamma","delta"]' > "$fixture/small.json"
"$jq" --arg mode exhaustive --slurpfile policy scripts/ci/platform-feature-policy.json -f scripts/ci/platform-feature-selections.jq "$fixture/small.json" > "$fixture/exhaustive.json"
if "$jq" --arg mode exhaustive --slurpfile policy scripts/ci/platform-feature-policy.json -f scripts/ci/platform-feature-selections.jq "$fixture/features.json" > "$fixture/oversized.json" 2> "$fixture/oversized.log"; then
    echo 'Unbounded production power set was admitted' >&2; exit 1
fi
grep -q 'limited to 12' "$fixture/oversized.log"
if bash scripts/ci/check-platform-features.sh --exhaustive > "$fixture/missing.log" 2>&1; then
    echo 'Exhaustive audit without an explicit scope was admitted' >&2; exit 1
fi
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
    "agent,candid-parser,cdk,gzip,pocket-ic,secp256k1,wire-schema",
    "bls,canister-sig-creation,certification,ed25519,signature-verification",
    ($f - ["agent","candid-parser","gzip","pocket-ic","secp256k1","wire-schema"] | join(","))] - $r | length) == 0
' >/dev/null
if "$jq" --arg mode typo --slurpfile policy scripts/ci/platform-feature-policy.json -f scripts/ci/platform-feature-selections.jq "$fixture/features.json" >/dev/null 2>&1; then
    echo 'Unknown matrix mode was admitted' >&2; exit 1
fi
if printf '[]' | "$jq" --arg mode routine --slurpfile policy scripts/ci/platform-feature-policy.json -f scripts/ci/platform-feature-selections.jq >/dev/null 2>&1; then
    echo 'Missing critical feature was admitted' >&2; exit 1
fi
printf 'Feature selections: real singles/pairs/critical groups and bounded exhaustive completeness passed (%s routine, %s synthetic exhaustive).\n' \
    "$("$jq" length "$fixture/routine.json")" "$("$jq" length "$fixture/exhaustive.json")"
