#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
mkdir -p target/feature-graphs
cargo metadata --no-deps --format-version 1 --locked > target/feature-graphs/metadata.json
"$root/.tools/host/bin/jq" -e '.packages[] | select(.name == "ic-platform") | (.features | keys) == ["agent", "bls", "candid", "candid-parser", "canister-sig-creation", "cdk", "certification", "default", "executor", "ic0", "icrc-ledger-types", "management-canister-types", "signature-verification"]' \
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
for capability in bls canister-sig-creation signature-verification cdk; do
    previous=("${all_selections[@]}")
    all_selections+=("$capability")
    for selection in "${previous[@]}"; do
        if [[ "$selection" != none && "$selection" != default ]]; then all_selections+=("$selection,$capability"); fi
    done
done
# Cache only within this invocation; never reuse graphs from a previous lockfile.
cache="$(mktemp -d "$root/target/feature-graphs/attempt.XXXXXX")"
printf "%s\n" "${all_selections[@]}" | "$root/.tools/host/bin/jq" -R -s 'split("\n") | map(select(length > 0))' |
    "$root/.tools/host/bin/jq" -r --slurpfile metadata target/feature-graphs/metadata.json \
    -f scripts/ci/platform-feature-closures.jq > "$cache/selections.tsv"
for target in "$(rustc -vV | sed -n 's/^host: //p')" wasm32-unknown-unknown; do
    while IFS=$'\t' read -r selection key; do
        graph="target/feature-graphs/$target-$selection.txt"
        args=(tree -p ic-platform --locked --target "$target" --edges normal,build --prefix none)
        if [[ "$selection" != default ]]; then args+=(--no-default-features); fi
        if [[ "$selection" != none && "$selection" != default ]]; then args+=(--features "$selection"); fi
        cached="$cache/$target-$key.txt"
        if [[ ! -f "$cached" ]]; then
            cargo "${args[@]}" > "$cached"
        fi
        cp "$cached" "$graph"
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
        cdk=false
        if [[ ",$selection," == *,cdk,* ]]; then cdk=true; fi
        if [[ "$cdk" == true || ",$selection," == *,executor,* ]]; then executor=true; fi
        raw_system=false
        verification=false
        if [[ ",$selection," == *,signature-verification,* ]]; then verification=true; fi
        creation=false
        if [[ "$verification" == true || ",$selection," == *,canister-sig-creation,* ]]; then creation=true; fi
        bls=false
        if [[ "$verification" == true || "$agent" == true || ",$selection," == *,bls,* ]]; then bls=true; fi
        if [[ "$executor" == true || "$creation" == true || ",$selection," == *,ic0,* ]]; then raw_system=true; fi
        certification=false
        if [[ "$creation" == true || "$agent" == true || ",$selection," == *,certification,* ]]; then certification=true; fi
        # Exact direct requirements do not exclude a second transitive version.
        # Only explicit CDK selection admits the qualified SDK frontend/macros.
        if ! awk -v executor="$executor" -v raw_system="$raw_system" -v cdk="$cdk" \
            -v generator="ic-platform-macros v$generator_version (proc-macro) ($root/crates/ic-platform-macros)" '
            $1 ~ /^ic-cdk/ {
                if ($1 == "ic-cdk-executor") { if ($2 != "v2.0.0" || executor != "true") exit 1 }
                else if ($1 == "ic-cdk" || $1 == "ic-cdk-macros") { if ($2 != "v0.20.3" || cdk != "true") exit 1 }
                else exit 1
            }
            $1 == "ic0" && ($2 != "v1.2.0" || raw_system != "true") { exit 1 }
            $1 == "ic-cdk-macros" && !(NF == 3 && $3 == "(proc-macro)" || NF == 4 && $3 == "(proc-macro)" && $4 == "(*)") { exit 1 }
            ($1 ~ /^ic-cdk/ && $1 != "ic-cdk-macros" || $1 == "ic0") && !(NF == 2 || (NF == 3 && $3 == "(*)")) { exit 1 }
            $1 == "ic-platform-macros" && (cdk != "true" || ($0 != generator && $0 != generator " (*)")) { exit 1 }
        ' "$graph"; then
            echo "Graph contains an unselected or unqualified CDK/system dependency: $graph" >&2; exit 1
        fi
        if [[ "$cdk" == false ]] && grep -Eq '^ic-platform-macros ' "$graph"; then
            echo "Platform generator must remain CDK opt-in only: $graph" >&2; exit 1
        fi
        if ! awk -v creation="$creation" -v verification="$verification" -v bls="$bls" -v certification="$certification" '
            $1 == "ic-canister-sig-creation" && ($2 != "v1.3.1" || creation != "true") { exit 1 }
            $1 == "ic-signature-verification" && ($2 != "v0.3.0" || verification != "true") { exit 1 }
            $1 == "ic-verify-bls-signature" && ($2 != "v0.6.0" || bls != "true") { exit 1 }
            $1 == "ic_bls12_381" && ($2 != "v0.10.1" || bls != "true") { exit 1 }
            $1 == "ic-certification" && ($2 != "v3.2.0" || certification != "true") { exit 1 }
            ($1 == "ic-canister-sig-creation" || $1 == "ic-signature-verification" || $1 == "ic-verify-bls-signature" || $1 == "ic_bls12_381" || $1 == "ic-certification") && !(NF == 2 || (NF == 3 && $3 == "(*)")) { exit 1 }
        ' "$graph"; then
            echo "Graph contains an unselected or unqualified signature dependency: $graph" >&2; exit 1
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
        if [[ ",$selection," != *,candid,* && "$cdk" == false && "$parser" == false && "$agent" == false && "$management" == false && "$ledger" == false && "$creation" == false ]] && grep -Eq '^(candid|candid_derive) ' "$graph"; then
            echo "Graph without candid must remain Candid-free: $graph" >&2; exit 1
        fi
    done < "$cache/selections.tsv"
done
cargo check -p ic-platform --lib --locked --no-default-features
cargo check -p ic-platform --lib --locked --no-default-features --features executor --target wasm32-unknown-unknown
