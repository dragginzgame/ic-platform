#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
mkdir -p "$root/target"
fixture="$(mktemp -d "$root/target/platform-feature-admission.XXXXXX")"
trap 'rm -rf "$fixture"' EXIT
cd "$root"
jq="$root/.tools/host/bin/jq"
cargo metadata --no-deps --format-version 1 --locked --offline > "$fixture/metadata.json"
admit() {
    local graph_target=native arg
    for arg in "$@"; do case "$arg" in target=*) graph_target="${arg#target=}" ;; esac; done
    cut -f1 "$fixture/mapping" | "$jq" -Rsc 'split("\n")[:-1]' |
        "$jq" -r --arg projection activation --arg target "$graph_target" \
            --slurpfile metadata "$fixture/metadata.json" --slurpfile policy "$root/scripts/ci/platform-feature-policy.json" \
            -f "$root/scripts/ci/platform-feature-closures.jq" > "$fixture/activation.tsv"
    awk -v activation="$fixture/activation.tsv" -f "$root/scripts/ci/platform-graph-policy.awk" "$@"
}
count=0
while IFS='|' read -r selection package expected; do
    printf 'ic-platform v0.1.13\n%s\n' "$package" > "$fixture/native-case.txt"
    # Supply the required guard for ordinary runtime cases; explicit guard cases
    # below exercise identity and feature-selection refusals.
    if [[ "$package" != ic-platform-runtime-guard* && ",$selection," =~ ,(executor|runtime|candid-endpoint), ]]; then
        printf 'ic-platform-runtime-guard v0.1.13 (/fixture)\n' >> "$fixture/native-case.txt"
    fi
    printf '%s\tcase\n' "$selection" > "$fixture/mapping"
    actual=reject
    if admit -v target=native -v cache="$fixture" -v guard='ic-platform-runtime-guard v0.1.13 (/fixture)' -v generator='ic-platform-macros v0.1.13 (proc-macro) (/fixture)' \
        -f "$root/scripts/ci/check-platform-feature-graphs.awk" "$fixture/mapping" > "$fixture/log" 2>&1; then actual=pass; fi
    [[ "$actual" == "$expected" ]] || { echo "Unexpected $actual for $selection: $package" >&2; cat "$fixture/log" >&2; exit 1; }
    count=$((count + 1))
