#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$root/scripts/ci/fixture-lifecycle.sh"
fixture="$(fixture_create "${TMPDIR:-/tmp}/platform-runtime-lock.XXXXXX")"
fixture_complete=false
trap 'fixture_finish "$?" "$fixture" "Runtime lock fixture retained"' EXIT
mkdir -p "$fixture/scripts/release" "$fixture/scripts/ci" "$fixture/scripts/fixtures/cdk-runtime" "$fixture/.tools" "$fixture/ci"
ln -s "$root/.tools/host" "$fixture/.tools/host"
cp "$root/scripts/release/"{runtime-lock.sh,runtime-pin.sh} "$fixture/scripts/release/"
cp "$root/scripts/ci/"{read-cargo-workspace-version.sh,StableVersion.pm} "$fixture/scripts/ci/"
cp "$root/ci/"{release-editor.env,dependency-pinning-exceptions.json} "$fixture/ci/"
# Admit the real prepared editor once; the disposable fixture owns no tool store.
editor="$(bash "$root/scripts/release/editor.sh" check)"
printf '#!/usr/bin/env bash\nprintf "%%s\\n" %q\n' "$editor" > "$fixture/scripts/release/editor.sh"
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
# Only local package versions and references to those identities may change.
# Registry identities and their references (including Testkit's separate
# Platform of the same version) must remain untouched.
for side in before after; do
    source="$lock"
    if [[ "$side" == before ]]; then source="$fixture/before.lock"; fi
    "$YQ" -p toml -o json '.' "$source" | jq -S --arg before "$version" '
        [.package[] | select(.source != null or (.name != "ic-platform" and .name != "ic-platform-macros" and .name != "ic-platform-runtime-guard" and .name != "ic-platform-types")) |
            if .source == null and .dependencies != null then
                .dependencies |= map(
                    if . == "ic-platform 99.0.0" then "ic-platform " + $before
                    elif . == "ic-platform-macros 99.0.0" then "ic-platform-macros " + $before
                    elif . == "ic-platform-runtime-guard 99.0.0" then "ic-platform-runtime-guard " + $before
                    elif . == "ic-platform-types 99.0.0" then "ic-platform-types " + $before
                    else . end)
            else . end]' > "$fixture/$side.json"
done
cmp "$fixture/before.json" "$fixture/after.json"
cp "$lock" "$fixture/prepared.lock"
bash "$fixture/scripts/release/runtime-lock.sh" prepare "$version"
cmp "$fixture/prepared.lock" "$lock"
# Once a qualified graph includes types, it follows the same workspace version.
if ! "$YQ" -p toml -o json '.' "$lock" | jq -e 'any(.package[]; .source == null and .name == "ic-platform-types")' > /dev/null; then
    printf '\n[[package]]\nname = "ic-platform-types"\nversion = "99.0.0"\n' >> "$lock"
fi
printf '[workspace]\n[workspace.package]\nversion = "99.0.1"\n' > "$fixture/Cargo.toml"
bash "$fixture/scripts/release/runtime-lock.sh" prepare 99.0.0
bash "$fixture/scripts/release/runtime-lock.sh" check
"$YQ" -p toml -o json '.' "$lock" | jq -e '
    [.package[] | select(.source == null and (.name | startswith("ic-platform"))) | .version] |
    length == 4 and all(.[]; . == "99.0.1")' > /dev/null
# Exact helper constraints and their policy exception advance together, and
# a stale/wrong-base exception must fail closed without changing other pins.
bash "$fixture/scripts/release/runtime-pin.sh" check > "$fixture/pin-stale.log" 2>&1 && exit 1
cp "$fixture/ci/dependency-pinning-exceptions.json" "$fixture/before-pins.json"
if bash "$fixture/scripts/release/runtime-pin.sh" prepare 98.0.0 2> "$fixture/pin-wrong.log"; then exit 1; fi
cmp "$fixture/before-pins.json" "$fixture/ci/dependency-pinning-exceptions.json"
bash "$fixture/scripts/release/runtime-pin.sh" prepare "$version"
bash "$fixture/scripts/release/runtime-pin.sh" check
jq 'map(select(.subject != "ic-platform-runtime-guard"))' "$fixture/before-pins.json" > "$fixture/before-other-pins.json"
jq 'map(select(.subject != "ic-platform-runtime-guard"))' "$fixture/ci/dependency-pinning-exceptions.json" > "$fixture/after-other-pins.json"
cmp "$fixture/before-other-pins.json" "$fixture/after-other-pins.json"
bash "$fixture/scripts/release/runtime-pin.sh" prepare "$version"
fixture_complete=true
echo 'Runtime lock drift, wrong-base refusal, coordinated update and retry checks passed'
