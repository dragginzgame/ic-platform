#!/usr/bin/env bash
set -euo pipefail
# Requires an explicitly prepared Testkit server or server URL; never installs it.
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
# shellcheck source=scripts/ci/runtime-selection.sh
source "$root/scripts/ci/runtime-selection.sh"
runtime_selection "$@"
if [[ "$mode" == run && -z "${POCKET_IC_BIN:-}" && -z "${IC_TESTKIT_POCKET_IC_URL:-}" ]]; then
    echo 'Set POCKET_IC_BIN (Testkit-admitted binary) or IC_TESTKIT_POCKET_IC_URL first' >&2
    exit 1
fi
mkdir -p target
fixture="$(mktemp -d "$root/target/cdk-runtime.XXXXXX")"
mkdir -p "$fixture/crates/canister/src/fixtures" "$fixture/apps/driver/src"
lock="scripts/fixtures/cdk-runtime/Cargo.lock"
printf '%s\n' "$qualification" > "$fixture/qualification"
printf '%s\n' "$mode" > "$fixture/operation"
finish_runtime() {
    local result=$?
    trap - EXIT
    refresh=()
    if [[ "$mode" == --refresh-lock ]]; then refresh=(--refresh-lock); fi
    perl "$root/scripts/ci/runtime-inputs.pl" verify "$root" "$fixture" "$lock" \
        ${refresh[@]+"${refresh[@]}"} || result=1
    printf '%s\n' "$result" > "$fixture/status"
    printf 'Runtime %s evidence: %s\n' "$qualification" "$fixture"
    exit "$result"
}
trap finish_runtime EXIT
# Capture before reading catalog/compiler selections or copying any fixture file.
perl "$root/scripts/ci/runtime-inputs.pl" capture "$root" "$fixture" "$lock"
input_root="$fixture/inputs"
mkdir -p "$input_root/.tools"
ln -s "$root/.tools/host" "$input_root/.tools/host"
if [[ "$mode" == --refresh-lock ]]; then
    printf 'not-qualified: lock refresh\n' > "$fixture/source-revision"
else
    git rev-parse HEAD > "$fixture/source-revision"
fi
msrv="$(bash "$input_root/scripts/ci/read-platform-msrv.sh")"
rustc -vV > "$fixture/compiler.txt"
# shellcheck source=scripts/ci/workspace-dependencies.sh
source "$root/scripts/ci/workspace-dependencies.sh"
platform_dependencies_read "$input_root"
testkit="$(platform_dependency ic-testkit)"
wire="$(platform_dependency_version ic-wire)"
serde="$(platform_dependency_version serde)"
tokio="$(platform_dependency tokio)"
selection='path = "../../crates/ic-platform"'
runtime_selection='path = "../../crates/ic-platform-runtime"'
if [[ -n "$published" ]]; then
    runtime_selection="version = \"=$published\""
    facade_version="$("$root/.tools/host/bin/yq" -p toml -o json -r '.package.version' "$input_root/crates/ic-platform/Cargo.toml")"
    selection="version = \"=$facade_version\""
fi
PLATFORM_FIXTURE_RUNTIME="$runtime_selection" PLATFORM_FIXTURE_MSRV="$msrv" PLATFORM_FIXTURE_PLATFORM="$selection" \
PLATFORM_FIXTURE_TESTKIT="$testkit" PLATFORM_FIXTURE_WIRE="$wire" PLATFORM_FIXTURE_SERDE="$serde" \
PLATFORM_FIXTURE_TOKIO="$tokio" \
    perl "$input_root/scripts/ci/render-runtime-fixture.pl" "$input_root/scripts/fixtures/cdk-runtime" "$fixture" \
        "$qualification" "$selection_mode" "$adapter_mode"
