#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
mkdir -p target/feature-graphs
cargo metadata --no-deps --format-version 1 --locked > target/feature-graphs/metadata.json
"$root/.tools/host/bin/jq" -e '.packages[] | select(.name == "ic-platform") | (.features | keys) == ["agent", "bls", "candid", "candid-parser", "canister-sig-creation", "cdk", "certification", "default", "ed25519", "executor", "ic0", "icrc-ledger-types", "management-canister-types", "pocket-ic", "secp256k1", "signature-verification"]' \
    target/feature-graphs/metadata.json > /dev/null || {
    echo 'Platform feature inventory changed; update the graph matrix and isolation policy' >&2; exit 1;
}
"$root/.tools/host/bin/jq" -e '.packages[] | select(.name == "ic-platform") | .features.default == []' \
    target/feature-graphs/metadata.json > /dev/null || {
    echo 'Platform must declare empty default features' >&2; exit 1;
}
"$root/.tools/host/bin/jq" -e '.packages[] | select(.name == "ic-platform") | .dependencies[] | select(.name == "ic-cdk-executor") | .req == "=2.0.0"' \
    target/feature-graphs/metadata.json > /dev/null || {
    echo 'Platform must preserve the exact qualified executor =2.0.0' >&2; exit 1;
}
"$root/.tools/host/bin/jq" -e '.packages[] | select(.name == "ic-platform") | . as $p | .features.cdk == ["candid", "executor", "ic0", "dep:ic-cdk", "dep:ic-platform-macros"] and ([.dependencies[] | select(.name == "ic-cdk")] | length == 1 and .[0].optional and .[0].req == "=0.20.3") and ([.dependencies[] | select(.name == "ic-platform-macros")] | length == 1 and .[0].optional and .[0].req == ("^" + $p.version))' \
    target/feature-graphs/metadata.json > /dev/null || {
    echo 'CDK must select the qualified runtime and coordinated generator' >&2; exit 1;
}
generator_version="$("$root/.tools/host/bin/jq" -r '.packages[] | select(.name == "ic-platform") | .version' target/feature-graphs/metadata.json)"
"$root/.tools/host/bin/jq" -e '.packages[] | select(.name == "ic-platform") | .features.secp256k1 == ["dep:ic-secp256k1"] and ([.dependencies[] | select(.name == "ic-secp256k1")] | length == 1 and .[0].optional and .[0].req == "=0.3.0" and .[0].target == "cfg(not(target_family = \"wasm\"))")' \
    target/feature-graphs/metadata.json > /dev/null || {
    echo 'secp256k1 must remain exact, optional and native-only' >&2; exit 1;
}
"$root/.tools/host/bin/jq" -e '.packages[] | select(.name == "ic-platform") | .dependencies[] | select(.name == "ic0") | .req == "=1.2.0"' \
    target/feature-graphs/metadata.json > /dev/null || {
    echo 'Platform must preserve the exact qualified ic0 =1.2.0' >&2; exit 1;
}
"$root/.tools/host/bin/jq" -e '.packages[] | select(.name == "ic-platform") | .features["management-canister-types"] == ["dep:ic-management-canister-types"] and ([.dependencies[] | select(.name == "ic-management-canister-types")] | length == 1 and .[0].req == "=0.11.0" and .[0].optional)' \
    target/feature-graphs/metadata.json > /dev/null || {
    echo 'Platform must preserve optional management types =0.11.0' >&2; exit 1;
}
"$root/.tools/host/bin/jq" -e '.packages[] | select(.name == "ic-platform") | .features["icrc-ledger-types"] == ["dep:icrc-ledger-types"] and ([.dependencies[] | select(.name == "icrc-ledger-types")] | length == 1 and .[0].optional and (.[0].uses_default_features | not) and .[0].features == [])' \
    target/feature-graphs/metadata.json > /dev/null || {
    echo 'Ledger types must be optional without upstream default/storage features' >&2; exit 1;
}
"$root/.tools/host/bin/jq" -e '.packages[] | select(.name == "ic-platform") | .features.bls == ["dep:ic-verify-bls-signature"] and ([.dependencies[] | select(.name == "ic-verify-bls-signature")] | length == 1 and .[0].optional and (.[0].uses_default_features | not) and .[0].features == ["alloc"])' \
    target/feature-graphs/metadata.json > /dev/null || {
    echo 'BLS must be optional with alloc and no upstream default/RNG features' >&2; exit 1;
}
"$root/.tools/host/bin/jq" -e '.packages[] | select(.name == "ic-platform") | .features.ed25519 == ["dep:curve25519-dalek", "dep:ed25519-dalek"] and ([.dependencies[] | select(.name == "curve25519-dalek" or .name == "ed25519-dalek")] | length == 2 and all(.[]; .optional and (.uses_default_features | not) and .features == [] and .target == null))' \
    target/feature-graphs/metadata.json > /dev/null || {
    echo 'Ed25519 must remain optional on native/Wasm without upstream default/RNG features' >&2; exit 1;
}
# These are production graphs including normal/build, excluding dev dependencies.
# Cargo tree prints canonical package names even for renamed deps.
selections=(none candid executor certification
    candid,executor candid,certification executor,certification candid,executor,certification
    candid-parser candid,candid-parser executor,candid-parser certification,candid-parser
    candid,executor,candid-parser candid,certification,candid-parser
    executor,certification,candid-parser candid,executor,certification,candid-parser)
