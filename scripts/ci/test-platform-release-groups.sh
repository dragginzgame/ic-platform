#!/usr/bin/env bash
# Public grouped consumer routing; actual recovery semantics are tested separately.
set -euo pipefail
unset PACKAGE RELEASE_GROUP
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
bash "$root/scripts/ci/check-make-execution.sh"
unset MAKEFLAGS MFLAGS MAKEOVERRIDES
fixture="$(mktemp -d "${TMPDIR:-/tmp}/platform-release-routing.XXXXXX")"
trap 'printf "Release routing evidence: %s\n" "$fixture"' EXIT
cd "$root"
bash scripts/ci/prepare-script-fixture.sh "$root" "$fixture" Makefile make/consumer.mk make/tools.mk make/release.mk make/rust-format.mk make/execution.mk scripts/ci/check-make-execution.sh scripts/ci/check-release-delivery.sh ci/release-delivery.txt
mkdir -p "$fixture/scripts/release" "$fixture/scripts/ci" "$fixture/scripts/dev"
ln -s "$root/.tools" "$fixture/.tools"
export ROUTING_OUTPUT="$fixture/selection" ROUTING_EFFECTS="$fixture/effects"
export PLATFORM_RELEASE_TEST_ROOT="$root" PLATFORM_RELEASE_TEST_FIXTURE="$fixture"
cargo metadata --no-deps --locked --offline --format-version 1 > "$fixture/metadata.json"
cat > "$fixture/scripts/ci/run-release.sh" <<'RUNNER'
#!/usr/bin/env bash
set -euo pipefail
[[ "$RELEASE_GROUP" == "$EXPECTED_GROUP" && "$1" == "$EXPECTED_OPERATION" ]]
printf '%s\n' "$RELEASE_GROUP" > "$ROUTING_OUTPUT"
printf 'release %s %s\n' "$1" "$RELEASE_GROUP" >> "$ROUTING_EFFECTS"
[[ "${2:-}" == review && "${3:-}" == release-review || "$1" == resume && "${2:-}" == 99.0.0 && "${3:-}" == review && "${4:-}" == release-review ]]
exit "${ROUTING_RUNNER_RESULT:-0}"
RUNNER
cp scripts/release/publish.sh "$fixture/scripts/release/"
cat > "$fixture/scripts/release/check-resume-policy.sh" <<'RESUME'
#!/usr/bin/env bash
[[ "$1" == 99.0.0 && -n "$RELEASE_GROUP" ]]
RESUME
cat > "$fixture/scripts/release/group.sh" <<'GROUP'
#!/usr/bin/env bash
exec bash "$PLATFORM_RELEASE_TEST_ROOT/scripts/release/group.sh" "$@"
GROUP
cat > "$fixture/scripts/dev/install-host-tools.sh" <<'HOST'
#!/usr/bin/env bash
printf 'host admission\n' >> "$ROUTING_EFFECTS"
HOST
cat > "$fixture/scripts/ci/cargo-publication-adapter.sh" <<'PUBLISH'
#!/usr/bin/env bash
set -euo pipefail
[[ "$CARGO_PUBLISH_REGISTRY" == crates-io && "$CARGO_PUBLISH_VERIFY_TARGETS" == clippy ]]
[[ "$CARGO_PUBLISH_ALL_FEATURES" == 1 && "$CARGO_PUBLISH_CHECK_ALLOW_DIRTY" == 1 ]]
[[ "$CARGO_PUBLISH_EVIDENCE_ROOT" == "$PLATFORM_RELEASE_TEST_FIXTURE/target/publication" ]]
[[ "$CARGO_PUBLISH_MAKE" == "$PLATFORM_RELEASE_TEST_FIXTURE/scripts/release/verify-publication.sh" ]]
[[ "$CARGO_PUBLISH_PACKAGES" == "$(bash "$PLATFORM_RELEASE_TEST_ROOT/scripts/release/group.sh" members)" ]]
perl "$PLATFORM_RELEASE_TEST_ROOT/scripts/ci/cargo-publication-selection.pl" \
    "$PLATFORM_RELEASE_TEST_FIXTURE/metadata.json" "$CARGO_PUBLISH_PACKAGES" "$CARGO_PUBLISH_SELECTION" crates-io \
    > "$PLATFORM_RELEASE_TEST_FIXTURE/selectors"
