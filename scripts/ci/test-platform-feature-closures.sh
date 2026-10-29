#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
jq="$root/.tools/host/bin/jq"
# Deliberately include a transitive implication and a dependency-feature edge.
# The latter must not be mistaken for a local feature.
metadata='[{"packages":[{"name":"ic-platform","features":{"default":[],"candid":["dep:candid","candid/printer"],"executor":[],"ic0":[],"cdk":["executor","ic0","dep:ic-platform-macros"],"nested":["cdk","candid"],"agent":["dep:ic-agent"]}}]}]'
actual="$(printf '%s' '["none","default","cdk","ic0,executor,candid,cdk","nested","nested,candid","agent","agent,cdk"]' |
    "$jq" -r --arg target native --argjson metadata "$metadata" --slurpfile policy scripts/ci/platform-feature-policy.json -f scripts/ci/platform-feature-closures.jq)"
expected="$(printf 'none\tnone\ndefault\tdefault\ncdk\tcdk,executor,ic0\nic0,executor,candid,cdk\tcandid,cdk,executor,ic0\nnested\tcandid,cdk,executor,ic0,nested\nnested,candid\tcandid,cdk,executor,ic0,nested\nagent\tagent\nagent,cdk\tagent,cdk,executor,ic0')"
[[ "$actual" == "$expected" ]] || { printf 'Unexpected closures:\n%s\n' "$actual" >&2; exit 1; }
echo 'Feature closure equivalence and isolation cases passed'
long_key="$(printf '%0300d' 0)"
actual="$(printf 'first\t%s\nsecond\t%s\nthird\tother\n' "$long_key" "$long_key" |
    awk -f scripts/ci/platform-feature-cache-keys.awk)"
[[ "$actual" == $'first\tgraph-1\nsecond\tgraph-1\nthird\tgraph-2' ]] || {
    echo 'Short graph IDs lost closure identity' >&2; exit 1;
}
if printf 'invalid\n' | awk -f scripts/ci/platform-feature-cache-keys.awk >/dev/null 2>&1; then
    echo 'Invalid closure mapping was admitted' >&2; exit 1;
fi
echo 'Long closure names retain distinct short cache IDs'
# Native-only leaves may be removed from a Wasm key only after exact admission.
native_metadata="$(printf '%s' "$metadata" | "$jq" '
  .[0].packages[0] |= (.features += {principal:[],agent:["principal","dep:ic-agent"],"candid-parser":["principal","dep:candid_parser"],secp256k1:["principal","dep:ic-secp256k1"],"pocket-ic":["principal","dep:pocket-ic"],"wire-schema":["principal","dep:ic-wire-metadata","dep:serde_json"]} |
  .dependencies = ["ic-agent","candid_parser","ic-secp256k1","pocket-ic","ic-wire-metadata","serde_json"] | .dependencies |= map({name:.,optional:true,target:"cfg(not(target_family = \"wasm\"))"}))')"
selections='["none","default","agent","agent,candid-parser,secp256k1,pocket-ic","agent,cdk"]'
actual="$(printf '%s' "$selections" | "$jq" -r --arg target wasm32-unknown-unknown --argjson metadata "$native_metadata" --slurpfile policy scripts/ci/platform-feature-policy.json -f scripts/ci/platform-feature-closures.jq)"
expected="$(printf 'none\tnone\ndefault\tdefault\nagent\tprincipal\nagent,candid-parser,secp256k1,pocket-ic\tprincipal\nagent,cdk\tcdk,executor,ic0,principal')"
[[ "$actual" == "$expected" ]] || { echo 'Unexpected target-aware closure' >&2; exit 1; }
for feature in agent candid-parser secp256k1 pocket-ic wire-schema; do
    # shellcheck disable=SC2016 # $feature is a jq variable.
    changed="$(printf '%s' "$native_metadata" | "$jq" --arg feature "$feature" '.[0].packages[0].features[$feature] += ["candid"]')"
    if printf '%s' "$selections" | "$jq" --arg target wasm32-unknown-unknown --argjson metadata "$changed" --slurpfile policy scripts/ci/platform-feature-policy.json -f scripts/ci/platform-feature-closures.jq >/dev/null 2>&1; then
        echo "Admitted non-leaf feature $feature" >&2; exit 1
    fi
