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
capabilities=(executor principal ecdsa ed25519 candid candid-service external-candid certification candid-parser dynamic agent management ic0 icrc bls canister-sig-creation signature-verification cdk runtime secp256k1 pocket-ic stable-structures)
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
features="\"$capability\""
if [[ "$capability" == executor ]]; then features='"executor", "ic0"'; fi
if [[ "$capability" == external-candid ]]; then features='"cdk", "candid-service"'; fi
if [[ "$capability" == agent ]]; then features='"agent", "certification"'; fi
if [[ "$capability" == dynamic ]]; then features='"candid", "candid-parser"'; fi
if [[ "$capability" == management ]]; then features='"management-canister-types", "candid"'; fi
if [[ "$capability" == icrc ]]; then features='"icrc-ledger-types", "candid"'; fi
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
[dependencies]
ic-platform.workspace = true
MANIFEST
if [[ "$capability" == principal ]]; then
    printf '\n[features]\ncandid = ["ic-platform/candid"]\n' >> "$fixture/crates/consumer/Cargo.toml"
fi
source="$capability"
if [[ "$capability" == external-candid ]]; then source=external_candid; fi
if [[ "$capability" == stable-structures ]]; then source=stable_structures; fi
if [[ "$capability" == candid-parser ]]; then source=parser; fi
if [[ "$capability" == candid-service ]]; then source=service; fi
if [[ "$capability" == pocket-ic ]]; then source=pocket_ic; fi
if [[ "$capability" == canister-sig-creation ]]; then source=canister_sig; fi
if [[ "$capability" == signature-verification ]]; then source=signature; fi
if [[ "$capability" == principal ]]; then source=principal; else source="${source}_consumer"; fi
cp "crates/ic-platform/tests/${source}.rs" "$fixture/crates/consumer/src/lib.rs"
if [[ "$capability" == management ]]; then
    mkdir -p "$fixture/crates/consumer/src/fixtures"
    cp crates/ic-platform/tests/fixtures/management-*.hex "$fixture/crates/consumer/src/fixtures/"
fi
if [[ "$capability" == icrc ]]; then
    mkdir -p "$fixture/crates/consumer/src/fixtures"
    cp crates/ic-platform/tests/fixtures/icrc-*.hex "$fixture/crates/consumer/src/fixtures/"
fi
if [[ "$capability" == dynamic || "$capability" == agent ]]; then
    mkdir -p "$fixture/crates/consumer/src/fixtures"
    cp "crates/ic-platform/tests/fixtures/$capability-"*.hex "$fixture/crates/consumer/src/fixtures/"
fi
if [[ "$capability" == bls || "$capability" == signature-verification ]]; then
    mkdir -p "$fixture/crates/consumer/src/fixtures"
    cp crates/ic-platform/tests/fixtures/bls-*.hex "$fixture/crates/consumer/src/fixtures/"
fi
if [[ "$capability" == signature-verification ]]; then
    cp crates/ic-platform/tests/fixtures/signature-*.hex "$fixture/crates/consumer/src/fixtures/"
fi
if [[ "$capability" == cdk || "$capability" == external-candid ]]; then
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
    "$root/.tools/host/bin/jq" -e --arg version "$published" '
        [.packages[] | select(.name == "ic-platform")] as $facades |
        ($facades | length) == 1 and $facades[0].version == $version and
        all(.packages[] | select(.name == "ic-platform" or .name == "ic-platform-macros");
            .source == "registry+https://github.com/rust-lang/crates.io-index")
    ' "$fixture/metadata.json" > /dev/null || {
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
CARGO_TARGET_DIR="$root/target/consumer-build" cargo test --manifest-path "$fixture/Cargo.toml" --frozen
CARGO_TARGET_DIR="$root/target/consumer-msrv" cargo +1.88.0 test --manifest-path "$fixture/Cargo.toml" --frozen
if [[ "$capability" != candid-parser && "$capability" != dynamic && "$capability" != agent ]]; then
    command=check
    if [[ "$capability" == cdk || "$capability" == external-candid ]]; then command=build; fi
    CARGO_TARGET_DIR="$root/target/consumer-build" cargo "$command" --manifest-path "$fixture/Cargo.toml" --lib --target wasm32-unknown-unknown --frozen
    CARGO_TARGET_DIR="$root/target/consumer-msrv" cargo +1.88.0 "$command" --manifest-path "$fixture/Cargo.toml" --lib --target wasm32-unknown-unknown --frozen
fi
printf 'Consumer evidence retained at %s\n' "$fixture"
done
