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
# Published qualification retains its historical compiler; source checks use the manifest.
msrv=1.91.0
if [[ -z "$published" ]]; then msrv="$(bash "$root/scripts/ci/read-platform-msrv.sh")"; fi
profiles="$root/scripts/ci/platform-consumer-profiles.tsv"
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
IFS=$'\t' read -r _ features source assets wasm graph_family <<< "$profile"
case "$graph_family" in
    -|minimal|principal|principal-external|principal-self-authenticating|ledger|cdk|runtime|external-candid|blake3|external-codecs) ;;
    *) echo "Unknown consumer graph family: $graph_family" >&2; exit 2 ;;
esac
features="\"${features//,/\", \"}\""
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
[lints.rust]
warnings = "deny"
[dependencies]
ic-platform.workspace = true
MANIFEST
if [[ "$capability" == principal ]]; then
    printf '\n[features]\ncandid = ["ic-platform/candid"]\n' >> "$fixture/crates/consumer/Cargo.toml"
fi
cp "crates/ic-platform/tests/${source}.rs" "$fixture/crates/consumer/src/lib.rs"
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
if [[ "$graph_family" != - ]]; then
    for graph_target in "$(rustc -vV | sed -n 's/^host: //p')" wasm32-unknown-unknown; do
        graph="$fixture/$graph_target.graph"
        if [[ "$graph_family" == blake3 ]]; then
            BLAKE3_PACKAGES_OUTPUT="$graph" bash scripts/ci/check-blake3-graph.sh "$fixture" platform-consumer-proof "$graph_target" -
        else
            cargo tree --manifest-path "$fixture/Cargo.toml" --frozen --target "$graph_target" --edges normal,build --prefix none > "$graph"
        fi
        case "$graph_family" in
            minimal)
                if grep -Eq '^(ic_principal|serde|serde_core|serde_derive|sha2|digest|crc32fast|data-encoding|candid|ic-agent) ' "$graph"; then
                    echo 'Minimal consumer acquired application types/codecs/crypto' >&2; exit 1
                fi ;;
            principal|principal-external|principal-self-authenticating|ledger)
                if grep -Eq '^(icrc-ledger-types|icrc-cbor|minicbor|base32|candid_parser|ic-agent|pocket-ic) ' "$graph"; then
                    echo 'Identity/ledger consumer acquired retired or unrelated integration machinery' >&2; exit 1
                fi
                if [[ "$graph_family" == principal || "$graph_family" == principal-external ]] &&
                    grep -Eq '^(sha2|digest|candid) ' "$graph"; then
                    echo 'Principal byte/text capability acquired hashing or Candid' >&2; exit 1
                fi
                if [[ "$graph_family" == principal ]] && grep -q '^ic_principal ' "$graph"; then
                    echo 'Owned Principal acquired the external identity' >&2; exit 1
                fi ;;
            cdk|runtime|external-candid)
                if [[ "$graph_family" == external-candid ]]; then
                    bash scripts/ci/check-platform-runtime-graph.sh "$graph" --external-candid
                else
                    bash scripts/ci/check-platform-runtime-graph.sh "$graph"
                fi
                if grep -Eq '^(darling|darling_core|darling_macro) ' "$graph"; then
                    echo 'Runtime/entrypoint consumer acquired retired Darling scaffolding' >&2; exit 1
                fi
                if [[ "$graph_family" == runtime ]] && grep -q '^ic-platform-macros ' "$graph"; then
                    echo 'Runtime-only consumer acquired the entrypoint generator' >&2; exit 1
                fi ;;
            blake3)
                if grep -Eq '^(candid|ic-cdk|ic-platform-macros|ic-agent|pocket-ic|ic_principal|serde) ' "$graph"; then
                    echo 'BLAKE3 consumer acquired unrelated IC/application dependencies' >&2; exit 1
                fi ;;
            external-codecs)
                if grep -Eq '^(ic-cdk[^ ]*|ic0|ic-platform-macros|ic-agent|pocket-ic|ic-testkit|candid_parser|reqwest|tokio) ' "$graph"; then
                    echo 'External codecs acquired transport, executor or host parsing' >&2; exit 1
                fi ;;
        esac
    done
fi
CARGO_TARGET_DIR="$root/target/consumer-build" cargo test --manifest-path "$fixture/Cargo.toml" --frozen
CARGO_TARGET_DIR="$root/target/consumer-msrv" cargo +"$msrv" test --manifest-path "$fixture/Cargo.toml" --frozen
if [[ "$wasm" != skip ]]; then
    CARGO_TARGET_DIR="$root/target/consumer-build" cargo "$wasm" --manifest-path "$fixture/Cargo.toml" --lib --target wasm32-unknown-unknown --frozen
    CARGO_TARGET_DIR="$root/target/consumer-msrv" cargo +"$msrv" "$wasm" --manifest-path "$fixture/Cargo.toml" --lib --target wasm32-unknown-unknown --frozen
fi
printf 'Consumer evidence retained at %s\n' "$fixture"
# Parse the exit with the loop before Cargo runs. A concurrent source edit must
# not make Bash resume at a shifted byte offset in the changed file afterward.
done; exit 0
