#!/usr/bin/env bash
# Run with Bash 3.2 on macOS as well as the prepared development shell.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
mkdir -p "$root/target"
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$root/scripts/ci/fixture-lifecycle.sh"
fixture_complete=false
fixture="$(fixture_create "$root/target/prepared-call-admission.XXXXXX")"
fixture="$(CDPATH='' cd "$fixture" && pwd -P)"
first_pid="" second_pid=""
finish() {
    local status=$?
    for pid in "$first_pid" "$second_pid"; do
        [[ -z "$pid" ]] || { kill "$pid" 2>/dev/null || :; wait "$pid" 2>/dev/null || :; }
    done
    fixture_finish "$status" "$fixture" 'Admission evidence retained'
}
trap finish EXIT
# Catalog projection keeps complete tables and explicit version selections. It
# takes one snapshot: later manifest edits cannot mix suppliers within one fixture.
# Like a Git archive used by the Wasm benchmark, the catalog has no tools.
mkdir -p "$fixture/catalog"
cat > "$fixture/catalog/Cargo.toml" <<'CATALOG'
[workspace.dependencies]
scalar = "1.2.3"
table = { version = "=4.5.6", default-features = false, features = ["one"] }
source = { git = "https://example.invalid/source", rev = "immutable" }
CATALOG
# shellcheck source=scripts/ci/workspace-dependencies.sh
source "$root/scripts/ci/workspace-dependencies.sh"
platform_dependencies_read "$fixture/catalog"
[[ "$(platform_dependency scalar)" == 'version = "1.2.3"' ]]
[[ "$(platform_dependency_version table)" == '=4.5.6' ]]
[[ "$(platform_dependency table)" == 'version = "=4.5.6", default-features = false, features = ["one"]' ]]
[[ "$(platform_dependency source)" == 'git = "https://example.invalid/source", rev = "immutable"' ]]
: > "$fixture/catalog/Cargo.toml"
[[ "$(platform_dependency_version scalar)" == '1.2.3' ]]
if platform_dependency missing > "$fixture/catalog/missing" 2>&1; then exit 1; fi
if platform_dependency_version source > "$fixture/catalog/version" 2>&1; then exit 1; fi
mkdir -p "$fixture/scripts/ci" "$fixture/scripts/fixtures/prepared-call-consumer" "$fixture/.tools" "$fixture/bin"
cp "$root/scripts/ci/"{check-prepared-call-consumer.sh,check-platform-runtime.sh,runtime-selection.sh,runtime-inputs.pl,retain-build-artifact.sh,workspace-dependencies.sh,render-runtime-fixture.pl,check-platform-runtime-graph.sh} "$fixture/scripts/ci/"
cp "$root/scripts/ci/platform-graph-policy.awk" "$fixture/scripts/ci/"
cp "$root/scripts/ci/"{check-qualification-identities.sh,qualification-identities.jq} "$fixture/scripts/ci/"
cp "$root/scripts/fixtures/prepared-call-consumer/"{lib,tests}.rs "$fixture/scripts/fixtures/prepared-call-consumer/"
cp -R "$root/scripts/fixtures/cdk-runtime" "$fixture/scripts/fixtures/"
mkdir -p "$fixture/crates/ic-platform/tests/fixtures"
cp "$root/crates/ic-platform/tests/management_consumer.rs" "$fixture/crates/ic-platform/tests/"
cp -R "$root/crates/ic-platform/tests/support" "$fixture/crates/ic-platform/tests/"
cp "$root/crates/ic-platform/tests/fixtures/management-"*.hex "$fixture/crates/ic-platform/tests/fixtures/"
cp "$root/"{Cargo.toml,Cargo.lock,rust-toolchain.toml} "$fixture/"
mkdir -p "$fixture/.cargo"
cp "$root/.cargo/config.toml" "$fixture/.cargo/"
cp "$root/scripts/ci/consumer-fixture.sh" "$fixture/scripts/ci/"
bash "$root/scripts/ci/prepare-script-fixture.sh" "$root" "$fixture" scripts/ci/check-cargo-dependency-boundaries.sh
ln -s "$root/.tools/host" "$fixture/.tools/host"
ln -s "$BASH" "$fixture/bin/bash"
cp "$root/scripts/ci/"{read-platform-msrv.sh,check-platform-compiler.sh} "$fixture/scripts/ci/"
export PLATFORM_ADMISSION_TOOLS="$root/scripts/fixtures/admission-tools"
export PLATFORM_ADMISSION_PACKAGE=platform-runtime-driver
export ADMISSION_SOURCE_ROOT="$fixture"
cp "$PLATFORM_ADMISSION_TOOLS/rustc.sh" "$fixture/bin/rustc"
cat > "$fixture/bin/cargo" <<'CARGO'
#!/usr/bin/env bash
set -euo pipefail
for arg in "$@"; do [[ -n "$arg" ]] || { echo 'empty argument reached Cargo' >&2; exit 31; }; done
printf '%s\n' "$*" >> "$ADMISSION_CALLS"
# Exercise real identity admission against a synthetic resolved owner graph.
# --no-deps remains the shared boundary fixture, including its CBOR refusal.
if [[ "$1" == metadata && " $* " != *' --no-deps '* ]]; then
    external=false
    case " $* " in *' --features external-candid '*|*' --features external-adapters '*) external=true ;; esac
    jq -n --arg root "$ADMISSION_SOURCE_ROOT" --argjson external "$external" '
        ["ic-platform", "ic-platform-types", "ic-platform-runtime", "ic-platform-macros",
         "ic-platform-runtime-guard", "ic-testkit", "platform-runtime-driver", "platform-runtime-canister"] as $names |
        {packages: [$names[] | {id: ., name: ., version: "0.0.0", source: null,
            manifest_path: ($root + "/crates/" + . + "/Cargo.toml")}],
         resolve: {nodes: [$names[] | . as $name | {id: $name, deps:
            ((if $name == "platform-runtime-driver" then ["ic-platform", "ic-testkit"]
              elif $name == "platform-runtime-canister" then ["ic-platform-runtime", "ic-platform-macros"] +
                (if $external then ["ic-platform"] else [] end)
              elif $name == "ic-platform-runtime" then ["ic-platform-runtime-guard", "ic-platform-types"]
              elif $name == "ic-platform" then ["ic-platform-types"] else [] end) |
             map({pkg: ., dep_kinds: [{kind: null}]}))}]}}'
    exit 0
