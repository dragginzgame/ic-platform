#!/usr/bin/env bash
set -euo pipefail
# Exercise the real consumer runner and graph checker without compiling fixtures.
# Run with the current shell, including Bash 3.2 on supported macOS hosts.
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/platform-consumer-admission.XXXXXX")"
trap 'rm -rf "$fixture"' EXIT
mkdir -p "$fixture/scripts/ci" "$fixture/crates/ic-platform/tests" "$fixture/bin"
cp "$root/scripts/ci/check-platform-consumer.sh" "$root/scripts/ci/check-platform-runtime-graph.sh" "$fixture/scripts/ci/"
cp "$root/scripts/ci/check-blake3-graph.sh" "$root/scripts/ci/check-cargo-graph.pl" "$fixture/scripts/ci/"
cp "$root/scripts/ci/platform-consumer-profiles.tsv" "$fixture/scripts/ci/"
touch "$fixture/Cargo.lock"
cp -R "$root/crates/ic-platform/tests/." "$fixture/crates/ic-platform/tests/"
ln -s "$BASH" "$fixture/bin/bash"
cat > "$fixture/bin/rustc" <<'STUB'
#!/usr/bin/env bash
printf 'host: x86_64-unknown-linux-gnu\n'
STUB
cat > "$fixture/bin/cargo" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
case "$1" in
    metadata) printf '{}\n' ;;
    tree)
        if [[ "${TEST_CAPABILITY:-}" == external-codecs ]]; then
            printf 'candid v0.10.39\n'
            if [[ "$TEST_REJECT" == yes ]]; then printf 'ic-agent v0.49.3\n'; fi
            exit 0
        fi
        if [[ "${TEST_CAPABILITY:-}" == blake3 ]]; then
            case " $* " in
                *' --format {f} '*) printf '%s\n' "${TEST_BLAKE3_FEATURES:-std}" ;;
                *)
                    printf 'platform-consumer-proof v0.1.0\nblake3 v1.8.7\n'
                    if [[ "${TEST_BLAKE3_DUPLICATE:-no}" == yes ]]; then
                        printf 'blake3 v1.8.6\n'
                    fi
                    ;;
            esac
            exit 0
        fi
        printf 'ic-cdk-executor v2.0.0\nic0 v1.2.0\n'
        if [[ "$TEST_CANDID" == yes ]]; then printf 'candid v0.10.38\n'; fi
        if [[ "$TEST_REJECT" == yes ]]; then printf 'ic-agent v0.49.2\n'; fi
        ;;
    *)
        printf '%s\n' "$*" >> "$TEST_CALLS"
        if [[ "${TEST_FAIL_LAST:-no}" == yes && "$1" == +1.91.0 && "$2" == build ]]; then
            exit 42
        fi
        if [[ "${TEST_EDIT:-no}" == yes && "$1" == +1.91.0 && "$2" == build && ! -e "$TEST_READY.seen" ]]; then
            : > "$TEST_READY.seen"
            printf 'ready\n' > "$TEST_READY"
            read -r _ < "$TEST_RESUME"
        fi
        ;;
esac
STUB
chmod +x "$fixture/bin/cargo" "$fixture/bin/rustc"
export PATH="$fixture/bin:$PATH" TEST_CALLS="$fixture/calls"
export TEST_REJECT=no TEST_CANDID=no
while IFS=$'\t' read -r capability _ _ _ wasm; do
    TEST_CANDID=no
    if [[ "$capability" == external-candid ]]; then TEST_CANDID=yes; fi
    : > "$TEST_CALLS"
    TEST_CAPABILITY="$capability" "$BASH" "$fixture/scripts/ci/check-platform-consumer.sh" "$capability" > "$fixture/log" 2>&1 || {
        cat "$fixture/log" >&2; exit 1;
    }
    expected=4
    if [[ "$wasm" == skip ]]; then expected=2; fi
    [[ "$(wc -l < "$TEST_CALLS" | tr -d ' ')" == "$expected" ]] || { cat "$fixture/log" >&2; exit 1; }
done < "$root/scripts/ci/platform-consumer-profiles.tsv"
: > "$TEST_CALLS"
if "$BASH" "$fixture/scripts/ci/check-platform-consumer.sh" unknown > "$fixture/log" 2>&1; then
    echo 'Unknown consumer profile was admitted' >&2; exit 1
fi
[[ ! -s "$TEST_CALLS" ]] || { echo 'Compiled an unknown consumer profile' >&2; exit 1; }
echo 'Every consumer profile prepares its source/assets and selects native/Wasm commands'
if TEST_CAPABILITY=external-codecs TEST_REJECT=yes "$BASH" "$fixture/scripts/ci/check-platform-consumer.sh" external-codecs > "$fixture/codec-rejection.log" 2>&1; then
    echo 'External codecs admitted host transport' >&2; exit 1
fi
grep -q 'External codecs acquired transport' "$fixture/codec-rejection.log"
for rejection in duplicate features; do
    duplicate=no feature_union=std
    if [[ "$rejection" == duplicate ]]; then
        duplicate=yes
        diagnostic='maximum 1 rendered package identities, observed 2'
    else
        feature_union=pure,std
        diagnostic='BLAKE3 must select only std'
    fi
    : > "$TEST_CALLS"
    if TEST_CAPABILITY=blake3 TEST_BLAKE3_DUPLICATE="$duplicate" TEST_BLAKE3_FEATURES="$feature_union" \
        "$BASH" "$fixture/scripts/ci/check-platform-consumer.sh" blake3 > "$fixture/log" 2>&1; then
        echo "Admitted rejected BLAKE3 $rejection graph" >&2; exit 1
    fi
    grep -Fq "$diagnostic" "$fixture/log"
    [[ ! -s "$TEST_CALLS" ]] || { echo 'Compiled after BLAKE3 graph rejection' >&2; exit 1; }
done
echo 'BLAKE3 consumer rejects duplicate identities and extra features before compilation'
for capability in cdk runtime external-candid; do
    export TEST_CANDID=no
    if [[ "$capability" == external-candid ]]; then TEST_CANDID=yes; fi
    for rejection in no yes; do
        export TEST_REJECT="$rejection"
        : > "$TEST_CALLS"
        if "$BASH" "$fixture/scripts/ci/check-platform-consumer.sh" "$capability" > "$fixture/log" 2>&1; then
            [[ "$rejection" == no ]] || { echo "Admitted rejected $capability graph" >&2; exit 1; }
            [[ "$(wc -l < "$TEST_CALLS" | tr -d ' ')" == 4 ]] || { cat "$fixture/log"; exit 1; }
        else
            [[ "$rejection" == yes ]] || { cat "$fixture/log"; exit 1; }
            grep -q 'Canister graph contains host parsing' "$fixture/log"
            [[ ! -s "$TEST_CALLS" ]] || { echo 'Compiled after graph rejection' >&2; exit 1; }
        fi
    done
done
echo 'Consumer admission: CDK/runtime/external Candid selection and failure propagation passed'

export TEST_FAIL_LAST=yes TEST_REJECT=no TEST_CANDID=no
if "$BASH" "$fixture/scripts/ci/check-platform-consumer.sh" cdk > "$fixture/failure.log" 2>&1; then
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
"$BASH" "$fixture/scripts/ci/check-platform-consumer.sh" cdk > "$fixture/edit.log" 2>&1 &
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
[[ "$(wc -l < "$TEST_CALLS" | tr -d ' ')" == 4 ]] || { cat "$fixture/edit.log" >&2; exit 1; }
echo 'Consumer runner completes its parsed command list after a concurrent edit'
