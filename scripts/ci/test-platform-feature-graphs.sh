#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/platform-feature-admission.XXXXXX")"
trap 'rm -rf "$fixture"' EXIT
count=0
while IFS='|' read -r selection package expected; do
    printf 'ic-platform v0.1.13\n%s\n' "$package" > "$fixture/native-case.txt"
    printf '%s\tcase\n' "$selection" > "$fixture/mapping"
    actual=reject
    if awk -v target=native -v cache="$fixture" -v generator='ic-platform-macros v0.1.13 (proc-macro) (/fixture)' \
        -f "$root/scripts/ci/check-platform-feature-graphs.awk" "$fixture/mapping" > "$fixture/log" 2>&1; then actual=pass; fi
    [[ "$actual" == "$expected" ]] || { echo "Unexpected $actual for $selection: $package" >&2; cat "$fixture/log" >&2; exit 1; }
    count=$((count + 1))
done <<'CASES'
none||pass
none|candid v0.10.38|reject
none|candid_derive v0.10.38|reject
candid|candid v0.10.38|pass
none|candid_parser v0.4.1|reject
candid-parser|candid_parser v0.4.1|pass
none|ic-agent v0.49.2|reject
agent|ic-agent v0.49.2|pass
none|ic-secp256k1 v0.3.0|reject
secp256k1|ic-secp256k1 v0.3.0|pass
secp256k1||reject
secp256k1|ic-secp256k1 v0.3.1|reject
secp256k1|ic-secp256k1 v0.3.0 (/local)|reject
none|pocket-ic v16.1.0|reject
none|ic-testkit v0.54.0|reject
none|ic-testkit-candid v0.53.0|reject
none|ic-cdk-executor v2.0.0|reject
executor|ic-cdk-executor v2.0.0|pass
executor|ic-cdk-executor v1.0.2|reject
executor|ic-cdk-executor v2.0.0 (/local)|reject
cdk|ic-cdk v0.20.3|pass
cdk|ic-cdk v0.19.0|reject
cdk|ic-cdk-timers v1.0.0|reject
cdk|ic-cdk-macros v0.20.3 (proc-macro)|pass
cdk|ic-cdk-macros v0.20.3 (proc-macro) (/local)|reject
cdk|ic-platform-macros v0.1.13 (proc-macro) (/fixture)|pass
none|ic-platform-macros v0.1.13 (proc-macro) (/fixture)|reject
cdk|ic-platform-macros v0.1.12 (proc-macro) (/fixture)|reject
ic0|ic0 v1.2.0|pass
ic0|ic0 v1.1.0|reject
ic0|ic0 v1.2.0 (/local)|reject
canister-sig-creation|ic-canister-sig-creation v1.3.1|pass
canister-sig-creation|ic-canister-sig-creation v1.3.2|reject
signature-verification|ic-signature-verification v0.3.0|pass
signature-verification|ic-signature-verification v0.2.0|reject
bls|ic-verify-bls-signature v0.6.0|pass
bls|ic_bls12_381 v0.10.1|pass
bls|ic_bls12_381 v0.10.2|reject
bls|ic-verify-bls-signature v0.6.0 (/local)|reject
certification|ic-certification v3.2.0|pass
certification|ic-certification v3.2.1|reject
none|ic-management-canister-types v0.11.0|reject
management-canister-types|ic-management-canister-types v0.11.0|pass
none|icrc-ledger-types v0.2.0|reject
icrc-ledger-types|icrc-ledger-types v0.2.0|pass
icrc-ledger-types|ic-stable-structures v0.7.2|reject
CASES
# A shared key must still be admitted separately for each original selection.
printf 'ic-platform v0.1.13\nic-agent v0.49.2\n' > "$fixture/native-case.txt"
printf 'agent\tcase\nnone\tcase\n' > "$fixture/mapping"
if awk -v target=native -v cache="$fixture" -f "$root/scripts/ci/check-platform-feature-graphs.awk" "$fixture/mapping" > "$fixture/log" 2>&1; then exit 1; fi
# Missing graphs fail closed rather than being treated as an empty package set.
rm "$fixture/native-case.txt"
if awk -v target=native -v cache="$fixture" -f "$root/scripts/ci/check-platform-feature-graphs.awk" "$fixture/mapping" > "$fixture/log" 2>&1; then exit 1; fi
printf 'Batch graph admission: %s cases plus shared-key and missing-graph refusals passed\n' "$count"