fi
# shellcheck source=scripts/fixtures/admission-tools/cargo.sh
source "$PLATFORM_ADMISSION_TOOLS/cargo.sh"
case "$1" in
    tree)
        printf 'ic-platform-runtime v0.17.0 (/local/runtime)\n'
        case " $* " in *' --features external-candid '*|*' --features external-adapters '*) printf 'ic-platform v0.17.0 (/local/platform)\nnum-bigint v0.5.1\n' ;; esac
        ;;
    +*)
        if [[ "$2" == clippy && "${ADMISSION_FAIL_CLIPPY:-no}" == yes ]]; then exit 43; fi
        if [[ "$2" == build ]]; then
            if [[ "${ADMISSION_FAIL_BUILD:-no}" == yes ]]; then exit 42; fi
            printf '%s\n' "$CARGO_TARGET_DIR" >> "$ADMISSION_TARGETS"
            case " $* " in
                *' --target wasm32-unknown-unknown '*)
                    mkdir -p "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release"
                    printf '%s\t%s\n' "${ADMISSION_RUN:-single}" "$*" > "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/platform_runtime_canister.wasm" ;;
                *)
                    mkdir -p "$CARGO_TARGET_DIR/debug"
                    printf '#!/usr/bin/env bash\nexit 0\n' > "$CARGO_TARGET_DIR/debug/platform-runtime-driver"
                    chmod +x "$CARGO_TARGET_DIR/debug/platform-runtime-driver" ;;
            esac
        fi
        if [[ "${ADMISSION_FAIL:-no}" == yes && "$2" == check ]]; then exit 42; fi ;;
