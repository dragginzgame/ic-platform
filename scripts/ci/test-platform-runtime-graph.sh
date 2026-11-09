#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
mkdir -p "$root/target"
fixture="$(mktemp -d "$root/target/platform-runtime-graph.XXXXXX")"
trap 'rm -rf "$fixture"' EXIT
cat > "$fixture/control" <<'GRAPH'
platform-runtime-canister v0.0.0
ic-platform-runtime v0.17.0 (/local/platform)
ic-platform-runtime v0.17.0 (/local/platform) (*)
GRAPH
bash "$root/scripts/ci/check-platform-runtime-graph.sh" "$fixture/control"
for dependency in 'candid v0.10.38' 'candid_derive v0.10.38' 'ic-cdk v0.20.3' 'ic-cdk-macros v0.20.3 (proc-macro)' 'candid_parser v0.4.0' 'ic-agent v0.49.2' 'ic-secp256k1 v0.3.0' 'pocket-ic v16.1.0' \
    'ic-cdk-executor v2.0.0' 'ic0 v1.2.0' 'ic-platform-runtime v0.16.0' 'ic-platform-runtime v0.17.0' 'ic-testkit v0.54.0' 'ic-testkit-candid v0.53.0' 'ic-cdk-executor v1.0.2' \
    'ic0 v1.1.0' 'ic-cdk v0.19.0' 'ic-cdk-macros v0.19.0' 'ic-cdk-timers v1.0.0' \
    'ic-cdk v0.20.3 (/tmp/replacement-cdk)' \
    'ic-cdk v0.20.3 (https://example.invalid/cdk#12345678)' \
    'ic-cdk-macros v0.20.3 (proc-macro) (/tmp/replacement-macros)' \
    'ic-cdk-macros v0.20.3 (proc-macro) (https://example.invalid/cdk#12345678)'; do
    cp "$fixture/control" "$fixture/rejected"
    printf '%s\n' "$dependency" >> "$fixture/rejected"
    if bash "$root/scripts/ci/check-platform-runtime-graph.sh" "$fixture/rejected" > "$fixture/log" 2>&1; then
        echo "Unexpectedly admitted $dependency" >&2; exit 1
    fi
done
awk '$1 != "ic-platform-runtime"' "$fixture/control" > "$fixture/rejected"
if bash "$root/scripts/ci/check-platform-runtime-graph.sh" "$fixture/rejected" > "$fixture/log" 2>&1; then
    echo 'Unexpectedly admitted missing ic-platform-runtime' >&2; exit 1
fi
echo 'Runtime graph control and rejection cases passed'
cp "$fixture/control" "$fixture/external"
printf 'candid v0.10.38\ncandid_derive v0.10.38\n' >> "$fixture/external"
if bash "$root/scripts/ci/check-platform-runtime-graph.sh" "$fixture/external" --external-candid > "$fixture/log" 2>&1; then exit 1; fi
if bash "$root/scripts/ci/check-platform-runtime-graph.sh" "$fixture/external" > "$fixture/log" 2>&1; then exit 1; fi
bash "$root/scripts/ci/check-platform-runtime-graph.sh" "$fixture/control" --external-candid
# Native and Wasm management both use owned records.
printf 'ic-management-canister-types v0.11.0\n' >> "$fixture/external"
if bash "$root/scripts/ci/check-platform-runtime-graph.sh" "$fixture/external" --external-candid > "$fixture/log" 2>&1; then exit 1; fi
if bash "$root/scripts/ci/check-platform-runtime-graph.sh" "$fixture/external" --external-candid --native-management > "$fixture/log" 2>&1; then exit 1; fi
printf 'ic-agent v0.49.2\n' >> "$fixture/external"
if bash "$root/scripts/ci/check-platform-runtime-graph.sh" "$fixture/external" --external-candid --native-management > "$fixture/log" 2>&1; then exit 1; fi
echo 'Retired native management exception and supplier graphs refuse'
# Selected registry signature creation may bring raw IC0, never an executor.
cp "$fixture/control" "$fixture/signature"
printf 'candid v0.10.39\nic-canister-sig-creation v1.3.1\nic0 v1.2.0\n' >> "$fixture/signature"
bash "$root/scripts/ci/check-platform-runtime-graph.sh" "$fixture/signature" --external-candid
for mutation in 's/ic0 v1.2.0/ic0 v1.1.0/' 's/ic0 v1.2.0/ic0 v1.2.0 (\/local)/' 's/ic-canister-sig-creation v1.3.1/ic-canister-sig-creation v1.3.2/' 's/ic-canister-sig-creation v1.3.1/ic-canister-sig-creation v1.3.1 (\/local)/'; do
    sed "$mutation" "$fixture/signature" > "$fixture/rejected"
    if bash "$root/scripts/ci/check-platform-runtime-graph.sh" "$fixture/rejected" --external-candid > "$fixture/log" 2>&1; then
        echo "Unexpectedly admitted signature IC0 supplier mutation: $mutation" >&2; exit 1
    fi
done
printf 'ic-cdk-executor v2.0.0\n' >> "$fixture/signature"
if bash "$root/scripts/ci/check-platform-runtime-graph.sh" "$fixture/signature" --external-candid > "$fixture/log" 2>&1; then exit 1; fi
echo 'Signature raw IC0 admission preserves executor/source isolation'
