#!/usr/bin/env bash
set -euo pipefail
# Exercise the real consumer runner and graph checker without compiling fixtures.
# Run with the current shell, including Bash 3.2 on supported macOS hosts.
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
mkdir -p "$root/target"
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$root/scripts/ci/fixture-lifecycle.sh"
fixture_complete=false
fixture="$(fixture_create "$root/target/platform-consumer-admission.XXXXXX")"
finish() {
    local status=$?
    fixture_finish "$status" "$fixture" 'Consumer admission evidence retained'
}
trap finish EXIT
mkdir -p "$fixture/scripts/ci" "$fixture/crates/ic-platform/tests" "$fixture/bin"
cp "$root/scripts/ci/check-platform-consumer.sh" "$root/scripts/ci/check-platform-runtime-graph.sh" "$fixture/scripts/ci/"
cp "$root/scripts/ci/platform-consumer-profiles.tsv" "$fixture/scripts/ci/"
cp "$root/scripts/ci/platform-graph-policy.awk" "$fixture/scripts/ci/"
touch "$fixture/Cargo.lock"
cp "$root/scripts/ci/consumer-fixture.sh" "$fixture/scripts/ci/"
bash "$root/scripts/ci/prepare-script-fixture.sh" "$root" "$fixture" scripts/ci/check-cargo-dependency-boundaries.sh
cp -R "$root/crates/ic-platform/tests/." "$fixture/crates/ic-platform/tests/"
ln -s "$BASH" "$fixture/bin/bash"
cp "$root/scripts/ci/"{read-platform-msrv.sh,check-platform-compiler.sh} "$fixture/scripts/ci/"
cp "$root/rust-toolchain.toml" "$fixture/"
cp "$root/Cargo.toml" "$fixture/Cargo.toml"
cp "$root/crates/ic-platform/Cargo.toml" "$fixture/crates/ic-platform/Cargo.toml"
mkdir -p "$fixture/.tools"
ln -s "$root/.tools/host" "$fixture/.tools/host"
export PLATFORM_ADMISSION_TOOLS="$root/scripts/fixtures/admission-tools"
export PLATFORM_ADMISSION_PACKAGE=platform-consumer-proof
cp "$PLATFORM_ADMISSION_TOOLS/rustc.sh" "$fixture/bin/rustc"
cat > "$fixture/bin/cargo" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=scripts/fixtures/admission-tools/cargo.sh
source "$PLATFORM_ADMISSION_TOOLS/cargo.sh"
case "$1" in
    tree)
        if [[ "${TEST_CAPABILITY:-}" == external-codecs ]]; then
            printf 'candid v0.10.39\n'
            if [[ "$TEST_REJECT" == yes ]]; then printf 'ic-agent v0.49.3\n'; fi
            exit 0
        fi
        printf 'ic-platform v0.17.0 (/local/platform)\n'
        if [[ "${TEST_PRINCIPAL:-no}" == yes ]]; then
            printf 'ic_principal v0.1.5\n'
        fi
        if [[ "${TEST_CAPABILITY:-}" == principal-self-authenticating ]]; then
            printf 'sha2 v0.11.1\n'
        fi
        case "${TEST_CAPABILITY:-runtime}" in
            runtime|owned-endpoint|executor|ic0)
                printf 'ic-platform-runtime v0.17.0 (/local/runtime)\n' ;;
        esac
        if [[ "$TEST_CANDID" == yes ]]; then printf 'candid v0.10.38\n'; fi
        if [[ "$TEST_REJECT" == yes ]]; then printf 'ic-agent v0.49.2\n'; fi
        ;;
    *)
        printf '%s\n' "$*" >> "$TEST_CALLS"
        if [[ "${TEST_FAIL_LAST:-no}" == yes && "$1" == +"$TEST_MSRV" && "$2" == build ]]; then
            exit 42
        fi
        if [[ "${TEST_EDIT:-no}" == yes && "$1" == +"$TEST_MSRV" && "$2" == build && ! -e "$TEST_READY.seen" ]]; then
            : > "$TEST_READY.seen"
            printf 'ready\n' > "$TEST_READY"
            read -r _ < "$TEST_RESUME"
        fi
        ;;
