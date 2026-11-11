#!/usr/bin/env bash
# Focused graph admission tests; no compilation or network required.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
node - "$root" > "$tmp/base.json" <<'JS'
const root = process.argv[2];
const packages = [], nodes = [];
function add(id, name, local, deps = [], version = '1.0.0') {
    packages.push({id, name, version, source: local ? null : 'registry+https://github.com/rust-lang/crates.io-index',
        manifest_path: local ? `${root}/crates/${name}/Cargo.toml` : `/registry/${id}/Cargo.toml`});
    nodes.push({id, deps: deps.map(pkg => ({pkg, dep_kinds: [{kind: null, target: null}]}))});
}
add('fixture', 'proof', false, ['platform', 'runtime', 'testkit']);
add('platform', 'ic-platform', true, ['types', 'agent']);
add('runtime', 'ic-platform-runtime', true, ['types', 'macros', 'guard']);
add('types', 'ic-platform-types', true);
add('macros', 'ic-platform-macros', true);
add('guard', 'ic-platform-runtime-guard', true);
add('agent', 'ic-agent', false, ['principal']);
add('principal', 'ic_principal', false);
add('testkit', 'ic-testkit', false, ['client']);
add('client', 'pocket-ic', false, ['management', 'candid']);
add('old-platform', 'ic-platform', false, ['old-types']);
add('old-types', 'ic-platform-types', false);
add('management', 'ic-management-canister-types', false, ['candid']);
add('candid', 'candid', false, ['principal']);
add('unused-runtime', 'ic-platform-runtime', false);
console.log(JSON.stringify({packages, resolve: {nodes}}));
JS
check() {
    bash "$root/scripts/ci/check-qualification-identities.sh" "$1" "$root" proof \
        ic-platform,ic-platform-runtime,ic-platform-macros,ic-platform-runtime-guard,ic-testkit "${@:2}"
}
reject() {
    if check "$tmp/case.json" "${@}" > "$tmp/out" 2>&1; then
        echo 'identity mismatch was accepted' >&2; exit 1
    fi
}
check "$tmp/base.json" > "$tmp/report.json"
jq -e '.selected_owners | length == 5' "$tmp/report.json" >/dev/null
jq -e '(.infrastructure[0].platform_owners | length == 0) and (.public_type_crossings | length == 1)' "$tmp/report.json" >/dev/null
jq '(.resolve.nodes[] | select(.id == "testkit").deps) = []' "$tmp/base.json" > "$tmp/case.json"
reject
jq '(.packages[] | select(.id == "platform").manifest_path) = "/wrong/Cargo.toml"' "$tmp/base.json" > "$tmp/case.json"
reject
jq '(.packages[] | select(.id == "platform").source) = "registry+https://github.com/rust-lang/crates.io-index"' "$tmp/base.json" > "$tmp/case.json"
reject
jq '(.resolve.nodes[] | select(.id == "runtime").deps[0].pkg) = "old-types"' "$tmp/base.json" > "$tmp/case.json"
reject
jq '(.resolve.nodes[] | select(.id == "fixture").deps) |= map(select(.pkg != "runtime"))' "$tmp/base.json" > "$tmp/case.json"
reject
jq '.packages += [(.packages[] | select(.id == "principal") | .id = "other-principal")] | (.resolve.nodes[] | select(.id == "candid").deps[0].pkg) = "other-principal"' "$tmp/base.json" > "$tmp/case.json"
reject
jq '(.packages[] | select(.source == null).source) = "registry+https://github.com/rust-lang/crates.io-index"' "$tmp/base.json" > "$tmp/published.json"
check "$tmp/published.json" published 1.0.0 > /dev/null
for owner in runtime macros guard; do
    jq --arg owner "$owner" '(.packages[] | select(.id == $owner).version) = "2.0.0"' "$tmp/published.json" > "$tmp/case.json"
    reject published 1.0.0
done
printf 'Qualification identity admission tests passed\n'