cp "$input_root"/scripts/fixtures/cdk-runtime/canister/mod.rs "$fixture/crates/canister/src/lib.rs"
cp "$input_root"/scripts/fixtures/cdk-runtime/canister/tests.rs "$fixture/crates/canister/src/tests.rs"
mkdir -p "$fixture/crates/canister/src/management_proof/fixtures"
cp "$input_root"/crates/ic-platform/tests/management_consumer.rs "$fixture/crates/canister/src/management_proof/mod.rs"
cp -R "$input_root"/crates/ic-platform/tests/support "$fixture/crates/canister/src/management_proof/"
cp "$input_root"/crates/ic-platform/tests/fixtures/management-*.hex "$fixture/crates/canister/src/management_proof/fixtures/"
cp -R "$input_root"/scripts/fixtures/cdk-runtime/{wire_canister,wire_protocol,protocol,codec,transport,external_canister,execution,cycles,stable_checks,system_checks} "$fixture/crates/canister/src/"
cp "$input_root"/scripts/fixtures/cdk-runtime/driver/mod.rs "$fixture/apps/driver/src/main.rs"
cp -R "$input_root/scripts/fixtures/cdk-runtime/pocketic_support" "$fixture/apps/driver/src/"
cp "$input_root"/scripts/fixtures/cdk-runtime/driver/tests.rs "$fixture/apps/driver/src/tests.rs"
cp -R "$input_root"/scripts/fixtures/cdk-runtime/{wire_driver,wire_protocol,protocol,external_driver,benchmarks,driver_scenarios} "$fixture/apps/driver/src/"
platform_fixture_policy "$input_root" "$fixture"
# Preparation modes may resolve/fetch only as explicitly requested. Qualification
# below this phase uses the maintained lock and admits graphs before compiling.
if [[ "$mode" == --refresh-lock ]]; then
    # Preserve the maintained foreign selections while resolving necessary
    # manifest/owner changes. Ordinary validation never rewrites this lock.
    cp "$input_root/$lock" "$fixture/Cargo.lock"
    cargo metadata --manifest-path "$fixture/Cargo.toml" --filter-platform "$(rustc -vV | sed -n 's/^host: //p')" --format-version 1 > "$fixture/metadata.json"
    cp "$fixture/Cargo.lock" "$lock"
    printf 'Review the refreshed qualification lock: %s\n' "$lock"
    exit 0
fi
cp "$input_root/$lock" "$fixture/Cargo.lock"

if [[ "$mode" == --prepare ]]; then
    cargo fetch --manifest-path "$fixture/Cargo.toml" --locked
    printf 'Runtime dependency preparation retained at %s\n' "$fixture"
    exit 0
fi
if [[ -n "$published" ]]; then
    # Resolve only the disposable lock from explicitly prepared registry caches.
    cargo metadata --manifest-path "$fixture/Cargo.toml" --offline --filter-platform "$(rustc -vV | sed -n 's/^host: //p')" --format-version 1 > "$fixture/metadata.json"
fi
cargo metadata --manifest-path "$fixture/Cargo.toml" --locked --offline --filter-platform "$(rustc -vV | sed -n 's/^host: //p')" --format-version 1 > "$fixture/metadata.json"
bash "$root/scripts/ci/check-cargo-dependency-boundaries.sh" --consumer "$fixture" --external-ic
# One selection owner for graph admission, compilation, artifacts and execution.
profiles=(raw-wire raw-external-candid)
if [[ "$transport_mode" == prepared ]]; then profiles+=(owned-wire owned-external-candid); fi
select_runtime_profile() {
    transport="${1%%-*}" protocol="${1#*-}"
    selected_features=() selected_admission=() selected_external=""
    case "$1" in
        raw-wire|owned-wire) ;;
        raw-external-candid|owned-external-candid)
            selected_features=(--features external-candid)
            if [[ "$adapter_mode" == external ]]; then selected_features=(--features external-adapters); fi
            selected_admission=(--external-candid)
            selected_external=1 ;;
        *) echo "Unknown runtime profile: $1" >&2; exit 2 ;;
    esac
    if [[ "$transport" == owned ]]; then selected_features+=(--features prepared-call); fi
    selected_artifact="$protocol"
    if [[ "$transport_mode" == prepared ]]; then selected_artifact="$1"; fi
}
for profile in "${profiles[@]}"; do
    for graph_target in "$(rustc -vV | sed -n 's/^host: //p')" wasm32-unknown-unknown; do
        select_runtime_profile "$profile"
        cargo tree --manifest-path "$fixture/Cargo.toml" --locked -p platform-runtime-canister ${selected_features[@]+"${selected_features[@]}"} --target "$graph_target" --edges normal,build --prefix none > "$fixture/$graph_target-$transport-$protocol.graph"
        cargo metadata --manifest-path "$fixture/Cargo.toml" --locked --offline --filter-platform "$graph_target" \
            ${selected_features[@]+"${selected_features[@]}"} --format-version 1 > "$fixture/$graph_target-$transport-$protocol.metadata.json"
        required=ic-platform-runtime,ic-platform-macros,ic-platform-runtime-guard
        if [[ "$protocol" == external-candid ]]; then required+=,ic-platform; fi
        bash "$root/scripts/ci/check-qualification-identities.sh" "$fixture/$graph_target-$transport-$protocol.metadata.json" \
            "$root" platform-runtime-canister "$required" "$selection_mode" "$published" \
            > "$fixture/$graph_target-$transport-$protocol.identities.json"
        bash scripts/ci/check-platform-runtime-graph.sh "$fixture/$graph_target-$transport-$protocol.graph" ${selected_admission[@]+"${selected_admission[@]}"}
        if [[ "$protocol" == wire ]] && grep -q '^ic-platform ' "$fixture/$graph_target-$transport-$protocol.graph"; then
            echo 'Runtime-only canister acquired the crypto facade' >&2; exit 1
        fi
        if grep -q '^ic-timers ' "$fixture/$graph_target-$transport-$protocol.graph"; then
            echo 'Owner runtime proof acquired Timers' >&2; exit 1
        fi
        if [[ "$graph_target" == wasm* ]] && grep -q '^pretty ' "$fixture/$graph_target-$transport-$protocol.graph"; then
            echo 'Canister runtime acquired an unselected Candid service printer' >&2; exit 1
        fi
    done
