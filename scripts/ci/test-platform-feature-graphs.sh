#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/platform-feature-admission.XXXXXX")"
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
    awk -v activation="$fixture/activation.tsv" "$@"
}
count=0
while IFS='|' read -r selection package expected; do
    printf 'ic-platform v0.1.13\n%s\n' "$package" > "$fixture/native-case.txt"
    printf '%s\tcase\n' "$selection" > "$fixture/mapping"
    actual=reject
    if admit -v target=native -v cache="$fixture" -v generator='ic-platform-macros v0.1.13 (proc-macro) (/fixture)' \
        -f "$root/scripts/ci/check-platform-feature-graphs.awk" "$fixture/mapping" > "$fixture/log" 2>&1; then actual=pass; fi
    [[ "$actual" == "$expected" ]] || { echo "Unexpected $actual for $selection: $package" >&2; cat "$fixture/log" >&2; exit 1; }
    count=$((count + 1))
done <<'CASES'
none||pass
none|ic-wire v0.6.4|reject
principal|ic-wire v0.6.4|reject
wire-schema|ic-wire v0.6.4|pass
wire-schema|ic-wire-derive v0.6.4|pass
executor|ic_principal v0.1.5|reject
ic0|serde v1.0.229|reject
executor,ic0|sha2 v0.10.9|reject
executor,ic0|serde_derive v1.0.229 (proc-macro)|reject
principal|ic_principal v0.1.5|pass
principal,executor|serde v1.0.229|pass
canister-sig-creation|ic-representation-independent-hash v3.2.0|reject
signature-verification|ic-representation-independent-hash v3.2.0|reject
canister-sig-creation|serde_cbor v0.11.2|reject
canister-sig-creation|half v1.8.3|reject
signature-verification|serde_cbor v0.11.2|pass
none|rand v0.8.5|reject
executor|rand_chacha v0.3.1|reject
cdk|getrandom v0.2.17|reject
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
cdk|candid v0.10.38|reject
runtime|candid v0.10.38|reject
candid-endpoint|candid v0.10.38|pass
candid-endpoint|pretty v0.12.5|reject
candid-endpoint|ic-cdk-executor v2.0.0|pass
candid-endpoint|ic-platform-macros v0.1.13 (proc-macro) (/fixture)|pass
runtime|ic-cdk-executor v2.0.0|pass
runtime|ic0 v1.2.0|pass
runtime|ic-platform-macros v0.1.13 (proc-macro) (/fixture)|reject
cdk|candid_derive v0.10.38|reject
cdk,candid|candid v0.10.38|pass
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
cdk,candid|pretty v0.12.5|reject
candid-service|pretty v0.12.5|pass
candid-parser|pretty v0.12.5|pass
management-canister-types|pretty v0.12.5|pass
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
cdk|ic-cdk v0.20.3|reject
cdk|ic-cdk v0.19.0|reject
cdk|ic-cdk-timers v1.0.0|reject
cdk|ic-cdk-macros v0.20.3 (proc-macro)|reject
cdk|ic-cdk-macros v0.20.3 (proc-macro) (/local)|reject
cdk|ic-platform-macros v0.1.13 (proc-macro) (/fixture)|pass
none|ic-platform-macros v0.1.13 (proc-macro) (/fixture)|reject
cdk|ic-platform-macros v0.1.12 (proc-macro) (/fixture)|reject
ic0|ic0 v1.2.0|pass
ic0|ic0 v1.1.0|reject
ic0|ic0 v1.2.0 (/local)|reject
canister-sig-creation|ic-canister-sig-creation v1.3.1|reject
canister-sig-creation|candid v0.10.38|reject
signature-verification|candid v0.10.38|reject
canister-sig-creation|ic0 v1.2.0|reject
signature-verification|ic0 v1.2.0|reject
cdk|ic-error-types v0.2.0|reject
canister-sig-creation|ic-canister-sig-creation v1.3.2|reject
signature-verification|ic-signature-verification v0.3.0|reject
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
stable-structures|ic-stable-structures v0.7.2|pass
stable-structures|ic-stable-structures v0.7.3|reject
stable-structures|ic-stable-structures v0.7.2 (/tmp/override)|reject
stable-structures|ic-platform v0.1.18|reject
stable-structures|candid v0.10.38|reject
CASES
# A shared key must still be admitted separately for each original selection.
printf 'ic-platform v0.1.13\nic-agent v0.49.2\n' > "$fixture/native-case.txt"
printf 'agent\tcase\nnone\tcase\n' > "$fixture/mapping"
if admit -v target=native -v cache="$fixture" -f "$root/scripts/ci/check-platform-feature-graphs.awk" "$fixture/mapping" > "$fixture/log" 2>&1; then exit 1; fi
# Missing graphs fail closed rather than being treated as an empty package set.
rm "$fixture/native-case.txt"
if admit -v target=native -v cache="$fixture" -f "$root/scripts/ci/check-platform-feature-graphs.awk" "$fixture/mapping" > "$fixture/log" 2>&1; then exit 1; fi
printf 'Batch graph admission: %s cases plus shared-key and missing-graph refusals passed\n' "$count"
# PocketIC is admitted only as a selected native dependency with its own DTOs.
for target in native wasm32-unknown-unknown; do
    for selection in pocket-ic none executor; do
        printf 'pocket-ic v16.1.0\nic-management-canister-types v0.8.0\ncandid v0.10.38\nic-certification v3.2.0\n' > "$fixture/$target-pocket.txt"
        printf '%s\tpocket\n' "$selection" > "$fixture/mapping"
        actual=reject
        if admit -v target="$target" -v cache="$fixture" -f "$root/scripts/ci/check-platform-feature-graphs.awk" "$fixture/mapping" > "$fixture/log" 2>&1; then actual=pass; fi
        expected=reject
        if [[ "$target" == native && "$selection" == pocket-ic ]]; then expected=pass; fi
        [[ "$actual" == "$expected" ]] || { cat "$fixture/log" >&2; exit 1; }
    done