done <<'CASES'
none||pass
executor|ic-platform-runtime-guard v0.1.13 (/fixture)|pass
none|ic-platform-runtime-guard v0.1.13 (/fixture)|reject
ic0|ic-platform-runtime-guard v0.1.13 (/fixture)|reject
executor|ic-platform-runtime-guard v0.1.12 (/fixture)|reject
runtime|ic-platform-runtime-guard v0.1.13 (/other)|reject
none|ic-wire v0.6.4|reject
principal|ic-wire v0.6.4|reject
principal|ic-wire-derive v0.9.0|reject
principal|ic-platform-runtime v0.17.0 (/fixture)|reject
executor|ic_principal v0.1.5|reject
ic0|serde v1.0.229|reject
executor,ic0|sha2 v0.10.9|reject
executor,ic0|serde_derive v1.0.229 (proc-macro)|reject
principal|ic_principal v0.1.5|reject
canister-sig-creation|ic_principal v0.1.5|pass
signature-verification|ic_principal v0.1.5|pass
signature-verification,principal-external|ic_principal v0.1.5|pass
principal-external|ic_principal v0.1.5|pass
principal-external|sha2 v0.10.9|reject
principal-self-authenticating|sha2 v0.10.9|pass
principal-self-authenticating|ic_principal v0.1.5|reject
principal-self-authenticating,principal-external|ic_principal v0.1.5|pass
runtime|ic_principal v0.1.5|reject
runtime|sha2 v0.10.9|reject
runtime|digest v0.10.7|reject
principal,executor|serde v1.0.229|pass
canister-sig-creation|ic-representation-independent-hash v3.2.0|pass
signature-verification|ic-representation-independent-hash v3.2.0|pass
canister-sig-creation|serde_cbor v0.11.2|pass
canister-sig-creation|half v1.8.3|pass
signature-verification|serde_cbor v0.11.2|pass
none|rand v0.8.5|reject
executor|rand_chacha v0.3.1|reject
runtime|getrandom v0.2.17|reject
bls|rand v0.8.5|reject
signature-verification|rand v0.8.5|reject
none|rand_core v0.6.4|pass
agent|rand v0.8.5|pass
candid-parser|getrandom v0.2.17|pass
none|k256 v0.13.4|reject
executor|k256 v0.13.4|reject
ecdsa|k256 v0.13.4|pass
ecdsa||reject
ecdsa|k256 v0.13.4 (/local)|reject
none|ed25519-dalek v2.2.0|reject
executor|curve25519-dalek v4.1.3|reject
ed25519||reject
ed25519|ed25519-dalek v2.2.0|reject
none|candid v0.10.38|reject
ic-protocol|candid v0.10.38|pass
ic-protocol|pretty v0.12.5|pass
ic-protocol|ic-transport-types v0.49.3|pass
ic-protocol-fixtures|ic-agent v0.49.3|reject
certification-cbor|ic-transport-types v0.49.3|reject
runtime|candid v0.10.38|reject
candid-endpoint|candid v0.10.38|pass
candid-endpoint|pretty v0.12.5|reject
candid-endpoint|ic-cdk-executor v2.0.0|reject
candid-endpoint|ic-platform-macros v0.1.13 (proc-macro) (/fixture)|pass
runtime|ic-cdk-executor v2.0.0|reject
runtime|ic0 v1.2.0|reject
runtime|ic-platform-macros v0.1.13 (proc-macro) (/fixture)|reject
runtime|candid_derive v0.10.38|reject
runtime,candid|candid v0.10.38|pass
none|candid_derive v0.10.38|reject
candid|candid v0.10.38|pass
cmc-types|candid v0.10.38|pass
icp-ledger-types|candid_derive v0.10.38|pass
cycles-ledger-types|ic_principal v0.1.5|pass
nns-registry-types|serde v1.0.229|pass
certified-assets-types|candid v0.10.38|pass
cmc-types|candid_parser v0.4.1|reject
cycles-ledger-types|ic-cdk v0.19.0|reject
icp-ledger-types|icrc-ledger-types v0.2.0|reject
certified-assets-types|ic-agent v0.49.3|reject
nns-registry-types|pocket-ic v16.1.0|reject
candid-service|candid v0.10.38|pass
candid|pretty v0.12.5|reject
runtime,candid|pretty v0.12.5|reject
candid-service|pretty v0.12.5|pass
candid-parser|pretty v0.12.5|pass
management-canister-types|pretty v0.12.5|pass
none|candid_parser v0.4.1|reject
candid-parser|candid_parser v0.4.1|pass
none|ic-agent v0.49.2|reject
agent|ic-agent v0.49.2|pass
none|ic-secp256k1 v0.3.0|reject
agent|ic-secp256k1 v0.3.0|reject
ecdsa|ic-secp256k1 v0.3.0|reject
none|pocket-ic v16.1.0|reject
none|ic-testkit v0.54.0|reject
none|ic-testkit-candid v0.53.0|reject
none|ic-cdk-executor v2.0.0|reject
executor|ic-cdk-executor v2.0.0|reject
executor|ic-cdk-executor v1.0.2|reject
executor|ic-cdk-executor v2.0.0 (/local)|reject
runtime|ic-cdk v0.20.3|reject
runtime|ic-cdk v0.19.0|reject
runtime|ic-cdk-timers v1.0.0|reject
runtime|ic-cdk-macros v0.20.3 (proc-macro)|reject
runtime|ic-cdk-macros v0.20.3 (proc-macro) (/local)|reject
none|ic-platform-macros v0.1.13 (proc-macro) (/fixture)|reject
candid-endpoint|ic-platform-macros v0.1.12 (proc-macro) (/fixture)|reject
ic0|ic0 v1.2.0|reject
ic0|ic0 v1.1.0|reject
ic0|ic0 v1.2.0 (/local)|reject
canister-sig-creation|ic-canister-sig-creation v1.3.1|pass
canister-sig-creation|candid v0.10.38|pass
signature-verification|candid v0.10.38|pass
canister-sig-creation|ic0 v1.2.0|pass
signature-verification|ic0 v1.2.0|pass
runtime|ic-error-types v0.2.0|reject
canister-sig-creation|ic-canister-sig-creation v1.3.2|reject
signature-verification|ic-signature-verification v0.3.0|pass
signature-verification|ic-signature-verification v0.2.0|reject
bls|ic-verify-bls-signature v0.6.0|pass
signature-verification|ic-verify-bls-signature v0.6.0|pass
agent|ic-verify-bls-signature v0.6.0|pass
bls|ic_bls12_381 v0.10.1|pass
signature-verification|ic_bls12_381 v0.10.1|pass
agent|ic_bls12_381 v0.10.1|pass
bls|ic_bls12_381 v0.10.2|reject
bls|ic-verify-bls-signature v0.6.0 (/local)|reject
certification|ic-certification v3.2.0|pass
certification-cbor|ic-certification v3.2.0|pass
certification-cbor|serde_cbor v0.11.2|pass
certification-cbor|half v1.8.3|pass
certification-cbor|candid v0.10.38|reject
certification-cbor|ic-platform-runtime v0.19.2 (/fixture)|reject
canister-sig-creation|ic-certification v3.2.0|pass
signature-verification|ic-certification v3.2.0|pass
agent|ic-certification v3.2.0|pass
certification|ic-certification v3.2.1|reject
none|ic-management-canister-types v0.11.0|reject
management-canister-types|ic-management-canister-types v0.11.0|pass
none|icrc-ledger-types v0.2.0|reject
icrc-ledger-types|icrc-ledger-types v0.2.0|reject
icrc-ledger-types|icrc-cbor v0.1.0|reject
icrc-ledger-types|pretty v0.12.5|reject
icrc-ledger-types|ic-stable-structures v0.7.2|reject
agent|ic-stable-structures v0.7.2|reject
principal|ic-stable-structures v0.7.3|reject
principal|ic-stable-structures v0.7.2 (/tmp/override)|reject
CASES
# A runtime selection must actually contain its guard, not merely avoid outsiders.
printf 'ic-platform v0.1.13\n' > "$fixture/native-case.txt"
printf 'executor\tcase\n' > "$fixture/mapping"
if admit -v target=native -v cache="$fixture" -v guard='ic-platform-runtime-guard v0.1.13 (/fixture)' \
    -f "$root/scripts/ci/check-platform-feature-graphs.awk" "$fixture/mapping" > "$fixture/log" 2>&1; then exit 1; fi
