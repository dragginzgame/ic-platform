#!/usr/bin/env bash
set -euo pipefail
# Requires an explicitly prepared Testkit server or server URL; never installs it.
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
qualification=owner
if [[ "${1:-}" == --timers ]]; then qualification=timers; shift; fi
adapters=""
if [[ "${1:-}" == --external-adapters ]]; then adapters=1; shift; fi
prepared_call=""
if [[ "${1:-}" == --prepared-call ]]; then prepared_call=1; shift; fi
published=""
if [[ "${1:-}" == --published ]]; then
    stable_version='^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$'
    [[ $# -ge 2 && "$2" =~ $stable_version ]] || exit 2
    published="$2"
    shift 2
fi
[[ -z "$prepared_call" || -z "$published" ]] || { echo 'The prepared-call fixture selects local sources; omit --published' >&2; exit 2; }
if [[ -z "$published" ]]; then adapters=1; fi
external_feature=external-candid
if [[ -n "$adapters" ]]; then external_feature=external-adapters; fi
mode="${1:-run}"
[[ $# -le 1 ]] || { echo "Expected [--timers] [--external-adapters] [--prepared-call | --published X.Y.Z] [mode]" >&2; exit 2; }
case "$mode" in run|--prepare|--check-graph|--build-only|--refresh-lock) ;; *) echo "Usage: $0 [--timers] [--external-adapters] [--prepared-call | --published X.Y.Z] [run|--prepare|--check-graph|--build-only|--refresh-lock]" >&2; exit 2 ;; esac
[[ ( -z "$published" && -z "$prepared_call" ) || "$mode" != --refresh-lock ]] || {
    echo 'Published or prepared-call qualification cannot rewrite the maintained local lock' >&2; exit 2;
}
if [[ "$mode" == run && -z "${POCKET_IC_BIN:-}" && -z "${IC_TESTKIT_POCKET_IC_URL:-}" ]]; then
    echo 'Set POCKET_IC_BIN (Testkit-admitted binary) or IC_TESTKIT_POCKET_IC_URL first' >&2
    exit 1
fi
msrv=1.91.0
if [[ -z "$published" ]]; then msrv="$(bash "$root/scripts/ci/read-platform-msrv.sh")"; fi
mkdir -p target
fixture="$(mktemp -d "$root/target/cdk-runtime.XXXXXX")"
mkdir -p "$fixture/crates/canister/src/fixtures" "$fixture/apps/driver/src"
# Inherit the reviewed test dependency selections from the root catalog.
development_toolchain="$("$root/.tools/host/bin/yq" -p toml -o yaml -r '.toolchain.channel' rust-toolchain.toml)"
[[ "$development_toolchain" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo 'Expected a pinned development compiler in rust-toolchain.toml' >&2; exit 1; }
testkit="$("$root/.tools/host/bin/yq" -p toml -o yaml -r '.workspace.dependencies.ic-testkit' Cargo.toml)"
wire="$("$root/.tools/host/bin/yq" -p toml -o yaml -r '.workspace.dependencies.ic-wire' Cargo.toml)"
serde="$("$root/.tools/host/bin/yq" -p toml -o yaml -r '.workspace.dependencies.serde.version' Cargo.toml)"
cbor="$("$root/.tools/host/bin/yq" -p toml -o yaml -r '.workspace.dependencies.serde_cbor' Cargo.toml)"
selection='path = "../../crates/ic-platform"'
runtime_selection='path = "../../crates/ic-platform-runtime"'
if [[ -n "$published" ]]; then
    runtime_selection="version = \"=$published\""
    facade_version="$("$root/.tools/host/bin/yq" -p toml -o json -r '.package.version' crates/ic-platform/Cargo.toml)"
    selection="version = \"=$facade_version\""
fi
PLATFORM_FIXTURE_RUNTIME="$runtime_selection" PLATFORM_FIXTURE_MSRV="$msrv" PLATFORM_FIXTURE_PLATFORM="$selection" PLATFORM_FIXTURE_TESTKIT="$testkit" \
PLATFORM_FIXTURE_WIRE="$wire" \
PLATFORM_FIXTURE_SERDE="$serde" PLATFORM_FIXTURE_CBOR="$cbor" \
    perl -pe 's/\@(RUNTIME|PLATFORM|TESTKIT|WIRE|SERDE|CBOR|MSRV)\@/$ENV{"PLATFORM_FIXTURE_$1"}/ge' \
        scripts/fixtures/cdk-runtime/Cargo.toml.in > "$fixture/Cargo.toml"
# Timers and the canister share the canonical Runtime owner.
# This qualification-only patch tests existing compatible consumer requirements;
# published consumers must adopt the supplying release themselves.
if [[ -z "$published" ]]; then
    cat >> "$fixture/Cargo.toml" <<'PATCH'

[patch.crates-io]
ic-platform = { path = "../../crates/ic-platform" }
ic-platform-runtime = { path = "../../crates/ic-platform-runtime" }
ic-platform-macros = { path = "../../crates/ic-platform-macros" }
ic-platform-runtime-guard = { path = "../../crates/ic-platform-runtime-guard" }
PATCH
fi
cp scripts/fixtures/cdk-runtime/canister.Cargo.toml "$fixture/crates/canister/Cargo.toml"
if [[ -n "$adapters" ]]; then
    sed -i.bak 's/external-adapters = \["external-candid"\]/external-adapters = ["external-candid", "ic-platform\/candid-endpoint"]/' "$fixture/crates/canister/Cargo.toml"
    rm "$fixture/crates/canister/Cargo.toml.bak"
fi

cp scripts/fixtures/cdk-runtime/driver.Cargo.toml "$fixture/apps/driver/Cargo.toml"
if [[ "$qualification" == timers ]]; then
    timers="$("$root/.tools/host/bin/yq" -p toml -o json '.workspace.dependencies.ic-timers' Cargo.toml | "$root/.tools/host/bin/jq" -r 'if type == "string" then {version: .} else . end | to_entries | map("\(.key) = \(.value | tojson)") | join(", ")')"
    printf '\nic-timers = { %s }\n' "$timers" >> "$fixture/crates/canister/Cargo.toml"
fi
cp scripts/fixtures/cdk-runtime/canister.rs "$fixture/crates/canister/src/lib.rs"
mkdir -p "$fixture/crates/canister/src/management_proof/fixtures"
cp crates/ic-platform/tests/management_consumer.rs "$fixture/crates/canister/src/management_proof/mod.rs"
cp crates/ic-platform/tests/fixtures/management-*.hex "$fixture/crates/canister/src/management_proof/fixtures/"
cp -R scripts/fixtures/cdk-runtime/{wire_canister,wire_protocol,protocol,codec,transport,external_canister} "$fixture/crates/canister/src/"
cp scripts/fixtures/cdk-runtime/driver.rs "$fixture/apps/driver/src/main.rs"
cp -R scripts/fixtures/cdk-runtime/{wire_driver,wire_protocol,protocol,external_driver} "$fixture/apps/driver/src/"
lock="$root/scripts/fixtures/cdk-runtime/Cargo.lock"
if [[ "$qualification" == timers ]]; then lock="$root/scripts/fixtures/timers-runtime/Cargo.lock"; fi
printf '%s\n' "$qualification" > "$fixture/qualification"
printf '%s\n' "$mode" > "$fixture/operation"
# Bind each result to the actual source, selected lock, generated manifests and
# compiler identities. Retain failed preparation as well as installed results.
{
    "$root/.tools/host/bin/rg" --files crates scripts/fixtures/cdk-runtime | sed "\|^scripts/fixtures/cdk-runtime/Cargo.lock$|d"
    printf '%s\n' Cargo.toml rust-toolchain.toml .cargo/config.toml scripts/ci/check-platform-runtime.sh "$lock"
    if [[ "$mode" == --refresh-lock ]]; then printf '%s\n' Cargo.lock; fi
} | LC_ALL=C sort -u > "$fixture/input-files"
source_fingerprints() {
    perl -MDigest::SHA -ne '
        chomp; open my $f, "<", $_ or die "$_: $!"; binmode $f;
        print Digest::SHA->new(256)->addfile($f)->hexdigest, "  $_\n";
    ' "$fixture/input-files"
}
source_fingerprints > "$fixture/source.sha256"
git rev-parse HEAD > "$fixture/source-revision"
rustc -vV > "$fixture/compiler.txt"
finish_runtime() {
    local result=$?
    trap - EXIT
    source_fingerprints > "$fixture/source-after.sha256" || result=1
    for side in source source-after; do
        if [[ "$mode" == --refresh-lock ]]; then
            grep -Fv "  $lock" "$fixture/$side.sha256" > "$fixture/$side.comparable"
        else
            cp "$fixture/$side.sha256" "$fixture/$side.comparable"
        fi
    done
    if ! cmp -s "$fixture/source.comparable" "$fixture/source-after.comparable"; then
        echo 'Runtime qualification inputs changed during observation' >&2
        result=1
    fi
    printf '%s\n' "$result" > "$fixture/status"
    printf 'Runtime %s evidence: %s\n' "$qualification" "$fixture"
    exit "$result"
}
trap finish_runtime EXIT
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

if [[ "$mode" == --prepare ]]; then
    [[ -z "$published" ]] || { echo "Prepare published caches explicitly before qualification" >&2; exit 2; }
    cargo fetch --manifest-path "$fixture/Cargo.toml" --locked
    printf 'Runtime dependency preparation retained at %s\n' "$fixture"
    exit 0
fi
if [[ -n "$published" ]]; then
    # Resolve only the disposable lock from explicitly prepared registry caches.
    cargo metadata --manifest-path "$fixture/Cargo.toml" --offline --filter-platform "$(rustc -vV | sed -n 's/^host: //p')" --format-version 1 > "$fixture/metadata.json"
    "$root/.tools/host/bin/jq" -e --arg version "$published" --argjson runtime_owner true --argjson require_generator true -f "$root/scripts/ci/published-runtime.jq" "$fixture/metadata.json" > /dev/null || {
        echo 'Published runtime requires the exact registry Runtime/generator and no local replacements' >&2
        exit 1
    }
fi
cargo metadata --manifest-path "$fixture/Cargo.toml" --locked --offline --filter-platform "$(rustc -vV | sed -n 's/^host: //p')" --format-version 1 > "$fixture/metadata.json"
# One selection owner for graph admission, compilation, artifacts and execution.
select_runtime_profile() {
    selected_features=()
    if [[ "$qualification" == timers ]]; then selected_features+=(--features timers-composition); fi
    selected_admission=()
    selected_external=""
    selected_artifact="$2"
    if [[ -n "$prepared_call" ]]; then selected_artifact="$1-$2"; fi
    if [[ "$1" == owned ]]; then selected_features+=(--features prepared-call); fi
    if [[ "$2" == external-candid ]]; then
        selected_features+=(--features "$external_feature")
        selected_admission=(--external-candid)
        selected_external=1
    fi
}
transports=(raw)
if [[ -n "$prepared_call" ]]; then transports+=(owned); fi
for transport in "${transports[@]}"; do
for graph_target in "$(rustc -vV | sed -n 's/^host: //p')" wasm32-unknown-unknown; do
    for protocol in wire external-candid; do
        select_runtime_profile "$transport" "$protocol"
        if [[ "$graph_target" != wasm* && -n "$selected_external" ]]; then
            selected_admission+=(--native-management)
        fi
        cargo tree --manifest-path "$fixture/Cargo.toml" --locked -p platform-runtime-canister ${selected_features[@]+"${selected_features[@]}"} --target "$graph_target" --edges normal,build --prefix none > "$fixture/$graph_target-$transport-$protocol.graph"
        bash scripts/ci/check-platform-runtime-graph.sh "$fixture/$graph_target-$transport-$protocol.graph" ${selected_admission[@]+"${selected_admission[@]}"}
        if [[ "$protocol" == wire ]] && grep -q '^ic-platform ' "$fixture/$graph_target-$transport-$protocol.graph"; then
            echo 'Runtime-only canister acquired the crypto facade' >&2; exit 1
        fi
        if [[ "$qualification" == owner ]]; then
            if grep -q '^ic-timers ' "$fixture/$graph_target-$transport-$protocol.graph"; then
                echo 'Owner runtime proof acquired Timers' >&2; exit 1
            fi
        else
            grep -q '^ic-timers ' "$fixture/$graph_target-$transport-$protocol.graph" || { echo 'Composition proof omitted Timers' >&2; exit 1; }
        fi
        if [[ "$graph_target" == wasm* ]] && grep -q '^pretty ' "$fixture/$graph_target-$transport-$protocol.graph"; then
            echo 'Canister runtime acquired an unselected Candid service printer' >&2; exit 1
        fi
    done
done
done
if [[ "$mode" == --check-graph ]]; then
    printf 'Runtime graph evidence retained at %s\n' "$fixture"
    exit 0
fi
build="${CARGO_TARGET_DIR:-$root/target/runtime-build}"
mkdir -p "$fixture/artifacts"
# Cargo releases its target lock before returning. Keep our own inherited lock
# through the copy, so another proof cannot replace the selected artifact first.
# Perl flock is available on both supported Unix hosts; the subshell owns fd 9.
retain_runtime_artifact() (
    cache="$1" artifact="$2" destination="$3"
    shift 3
    mkdir -p "$cache"
    exec 9>>"$cache/.platform-artifact.lock"
    perl -MFcntl=:flock -e 'open my $lock, "<&=9" or die "$ARGV[0]: $!"; flock($lock, LOCK_EX) or die "$ARGV[0]: $!"' "$cache/.platform-artifact.lock"
    export CARGO_TARGET_DIR="$cache"
    "$@"
    cp "$cache/$artifact" "$destination"
)
driver_features=()
if [[ "$qualification" == timers ]]; then driver_features=(--features timers-composition); fi
build_runtime_driver() {
    cargo +"$msrv" test --manifest-path "$fixture/Cargo.toml" --locked -p platform-runtime-driver ${driver_features[@]+"${driver_features[@]}"}
    cargo +"$msrv" build --manifest-path "$fixture/Cargo.toml" --locked -p platform-runtime-driver ${driver_features[@]+"${driver_features[@]}"}
}
retain_runtime_artifact "$build/host-$msrv-$qualification" debug/platform-runtime-driver \
    "$fixture/artifacts/platform-runtime-driver" build_runtime_driver
for transport in "${transports[@]}"; do
for toolchain in "$development_toolchain" "$msrv"; do
    rustc +"$toolchain" -vV > "$fixture/compiler-$toolchain.txt"
    for protocol in wire external-candid; do
        select_runtime_profile "$transport" "$protocol"
        mkdir -p "$fixture/artifacts/$toolchain"
        retain_runtime_artifact "$build/$toolchain" \
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
                    if [[ -n "$adapters" ]]; then export PLATFORM_EXTERNAL_ADAPTERS=1; else unset PLATFORM_EXTERNAL_ADAPTERS; fi
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
done
done
printf 'Runtime sources, lock and metadata retained at %s\n' "$fixture"
