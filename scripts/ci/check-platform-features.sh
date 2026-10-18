#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
mkdir -p target/feature-graphs
cargo metadata --no-deps --format-version 1 --locked > target/feature-graphs/metadata.json
"$root/.tools/host/bin/jq" -e '.packages[] | select(.name == "ic-platform") | (.features | keys) == ["agent", "candid", "candid-parser", "certification", "default", "executor", "ic0", "icrc-ledger-types", "management-canister-types"]' \
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
# These are production package graphs, including build dependencies but not dev
# dependencies. Cargo tree prints canonical package names even for renamed deps.
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
for target in "$(rustc -vV | sed -n 's/^host: //p')" wasm32-unknown-unknown; do
    for selection in "${all_selections[@]}"; do
        graph="target/feature-graphs/$target-$selection.txt"
        args=(tree -p ic-platform --locked --target "$target" --edges normal,build --prefix none)
        if [[ "$selection" != default ]]; then args+=(--no-default-features); fi
        if [[ "$selection" != none && "$selection" != default ]]; then args+=(--features "$selection"); fi
        cargo "${args[@]}" > "$graph"
        parser=false
        if [[ "$target" != wasm* && "$selection" == *candid-parser* ]]; then parser=true; fi
        agent=false
        if [[ "$target" != wasm* && ",$selection," == *,agent,* ]]; then agent=true; fi
        if grep -Eq '^(pocket-ic|ic-testkit|ic-testkit-candid) ' "$graph"; then
            echo "Production graph contains Testkit transport: $graph" >&2; exit 1
        fi
        if [[ "$parser" == false ]] && grep -Eq '^candid_parser ' "$graph"; then
            echo "Parser must remain native host opt-in only: $graph" >&2; exit 1
        fi
        if [[ "$agent" == false ]] && grep -Eq '^ic-agent ' "$graph"; then
            echo "Agent must remain native host opt-in only: $graph" >&2; exit 1
        fi
        executor=false
        if [[ ",$selection," == *,executor,* ]]; then executor=true; fi
        raw_system=false
        if [[ "$executor" == true || ",$selection," == *,ic0,* ]]; then raw_system=true; fi
        # Exact direct requirements do not exclude a second transitive version.
        # Selecting the executor never admits the unqualified CDK frontend.
        if ! awk -v executor="$executor" -v raw_system="$raw_system" '
            $1 ~ /^ic-cdk/ && ($1 != "ic-cdk-executor" || $2 != "v2.0.0" || executor != "true") { exit 1 }
            $1 == "ic0" && ($2 != "v1.2.0" || raw_system != "true") { exit 1 }
            ($1 ~ /^ic-cdk/ || $1 == "ic0") && !(NF == 2 || (NF == 3 && $3 == "(*)")) { exit 1 }
        ' "$graph"; then
            echo "Graph contains an unselected or unqualified CDK/system dependency: $graph" >&2; exit 1
        fi
        management=false
        if [[ ",$selection," == *,management-canister-types,* ]]; then management=true; fi
        if [[ "$management" == false ]] && grep -Eq '^ic-management-canister-types ' "$graph"; then
            echo "Management types must remain opt-in only: $graph" >&2; exit 1
        fi
        ledger=false
        if [[ ",$selection," == *,icrc-ledger-types,* ]]; then ledger=true; fi
        if [[ "$ledger" == false ]] && grep -Eq '^icrc-ledger-types ' "$graph"; then
            echo "Ledger types must remain opt-in only: $graph" >&2; exit 1
        fi
        if grep -Eq '^ic-stable-structures ' "$graph"; then
            echo "Ledger storage integration is outside the Platform contract: $graph" >&2; exit 1
        fi
        # Protocol DTOs, host parser and Agent depend on upstream Candid independently.
        if [[ ",$selection," != *,candid,* && "$parser" == false && "$agent" == false && "$management" == false && "$ledger" == false ]] && grep -Eq '^(candid|candid_derive) ' "$graph"; then
            echo "Graph without candid must remain Candid-free: $graph" >&2; exit 1
        fi
        if [[ "$selection" != *certification* && "$agent" == false ]] && grep -Eq '^ic-certification ' "$graph"; then
            echo "Certification must remain optional: $graph" >&2; exit 1
        fi
    done
done
cargo check -p ic-platform --lib --locked --no-default-features
cargo check -p ic-platform --lib --locked --no-default-features --features executor --target wasm32-unknown-unknown