grep -q 'missing runtime singleton guard' "$fixture/log"
# A shared key must still be admitted separately for each original selection.
printf 'ic-platform v0.1.13\nic-agent v0.49.2\n' > "$fixture/native-case.txt"
printf 'agent\tcase\nnone\tcase\n' > "$fixture/mapping"
if admit -v target=native -v cache="$fixture" -f "$root/scripts/ci/check-platform-feature-graphs.awk" "$fixture/mapping" > "$fixture/log" 2>&1; then exit 1; fi
# Missing graphs fail closed rather than being treated as an empty package set.
rm "$fixture/native-case.txt"
if admit -v target=native -v cache="$fixture" -f "$root/scripts/ci/check-platform-feature-graphs.awk" "$fixture/mapping" > "$fixture/log" 2>&1; then exit 1; fi
printf 'Batch graph admission: %s cases plus shared-key and missing-graph refusals passed\n' "$count"
# The retired PocketIC owner is rejected on every target.
for target in native wasm32-unknown-unknown; do
    for selection in none executor; do
        printf 'ic-platform-pocket-ic v0.1.13 (/fixture)\nic-management-canister-types v0.11.0\ncandid v0.10.38\nic-certification v3.2.0\n' > "$fixture/$target-pocket.txt"
        printf '%s\tpocket\n' "$selection" > "$fixture/mapping"
        actual=reject
        if admit -v target="$target" -v cache="$fixture" -f "$root/scripts/ci/check-platform-feature-graphs.awk" "$fixture/mapping" > "$fixture/log" 2>&1; then actual=pass; fi
        expected=reject
        [[ "$actual" == "$expected" ]] || { cat "$fixture/log" >&2; exit 1; }
    done
done
# Pure hashing no longer belongs to the facade, on either target.
for target in native wasm32-unknown-unknown; do
    printf 'blake3 v1.8.7\n' > "$fixture/$target-blake3.txt"
    printf 'none\tblake3\n' > "$fixture/mapping"
    if admit -v target="$target" -v cache="$fixture" \
        -f "$root/scripts/ci/check-platform-feature-graphs.awk" "$fixture/mapping" > "$fixture/log" 2>&1; then exit 1; fi
done
for package in 'pocket-ic v16.1.0' 'ic-platform-pocket-ic v0.1.12 (/fixture)' 'ic-platform-pocket-ic v0.1.13 (/other)' 'ic-platform-pocket-ic v0.1.13' 'ic-management-canister-types v0.8.0'; do
    printf '%s\n' "$package" > "$fixture/native-pocket.txt"
    printf 'none\tpocket\n' > "$fixture/mapping"
    if admit -v target=native -v cache="$fixture" -f "$root/scripts/ci/check-platform-feature-graphs.awk" "$fixture/mapping" > "$fixture/log" 2>&1; then exit 1; fi
done
echo 'Retired PocketIC client refuses in native and Wasm graphs'
# Native host selections cannot authorize entropy dependencies in a Wasm graph.
for selection in agent candid-parser; do
    printf 'getrandom v0.2.17\n' > "$fixture/wasm32-unknown-unknown-entropy.txt"
    printf '%s\tentropy\n' "$selection" > "$fixture/mapping"
    if admit -v target=wasm32-unknown-unknown -v cache="$fixture" \
        -f "$root/scripts/ci/check-platform-feature-graphs.awk" "$fixture/mapping" > "$fixture/log" 2>&1; then
        echo "Unexpected Wasm entropy admission for $selection" >&2; exit 1
    fi
    rg -q 'unselected RNG/entropy scaffolding' "$fixture/log"
