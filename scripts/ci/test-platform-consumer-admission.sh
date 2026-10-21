#!/usr/bin/env bash
set -euo pipefail
# Exercise the real consumer runner and graph checker without compiling fixtures.
# Run with the current shell, including Bash 3.2 on supported macOS hosts.
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/platform-consumer-admission.XXXXXX")"
trap 'rm -rf "$fixture"' EXIT
mkdir -p "$fixture/scripts/ci" "$fixture/crates/ic-platform/tests" "$fixture/bin"
cp "$root/scripts/ci/check-platform-consumer.sh" "$root/scripts/ci/check-platform-runtime-graph.sh" "$fixture/scripts/ci/"
touch "$fixture/Cargo.lock"
for source in cdk runtime external_candid; do
    touch "$fixture/crates/ic-platform/tests/${source}_consumer.rs"
done
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
        printf 'ic-cdk-executor v2.0.0\nic0 v1.2.0\n'
        if [[ "$TEST_CANDID" == yes ]]; then printf 'candid v0.10.38\n'; fi
        if [[ "$TEST_REJECT" == yes ]]; then printf 'ic-agent v0.49.2\n'; fi
        ;;
    *) printf '%s\n' "$*" >> "$TEST_CALLS" ;;
esac
STUB
chmod +x "$fixture/bin/cargo" "$fixture/bin/rustc"
export PATH="$fixture/bin:$PATH" TEST_CALLS="$fixture/calls"
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
