#!/usr/bin/env bash
set -euo pipefail
# A separate Cargo package cannot accidentally use Platform's private dependencies.
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
# shellcheck source=scripts/ci/consumer-fixture.sh
source "$root/scripts/ci/consumer-fixture.sh"
mkdir -p target
published=""
if [[ "${1:-}" == --published ]]; then
    stable_version='^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$'
    [[ $# -ge 2 && "$2" =~ $stable_version ]] || {
        echo 'Usage: check-platform-consumer.sh [--published X.Y.Z] [capability ...]' >&2; exit 2;
    }
    published="$2"
    shift 2
fi
# Both source and selected archive qualification use the current owner policy.
msrv="$(bash "$root/scripts/ci/read-platform-msrv.sh")"
profiles="$root/scripts/ci/platform-consumer-profiles.tsv"
awk -f "$root/scripts/ci/platform-graph-policy.awk" /dev/null
tests="$("$root/.tools/host/bin/yq" -p toml -o json '.test' crates/ic-platform/Cargo.toml)"
capabilities=()
while IFS=$'\t' read -r capability _; do
    capabilities+=("$capability")
done < "$profiles"
if [[ $# -gt 0 ]]; then
    for requested in "$@"; do
        case " ${capabilities[*]} " in
            *" $requested "*) ;;
            *) echo "Unknown consumer capability: $requested" >&2; exit 2 ;;
        esac
    done
    capabilities=("$@")
fi
for capability in "${capabilities[@]}"; do
profile="$(awk -F '\t' -v capability="$capability" '$1 == capability { print }' "$profiles")"
IFS=$'\t' read -r _ source assets wasm graph_family <<< "$profile"
case "$graph_family" in
    -|minimal|principal|principal-self-authenticating|ledger|runtime|owned-endpoint|external-codecs) ;;
    *) echo "Unknown consumer graph family: $graph_family" >&2; exit 2 ;;
esac
# shellcheck disable=SC2016 # $source is a jq binding selecting the test declaration.
features="$(printf '%s\n' "$tests" | "$root/.tools/host/bin/jq" -er --arg source "tests/$source.rs" '
    [.[] | select(.path == $source)] |
    if length == 1 and (.[0]["required-features"] | type == "array" and length > 0)
    then .[0]["required-features"] | map(@json) | join(", ")
    else error("expected one Cargo test with explicit required features: " + $source) end')"
selection='path = "../../crates/ic-platform"'
if [[ -n "$published" ]]; then selection="version = \"=$published\""; fi
consumer_fixture "$root" boundary-consumer crates/consumer/src
cat > "$fixture/Cargo.toml" <<MANIFEST
[workspace]
members = ["crates/consumer"]
resolver = "3"
[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "$msrv"
[workspace.dependencies]
ic-platform = { $selection, default-features = false, features = [$features] }
MANIFEST
cat > "$fixture/crates/consumer/Cargo.toml" <<'MANIFEST'
[package]
name = "platform-consumer-proof"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
publish = false

[package.metadata.toko-tooling.loc]
category = "tests"
[lints.rust]
warnings = "deny"
[dependencies]
ic-platform.workspace = true
MANIFEST
cp "crates/ic-platform/tests/${source}.rs" "$fixture/crates/consumer/src/lib.rs"
cp -R crates/ic-platform/tests/support "$fixture/crates/consumer/src/"
if [[ "$assets" != - ]]; then
    mkdir -p "$fixture/crates/consumer/src/fixtures"
    IFS=, read -r -a patterns <<< "$assets"
    for pattern in "${patterns[@]}"; do
        # The reviewed profile supplies a filename glob, not a literal path.
        # shellcheck disable=SC2086
        cp crates/ic-platform/tests/fixtures/$pattern "$fixture/crates/consumer/src/fixtures/"
    done
fi
if [[ "$wasm" == build ]]; then
    printf '\n[lib]\ncrate-type = ["cdylib", "rlib"]\n' >> "$fixture/crates/consumer/Cargo.toml"
fi
consumer_metadata "$root"
if [[ -n "$published" ]]; then
    # Refuse path/Git replacement of either the facade or its generator.
    "$root/.tools/host/bin/jq" -e --arg version "$published" --argjson require_generator false \
        -f "$root/scripts/ci/published-runtime.jq" "$fixture/metadata.json" > /dev/null || {
        echo 'Published consumer must resolve the exact registry facade and registry generator' >&2; exit 1;
    }
fi
graph_has_forbidden() {
    awk -v family="$1" -f "$root/scripts/ci/platform-graph-policy.awk" "$graph"
}
if [[ "$graph_family" != - ]]; then
    for graph_target in "$(rustc -vV | sed -n 's/^host: //p')" wasm32-unknown-unknown; do
        graph="$fixture/$graph_target.graph"
        cargo tree --manifest-path "$fixture/Cargo.toml" --frozen --target "$graph_target" --edges normal,build --prefix none > "$graph"
        case "$graph_family" in
            minimal)
                if graph_has_forbidden minimal; then
                    echo 'Minimal consumer acquired application types/codecs/crypto' >&2; exit 1
                fi ;;
            principal|principal-self-authenticating|ledger)
                if graph_has_forbidden identity; then
                    echo 'Identity/ledger consumer acquired retired or unrelated integration machinery' >&2; exit 1
                fi
                if [[ "$graph_family" == principal ]] &&
                    graph_has_forbidden identity-codecs; then
                    echo 'Principal byte/text capability acquired hashing or Candid' >&2; exit 1
                fi
                if [[ "$graph_family" == principal || "$graph_family" == principal-self-authenticating ]] && graph_has_forbidden external-principal; then
                    echo 'Owned Principal acquired the external identity' >&2; exit 1
                fi ;;
            runtime|owned-endpoint)
                bash scripts/ci/check-platform-runtime-graph.sh "$graph"
                if graph_has_forbidden retired-darling; then
                    echo 'Runtime/entrypoint consumer acquired retired Darling scaffolding' >&2; exit 1
                fi
                if [[ "$graph_family" == runtime ]] && graph_has_forbidden generator; then
                    echo 'Runtime-only consumer acquired the entrypoint generator' >&2; exit 1
                fi ;;
            external-codecs)
                if graph_has_forbidden external-codecs; then
                    echo 'External codecs acquired transport, executor or host parsing' >&2; exit 1
                fi ;;
        esac
    done
fi
bash "$root/scripts/ci/check-platform-compiler.sh" > "$fixture/compiler.txt"
CARGO_TARGET_DIR="$root/target/consumer-build" cargo +"$msrv" test --manifest-path "$fixture/Cargo.toml" --frozen
if [[ "$wasm" != skip ]]; then
    CARGO_TARGET_DIR="$root/target/consumer-build" cargo +"$msrv" "$wasm" --manifest-path "$fixture/Cargo.toml" --lib --target wasm32-unknown-unknown --frozen
fi
printf 'Consumer evidence retained at %s\n' "$fixture"
# Parse the exit with the loop before Cargo runs. A concurrent source edit must
# not make Bash resume at a shifted byte offset in the changed file afterward.
done; exit 0
