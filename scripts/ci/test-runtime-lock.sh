#!/usr/bin/env bash
set -euo pipefail
unset RELEASE_GROUP RELEASE_COMMIT
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$root/scripts/ci/fixture-lifecycle.sh"
fixture="$(fixture_create "${TMPDIR:-/tmp}/platform-runtime-lock.XXXXXX")"
fixture_complete=false
trap 'fixture_finish "$?" "$fixture" "Runtime lock fixture retained"' EXIT
mkdir -p "$fixture/scripts/release" "$fixture/scripts/ci" "$fixture/scripts/fixtures/cdk-runtime" "$fixture/.tools" "$fixture/ci" "$fixture/crates/ic-platform-types" "$fixture/crates/ic-platform"
ln -s "$root/.tools/host" "$fixture/.tools/host"
cp "$root/scripts/release/"{runtime-lock.sh,runtime-pin.sh} "$fixture/scripts/release/"
cp "$root/ci/"{release-editor.env,dependency-pinning-exceptions.json} "$fixture/ci/"
editor="$(bash "$root/scripts/release/editor.sh" check)"
printf '#!/usr/bin/env bash\nprintf "%%s\\n" %q\n' "$editor" > "$fixture/scripts/release/editor.sh"
lock="$fixture/scripts/fixtures/cdk-runtime/Cargo.lock"
cp "$root/scripts/fixtures/cdk-runtime/Cargo.lock" "$lock"
export YQ="$root/.tools/host/bin/yq"
export PATH="$root/.tools/host/bin:$PATH"
version="$("$YQ" -p toml -o json -r '.package.version' "$root/crates/ic-platform/Cargo.toml")"
types="$("$YQ" -p toml -o json -r '.package.version' "$root/crates/ic-platform-types/Cargo.toml")"
printf '[workspace]\n' > "$fixture/Cargo.toml"
printf '[package]\nversion = "%s"\n' "$version" > "$fixture/crates/ic-platform/Cargo.toml"
printf '[package]\nversion = "%s"\n' "$types" > "$fixture/crates/ic-platform-types/Cargo.toml"
bash "$fixture/scripts/release/runtime-lock.sh" check
# The historical actual lock contains no local Types record. Types preparation
# must preserve that graph rather than inventing a newly qualified owner.
if ! "$YQ" -p toml -o json '.' "$lock" | jq -e 'any(.package[]; .source == null and .name == "ic-platform-types")' > /dev/null; then
    cp "$lock" "$fixture/historical.lock"
    printf '[package]\nversion = "99.1.0"\n' > "$fixture/crates/ic-platform-types/Cargo.toml"
    RELEASE_GROUP=types bash "$fixture/scripts/release/runtime-lock.sh" prepare "$types"
    cmp "$lock" "$fixture/historical.lock"
    printf '[package]\nversion = "%s"\n' "$types" > "$fixture/crates/ic-platform-types/Cargo.toml"
    printf '\n[[package]]\nname = "ic-platform-types"\nversion = "%s"\n' "$types" >> "$lock"
fi
cp "$lock" "$fixture/before.lock"
printf '[package]\nversion = "99.0.0"\n' > "$fixture/crates/ic-platform/Cargo.toml"
if bash "$fixture/scripts/release/runtime-lock.sh" check > "$fixture/stale.log" 2>&1; then exit 1; fi
if RELEASE_GROUP=platform bash "$fixture/scripts/release/runtime-lock.sh" prepare 98.0.0 > "$fixture/wrong-base.log" 2>&1; then exit 1; fi
cmp "$lock" "$fixture/before.lock"
RELEASE_GROUP=platform bash "$fixture/scripts/release/runtime-lock.sh" prepare "$version"
bash "$fixture/scripts/release/runtime-lock.sh" check
# Runtime-only changes preserve the independent owner and all registry records.
for side in before after; do
    selected="$lock"; [[ "$side" != before ]] || selected="$fixture/before.lock"
    "$YQ" -p toml -o json '.' "$selected" | jq -S --arg before "$version" '
      [.package[] | select(.source != null or (.name != "ic-platform" and .name != "ic-platform-macros" and .name != "ic-platform-runtime-guard")) |
       if .source == null and .dependencies != null then .dependencies |= map(
         if . == "ic-platform 99.0.0" then "ic-platform " + $before
         elif . == "ic-platform-macros 99.0.0" then "ic-platform-macros " + $before
         elif . == "ic-platform-runtime-guard 99.0.0" then "ic-platform-runtime-guard " + $before else . end)
       else . end]' > "$fixture/$side.json"
done
cmp "$fixture/before.json" "$fixture/after.json"
cp "$lock" "$fixture/runtime.lock"
RELEASE_GROUP=platform bash "$fixture/scripts/release/runtime-lock.sh" prepare "$version"
cmp "$lock" "$fixture/runtime.lock"
# The reverse direction updates only the recorded Types owner.
printf '[package]\nversion = "99.1.0"\n' > "$fixture/crates/ic-platform-types/Cargo.toml"
if RELEASE_GROUP=types bash "$fixture/scripts/release/runtime-lock.sh" prepare 98.0.0 > "$fixture/type-wrong.log" 2>&1; then exit 1; fi
cmp "$lock" "$fixture/runtime.lock"
RELEASE_GROUP=types bash "$fixture/scripts/release/runtime-lock.sh" prepare "$types"
RELEASE_GROUP=types bash "$fixture/scripts/release/runtime-lock.sh" check
for side in before after; do
    selected="$lock"; [[ "$side" != before ]] || selected="$fixture/runtime.lock"
    "$YQ" -p toml -o json '.' "$selected" | jq -S '[.package[] | select(.source != null or .name != "ic-platform-types")]' > "$fixture/$side-types.json"
done
cmp "$fixture/before-types.json" "$fixture/after-types.json"
cp "$lock" "$fixture/types.lock"
RELEASE_GROUP=types bash "$fixture/scripts/release/runtime-lock.sh" prepare "$types"
cmp "$lock" "$fixture/types.lock"
# Exact guard-pin custody remains with the platform group.
if bash "$fixture/scripts/release/runtime-pin.sh" check > "$fixture/pin-stale.log" 2>&1; then exit 1; fi
cp "$fixture/ci/dependency-pinning-exceptions.json" "$fixture/before-pins.json"
if RELEASE_GROUP=platform bash "$fixture/scripts/release/runtime-pin.sh" prepare 98.0.0 > "$fixture/pin-wrong.log" 2>&1; then exit 1; fi
cmp "$fixture/before-pins.json" "$fixture/ci/dependency-pinning-exceptions.json"
RELEASE_GROUP=platform bash "$fixture/scripts/release/runtime-pin.sh" prepare "$version"
bash "$fixture/scripts/release/runtime-pin.sh" check
for side in before after; do
    selected="$fixture/ci/dependency-pinning-exceptions.json"; [[ "$side" != before ]] || selected="$fixture/before-pins.json"
    jq 'map(select(.subject != "ic-platform-runtime-guard"))' "$selected" > "$fixture/$side-other-pins.json"
done
cmp "$fixture/before-other-pins.json" "$fixture/after-other-pins.json"
fixture_complete=true
echo 'Independent runtime/Types lock updates, historical graph preservation, wrong-base refusal and retry passed'
