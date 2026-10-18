#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
jq="$root/.tools/host/bin/jq"
# Deliberately include a transitive implication and a dependency-feature edge.
# The latter must not be mistaken for a local feature.
metadata='[{"packages":[{"name":"ic-platform","features":{"default":[],"candid":["dep:candid","candid/printer"],"executor":[],"ic0":[],"cdk":["candid","executor","ic0","dep:ic-cdk"],"nested":["cdk"],"agent":["dep:ic-agent"]}}]}]'
actual="$(printf '%s' '["none","default","cdk","ic0,executor,candid,cdk","nested","nested,candid","agent","agent,cdk"]' |
    "$jq" -r --arg target native --argjson metadata "$metadata" -f scripts/ci/platform-feature-closures.jq)"
expected="$(printf 'none\tnone\ndefault\tdefault\ncdk\tcandid,cdk,executor,ic0\nic0,executor,candid,cdk\tcandid,cdk,executor,ic0\nnested\tcandid,cdk,executor,ic0,nested\nnested,candid\tcandid,cdk,executor,ic0,nested\nagent\tagent\nagent,cdk\tagent,candid,cdk,executor,ic0')"
[[ "$actual" == "$expected" ]] || { printf 'Unexpected closures:\n%s\n' "$actual" >&2; exit 1; }
echo 'Feature closure equivalence and isolation cases passed'
# Native-only leaves may be removed from a Wasm key only after exact admission.
native_metadata="$(printf '%s' "$metadata" | "$jq" '
  .[0].packages[0] |= (.features += {"candid-parser":["dep:candid_parser"],secp256k1:["dep:ic-secp256k1"]} |
  .dependencies = ["ic-agent","candid_parser","ic-secp256k1"] | .dependencies |= map({name:.,optional:true,target:"cfg(not(target_family = \"wasm\"))"}))')"
selections='["none","default","agent","agent,candid-parser,secp256k1","agent,cdk"]'
actual="$(printf '%s' "$selections" | "$jq" -r --arg target wasm32-unknown-unknown --argjson metadata "$native_metadata" -f scripts/ci/platform-feature-closures.jq)"
expected="$(printf 'none\tnone\ndefault\tdefault\nagent\tnone\nagent,candid-parser,secp256k1\tnone\nagent,cdk\tcandid,cdk,executor,ic0')"
[[ "$actual" == "$expected" ]] || { echo 'Unexpected target-aware closure' >&2; exit 1; }
for feature in agent candid-parser secp256k1; do
    changed="$(printf '%s' "$native_metadata" | "$jq" --arg feature "$feature" '.[0].packages[0].features[$feature] += ["candid"]')"
    if printf '%s' "$selections" | "$jq" --arg target wasm32-unknown-unknown --argjson metadata "$changed" -f scripts/ci/platform-feature-closures.jq >/dev/null 2>&1; then
        echo "Admitted non-leaf feature $feature" >&2; exit 1
    fi
done
for change in '.dependencies[0].target = null' '.dependencies[1].optional = false' '.dependencies += [.dependencies[2]]' '.dependencies[0].rename = "renamed"'; do
    changed="$(printf '%s' "$native_metadata" | "$jq" ".[0].packages[0] |= ($change)")"
    if printf '%s' "$selections" | "$jq" --arg target wasm32-unknown-unknown --argjson metadata "$changed" -f scripts/ci/platform-feature-closures.jq >/dev/null 2>&1; then
        echo "Admitted changed dependency contract: $change" >&2; exit 1
    fi
done
echo 'Native-only Wasm equivalence and seven fail-closed contract cases passed'
