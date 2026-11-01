#!/usr/bin/env bash
set -euo pipefail
# Exercise the actual renderer/resolver with an incompatible scheduler generation.
# No builds, server, index refresh or mutation of the working selection.
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/platform-runtime-independence.XXXXXX")"
trap 'printf "Runtime independence evidence: %s\n" "$fixture"' EXIT
mkdir -p "$fixture/repo/scripts/ci" "$fixture/repo/scripts/fixtures" "$fixture/repo/.tools"
cp -R "$root/crates" "$root/.cargo" "$fixture/repo/"
cp "$root/"{Cargo.toml,Cargo.lock,rust-toolchain.toml} "$fixture/repo/"
cp "$root/scripts/ci/"{check-platform-runtime.sh,check-platform-runtime-graph.sh,read-platform-msrv.sh,platform-graph-policy.awk} "$fixture/repo/scripts/ci/"
cp -R "$root/scripts/fixtures/"{cdk-runtime,timers-runtime} "$fixture/repo/scripts/fixtures/"
ln -s "$root/.tools/host" "$fixture/repo/.tools/host"
# The source provenance is the current repository; no temporary commits needed.
export GIT_DIR="$(git -C "$root" rev-parse --absolute-git-dir)"
export CARGO_NET_OFFLINE=true
sed -i.bak 's/^ic-timers = .*/ic-timers = "=0.35.1"/' "$fixture/repo/Cargo.toml"
rm "$fixture/repo/Cargo.toml.bak"
bash "$fixture/repo/scripts/ci/check-platform-runtime.sh" --prepared-call --check-graph > "$fixture/owner.log" 2>&1
if bash "$fixture/repo/scripts/ci/check-platform-runtime.sh" --timers --prepared-call --check-graph > "$fixture/composition.log" 2>&1; then
    echo 'Incompatible Timers unexpectedly qualified' >&2; exit 1
fi
grep -q 'only one package in the dependency graph may specify the same links value' "$fixture/composition.log"
printf 'Owner graphs pass independently; incompatible Timers fails singleton admission\n'
