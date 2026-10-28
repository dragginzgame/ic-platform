#!/usr/bin/env bash
set -euo pipefail
# Requires an explicitly prepared Testkit server or server URL; never installs it.
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
ic_call=""
if [[ "${1:-}" == --ic-call ]]; then ic_call=1; shift; fi
published=""
if [[ "${1:-}" == --published ]]; then
    stable_version='^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$'
    [[ $# -ge 2 && "$2" =~ $stable_version ]] || exit 2
    published="$2"
    shift 2
fi
[[ -z "$ic_call" || -z "$published" ]] || { echo 'The ic-call fixture selects local sources; omit --published' >&2; exit 2; }
mode="${1:-run}"
[[ $# -le 1 ]] || { echo "Expected [--ic-call | --published X.Y.Z] [mode]" >&2; exit 2; }
case "$mode" in run|--prepare|--check-graph|--build-only|--refresh-lock) ;; *) echo "Usage: $0 [--ic-call | --published X.Y.Z] [run|--prepare|--check-graph|--build-only|--refresh-lock]" >&2; exit 2 ;; esac
[[ ( -z "$published" && -z "$ic_call" ) || "$mode" != --refresh-lock ]] || {
    echo 'Published or ic-call qualification cannot rewrite the maintained local lock' >&2; exit 2;
}
if [[ "$mode" == run && -z "${POCKET_IC_BIN:-}" && -z "${IC_TESTKIT_POCKET_IC_URL:-}" ]]; then
    echo 'Set POCKET_IC_BIN (Testkit-admitted binary) or IC_TESTKIT_POCKET_IC_URL first' >&2
    exit 1
fi
mkdir -p target
fixture="$(mktemp -d "$root/target/cdk-runtime.XXXXXX")"
mkdir -p "$fixture/crates/canister/src/fixtures" "$fixture/apps/driver/src"
# Inherit the reviewed test dependency selections from the root catalog.
development_toolchain="$("$root/.tools/host/bin/yq" -p toml -o yaml -r '.toolchain.channel' rust-toolchain.toml)"
[[ "$development_toolchain" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo 'Expected a pinned development compiler in rust-toolchain.toml' >&2; exit 1; }
testkit="$("$root/.tools/host/bin/yq" -p toml -o yaml -r '.workspace.dependencies.ic-testkit' Cargo.toml)"
timers="$("$root/.tools/host/bin/yq" -p toml -o yaml -r '.workspace.dependencies.ic-timers' Cargo.toml)"
wire="$("$root/.tools/host/bin/yq" -p toml -o yaml -r '.workspace.dependencies.ic-wire' Cargo.toml)"
serde="$("$root/.tools/host/bin/yq" -p toml -o yaml -r '.workspace.dependencies.serde.version' Cargo.toml)"
cbor="$("$root/.tools/host/bin/yq" -p toml -o yaml -r '.workspace.dependencies.serde_cbor' Cargo.toml)"
selection='path = "../../crates/ic-platform"'
if [[ -n "$published" ]]; then selection="version = \"=$published\""; fi
PLATFORM_FIXTURE_PLATFORM="$selection" PLATFORM_FIXTURE_TESTKIT="$testkit" \
PLATFORM_FIXTURE_TIMERS="$timers" PLATFORM_FIXTURE_WIRE="$wire" \
PLATFORM_FIXTURE_SERDE="$serde" PLATFORM_FIXTURE_CBOR="$cbor" \
    perl -pe 's/\@(PLATFORM|TESTKIT|TIMERS|WIRE|SERDE|CBOR)\@/$ENV{"PLATFORM_FIXTURE_$1"}/ge' \
        scripts/fixtures/cdk-runtime/Cargo.toml.in > "$fixture/Cargo.toml"
if [[ -z "$published" ]]; then
    cat >> "$fixture/Cargo.toml" <<'MANIFEST'
[patch.crates-io]
ic-platform = { path = "../../crates/ic-platform" }
MANIFEST
fi
cp scripts/fixtures/cdk-runtime/canister.Cargo.toml "$fixture/crates/canister/Cargo.toml"
if [[ -n "$ic_call" ]]; then
    sed -i.bak 's/ic-call = \[\]/ic-call = ["dep:ic-call"]/' "$fixture/crates/canister/Cargo.toml"
    rm "$fixture/crates/canister/Cargo.toml.bak"
    printf '\nic-call = { path = "../../../../crates/ic-call", default-features = false, optional = true }\n' >> "$fixture/crates/canister/Cargo.toml"
fi
cp scripts/fixtures/cdk-runtime/driver.Cargo.toml "$fixture/apps/driver/Cargo.toml"
cp scripts/fixtures/cdk-runtime/canister.rs "$fixture/crates/canister/src/lib.rs"
cp -R scripts/fixtures/cdk-runtime/{wire_canister,wire_protocol,protocol,codec,transport} "$fixture/crates/canister/src/"
cp scripts/fixtures/cdk-runtime/driver.rs "$fixture/apps/driver/src/main.rs"
cp -R scripts/fixtures/cdk-runtime/{wire_driver,wire_protocol,protocol} "$fixture/apps/driver/src/"
lock="$root/scripts/fixtures/cdk-runtime/Cargo.lock"
# Preparation modes may resolve/fetch only as explicitly requested. Qualification
# below this phase uses the maintained lock and admits graphs before compiling.
if [[ "$mode" == --refresh-lock ]]; then
    # Explicitly rebase on the production lock, resolving the test-only additions.
    # Ordinary validation never resolves or rewrites the maintained selection.
    cp Cargo.lock "$fixture/Cargo.lock"
    cargo metadata --manifest-path "$fixture/Cargo.toml" --filter-platform "$(rustc -vV | sed -n 's/^host: //p')" --format-version 1 > "$fixture/metadata.json"
    cp "$fixture/Cargo.lock" "$lock"
    printf 'Review the refreshed qualification lock: %s\n' "$lock"
    exit 0
fi
cp "$lock" "$fixture/Cargo.lock"
if [[ -n "$ic_call" ]]; then
    # Add only local request-custody packages in the disposable consumer lock.
    # Registry package identities and edges must remain exactly the maintained set.
    resolution=(--offline)
    if [[ "$mode" == --prepare ]]; then resolution=(); fi
    cargo metadata --manifest-path "$fixture/Cargo.toml" ${resolution[@]+"${resolution[@]}"} --filter-platform "$(rustc -vV | sed -n 's/^host: //p')" --format-version 1 > "$fixture/metadata.json"
    for side in baseline consumer; do
        selected="$lock"
        if [[ "$side" == consumer ]]; then selected="$fixture/Cargo.lock"; fi
        "$root/.tools/host/bin/yq" -p toml -o json '.' "$selected" |
            "$root/.tools/host/bin/jq" -S '[.package[] | select(.source != null)] | sort_by(.name, .version, .source)' > "$fixture/$side-registry.json"
    done
    cmp "$fixture/baseline-registry.json" "$fixture/consumer-registry.json"
fi
if [[ "$mode" == --prepare ]]; then
    [[ -z "$published" ]] || { echo "Prepare published caches explicitly before qualification" >&2; exit 2; }
    cargo fetch --manifest-path "$fixture/Cargo.toml" --locked
    printf 'Runtime dependency preparation retained at %s\n' "$fixture"
    exit 0
fi
if [[ -n "$published" ]]; then
    # Resolve only the disposable lock from explicitly prepared registry caches.
    cargo metadata --manifest-path "$fixture/Cargo.toml" --offline --filter-platform "$(rustc -vV | sed -n 's/^host: //p')" --format-version 1 > "$fixture/metadata.json"
    "$root/.tools/host/bin/jq" -e --arg version "$published" --argjson require_generator true -f "$root/scripts/ci/published-runtime.jq" "$fixture/metadata.json" > /dev/null || {
        echo 'Published runtime requires the exact registry facade/generator and no local replacements' >&2
        exit 1
    }
fi
cargo metadata --manifest-path "$fixture/Cargo.toml" --locked --offline --filter-platform "$(rustc -vV | sed -n 's/^host: //p')" --format-version 1 > "$fixture/metadata.json"
transports=(raw)
if [[ -n "$ic_call" ]]; then transports+=(owned); fi
for transport in "${transports[@]}"; do
for graph_target in "$(rustc -vV | sed -n 's/^host: //p')" wasm32-unknown-unknown; do
    for protocol in wire external-candid; do
        features=()
        if [[ "$transport" == owned ]]; then features+=(--features ic-call); fi
        admission=()
        if [[ "$protocol" == external-candid ]]; then features+=(--features external-candid); admission=(--external-candid); fi
        cargo tree --manifest-path "$fixture/Cargo.toml" --locked -p platform-runtime-canister ${features[@]+"${features[@]}"} --target "$graph_target" --edges normal,build --prefix none > "$fixture/$graph_target-$transport-$protocol.graph"
        bash scripts/ci/check-platform-runtime-graph.sh "$fixture/$graph_target-$transport-$protocol.graph" ${admission[@]+"${admission[@]}"}
    done
done
done
if [[ "$mode" == --check-graph ]]; then
    printf 'Runtime graph evidence retained at %s\n' "$fixture"
    exit 0
fi
CARGO_TARGET_DIR="$fixture/build/host" cargo +1.91.0 test --manifest-path "$fixture/Cargo.toml" --locked -p platform-runtime-driver
CARGO_TARGET_DIR="$fixture/build/host" cargo +1.91.0 build --manifest-path "$fixture/Cargo.toml" --locked -p platform-runtime-driver
for transport in "${transports[@]}"; do
for toolchain in "$development_toolchain" 1.91.0; do
    for protocol in wire external-candid; do
        features=()
        if [[ "$transport" == owned ]]; then features+=(--features ic-call); fi
        if [[ "$protocol" == external-candid ]]; then features+=(--features external-candid); fi
        CARGO_TARGET_DIR="$fixture/build/$toolchain" cargo +"$toolchain" build --manifest-path "$fixture/Cargo.toml" --locked -p platform-runtime-canister ${features[@]+"${features[@]}"} --release --target wasm32-unknown-unknown
        mkdir -p "$fixture/artifacts/$toolchain"
        artifact="$protocol"
        if [[ -n "$ic_call" ]]; then artifact="$transport-$protocol"; fi
        cp "$fixture/build/$toolchain/wasm32-unknown-unknown/release/platform_runtime_canister.wasm" "$fixture/artifacts/$toolchain/$artifact.wasm"
        if [[ "$mode" == run ]]; then
            if [[ "$protocol" == external-candid ]]; then
                PLATFORM_EXTERNAL_CANDID=1 "$fixture/build/host/debug/platform-runtime-driver" "$fixture/artifacts/$toolchain/$artifact.wasm"
            else
                (
                    unset PLATFORM_EXTERNAL_CANDID
                    "$fixture/build/host/debug/platform-runtime-driver" "$fixture/artifacts/$toolchain/$artifact.wasm"
                )
            fi
        fi
    done
done
done
printf 'Runtime sources, lock and metadata retained at %s\n' "$fixture"
