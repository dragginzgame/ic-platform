#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
jq="$root/.tools/host/bin/jq"
# Deliberately include a transitive implication and a dependency-feature edge.
# The latter must not be mistaken for a local feature.
metadata='[{"packages":[{"name":"ic-platform","features":{"default":[],"candid":["dep:candid","candid/printer"],"executor":[],"ic0":[],"cdk":["candid","executor","ic0","dep:ic-cdk"],"nested":["cdk"],"agent":["dep:ic-agent"]}}]}]'
actual="$(printf '%s' '["none","default","cdk","ic0,executor,candid,cdk","nested","nested,candid","agent","agent,cdk"]' |
    "$jq" -r --argjson metadata "$metadata" -f scripts/ci/platform-feature-closures.jq)"
expected="$(printf 'none\tnone\ndefault\tdefault\ncdk\tcandid,cdk,executor,ic0\nic0,executor,candid,cdk\tcandid,cdk,executor,ic0\nnested\tcandid,cdk,executor,ic0,nested\nnested,candid\tcandid,cdk,executor,ic0,nested\nagent\tagent\nagent,cdk\tagent,candid,cdk,executor,ic0')"
[[ "$actual" == "$expected" ]] || { printf 'Unexpected closures:\n%s\n' "$actual" >&2; exit 1; }
echo 'Feature closure equivalence and isolation cases passed'
