#!/usr/bin/env bash
# Consumer routing only; substitutes effectful runners and performs no releases/uploads.
set -euo pipefail
unset PACKAGE RELEASE_GROUP
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
# Qualify execution before isolating fixture Make variables from the caller's
# release-group/PACKAGE command-line overrides.
bash "$root/scripts/ci/check-make-execution.sh"
unset MAKEFLAGS MFLAGS MAKEOVERRIDES
fixture="$(mktemp -d "${TMPDIR:-/tmp}/platform-release-routing.XXXXXX")"
trap 'printf "Release routing evidence: %s\n" "$fixture"' EXIT
cd "$root"
bash scripts/ci/prepare-script-fixture.sh "$root" "$fixture" Makefile make/consumer.mk make/tools.mk make/release.mk make/rust-format.mk make/execution.mk scripts/ci/check-make-execution.sh scripts/ci/check-release-delivery.sh ci/release-delivery.txt
mkdir -p "$fixture/scripts/release" "$fixture/scripts/ci"
cat > "$fixture/scripts/ci/run-release.sh" <<'RUNNER'
#!/usr/bin/env bash
set -euo pipefail
[[ "$1" == minor && "$RELEASE_GROUP" == "$EXPECTED_GROUP" ]]
printf '%s\n' "$RELEASE_GROUP" > "$ROUTING_OUTPUT"
RUNNER
export ROUTING_OUTPUT="$fixture/release-selection"
(unset RELEASE_GROUP; EXPECTED_GROUP=platform make --no-print-directory -s -C "$fixture" "SHARED_TOOLING_ROOT=$fixture" release-minor)
[[ "$(cat "$ROUTING_OUTPUT")" == platform ]]
EXPECTED_GROUP=platform make --no-print-directory -s -C "$fixture" "SHARED_TOOLING_ROOT=$fixture" RELEASE_GROUP=platform release-minor
expected="$(bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml)"
[[ "$(make --no-print-directory -s release-version)" == "$expected" ]]
make --no-print-directory -s release-files > "$fixture/platform-files"
tr '\0' '\n' < "$fixture/platform-files" > "$fixture/platform-lines"
grep -Fx 'scripts/fixtures/cdk-runtime/Cargo.lock' "$fixture/platform-lines"
grep -Fx 'ci/dependency-pinning-exceptions.json' "$fixture/platform-lines"
bash scripts/ci/cargo-release-group.sh files platform > "$fixture/expected-platform-files"
cmp "$fixture/platform-files" "$fixture/expected-platform-files"
for group in types unknown; do
    if make --no-print-directory -s release-version RELEASE_GROUP="$group" > "$fixture/$group.log" 2>&1; then exit 1; fi
done
# Keep real group selection; replace only the effectful Shared publication provider.
cp scripts/release/publish.sh "$fixture/scripts/release/"
export PLATFORM_RELEASE_TEST_ROOT="$root"
cat > "$fixture/scripts/release/group.sh" <<'GROUP'
#!/usr/bin/env bash
exec bash "$PLATFORM_RELEASE_TEST_ROOT/scripts/release/group.sh" "$@"
GROUP
cat > "$fixture/scripts/ci/cargo-publication-adapter.sh" <<'PUBLISH'
#!/usr/bin/env bash
set -euo pipefail
[[ "$1" == check && "$CARGO_PUBLISH_REGISTRY" == crates-io && "$CARGO_PUBLISH_VERIFY_TARGETS" == clippy ]]
[[ "$CARGO_PUBLISH_ALL_FEATURES" == 1 && "$CARGO_PUBLISH_CHECK_ALLOW_DIRTY" == 1 ]]
[[ "$CARGO_PUBLISH_EVIDENCE_ROOT" == "$PLATFORM_RELEASE_TEST_FIXTURE/target/publication" ]]
[[ "$CARGO_PUBLISH_MAKE" == "$PLATFORM_RELEASE_TEST_FIXTURE/scripts/release/verify-publication.sh" ]]
[[ "$CARGO_PUBLISH_PACKAGES" == 'ic-platform-types ic-platform-runtime-guard ic-platform-macros ic-platform' ]]
printf '%s\n' "$CARGO_PUBLISH_SELECTION" > "$ROUTING_OUTPUT"
PUBLISH
export PLATFORM_RELEASE_TEST_FIXTURE="$fixture"
bash "$fixture/scripts/release/publish.sh" check
[[ "$(cat "$ROUTING_OUTPUT")" == "$(bash scripts/release/group.sh members)" ]]
for package in ic-platform-types ic-platform-runtime-guard ic-platform-macros ic-platform; do
    [[ " $(cat "$ROUTING_OUTPUT") " == *" $package "* ]]
done
PACKAGE='ic-platform-types ic-platform' bash "$fixture/scripts/release/publish.sh" check
[[ "$(cat "$ROUTING_OUTPUT")" == 'ic-platform-types ic-platform' ]]
if PACKAGE='' bash "$fixture/scripts/release/publish.sh" check > "$fixture/empty-package.log" 2>&1; then exit 1; fi
cargo metadata --no-deps --locked --offline --format-version 1 > "$fixture/metadata.json"
perl scripts/ci/cargo-publication-selection.pl "$fixture/metadata.json" \
    'ic-platform-types ic-platform-runtime-guard ic-platform-macros ic-platform' \
    'ic-platform-types' crates-io > "$fixture/type-package"
expected="$(jq -er '.packages[] | select(.name=="ic-platform-types") | .name+"@"+.version' "$fixture/metadata.json")"
[[ "$(tr -d '\0' < "$fixture/type-package")" == "$expected" ]]
jq -e --arg version "$(bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml)" '
    [.packages[] | select(.name | startswith("ic-platform"))] |
    length == 4 and all(.[]; .version == $version)' "$fixture/metadata.json" > /dev/null
echo 'One coordinated four-package release/publication, metadata custody and explicit subset selection passed.'
