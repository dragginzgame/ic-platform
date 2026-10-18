#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/platform-runtime-graph.XXXXXX")"
trap 'rm -rf "$fixture"' EXIT
cat > "$fixture/control" <<'GRAPH'
platform-runtime-canister v0.0.0
ic-cdk v0.20.3
ic-cdk-macros v0.20.3 (proc-macro)
ic-cdk-executor v2.0.0
ic-cdk-executor v2.0.0 (*)
ic0 v1.2.0
ic0 v1.2.0 (*)
GRAPH
bash "$root/scripts/ci/check-platform-runtime-graph.sh" "$fixture/control"
for dependency in 'candid_parser v0.4.0' 'ic-agent v0.49.2' 'ic-secp256k1 v0.3.0' 'pocket-ic v16.1.0' \
    'ic-testkit v0.54.0' 'ic-testkit-candid v0.53.0' 'ic-cdk-executor v1.0.2' \
    'ic0 v1.1.0' 'ic-cdk v0.19.0' 'ic-cdk-macros v0.19.0' 'ic-cdk-timers v1.0.0'; do
    cp "$fixture/control" "$fixture/rejected"
    printf '%s\n' "$dependency" >> "$fixture/rejected"
    if bash "$root/scripts/ci/check-platform-runtime-graph.sh" "$fixture/rejected" > "$fixture/log" 2>&1; then
        echo "Unexpectedly admitted $dependency" >&2; exit 1
    fi
done
for missing in ic0 ic-cdk-executor; do
    awk -v missing="$missing" '$1 != missing' "$fixture/control" > "$fixture/rejected"
    if bash "$root/scripts/ci/check-platform-runtime-graph.sh" "$fixture/rejected" > "$fixture/log" 2>&1; then
        echo "Unexpectedly admitted missing $missing" >&2; exit 1
    fi
done
echo 'Runtime graph control and thirteen rejection cases passed'