esac
CARGO
chmod +x "$fixture/bin/cargo" "$fixture/bin/rustc"
git -C "$fixture" init -q
git -C "$fixture" add .
git -C "$fixture" -c user.name='Platform admission fixture' -c user.email='platform-fixture@example.invalid' commit -qm 'Runtime admission source'
export PATH="$fixture/bin:$PATH" ADMISSION_CALLS="$fixture/calls" ADMISSION_TARGETS="$fixture/targets"
ADMISSION_COMPILER="$(bash "$root/scripts/ci/read-platform-msrv.sh")"
export ADMISSION_COMPILER
"$BASH" "$fixture/scripts/ci/check-prepared-call-consumer.sh" --prepare > "$fixture/prepare.log" 2>&1 || { cat "$fixture/prepare.log"; exit 1; }
[[ "$(grep -c '^fetch ' "$ADMISSION_CALLS")" == 1 ]]
[[ "$(grep -vc '^locate-project ' "$ADMISSION_CALLS")" == 1 ]]
: > "$ADMISSION_CALLS"
"$BASH" "$fixture/scripts/ci/check-prepared-call-consumer.sh" > "$fixture/pass.log" 2>&1 || { cat "$fixture/pass.log"; exit 1; }
if grep -q '^fetch ' "$ADMISSION_CALLS"; then echo 'Offline validation fetched dependencies' >&2; exit 1; fi
# All three selections reach strict Clippy, tests and both target checks.
[[ "$(grep -c '^clippy ' "$ADMISSION_CALLS")" == 3 ]]
[[ "$(grep -c '^+.* test ' "$ADMISSION_CALLS")" == 3 ]]
[[ "$(grep -c '^+.* check ' "$ADMISSION_CALLS")" == 3 ]]
[[ "$(grep -c '^tree ' "$ADMISSION_CALLS")" == 6 ]]
[[ "$(grep '^tree ' "$ADMISSION_CALLS" | grep -c -- ' --features wire ')" == 2 ]]
[[ "$(grep '^tree ' "$ADMISSION_CALLS" | grep -c -- ' --features external-candid ')" == 2 ]]
[[ "$(grep '^tree ' "$ADMISSION_CALLS" | grep -vc -- ' --features ')" == 2 ]]
if ADMISSION_FAIL=yes "$BASH" "$fixture/scripts/ci/check-prepared-call-consumer.sh" > "$fixture/fail.log" 2>&1; then
    echo 'Consumer hid a failing Wasm check' >&2; exit 1
else
    [[ $? == 42 ]] || { cat "$fixture/fail.log"; exit 1; }
fi
: > "$ADMISSION_CALLS"
if ADMISSION_DIRECT_CBOR=yes "$BASH" "$fixture/scripts/ci/check-platform-runtime.sh" --check-graph > "$fixture/cbor-refusal.log" 2>&1; then
    echo 'Runtime driver admitted direct CBOR' >&2; exit 1
fi
grep -q 'platform-runtime-driver -> serde_cbor' "$fixture/cbor-refusal.log"
if grep -Eq '^tree |^\+.* (build|test) ' "$ADMISSION_CALLS"; then
    echo 'Runtime proof continued after direct CBOR rejection' >&2; exit 1
fi
: > "$ADMISSION_CALLS"
"$BASH" "$fixture/scripts/ci/check-platform-runtime.sh" --prepared-call --check-graph > "$fixture/runtime.log" 2>&1 || { cat "$fixture/runtime.log"; exit 1; }
[[ "$(grep -c '^tree ' "$ADMISSION_CALLS")" == 8 ]]
[[ "$(grep '^tree ' "$ADMISSION_CALLS" | grep -c -- ' --features external-adapters ')" == 4 ]]
for attempt in first second; do
    : > "$ADMISSION_TARGETS"
    "$BASH" "$fixture/scripts/ci/check-platform-runtime.sh" --prepared-call --build-only > "$fixture/$attempt-build.log" 2>&1 || { cat "$fixture/$attempt-build.log"; exit 1; }
    LC_ALL=C sort -u "$ADMISSION_TARGETS" > "$fixture/$attempt-targets"
done
cmp "$fixture/first-targets" "$fixture/second-targets"
[[ "$(wc -l < "$fixture/first-targets" | tr -d ' ')" == 2 ]]
while IFS= read -r retained; do
    [[ -f "$retained/artifacts/platform-runtime-driver" ]]
    [[ "$(find "$retained/artifacts" -name '*.wasm' | wc -l | tr -d ' ')" == 4 ]]