esac
STUB
chmod +x "$fixture/bin/cargo" "$fixture/bin/rustc"
export PATH="$fixture/bin:$PATH" TEST_CALLS="$fixture/calls"
TEST_MSRV="$("$BASH" "$root/scripts/ci/read-platform-msrv.sh")"
export TEST_MSRV
export TEST_REJECT=no TEST_CANDID=no
: > "$TEST_CALLS"
if TEST_DIRECT_CBOR=yes "$BASH" "$fixture/scripts/ci/check-platform-consumer.sh" principal > "$fixture/cbor-refusal.log" 2>&1; then
    echo 'Standalone consumer admitted direct CBOR' >&2; exit 1
fi
grep -q 'platform-consumer-proof -> serde_cbor' "$fixture/cbor-refusal.log"
[[ ! -s "$TEST_CALLS" ]] || { echo 'Compiled after direct CBOR rejection' >&2; exit 1; }
while IFS=$'\t' read -r capability _ _ wasm _; do
    TEST_CANDID=no
    : > "$TEST_CALLS"
    TEST_CAPABILITY="$capability" "$BASH" "$fixture/scripts/ci/check-platform-consumer.sh" "$capability" > "$fixture/log" 2>&1 || {
        cat "$fixture/log" >&2; exit 1;
    }
    expected=2
    if [[ "$wasm" == skip ]]; then expected=1; fi
    [[ "$(wc -l < "$TEST_CALLS" | tr -d ' ')" == "$expected" ]] || { cat "$fixture/log" >&2; exit 1; }
done < "$root/scripts/ci/platform-consumer-profiles.tsv"
: > "$TEST_CALLS"
for capability in principal principal-self-authenticating; do
    if TEST_CAPABILITY="$capability" TEST_PRINCIPAL=yes "$BASH" "$fixture/scripts/ci/check-platform-consumer.sh" "$capability" > "$fixture/principal-refusal.log" 2>&1; then
        echo "$capability admitted the retired registry provider" >&2; exit 1
    fi
    grep -q 'Owned Principal acquired the external identity' "$fixture/principal-refusal.log"
    [[ ! -s "$TEST_CALLS" ]] || { echo 'Compiled after Principal graph rejection' >&2; exit 1; }
done
if TEST_CAPABILITY=principal-self-authenticating TEST_REJECT=yes "$BASH" "$fixture/scripts/ci/check-platform-consumer.sh" principal-self-authenticating > "$fixture/self-auth-refusal.log" 2>&1; then
    echo 'Self-authentication admitted unrelated Agent machinery' >&2; exit 1
fi
grep -q 'Identity/ledger consumer acquired retired or unrelated integration machinery' "$fixture/self-auth-refusal.log"
[[ ! -s "$TEST_CALLS" ]] || { echo 'Compiled after self-authentication graph rejection' >&2; exit 1; }
echo 'Self-authentication admits upstream hashing; registry Principal and unrelated clients remain refused'
: > "$TEST_CALLS"
if "$BASH" "$fixture/scripts/ci/check-platform-consumer.sh" unknown > "$fixture/log" 2>&1; then
    echo 'Unknown consumer profile was admitted' >&2; exit 1
fi
[[ ! -s "$TEST_CALLS" ]] || { echo 'Compiled an unknown consumer profile' >&2; exit 1; }
cp "$fixture/scripts/ci/platform-consumer-profiles.tsv" "$fixture/profiles.before"
sed 's/minimal$/unknown-graph/' "$fixture/profiles.before" > "$fixture/scripts/ci/platform-consumer-profiles.tsv"
if "$BASH" "$fixture/scripts/ci/check-platform-consumer.sh" executor > "$fixture/invalid-family.log" 2>&1; then
    echo 'Unknown consumer graph family was admitted' >&2; exit 1
