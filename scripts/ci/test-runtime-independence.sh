#!/usr/bin/env bash
set -euo pipefail
# Exercise the actual renderer/resolver with an incompatible scheduler generation.
# No builds, server, index refresh or mutation of the working selection.
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
prepare=false
case "$#:${1:-}" in
    0:) ;;
    1:--prepare) prepare=true ;;
    *) echo "usage: $0 [--prepare]" >&2; exit 2 ;;
esac
fixture="$(mktemp -d "${TMPDIR:-/tmp}/platform-runtime-independence.XXXXXX")"
finish() {
    result=$?
    if [[ "$result" != 0 ]]; then
        for log in "$fixture"/*.log; do
            [[ -f "$log" ]] || continue
            printf '\nEvidence: %s\n' "$log" >&2
            cat "$log" >&2
        done
    fi
    printf 'Runtime independence evidence: %s\n' "$fixture"
    exit "$result"
}
trap finish EXIT
if [[ "$prepare" == true ]]; then
    mkdir "$fixture/src"
    printf '// Resolver-only historical composition input.\n' > "$fixture/src/lib.rs"
    cat > "$fixture/Cargo.toml" <<'MANIFEST'
[package]
name = "runtime-independence-input"
version = "0.0.0"
edition = "2024"
[workspace]
[dependencies]
ic-timers = "=0.35.1"
MANIFEST
    cargo fetch --manifest-path "$fixture/Cargo.toml"
    exit 0
fi
mkdir -p "$fixture/repo/scripts/ci" "$fixture/repo/scripts/fixtures" "$fixture/repo/.tools"
cp -R "$root/crates" "$root/.cargo" "$fixture/repo/"
cp "$root/"{Cargo.toml,Cargo.lock,rust-toolchain.toml} "$fixture/repo/"
cp "$root/scripts/ci/"{check-platform-runtime.sh,check-platform-runtime-graph.sh,check-platform-compiler.sh,read-platform-msrv.sh,platform-graph-policy.awk} "$fixture/repo/scripts/ci/"
cp -R "$root/scripts/fixtures/"{cdk-runtime,timers-runtime} "$fixture/repo/scripts/fixtures/"
ln -s "$root/.tools/host" "$fixture/repo/.tools/host"
# The source provenance is the current repository; no temporary commits needed.
GIT_DIR="$(git -C "$root" rev-parse --absolute-git-dir)"
export GIT_DIR
export CARGO_NET_OFFLINE=true
sed -i.bak 's/^ic-timers = .*/ic-timers = "=0.35.1"/' "$fixture/repo/Cargo.toml"
rm "$fixture/repo/Cargo.toml.bak"
bash "$fixture/repo/scripts/ci/check-platform-runtime.sh" --prepared-call --check-graph > "$fixture/owner.log" 2>&1
if bash "$fixture/repo/scripts/ci/check-platform-runtime.sh" --timers --prepared-call --check-graph > "$fixture/composition.log" 2>&1; then
    echo 'Incompatible Timers unexpectedly qualified' >&2; exit 1
fi
grep -q 'only one package in the dependency graph may specify the same links value' "$fixture/composition.log"
printf 'Owner graphs pass independently; incompatible Timers fails singleton admission\n'