done
echo 'Native-only feature names cannot admit Wasm entropy dependencies'
# Wire belongs to tooling only; Principal cannot admit it into deployed graphs.
for package in ic-wire ic-wire-derive; do
    printf '%s v0.6.4\n' "$package" > "$fixture/wasm32-unknown-unknown-wire.txt"
    printf 'principal\twire\n' > "$fixture/mapping"
    if admit -v target=wasm32-unknown-unknown -v cache="$fixture" \
        -f "$root/scripts/ci/check-platform-feature-graphs.awk" "$fixture/mapping" > "$fixture/log" 2>&1; then
        echo 'Wasm production graph admitted Wire' >&2; exit 1
    fi
done
# Native and Wasm Platform graphs exclude the generic Host codec supplier.
for target in native wasm32-unknown-unknown; do
    printf 'ic-host-artifacts v0.71.0\n' > "$fixture/$target-host-codec.txt"
    printf 'none\thost-codec\n' > "$fixture/mapping"
    if admit -v target="$target" -v cache="$fixture" \
        -f "$root/scripts/ci/check-platform-feature-graphs.awk" "$fixture/mapping" > "$fixture/log" 2>&1; then
        echo 'Platform admitted a host artifact codec' >&2; exit 1
    fi
done
# Management DTO adoption must exclude the registry supplier and implicit printer from Wasm.
for package in 'ic-management-canister-types v0.11.0' 'pretty v0.12.5'; do
    printf 'ic-platform v0.1.13\n%s\n' "$package" > "$fixture/wasm32-unknown-unknown-management.txt"
    printf 'management-canister-types\tmanagement\n' > "$fixture/mapping"
    if admit -v target=wasm32-unknown-unknown -v cache="$fixture" \
        -f "$root/scripts/ci/check-platform-feature-graphs.awk" "$fixture/mapping" > "$fixture/log" 2>&1; then
        echo "Wasm management admitted native-only supplier/printer: $package" >&2; exit 1
    fi
done
echo 'Wasm management supplier and implicit-printer refusals passed'
# Runtime's single-pass admission retains each independent refusal and exception.
while IFS='|' read -r mode packages expected diagnostic; do
    printf 'ic-platform-runtime v0.20.1 (/fixture)\n%b\n' "$packages" > "$fixture/runtime.graph"
    arguments=()
    case "$mode" in
        wire) ;;
        external) arguments=(--external-candid) ;;
        native) arguments=(--external-candid --native-management) ;;
    esac
    actual=reject
    if bash "$root/scripts/ci/check-platform-runtime-graph.sh" "$fixture/runtime.graph" ${arguments[@]+"${arguments[@]}"} > "$fixture/runtime.log" 2>&1; then actual=pass; fi
    [[ "$actual" == "$expected" ]] || { cat "$fixture/runtime.log" >&2; echo "Unexpected runtime admission: $mode / $packages" >&2; exit 1; }
    [[ "$expected" == pass ]] || grep -Fq "$diagnostic" "$fixture/runtime.log"
done <<'CASES'
wire||pass|
wire|candid v0.10.40|reject|Wire runtime acquired Candid
wire|candid_derive v0.10.40|reject|Wire runtime acquired Candid
external||reject|Missing selected external Candid adapter
external|candid v0.10.40|pass|
wire|ic-agent v0.49.3|reject|Canister graph contains host parsing
external|candid v0.10.40\nic-management-canister-types v0.11.0|reject|Canister graph contains host parsing
native|candid v0.10.40\nic-management-canister-types v0.11.0|pass|
wire|ic0 v1.2.0|reject|Retired full CDK runtime/generator
wire|ic-cdk-executor v2.0.0|reject|Retired full CDK runtime/generator
wire|ic-canister-sig-creation v1.3.1\nic0 v1.2.0|pass|
wire|ic-canister-sig-creation v1.3.1 (*)\nic0 v1.2.0 (*)|pass|
wire|ic-canister-sig-creation v1.3.1\nic0 v9.0.0|reject|Unqualified signature IC0 supplier
wire|ic-canister-sig-creation v1.3.1\nic-cdk v0.20.3|reject|Retired full CDK runtime/generator
wire|ic-platform-runtime v0.20.1 (/fixture) (*)|pass|
wire|ic-platform-runtime v0.20.1 (/other)|reject|one canonical Runtime executor owner
CASES
printf 'slotmap v1.1.1\n' > "$fixture/runtime.graph"
if bash "$root/scripts/ci/check-platform-runtime-graph.sh" "$fixture/runtime.graph" > "$fixture/runtime.log" 2>&1; then exit 1; fi
grep -q 'one canonical Runtime executor owner' "$fixture/runtime.log"
echo 'Single-pass runtime graph admission: Candid, host, IC0 and owner-identity cases passed'