printf '%s\n' "$CARGO_PUBLISH_SELECTION" > "$ROUTING_OUTPUT"
printf 'publication %s %s\n' "$1" "$RELEASE_GROUP" >> "$ROUTING_EFFECTS"
PUBLISH
# Missing, empty and unknown selections refuse at Make parsing, before any
# fixture tool gate/runner. PACKAGE must never infer the release group.
for operation in release-patch release-minor release-major release-resume publish publish-check; do
    for selection in missing empty unknown; do
        arguments=()
        case "$selection" in empty) arguments=(RELEASE_GROUP=) ;; unknown) arguments=(RELEASE_GROUP=unknown) ;; esac
        if make --no-print-directory -s -C "$fixture" "$operation" PACKAGE=ic-platform-types ${arguments[@]+"${arguments[@]}"} > "$fixture/refusal.log" 2>&1; then exit 1; fi
        grep -q 'Select RELEASE_GROUP=platform or RELEASE_GROUP=types' "$fixture/refusal.log"
        [[ ! -e "$ROUTING_EFFECTS" ]]
    done
done
for group in platform types; do
    for operation in release-patch release-minor release-major release-resume; do
        for result in 0 17; do
            status=0
            EXPECTED_GROUP="$group" EXPECTED_OPERATION="${operation#release-}" ROUTING_RUNNER_RESULT="$result" make --no-print-directory -s -C "$fixture" "$operation" RELEASE_GROUP="$group" VERSION=99.0.0 RELEASE_REMOTE=review RELEASE_BRANCH=release-review > "$fixture/runner.log" 2>&1 || status=$?
            [[ "$(cat "$ROUTING_OUTPUT")" == "$group" ]]
            if [[ "$result" == 0 ]]; then [[ "$status" == 0 ]]; else [[ "$status" != 0 ]]; fi
        done
    done
    # Explicit environment selection works as well as a Make command-line value.
    RELEASE_GROUP="$group" EXPECTED_GROUP="$group" EXPECTED_OPERATION=patch make --no-print-directory -s -C "$fixture" release-patch RELEASE_REMOTE=review RELEASE_BRANCH=release-review
    cp "$ROUTING_EFFECTS" "$fixture/before-conflict"
    if make --no-print-directory -s -C "$fixture" release-patch release-minor RELEASE_GROUP="$group" > "$fixture/conflict.log" 2>&1; then exit 1; fi
    cmp "$ROUTING_EFFECTS" "$fixture/before-conflict"
    expected="$(RELEASE_GROUP="$group" bash scripts/release/group.sh version)"
    [[ "$(make --no-print-directory -s release-version RELEASE_GROUP="$group")" == "$expected" ]]
    make --no-print-directory -s release-files RELEASE_GROUP="$group" > "$fixture/$group-files"
    bash scripts/ci/cargo-release-group.sh files "$group" > "$fixture/$group-expected-files"
    cmp "$fixture/$group-files" "$fixture/$group-expected-files"
    tr '\0' '\n' < "$fixture/$group-files" | grep -Fx 'scripts/fixtures/cdk-runtime/Cargo.lock'
    for operation in publish publish-check; do
        make --no-print-directory -s -C "$fixture" "$operation" RELEASE_GROUP="$group"
        [[ "$(cat "$ROUTING_OUTPUT")" == "$(RELEASE_GROUP="$group" bash scripts/release/group.sh members)" ]]
    done
    if [[ "$group" == platform ]]; then subset=ic-platform; outside=ic-platform-types; else subset=ic-platform-types; outside=ic-platform; fi
    RELEASE_GROUP="$group" PACKAGE="$subset" bash "$fixture/scripts/release/publish.sh" check
    [[ "$(cat "$ROUTING_OUTPUT")" == "$subset" ]]
    if RELEASE_GROUP="$group" PACKAGE="$outside" bash "$fixture/scripts/release/publish.sh" check > "$fixture/cross-group.log" 2>&1; then exit 1; fi
    grep -q 'outside the publication roster' "$fixture/cross-group.log"
    if RELEASE_GROUP="$group" PACKAGE='' bash "$fixture/scripts/release/publish.sh" check > "$fixture/empty-package.log" 2>&1; then exit 1; fi
done
for operation in version files primary; do
    if bash scripts/release/group.sh "$operation" > "$fixture/cli-refusal.log" 2>&1; then exit 1; fi
    grep -q 'Select RELEASE_GROUP=platform or RELEASE_GROUP=types' "$fixture/cli-refusal.log"
done
for operation in preflight prepare check commit-check; do
    if bash scripts/release/adapter.sh "$operation" > "$fixture/adapter-refusal.log" 2>&1; then exit 1; fi
    grep -q 'Select RELEASE_GROUP=platform or RELEASE_GROUP=types' "$fixture/adapter-refusal.log"
done
for operation in check publish; do
    if bash scripts/release/publish.sh "$operation" > "$fixture/publication-refusal.log" 2>&1; then exit 1; fi
    grep -q 'Select RELEASE_GROUP=platform or RELEASE_GROUP=types' "$fixture/publication-refusal.log"
done
echo 'Explicit Make/CLI group admission, both rosters and same-group publication subsets passed.'
