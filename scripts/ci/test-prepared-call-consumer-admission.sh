#!/usr/bin/env bash
# Run with Bash 3.2 on macOS as well as the prepared development shell.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/prepared-call-admission.XXXXXX")"
fixture="$(CDPATH='' cd "$fixture" && pwd -P)"
first_pid="" second_pid=""
finish() {
    local status=$?
    for pid in "$first_pid" "$second_pid"; do
        [[ -z "$pid" ]] || { kill "$pid" 2>/dev/null || :; wait "$pid" 2>/dev/null || :; }
    done
    if [[ "$status" == 0 ]]; then rm -rf "$fixture"
    else printf 'Admission evidence retained: %s\n' "$fixture" >&2; fi
    exit "$status"
}
trap finish EXIT
mkdir -p "$fixture/scripts/ci" "$fixture/scripts/fixtures/prepared-call-consumer" "$fixture/.tools" "$fixture/bin"
cp "$root/scripts/ci/"{check-prepared-call-consumer.sh,check-platform-runtime.sh,check-platform-runtime-graph.sh} "$fixture/scripts/ci/"
cp "$root/scripts/ci/platform-graph-policy.awk" "$fixture/scripts/ci/"
cp "$root/scripts/fixtures/prepared-call-consumer/lib.rs" "$fixture/scripts/fixtures/prepared-call-consumer/"
cp -R "$root/scripts/fixtures/cdk-runtime" "$fixture/scripts/fixtures/"
mkdir -p "$fixture/crates/ic-platform/tests/fixtures"
cp "$root/crates/ic-platform/tests/management_consumer.rs" "$fixture/crates/ic-platform/tests/"
cp "$root/crates/ic-platform/tests/fixtures/management-"*.hex "$fixture/crates/ic-platform/tests/fixtures/"
cp "$root/"{Cargo.toml,Cargo.lock,rust-toolchain.toml} "$fixture/"
cp "$root/scripts/ci/consumer-fixture.sh" "$fixture/scripts/ci/"
ln -s "$root/.tools/host" "$fixture/.tools/host"
ln -s "$BASH" "$fixture/bin/bash"
cp "$root/scripts/ci/read-platform-msrv.sh" "$fixture/scripts/ci/"
cat > "$fixture/bin/rustc" <<'RUSTC'
#!/usr/bin/env bash
printf 'host: x86_64-unknown-linux-gnu\n'
RUSTC
cat > "$fixture/bin/cargo" <<'CARGO'
#!/usr/bin/env bash
set -euo pipefail
for arg in "$@"; do [[ -n "$arg" ]] || { echo 'empty argument reached Cargo' >&2; exit 31; }; done
printf '%s\n' "$*" >> "$ADMISSION_CALLS"
case "$1" in
    locate-project) exit 0 ;;
    metadata) printf '{}\n' ;;
    tree)
        printf 'ic-platform v0.8.5 (/local/platform)\n'
        case " $* " in *' --features external-candid '*|*' --features external-adapters '*) printf 'candid v0.10.39\n' ;; esac
        ;;
    +*)
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
export PATH="$fixture/bin:$PATH" ADMISSION_CALLS="$fixture/calls" ADMISSION_TARGETS="$fixture/targets"
"$BASH" "$fixture/scripts/ci/check-prepared-call-consumer.sh" --prepare > "$fixture/prepare.log" 2>&1 || { cat "$fixture/prepare.log"; exit 1; }
[[ "$(grep -c '^fetch ' "$ADMISSION_CALLS")" == 1 ]]
[[ "$(grep -vc '^locate-project ' "$ADMISSION_CALLS")" == 1 ]]
: > "$ADMISSION_CALLS"
"$BASH" "$fixture/scripts/ci/check-prepared-call-consumer.sh" > "$fixture/pass.log" 2>&1 || { cat "$fixture/pass.log"; exit 1; }
if grep -q '^fetch ' "$ADMISSION_CALLS"; then echo 'Offline validation fetched dependencies' >&2; exit 1; fi
# All three selections reach strict Clippy, tests and both target checks.
[[ "$(grep -c '^clippy ' "$ADMISSION_CALLS")" == 3 ]]
[[ "$(grep -c '^+.* test ' "$ADMISSION_CALLS")" == 6 ]]
[[ "$(grep -c '^+.* check ' "$ADMISSION_CALLS")" == 6 ]]
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
"$BASH" "$fixture/scripts/ci/check-platform-runtime.sh" --prepared-call --check-graph > "$fixture/runtime.log" 2>&1 || { cat "$fixture/runtime.log"; exit 1; }
[[ "$(grep -c '^tree ' "$ADMISSION_CALLS")" == 8 ]]
[[ "$(grep '^tree ' "$ADMISSION_CALLS" | grep -c -- ' --features external-adapters ')" == 4 ]]
for attempt in first second; do
    : > "$ADMISSION_TARGETS"
    "$BASH" "$fixture/scripts/ci/check-platform-runtime.sh" --prepared-call --build-only > "$fixture/$attempt-build.log" 2>&1 || { cat "$fixture/$attempt-build.log"; exit 1; }
    LC_ALL=C sort -u "$ADMISSION_TARGETS" > "$fixture/$attempt-targets"
done
cmp "$fixture/first-targets" "$fixture/second-targets"
[[ "$(wc -l < "$fixture/first-targets" | tr -d ' ')" == 3 ]]
while IFS= read -r retained; do
    [[ -f "$retained/artifacts/platform-runtime-driver" ]]
    [[ "$(find "$retained/artifacts" -name '*.wasm' | wc -l | tr -d ' ')" == 8 ]]
done < <(sed -n 's/^Runtime sources, lock and metadata retained at //p' "$fixture/first-build.log" "$fixture/second-build.log")
if ADMISSION_FAIL_BUILD=yes "$BASH" "$fixture/scripts/ci/check-platform-runtime.sh" --build-only > "$fixture/build-failure.log" 2>&1; then
    echo 'Runtime proof hid a failed locked build' >&2; exit 1
else
    [[ $? == 42 ]] || { cat "$fixture/build-failure.log"; exit 1; }
fi

# Pause A after Cargo returns and before copying its first Wasm. B must wait
# for that entire interval, then retain its own output from the shared cache.
export ADMISSION_REAL_CP="$(command -v cp)" ADMISSION_REAL_PERL="$(command -v perl)"
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
if "$BASH" "$fixture/scripts/ci/check-platform-runtime.sh" --published invalid --check-graph > "$fixture/version.log" 2>&1; then
    echo 'Runtime admitted an invalid published version' >&2; exit 1
else
    [[ $? == 2 ]] || { cat "$fixture/version.log"; exit 1; }
fi
echo "prepared-call admission: raw/Wire/external-Candid arguments and failure propagation passed on Bash $BASH_VERSION"