done
bash "$root/scripts/ci/check-qualification-identities.sh" "$fixture/metadata.json" "$root" \
    platform-runtime-driver ic-platform,ic-testkit "$selection_mode" "$published" > "$fixture/driver-identities.json"
if [[ "$mode" == --check-graph ]]; then
    printf 'Runtime graph evidence retained at %s\n' "$fixture"
    exit 0
fi
build="${CARGO_TARGET_DIR:-$root/target/runtime-build}"
bash "$root/scripts/ci/check-platform-compiler.sh" > "$fixture/compiler-admission.txt"
mkdir -p "$fixture/artifacts"
# shellcheck source=scripts/ci/retain-build-artifact.sh
source "$root/scripts/ci/retain-build-artifact.sh"
driver_features=()
build_runtime_driver() {
    cargo +"$msrv" clippy --manifest-path "$fixture/Cargo.toml" --locked -p platform-runtime-driver --all-targets ${driver_features[@]+"${driver_features[@]}"} -- -D warnings || return
    cargo +"$msrv" test --manifest-path "$fixture/Cargo.toml" --locked -p platform-runtime-driver ${driver_features[@]+"${driver_features[@]}"} || return
    cargo +"$msrv" build --manifest-path "$fixture/Cargo.toml" --locked -p platform-runtime-driver ${driver_features[@]+"${driver_features[@]}"}
}
retain_build_artifact "$build/host-$msrv-$qualification" debug/platform-runtime-driver \
    "$fixture/artifacts/platform-runtime-driver" build_runtime_driver
toolchain="$msrv"
rustc +"$toolchain" -vV > "$fixture/compiler-$toolchain.txt"
for profile in "${profiles[@]}"; do
        select_runtime_profile "$profile"
        mkdir -p "$fixture/artifacts/$toolchain"
        cargo +"$toolchain" clippy --manifest-path "$fixture/Cargo.toml" --locked -p platform-runtime-canister ${selected_features[@]+"${selected_features[@]}"} --lib --target wasm32-unknown-unknown -- -D warnings
        retain_build_artifact "$build/$toolchain" \
            wasm32-unknown-unknown/release/platform_runtime_canister.wasm \
            "$fixture/artifacts/$toolchain/$selected_artifact.wasm" \
            cargo +"$toolchain" build --manifest-path "$fixture/Cargo.toml" --locked -p platform-runtime-canister ${selected_features[@]+"${selected_features[@]}"} --release --target wasm32-unknown-unknown
        printf '%s %s %s target=wasm32-unknown-unknown features=%s\n' "$toolchain" "$transport" "$protocol" "${selected_features[*]:-}" >> "$fixture/profiles.log"
        perl -MDigest::SHA -e 'for (@ARGV) { open my $f, "<", $_ or die $!; binmode $f; print Digest::SHA->new(256)->addfile($f)->hexdigest, "  $_\n"; }' \
            "$fixture/artifacts/$toolchain/$selected_artifact.wasm" >> "$fixture/artifacts.sha256"
        if [[ "$mode" == run ]]; then
            if [[ -n "$selected_external" ]]; then
                (
                    export PLATFORM_EXTERNAL_CANDID=1
                    if [[ "$adapter_mode" == external ]]; then export PLATFORM_EXTERNAL_ADAPTERS=1; else unset PLATFORM_EXTERNAL_ADAPTERS; fi
                    "$fixture/artifacts/platform-runtime-driver" "$fixture/artifacts/$toolchain/$selected_artifact.wasm"
                )
            else
                (
                    unset PLATFORM_EXTERNAL_CANDID PLATFORM_EXTERNAL_ADAPTERS
                    "$fixture/artifacts/platform-runtime-driver" "$fixture/artifacts/$toolchain/$selected_artifact.wasm"
                )
            fi
            printf 'PASS %s %s %s\n' "$toolchain" "$transport" "$protocol" >> "$fixture/profiles.log"
        fi
done
printf 'Runtime sources, lock and metadata retained at %s\n' "$fixture"