done < <(sed -n 's/^Runtime sources, lock and metadata retained at //p' "$fixture/first-build.log" "$fixture/second-build.log")
if ADMISSION_FAIL_BUILD=yes "$BASH" "$fixture/scripts/ci/check-platform-runtime.sh" --build-only > "$fixture/build-failure.log" 2>&1; then
    echo 'Runtime proof hid a failed locked build' >&2; exit 1
else
    [[ $? == 42 ]] || { cat "$fixture/build-failure.log"; exit 1; }
fi
: > "$ADMISSION_CALLS"
if ADMISSION_FAIL_CLIPPY=yes "$BASH" "$fixture/scripts/ci/check-platform-runtime.sh" --build-only > "$fixture/clippy-failure.log" 2>&1; then
    echo 'Runtime copied an artifact after failed Clippy' >&2; exit 1
else
    [[ $? == 43 ]] || { cat "$fixture/clippy-failure.log"; exit 1; }
fi
if grep -q ' build ' "$ADMISSION_CALLS"; then echo 'Runtime built after failed Clippy' >&2; exit 1; fi

# Pause A after Cargo returns and before copying its first Wasm. B must wait
# for that entire interval, then retain its own output from the shared cache.
ADMISSION_REAL_CP="$(command -v cp)" ADMISSION_REAL_PERL="$(command -v perl)"
export ADMISSION_REAL_CP ADMISSION_REAL_PERL
export ADMISSION_FIRST_READY="$fixture/first-ready" ADMISSION_SECOND_READY="$fixture/second-ready" ADMISSION_RESUME="$fixture/resume"
development_toolchain="$("$root/.tools/host/bin/yq" -p toml -r '.toolchain.channel' "$root/rust-toolchain.toml")"
export ADMISSION_WASM_LOCK="$fixture/target/runtime-build/$development_toolchain/.platform-artifact.lock"
mkfifo "$ADMISSION_FIRST_READY" "$ADMISSION_SECOND_READY" "$ADMISSION_RESUME"
exec 6<>"$ADMISSION_FIRST_READY" 7<>"$ADMISSION_SECOND_READY" 8<>"$ADMISSION_RESUME"
cat > "$fixture/bin/cp" <<'CP'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${ADMISSION_RUN:-}" == A && "$1" == */platform_runtime_canister.wasm && ! -e "$ADMISSION_FIRST_READY.seen" ]]; then
    : > "$ADMISSION_FIRST_READY.seen"
    printf 'ready\n' > "$ADMISSION_FIRST_READY"
    read -r -t 20 _ < "$ADMISSION_RESUME"
fi
exec "$ADMISSION_REAL_CP" "$@"
CP
cat > "$fixture/bin/perl" <<'PERL'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${ADMISSION_RUN:-}" == B && "${1:-}" == -MFcntl=:flock && "${4:-}" == "$ADMISSION_WASM_LOCK" && ! -e "$ADMISSION_SECOND_READY.seen" ]]; then
    : > "$ADMISSION_SECOND_READY.seen"
    printf 'waiting\n' > "$ADMISSION_SECOND_READY"
fi
exec "$ADMISSION_REAL_PERL" "$@"
PERL
chmod +x "$fixture/bin/cp" "$fixture/bin/perl"
ADMISSION_RUN=A "$BASH" "$fixture/scripts/ci/check-platform-runtime.sh" --prepared-call --build-only > "$fixture/concurrent-a.log" 2>&1 &
first_pid=$!
read -r -t 15 _ <&6
ADMISSION_RUN=B "$BASH" "$fixture/scripts/ci/check-platform-runtime.sh" --prepared-call --build-only > "$fixture/concurrent-b.log" 2>&1 &
second_pid=$!
read -r -t 15 _ <&7
# shellcheck disable=SC2016 # Perl variables must reach the Perl interpreter literally.
"$ADMISSION_REAL_PERL" -MFcntl=:flock -e 'open my $lock, ">>", $ARGV[0] or die $!; exit(flock($lock, LOCK_EX | LOCK_NB) ? 1 : 0)' "$ADMISSION_WASM_LOCK"
printf 'resume\n' >&8
wait "$first_pid"; first_pid=""
wait "$second_pid"; second_pid=""
for run in a b; do
    retained="$(sed -n 's/^Runtime sources, lock and metadata retained at //p' "$fixture/concurrent-$run.log")"
    [[ -n "$retained" ]]
    expected=A; if [[ "$run" == b ]]; then expected=B; fi
    while IFS= read -r artifact; do
        [[ "$(cut -f1 "$artifact")" == "$expected" ]]
    done < <(find "$retained/artifacts" -name '*.wasm')