done
for change in '.dependencies[0].target = null' '.dependencies[1].optional = false' '.dependencies += [.dependencies[2]]' '.dependencies[0].rename = "renamed"'; do
    changed="$(printf '%s' "$native_metadata" | "$jq" ".[0].packages[0] |= ($change)")"
    if printf '%s' "$selections" | "$jq" --arg target wasm32-unknown-unknown --argjson metadata "$changed" --slurpfile policy scripts/ci/platform-feature-policy.json -f scripts/ci/platform-feature-closures.jq >/dev/null 2>&1; then
        echo "Admitted changed dependency contract: $change" >&2; exit 1
    fi
done
echo 'Native-only Wasm equivalence and fail-closed contract cases passed'
# Both metadata dependencies must be admitted; retain Principal's implication.
selections='["principal","wire-schema","wire-schema,principal","wire-schema,cdk"]'
actual="$(printf '%s' "$selections" | "$jq" -r --arg target wasm32-unknown-unknown --argjson metadata "$native_metadata" --slurpfile policy scripts/ci/platform-feature-policy.json -f scripts/ci/platform-feature-closures.jq)"
expected="$(printf 'principal\tprincipal\nwire-schema\tprincipal\nwire-schema,principal\tprincipal\nwire-schema,cdk\tcdk,executor,ic0,principal')"
[[ "$actual" == "$expected" ]] || { echo 'Wire metadata implication lost' >&2; exit 1; }
actual="$(printf '%s' '["wire-schema"]' | "$jq" -r --arg target native --argjson metadata "$native_metadata" --slurpfile policy scripts/ci/platform-feature-policy.json -f scripts/ci/platform-feature-closures.jq)"
[[ "$actual" == $'wire-schema\tprincipal,wire-schema' ]] || { echo 'Native metadata was collapsed' >&2; exit 1; }
for change in '.dependencies[4].target = null' '.dependencies[5].optional = false' '.dependencies |= map(select(.name != "serde_json"))' '.dependencies += [.dependencies[5]]'; do
    changed="$(printf '%s' "$native_metadata" | "$jq" ".[0].packages[0] |= ($change)")"
    if printf '%s' "$selections" | "$jq" --arg target wasm32-unknown-unknown --argjson metadata "$changed" --slurpfile policy scripts/ci/platform-feature-policy.json -f scripts/ci/platform-feature-closures.jq >/dev/null 2>&1; then
        echo "Admitted changed metadata contract: $change" >&2; exit 1
    fi
done
echo 'Wire metadata target isolation and both dependency guards passed'

fixture="$(mktemp -d "${TMPDIR:-/tmp}/platform-native-closures.XXXXXX")"
trap 'rm -rf "$fixture"' EXIT
for change in 'del(.native_leaves.agent.prerequisites)' '.native_leaves.agent.prerequisites = "principal"' '.native_leaves.agent.dependencies = []'; do
    "$jq" "$change" scripts/ci/platform-feature-policy.json > "$fixture/policy.json"
    if printf '%s' '["agent"]' | "$jq" --arg target wasm32-unknown-unknown --argjson metadata "$native_metadata" --slurpfile policy "$fixture/policy.json" -f scripts/ci/platform-feature-closures.jq >/dev/null 2>&1; then
        echo "Malformed native policy was admitted: $change" >&2; exit 1
    fi
done
cargo metadata --no-deps --format-version 1 --locked --offline > "$fixture/metadata.json"
actual="$(printf '%s' '["none","executor","agent","agent,executor","wire-schema"]' | "$jq" -r --arg target wasm32-unknown-unknown --slurpfile metadata "$fixture/metadata.json" --slurpfile policy scripts/ci/platform-feature-policy.json -f scripts/ci/platform-feature-closures.jq)"
expected="$(printf 'none\tnone\nexecutor\texecutor\nagent\tprincipal\nagent,executor\texecutor,principal\nwire-schema\tprincipal')"
[[ "$actual" == "$expected" ]] || { echo 'Actual Cargo native-leaf closure failed' >&2; exit 1; }
echo 'Actual native/Wasm closures and explicit prerequisite guards passed'