all_selections=(default "${selections[@]}" agent)
for selection in "${selections[@]}"; do
    if [[ "$selection" != none ]]; then all_selections+=("$selection,agent"); fi
done
without_system=("${all_selections[@]}")
all_selections+=(ic0)
for selection in "${without_system[@]}"; do
    if [[ "$selection" != none && "$selection" != default ]]; then all_selections+=("$selection,ic0"); fi
done
without_management=("${all_selections[@]}")
all_selections+=(management-canister-types)
for selection in "${without_management[@]}"; do
    if [[ "$selection" != none && "$selection" != default ]]; then all_selections+=("$selection,management-canister-types"); fi
done
without_ledger=("${all_selections[@]}")
all_selections+=(icrc-ledger-types)
for selection in "${without_ledger[@]}"; do
    if [[ "$selection" != none && "$selection" != default ]]; then all_selections+=("$selection,icrc-ledger-types"); fi
done
for capability in ed25519 bls canister-sig-creation signature-verification cdk secp256k1 pocket-ic; do
    previous=("${all_selections[@]}")
    all_selections+=("$capability")
    for selection in "${previous[@]}"; do
        if [[ "$selection" != none && "$selection" != default ]]; then all_selections+=("$selection,$capability"); fi
    done
done
# Cache only within this invocation; never reuse graphs from a previous lockfile.
cache="$(mktemp -d "$root/target/feature-graphs/attempt.XXXXXX")"
for target in "$(rustc -vV | sed -n 's/^host: //p')" wasm32-unknown-unknown; do
    mapping="$cache/$target.tsv"
    printf "%s\n" "${all_selections[@]}" | "$root/.tools/host/bin/jq" -R -s 'split("\n") | map(select(length > 0))' |
        "$root/.tools/host/bin/jq" -r --arg target "$target" --slurpfile metadata target/feature-graphs/metadata.json \
        -f scripts/ci/platform-feature-closures.jq > "$mapping"
    while IFS=$'\t' read -r selection key; do
        cached="$cache/$target-$key.txt"
        if [[ -f "$cached" ]]; then continue; fi
        args=(tree -p ic-platform --locked --target "$target" --edges normal,build --prefix none)
        if [[ "$selection" != default ]]; then args+=(--no-default-features); fi
        if [[ "$selection" != none && "$selection" != default ]]; then args+=(--features "$selection"); fi
        cargo "${args[@]}" > "$cached"
    done < "$mapping"
    awk -v target="$target" -v cache="$cache" \
        -v generator="ic-platform-macros v$generator_version (proc-macro) ($root/crates/ic-platform-macros)" \
        -f scripts/ci/check-platform-feature-graphs.awk "$mapping"
done
printf 'Feature selections, mappings and Cargo graphs retained at %s\n' "$cache"
cargo check -p ic-platform --lib --locked --no-default-features
cargo check -p ic-platform --lib --locked --no-default-features --features executor --target wasm32-unknown-unknown