done
# BLAKE3 is independent of application/runtime features and registry-selected.
for target in native wasm32-unknown-unknown; do
    for selection in none blake3; do
        printf 'blake3 v1.8.7\n' > "$fixture/$target-blake3.txt"
        printf '%s\tblake3\n' "$selection" > "$fixture/mapping"
        actual=reject
        if admit -v target="$target" -v cache="$fixture" \
            -f "$root/scripts/ci/check-platform-feature-graphs.awk" "$fixture/mapping" > "$fixture/log" 2>&1; then actual=pass; fi
        expected=reject
        [[ "$selection" != blake3 ]] || expected=pass
        [[ "$actual" == "$expected" ]] || { cat "$fixture/log" >&2; exit 1; }
    done
done
for package in 'pocket-ic v16.0.0' 'pocket-ic v16.1.0 (/local)' 'ic-management-canister-types v0.8.0'; do
    printf '%s\n' "$package" > "$fixture/native-pocket.txt"
    printf 'pocket-ic\tpocket\n' > "$fixture/mapping"
    if admit -v target=native -v cache="$fixture" -f "$root/scripts/ci/check-platform-feature-graphs.awk" "$fixture/mapping" > "$fixture/log" 2>&1; then exit 1; fi
done
echo 'PocketIC native selection, Wasm exclusion and identity refusals passed'
# Native host selections cannot authorize entropy dependencies in a Wasm graph.
for selection in agent candid-parser pocket-ic secp256k1; do
    printf 'getrandom v0.2.17\n' > "$fixture/wasm32-unknown-unknown-entropy.txt"
    printf '%s\tentropy\n' "$selection" > "$fixture/mapping"
    if admit -v target=wasm32-unknown-unknown -v cache="$fixture" \
        -f "$root/scripts/ci/check-platform-feature-graphs.awk" "$fixture/mapping" > "$fixture/log" 2>&1; then
        echo "Unexpected Wasm entropy admission for $selection" >&2; exit 1
    fi
    rg -q 'unselected RNG/entropy scaffolding' "$fixture/log"
done
echo 'Native-only feature names cannot admit Wasm entropy dependencies'
# Selecting metadata cannot admit its native dependencies into deployed graphs.
for package in ic-wire ic-wire-derive; do
    printf '%s v0.6.4\n' "$package" > "$fixture/wasm32-unknown-unknown-wire.txt"
    printf 'wire-schema\twire\n' > "$fixture/mapping"
    if admit -v target=wasm32-unknown-unknown -v cache="$fixture" \
        -f "$root/scripts/ci/check-platform-feature-graphs.awk" "$fixture/mapping" > "$fixture/log" 2>&1; then
        echo 'Wasm graph admitted host Wire metadata' >&2; exit 1
    fi
done
# Admit only the declared codec identity, on native gzip selections. Registry
# duplicates cannot conceal themselves beside a valid pinned Git node.
pinned='(https://github.com/dragginzgame/ic-tooling.git?rev=c20dbd11785c17d369651f51fa0967fbec69c57a#c20dbd11)'
for declared in '' "$pinned"; do
    for target in native wasm32-unknown-unknown; do
        for selection in gzip none executor; do
            for source in '' "$pinned" "$pinned (*)" '(/local)' \
                '(https://github.com/dragginzgame/ic-tooling.git?rev=0000000000000000000000000000000000000000#00000000)' \
                '(https://github.com/other/ic-tooling.git?rev=c20dbd11785c17d369651f51fa0967fbec69c57a#c20dbd11)'; do
                printf 'ic-host-artifacts v0.69.13 %s\n' "$source" > "$fixture/$target-gzip.txt"
                printf '%s\tgzip\n' "$selection" > "$fixture/mapping"
                actual=reject
                if admit -v target="$target" -v cache="$fixture" -v gzip_source="$declared" \
                    -f "$root/scripts/ci/check-platform-feature-graphs.awk" "$fixture/mapping" > "$fixture/log" 2>&1; then actual=pass; fi
                expected=reject
                if [[ "$target" == native && "$selection" == gzip && ( "$source" == "$declared" || ( -n "$declared" && "$source" == "$declared (*)" ) ) ]]; then expected=pass; fi
                [[ "$actual" == "$expected" ]] || { cat "$fixture/log" >&2; exit 1; }
            done
        done
    done
done
printf 'ic-host-artifacts v0.69.13 %s\nic-host-artifacts v0.69.12\n' "$pinned" > "$fixture/native-gzip.txt"
printf 'gzip\tgzip\n' > "$fixture/mapping"
if admit -v target=native -v cache="$fixture" -v gzip_source="$pinned" \
    -f "$root/scripts/ci/check-platform-feature-graphs.awk" "$fixture/mapping" > "$fixture/log" 2>&1; then exit 1; fi
echo 'Host gzip identity, selection, Wasm and mixed-source refusals passed'
