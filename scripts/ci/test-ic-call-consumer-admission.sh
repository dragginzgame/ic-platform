#!/usr/bin/env bash
# Run with Bash 3.2 on macOS as well as the prepared development shell.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/ic-call-admission.XXXXXX")"
trap 'rm -rf "$fixture"' EXIT
mkdir -p "$fixture/scripts/ci" "$fixture/scripts/fixtures/ic-call-consumer" "$fixture/.tools" "$fixture/bin"
cp "$root/scripts/ci/"{check-ic-call-consumer.sh,check-platform-runtime.sh,check-platform-runtime-graph.sh} "$fixture/scripts/ci/"
cp "$root/scripts/fixtures/ic-call-consumer/lib.rs" "$fixture/scripts/fixtures/ic-call-consumer/"
cp -R "$root/scripts/fixtures/cdk-runtime" "$fixture/scripts/fixtures/"
cp "$root/"{Cargo.toml,Cargo.lock,rust-toolchain.toml} "$fixture/"
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
        printf 'ic-cdk-executor v2.0.0\nic0 v1.2.0\n'
        case " $* " in *' --features external-candid '*|*' --features external-adapters '*) printf 'candid v0.10.39\n' ;; esac
        ;;
    +*)
        if [[ "${ADMISSION_FAIL:-no}" == yes && "$2" == check ]]; then exit 42; fi ;;
esac
CARGO
chmod +x "$fixture/bin/cargo" "$fixture/bin/rustc"
export PATH="$fixture/bin:$PATH" ADMISSION_CALLS="$fixture/calls"
"$BASH" "$fixture/scripts/ci/check-ic-call-consumer.sh" > "$fixture/pass.log" 2>&1 || { cat "$fixture/pass.log"; exit 1; }
# All three selections reach strict Clippy, tests and both target checks.
[[ "$(grep -c '^clippy ' "$ADMISSION_CALLS")" == 3 ]]
[[ "$(grep -c '^+.* test ' "$ADMISSION_CALLS")" == 6 ]]
[[ "$(grep -c '^+.* check ' "$ADMISSION_CALLS")" == 6 ]]
[[ "$(grep -c '^tree ' "$ADMISSION_CALLS")" == 6 ]]
[[ "$(grep '^tree ' "$ADMISSION_CALLS" | grep -c -- ' --features wire ')" == 2 ]]
[[ "$(grep '^tree ' "$ADMISSION_CALLS" | grep -c -- ' --features external-candid ')" == 2 ]]
[[ "$(grep '^tree ' "$ADMISSION_CALLS" | grep -vc -- ' --features ')" == 2 ]]
if ADMISSION_FAIL=yes "$BASH" "$fixture/scripts/ci/check-ic-call-consumer.sh" > "$fixture/fail.log" 2>&1; then
    echo 'Consumer hid a failing Wasm check' >&2; exit 1
else
    [[ $? == 42 ]] || { cat "$fixture/fail.log"; exit 1; }
fi
: > "$ADMISSION_CALLS"
"$BASH" "$fixture/scripts/ci/check-platform-runtime.sh" --ic-call --check-graph > "$fixture/runtime.log" 2>&1 || { cat "$fixture/runtime.log"; exit 1; }
[[ "$(grep -c '^tree ' "$ADMISSION_CALLS")" == 8 ]]
[[ "$(grep '^tree ' "$ADMISSION_CALLS" | grep -c -- ' --features external-adapters ')" == 4 ]]
if "$BASH" "$fixture/scripts/ci/check-platform-runtime.sh" --published invalid --check-graph > "$fixture/version.log" 2>&1; then
    echo 'Runtime admitted an invalid published version' >&2; exit 1
else
    [[ $? == 2 ]] || { cat "$fixture/version.log"; exit 1; }
fi
echo "ic-call admission: raw/Wire/external-Candid arguments and failure propagation passed on Bash $BASH_VERSION"
