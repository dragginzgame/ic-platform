#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
mode=routine
case "${1:-}" in
    "") [[ $# == 0 ]] || exit 2 ;;
    --exhaustive) [[ $# == 1 ]] || exit 2; mode=exhaustive ;;
    *) echo 'usage: check-platform-features.sh [--exhaustive]' >&2; exit 2 ;;
esac
mkdir -p target/feature-graphs
cargo metadata --no-deps --format-version 1 --frozen > target/feature-graphs/metadata.json
"$root/.tools/host/bin/jq" -e '.packages[] | select(.name == "ic-platform") | (.features | keys) == ["agent", "bls", "candid", "candid-parser", "canister-sig-creation", "cdk", "certification", "default", "ecdsa", "ed25519", "executor", "ic0", "icrc-ledger-types", "management-canister-types", "pocket-ic", "secp256k1", "signature-verification", "stable-structures"]' \
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
"$root/.tools/host/bin/jq" -e '.packages[] | select(.name == "ic-platform") | .features["stable-structures"] == ["dep:ic-stable-structures"] and ([.dependencies[] | select(.name == "ic-stable-structures")] | length == 1 and .[0].optional and .[0].req == "=0.7.2" and (.[0].uses_default_features | not) and .[0].features == [] and .[0].target == null)' \
    target/feature-graphs/metadata.json > /dev/null || {
    echo 'Storage substrate must remain optional, portable and aligned with Memory =0.7.2' >&2; exit 1;
}
# These are production graphs including normal/build, excluding dev dependencies.
"$root/.tools/host/bin/jq" -e '.packages[] | select(.name == "ic-platform") | .features.ecdsa == ["dep:k256"] and ([.dependencies[] | select(.name == "k256")] | length == 1 and .[0].optional and (.[0].uses_default_features | not) and .[0].features == ["ecdsa"] and .[0].target == null)' \
    target/feature-graphs/metadata.json > /dev/null || {
    echo 'ECDSA must remain optional on native/Wasm without default/RNG features' >&2; exit 1;
}
# Cargo tree prints canonical package names even for renamed deps.
# Cache only within this invocation; never reuse graphs from a previous lockfile.
cache="$(mktemp -d "$root/target/feature-graphs/attempt.XXXXXX")"
"$root/.tools/host/bin/jq" '.packages[] | select(.name == "ic-platform") | .features | keys - ["default"]' target/feature-graphs/metadata.json |
    "$root/.tools/host/bin/jq" --arg mode "$mode" -f scripts/ci/platform-feature-selections.jq > "$cache/selections.json"
selection_count="$("$root/.tools/host/bin/jq" length "$cache/selections.json")"
printf 'Feature matrix mode: %s\n' "$mode"
for target in "$(rustc -vV | sed -n 's/^host: //p')" wasm32-unknown-unknown; do
    mapping="$cache/$target.tsv"
    "$root/.tools/host/bin/jq" -r --arg target "$target" --slurpfile metadata target/feature-graphs/metadata.json \
        -f scripts/ci/platform-feature-closures.jq "$cache/selections.json" > "$mapping"
    # Resolve/download once during setup, not in every exhaustive graph query.
    # Keep all selection admissions, including those sharing an equivalent key.
    queries="$(cut -f2 "$mapping" | LC_ALL=C sort -u | wc -l | tr -d ' ')"
    completed=0
    printf 'Feature graphs: %s, %s unique offline queries (%s selections). Evidence: %s\n' \
        "$target" "$queries" "$selection_count" "$cache"
    while IFS=$'\t' read -r selection key; do
        cached="$cache/$target-$key.txt"
        if [[ -f "$cached" ]]; then continue; fi
        args=(tree -p ic-platform --frozen --target "$target" --edges normal,build --prefix none)
        if [[ "$selection" != default ]]; then args+=(--no-default-features); fi
        if [[ "$selection" != none && "$selection" != default ]]; then args+=(--features "$selection"); fi
        if ! cargo "${args[@]}" > "$cached"; then
            printf 'Feature query failed: %s / %s. If registry inputs are missing, run cargo fetch --locked --target %s before retrying.\n' "$target" "$selection" "$target" >&2
            exit 1
        fi
        completed=$((completed + 1))
        if (( completed % 100 == 0 || completed == queries )); then
            printf 'Feature graphs: %s %s/%s complete\n' "$target" "$completed" "$queries"
        fi
    done < "$mapping"
    awk -v target="$target" -v cache="$cache" \
        -v generator="ic-platform-macros v$generator_version (proc-macro) ($root/crates/ic-platform-macros)" \
        -f scripts/ci/check-platform-feature-graphs.awk "$mapping"
done
printf 'Feature selections, mappings and Cargo graphs retained at %s\n' "$cache"
cargo check -p ic-platform --lib --frozen --no-default-features
cargo check -p ic-platform --lib --frozen --no-default-features --features executor --target wasm32-unknown-unknown