done
exec 6>&- 7>&- 8>&-
# Snapshot copies have their own identity; additions, source edits and snapshot
# edits refuse, while explicit refresh tolerates only the selected lock changing.
proof="$fixture/input-proof"
mkdir "$proof"
input_lock=scripts/fixtures/cdk-runtime/Cargo.lock
input_file=scripts/fixtures/cdk-runtime/canister.rs
perl "$fixture/scripts/ci/runtime-inputs.pl" capture "$fixture" "$proof" "$input_lock"
cmp "$fixture/$input_file" "$proof/inputs/$input_file"
printf '// added input\n' > "$fixture/crates/new-input.rs"
if perl "$fixture/scripts/ci/runtime-inputs.pl" verify "$fixture" "$proof" "$input_lock" > "$proof/roster.log" 2>&1; then exit 1; fi
grep -q 'input roster changed' "$proof/roster.log"
rm "$fixture/crates/new-input.rs"
printf '\n// source edit\n' >> "$fixture/$input_file"
if perl "$fixture/scripts/ci/runtime-inputs.pl" verify "$fixture" "$proof" "$input_lock" > "$proof/source.log" 2>&1; then exit 1; fi
grep -q 'inputs changed during observation' "$proof/source.log"
cp "$proof/inputs/$input_file" "$fixture/$input_file"
printf '\n# refreshed lock\n' >> "$fixture/$input_lock"
if perl "$fixture/scripts/ci/runtime-inputs.pl" verify "$fixture" "$proof" "$input_lock" > "$proof/lock.log" 2>&1; then exit 1; fi
perl "$fixture/scripts/ci/runtime-inputs.pl" verify "$fixture" "$proof" "$input_lock" --refresh-lock
cp "$proof/inputs/$input_lock" "$fixture/$input_lock"
printf '\n// snapshot edit\n' >> "$proof/inputs/$input_file"
if perl "$fixture/scripts/ci/runtime-inputs.pl" verify "$fixture" "$proof" "$input_lock" > "$proof/snapshot.log" 2>&1; then exit 1; fi
grep -q 'input snapshot changed' "$proof/snapshot.log"
raw_proof="$fixture/raw-rand-input-proof"
mkdir -p "$raw_proof" "$fixture/scripts/benchmarks"
cp -R "$root/scripts/fixtures/raw-rand" "$fixture/scripts/fixtures/"
cp "$root/scripts/ci/check-platform-raw-rand.sh" "$fixture/scripts/ci/"
cp "$root/scripts/benchmarks/wasm-sections.pl" "$fixture/scripts/benchmarks/"
perl "$fixture/scripts/ci/runtime-inputs.pl" capture "$fixture" "$raw_proof" Cargo.lock --raw-rand
cmp "$fixture/scripts/fixtures/raw-rand/canister/mod.rs" "$raw_proof/inputs/scripts/fixtures/raw-rand/canister/mod.rs"
printf '// added raw_rand input\n' > "$fixture/scripts/fixtures/raw-rand/new.rs"
if perl "$fixture/scripts/ci/runtime-inputs.pl" verify "$fixture" "$raw_proof" Cargo.lock --raw-rand > "$raw_proof/roster.log" 2>&1; then exit 1; fi
grep -q 'input roster changed' "$raw_proof/roster.log"
rm "$fixture/scripts/fixtures/raw-rand/new.rs"
perl "$fixture/scripts/ci/runtime-inputs.pl" verify "$fixture" "$raw_proof" Cargo.lock --raw-rand
if "$BASH" "$fixture/scripts/ci/check-platform-runtime.sh" --published invalid --check-graph > "$fixture/version.log" 2>&1; then
    echo 'Runtime admitted an invalid published version' >&2; exit 1
else
    [[ $? == 2 ]] || { cat "$fixture/version.log"; exit 1; }
fi
echo "prepared-call admission: raw/Wire/external-Candid arguments and failure propagation passed on Bash $BASH_VERSION"
fixture_complete=true