fi
grep -q 'Unknown consumer graph family' "$fixture/invalid-family.log"
[[ ! -s "$TEST_CALLS" ]] || { echo 'Compiled an unknown graph family' >&2; exit 1; }
cp "$fixture/profiles.before" "$fixture/scripts/ci/platform-consumer-profiles.tsv"
echo 'Every consumer profile prepares its source/assets and selects native/Wasm commands'
# The test declaration is the feature owner, even after a later edit.
cp "$fixture/crates/ic-platform/Cargo.toml" "$fixture/test-manifest.before"
perl -0777 -i -pe 's/(name = "ecdsa_consumer".*?required-features = )\["ecdsa"\]/${1}["ecdsa", "ed25519"]/s' "$fixture/crates/ic-platform/Cargo.toml"
"$BASH" "$fixture/scripts/ci/check-platform-consumer.sh" ecdsa > "$fixture/derived-features.log" 2>&1
derived="$(sed -n 's/^Consumer evidence retained at //p' "$fixture/derived-features.log")"
grep -Fq 'features = ["ecdsa", "ed25519"]' "$derived/Cargo.toml"
cp "$fixture/test-manifest.before" "$fixture/crates/ic-platform/Cargo.toml"
echo 'Standalone feature selection follows the Cargo test declaration'
for capability in runtime owned-endpoint; do
    export TEST_CANDID=no
    for rejection in no yes; do
        export TEST_REJECT="$rejection"
        : > "$TEST_CALLS"
        if "$BASH" "$fixture/scripts/ci/check-platform-consumer.sh" "$capability" > "$fixture/log" 2>&1; then
            [[ "$rejection" == no ]] || { echo "Admitted rejected $capability graph" >&2; exit 1; }
            [[ "$(wc -l < "$TEST_CALLS" | tr -d ' ')" == 2 ]] || { cat "$fixture/log"; exit 1; }
        else
            [[ "$rejection" == yes ]] || { cat "$fixture/log"; exit 1; }
            grep -q 'Canister graph contains host parsing' "$fixture/log"
            [[ ! -s "$TEST_CALLS" ]] || { echo 'Compiled after graph rejection' >&2; exit 1; }
        fi
    done
done
echo 'Consumer admission: runtime/owned endpoint selection and failure propagation passed'

export TEST_FAIL_LAST=yes TEST_REJECT=no TEST_CANDID=no
if "$BASH" "$fixture/scripts/ci/check-platform-consumer.sh" owned-endpoint > "$fixture/failure.log" 2>&1; then
    echo 'Final Cargo failure was masked by consumer completion' >&2; exit 1
else
    [[ $? == 42 ]] || { cat "$fixture/failure.log" >&2; exit 1; }
fi
export TEST_FAIL_LAST=no
echo 'Final Cargo failure still propagates before successful completion'

# Cargo may run for minutes while a maintainer edits this checkout. Rewrite the
# same inode while the last call is paused, shifting the already-read file tail.
mkfifo "$fixture/ready" "$fixture/resume"
export TEST_EDIT=yes TEST_REJECT=no TEST_CANDID=no
export TEST_READY="$fixture/ready" TEST_RESUME="$fixture/resume"
: > "$TEST_CALLS"
"$BASH" "$fixture/scripts/ci/check-platform-consumer.sh" owned-endpoint > "$fixture/edit.log" 2>&1 &
runner_pid=$!
read -r _ < "$TEST_READY"
{
    printf '# Concurrent edit: %0200d\n' 0
    cat "$fixture/scripts/ci/check-platform-consumer.sh"
    printf '\nexit 97\n'
} > "$fixture/changed.sh"
cat "$fixture/changed.sh" > "$fixture/scripts/ci/check-platform-consumer.sh"
printf 'resume\n' > "$TEST_RESUME"
if ! wait "$runner_pid"; then
    cat "$fixture/edit.log" >&2
    echo 'Consumer runner read a shifted script tail after its final Cargo call' >&2
    exit 1
fi
[[ "$(wc -l < "$TEST_CALLS" | tr -d ' ')" == 2 ]] || { cat "$fixture/edit.log" >&2; exit 1; }
echo 'Consumer runner completes its parsed command list after a concurrent edit'
fixture_complete=true
