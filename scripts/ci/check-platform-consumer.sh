#!/usr/bin/env bash
set -euo pipefail
# A separate Cargo package cannot accidentally use Platform's private dependencies.
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
mkdir -p target
published=""
if [[ "${1:-}" == --published ]]; then
    [[ $# -ge 2 && "$2" =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]] || {
        echo 'Usage: check-platform-consumer.sh [--published X.Y.Z] [capability ...]' >&2; exit 2;
    }
    published="$2"
    shift 2
fi
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
IFS=$'\t' read -r _ features source assets wasm <<< "$profile"
features="\"${features//,/\", \"}\""
selection='path = "../../crates/ic-platform"'
if [[ -n "$published" ]]; then selection="version = \"=$published\""; fi
fixture="$(mktemp -d "$root/target/boundary-consumer.XXXXXX")"
mkdir -p "$fixture/crates/consumer/src"
cat > "$fixture/Cargo.toml" <<MANIFEST
[workspace]
members = ["crates/consumer"]
resolver = "3"
[workspace.package]
version = "0.1.0"
edition = "2024"
[workspace.dependencies]
ic-platform = { $selection, default-features = false, features = [$features] }
MANIFEST
cat > "$fixture/crates/consumer/Cargo.toml" <<'MANIFEST'
[package]
name = "platform-consumer-proof"
version.workspace = true
edition.workspace = true
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
cp Cargo.lock "$fixture/Cargo.lock"
# Seed reviewed dependencies; metadata adds the disposable consumer and, when
# requested, registry facade/generator identities in this disposable lock only.
# Filter metadata to the selected host: unfiltered offline metadata otherwise
# requires cached source for foreign targets (for example windows-link on macOS).
cargo metadata --manifest-path "$fixture/Cargo.toml" --offline --filter-platform "$(rustc -vV | sed -n 's/^host: //p')" --format-version 1 > "$fixture/metadata.json"
if [[ -n "$published" ]]; then
    # Refuse path/Git replacement of either the facade or its generator.
    "$root/.tools/host/bin/jq" -e --arg version "$published" --argjson require_generator false \
        -f "$root/scripts/ci/published-runtime.jq" "$fixture/metadata.json" > /dev/null || {
        echo 'Published consumer must resolve the exact registry facade and registry generator' >&2; exit 1;
    }
fi
if [[ "$capability" == executor || "$capability" == ic0 ]]; then
    for graph_target in "$(rustc -vV | sed -n 's/^host: //p')" wasm32-unknown-unknown; do
        cargo tree --manifest-path "$fixture/Cargo.toml" --frozen --target "$graph_target" --edges normal,build --prefix none > "$fixture/$graph_target.graph"
        if grep -Eq '^(ic_principal|serde|serde_core|serde_derive|sha2|digest|crc32fast|data-encoding|candid|ic-agent) ' "$fixture/$graph_target.graph"; then
            echo 'Minimal consumer acquired application types/codecs/crypto' >&2; exit 1
        fi
    done
fi
if [[ "$capability" == cdk || "$capability" == runtime || "$capability" == external-candid ]]; then
    for graph_target in "$(rustc -vV | sed -n 's/^host: //p')" wasm32-unknown-unknown; do
        cargo tree --manifest-path "$fixture/Cargo.toml" --frozen --target "$graph_target" --edges normal,build --prefix none > "$fixture/$graph_target.graph"
        # Bash 3.2 treats an empty array as unset under nounset.
        if [[ "$capability" == external-candid ]]; then
            bash scripts/ci/check-platform-runtime-graph.sh "$fixture/$graph_target.graph" --external-candid
        else
            bash scripts/ci/check-platform-runtime-graph.sh "$fixture/$graph_target.graph"
        fi
        if grep -Eq '^(darling|darling_core|darling_macro) ' "$fixture/$graph_target.graph"; then
            echo 'Runtime/entrypoint consumer acquired retired Darling scaffolding' >&2; exit 1
        fi
        if [[ "$capability" == runtime ]] && grep -q '^ic-platform-macros ' "$fixture/$graph_target.graph"; then
            echo 'Runtime-only consumer acquired the entrypoint generator' >&2; exit 1
        fi
    done
fi
if [[ "$capability" == blake3 ]]; then
    for graph_target in "$(rustc -vV | sed -n 's/^host: //p')" wasm32-unknown-unknown; do
        bash scripts/ci/check-blake3-graph.sh "$fixture" platform-consumer-proof "$graph_target" -
        cargo tree --manifest-path "$fixture/Cargo.toml" --frozen --target "$graph_target" \
            --edges normal,build --prefix none > "$fixture/$graph_target.graph"
        if grep -Eq '^(candid|ic-cdk|ic-platform-macros|ic-agent|pocket-ic|ic_principal|serde) ' "$fixture/$graph_target.graph"; then
            echo 'BLAKE3 consumer acquired unrelated IC/application dependencies' >&2; exit 1
        fi
    done
fi
if [[ "$capability" == external-codecs ]]; then
    for graph_target in "$(rustc -vV | sed -n 's/^host: //p')" wasm32-unknown-unknown; do
        cargo tree --manifest-path "$fixture/Cargo.toml" --frozen --target "$graph_target" --edges normal,build --prefix none > "$fixture/$graph_target.graph"
        if grep -Eq '^(ic-cdk[^ ]*|ic0|ic-platform-macros|ic-agent|pocket-ic|ic-testkit|candid_parser|reqwest|tokio) ' "$fixture/$graph_target.graph"; then
            echo 'External codecs acquired transport, executor or host parsing' >&2; exit 1
        fi
    done
fi
CARGO_TARGET_DIR="$root/target/consumer-build" cargo test --manifest-path "$fixture/Cargo.toml" --frozen
CARGO_TARGET_DIR="$root/target/consumer-msrv" cargo +1.91.0 test --manifest-path "$fixture/Cargo.toml" --frozen
if [[ "$wasm" != skip ]]; then
    CARGO_TARGET_DIR="$root/target/consumer-build" cargo "$wasm" --manifest-path "$fixture/Cargo.toml" --lib --target wasm32-unknown-unknown --frozen
    CARGO_TARGET_DIR="$root/target/consumer-msrv" cargo +1.91.0 "$wasm" --manifest-path "$fixture/Cargo.toml" --lib --target wasm32-unknown-unknown --frozen
fi
printf 'Consumer evidence retained at %s\n' "$fixture"
# Parse the exit with the loop before Cargo runs. A concurrent source edit must
# not make Bash resume at a shifted byte offset in the changed file afterward.
done; exit 0
