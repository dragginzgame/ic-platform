#!/usr/bin/env bash
set -euo pipefail
unset RELEASE_GROUP RELEASE_COMMIT
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
source "$root/scripts/ci/fixture-lifecycle.sh"
fixture="$(fixture_create "${TMPDIR:-/tmp}/platform-runtime-lock.XXXXXX")"
fixture_complete=false
trap 'fixture_finish "$?" "$fixture" "Runtime lock fixture retained"' EXIT
mkdir -p "$fixture/scripts/release" "$fixture/scripts/fixtures" "$fixture/.tools" "$fixture/ci" "$fixture/crates"
ln -s "$root/.tools/host" "$fixture/.tools/host"
cp "$root/scripts/release/"{runtime-lock.sh,runtime-pin.sh} "$fixture/scripts/release/"
cp "$root/ci/"{cargo-release-groups.json,release-editor.env,dependency-pinning-exceptions.json} "$fixture/ci/"
cp -R "$root/scripts/fixtures/"{cdk-runtime,timers-runtime} "$fixture/scripts/fixtures/"
editor="$(bash "$root/scripts/release/editor.sh" check)"
printf '#!/usr/bin/env bash\nprintf "%%s\\n" %q\n' "$editor" > "$fixture/scripts/release/editor.sh"
export PATH="$root/.tools/host/bin:$PATH"
for package in ic-platform ic-platform-pocket-ic ic-platform-runtime ic-platform-types; do
    mkdir "$fixture/crates/$package"
    cp "$root/crates/$package/Cargo.toml" "$fixture/crates/$package/"
done
printf '[workspace]\n' > "$fixture/Cargo.toml"
bash "$fixture/scripts/release/runtime-lock.sh" check
for group in platform pocket-ic runtime types; do
    primary="$(jq -r --arg group "$group" '.groups[] | select(.name == $group) | .primary' "$fixture/ci/cargo-release-groups.json")"
    members="$(jq -r --arg group "$group" '.groups[] | select(.name == $group) | .members | join(" ")' "$fixture/ci/cargo-release-groups.json")"
    previous="$(yq -p toml -r '.package.version' "$fixture/crates/$primary/Cargo.toml")"
    for profile in cdk-runtime timers-runtime; do
        cp "$fixture/scripts/fixtures/$profile/Cargo.lock" "$fixture/$profile.before"
    done
    printf '[package]\nversion = "99.1.0"\n' > "$fixture/crates/$primary/Cargo.toml"
    if bash "$fixture/scripts/release/runtime-lock.sh" check > "$fixture/$group-stale.log" 2>&1; then exit 1; fi
    if RELEASE_GROUP="$group" bash "$fixture/scripts/release/runtime-lock.sh" prepare 98.0.0 > "$fixture/$group-wrong.log" 2>&1; then exit 1; fi
    for profile in cdk-runtime timers-runtime; do cmp "$fixture/scripts/fixtures/$profile/Cargo.lock" "$fixture/$profile.before"; done
    RELEASE_GROUP="$group" bash "$fixture/scripts/release/runtime-lock.sh" prepare "$previous"
    bash "$fixture/scripts/release/runtime-lock.sh" check
    for profile in cdk-runtime timers-runtime; do
        lock="$fixture/scripts/fixtures/$profile/Cargo.lock"
        # shellcheck disable=SC2086 # Validated group roster contains package names.
        "$editor" --max-input-bytes 1048576 lock "$lock" 99.1.0 "$previous" $members > "$fixture/restored"
        cmp "$fixture/restored" "$fixture/$profile.before"
        cp "$lock" "$fixture/$profile.after"
    done
    RELEASE_GROUP="$group" bash "$fixture/scripts/release/runtime-lock.sh" prepare "$previous"
    for profile in cdk-runtime timers-runtime; do cmp "$fixture/scripts/fixtures/$profile/Cargo.lock" "$fixture/$profile.after"; done
    if [[ "$group" == runtime ]]; then
        if bash "$fixture/scripts/release/runtime-pin.sh" check > "$fixture/pin-stale.log" 2>&1; then exit 1; fi
        cp "$fixture/ci/dependency-pinning-exceptions.json" "$fixture/pins-before"
        if RELEASE_GROUP=runtime bash "$fixture/scripts/release/runtime-pin.sh" prepare 98.0.0 > "$fixture/pin-wrong.log" 2>&1; then exit 1; fi
        cmp "$fixture/pins-before" "$fixture/ci/dependency-pinning-exceptions.json"
        RELEASE_GROUP=runtime bash "$fixture/scripts/release/runtime-pin.sh" prepare "$previous"
    fi
    bash "$fixture/scripts/release/runtime-pin.sh" check
done
# Missing or duplicated singleton evidence never silently passes custody.
lock="$fixture/scripts/fixtures/cdk-runtime/Cargo.lock"
cp "$lock" "$fixture/complete.lock"
perl -0pi -e 's/\[\[package\]\]\nname = "ic-platform-runtime-guard"\n.*?(?=\[\[package\]\]|\z)//s' "$lock"
if bash "$fixture/scripts/release/runtime-lock.sh" check > "$fixture/missing.log" 2>&1; then exit 1; fi
grep -q 'missing local ic-platform-runtime-guard' "$fixture/missing.log"
cp "$fixture/complete.lock" "$lock"
printf '\n[[package]]\nname = "ic-platform-runtime-guard"\nversion = "99.1.0"\n' >> "$lock"
if bash "$fixture/scripts/release/runtime-lock.sh" check > "$fixture/duplicate.log" 2>&1; then exit 1; fi
for temporary in "$fixture"/scripts/fixtures/*/Cargo.lock.*; do [[ ! -e "$temporary" ]] || exit 1; done
fixture_complete=true
echo 'All four groups preserve unrelated lock identities; wrong-base/missing/duplicate refusals and retry pass.'
