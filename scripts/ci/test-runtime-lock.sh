#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/platform-runtime-lock.XXXXXX")"
trap 'rm -rf "$fixture"' EXIT
mkdir -p "$fixture/scripts/release" "$fixture/scripts/ci" "$fixture/scripts/fixtures/cdk-runtime" "$fixture/.tools"
ln -s "$root/.tools/host" "$fixture/.tools/host"
cp "$root/scripts/release/runtime-lock.sh" "$fixture/scripts/release/"
cp "$root/scripts/ci/"{read-cargo-workspace-version.sh,rewrite-local-lock-versions.pl,StableVersion.pm} "$fixture/scripts/ci/"
cp "$root/scripts/fixtures/cdk-runtime/Cargo.lock" "$fixture/scripts/fixtures/cdk-runtime/"
export YQ="$root/.tools/host/bin/yq"
export PATH="$root/.tools/host/bin:$PATH"
version="$(bash "$root/scripts/ci/read-cargo-workspace-version.sh" --stable "$root/Cargo.toml")"
printf '[workspace]\n[workspace.package]\nversion = "%s"\n' "$version" > "$fixture/Cargo.toml"
bash "$fixture/scripts/release/runtime-lock.sh" check
# A synthetic later identity avoids coupling the regression test to release numbers.
printf '[workspace]\n[workspace.package]\nversion = "99.0.0"\n' > "$fixture/Cargo.toml"
lock="$fixture/scripts/fixtures/cdk-runtime/Cargo.lock"
cp "$lock" "$fixture/before.lock"
if bash "$fixture/scripts/release/runtime-lock.sh" check 2> "$fixture/stale.log"; then exit 1; fi
if bash "$fixture/scripts/release/runtime-lock.sh" prepare 98.0.0 2> "$fixture/wrong.log"; then exit 1; fi
cmp "$fixture/before.lock" "$lock"
bash "$fixture/scripts/release/runtime-lock.sh" prepare "$version"
bash "$fixture/scripts/release/runtime-lock.sh" check
# No external package or fixture-local identity may change during release preparation.
for side in before after; do
    source="$lock"
    if [[ "$side" == before ]]; then source="$fixture/before.lock"; fi
    "$YQ" -p toml -o json '.' "$source" | jq -S '[.package[] | select(.source != null or (.name != "ic-platform" and .name != "ic-platform-macros"))]' > "$fixture/$side.json"
done
cmp "$fixture/before.json" "$fixture/after.json"
cp "$lock" "$fixture/prepared.lock"
bash "$fixture/scripts/release/runtime-lock.sh" prepare "$version"
cmp "$fixture/prepared.lock" "$lock"
echo 'Runtime lock drift, wrong-base refusal, coordinated update and retry checks passed'
