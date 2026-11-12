#!/usr/bin/env bash
# Public grouped consumer routing; actual recovery semantics are tested separately.
set -euo pipefail
unset PACKAGE RELEASE_GROUP
unset RELEASE_GROUPS RELEASE_PUBLISH_TARGET RELEASE_BATCH_ACTIVE
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
bash "$root/scripts/ci/check-make-execution.sh"
unset MAKEFLAGS MFLAGS MAKEOVERRIDES
mkdir -p "$root/target"
fixture="$(mktemp -d "$root/target/platform-release-routing.XXXXXX")"
trap 'printf "Release routing evidence: %s\n" "$fixture"' EXIT
cd "$root"
bash scripts/ci/prepare-script-fixture.sh "$root" "$fixture" Makefile make/consumer.mk make/tools.mk make/release.mk make/rust-format.mk make/execution.mk make/cargo-dependencies.mk scripts/ci/check-make-execution.sh scripts/ci/check-release-delivery.sh scripts/ci/check-retained-state.pl ci/release-delivery.txt
git -C "$fixture" init -q
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
cat > "$fixture/scripts/ci/run-release-batch.pl" <<'FROZEN'
use strict;
use warnings;
use Getopt::Long qw(GetOptions);
my ($root,$groups,$kind,$remote,$branch,$setup,$publish,$delivery,$status,$resume,$reconcile,$retire);
GetOptions('root=s'=>\$root,'groups=s'=>\$groups,'kind=s'=>\$kind,'remote=s'=>\$remote,
    'branch=s'=>\$branch,'setup-target=s'=>\$setup,'publish-target=s'=>\$publish,'delivery=s'=>\$delivery,
    'status=s'=>\$status,'resume=s'=>\$resume,'reconcile=s'=>\$reconcile,'retire=s'=>\$retire) or die;
my ($operation,$selection);
if (defined $groups) {
    $setup eq 'release-candidate-setup' && $remote eq 'review' && $branch eq 'release-review' or die;
    ($operation,$selection)=($kind,$groups);
} else {
    for my $entry ([status=>$status],[resume=>$resume],[reconcile=>$reconcile],[retire=>$retire]) {
        ($operation,$selection)=@$entry if defined $entry->[1];
    }
}
$operation eq $ENV{EXPECTED_OPERATION} && $selection eq $ENV{EXPECTED_GROUP} or die;
open my $out,'>',$ENV{ROUTING_OUTPUT} or die $!; print {$out} "$selection\n"; close $out or die $!;
open my $events,'>>',$ENV{ROUTING_EFFECTS} or die $!;
print {$events} "frozen $operation $selection publish=",($publish//''),"\n"; close $events or die $!;
exit($ENV{ROUTING_RUNNER_RESULT}//0);
FROZEN
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
cat > "$fixture/scripts/dev/toko-tooling.sh" <<'TOOLING'
#!/usr/bin/env bash
set -euo pipefail
[[ "$1" == check ]]
printf 'tooling admission\n' >> "$ROUTING_EFFECTS"
[[ "${ROUTING_TOOLING_REFUSE:-0}" == 0 ]]
TOOLING
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
    for selection in missing empty unknown retired; do
        arguments=()
        case "$selection" in empty) arguments=(RELEASE_GROUP=) ;; unknown) arguments=(RELEASE_GROUP=unknown) ;; retired) arguments=(RELEASE_GROUP=pocket-ic) ;; esac
        if make --no-print-directory -s -C "$fixture" "$operation" PACKAGE=ic-platform-types ${arguments[@]+"${arguments[@]}"} > "$fixture/refusal.log" 2>&1; then exit 1; fi
        grep -q 'Select RELEASE_GROUP=platform, RELEASE_GROUP=runtime or RELEASE_GROUP=types' "$fixture/refusal.log"
        [[ ! -e "$ROUTING_EFFECTS" ]]
    done
done
# The installed interface rejects retired configuration before any tool/runner
# effects, including an explicitly empty value. Never alias an old roster.
for setting in RELEASE_FROZEN_GROUPS RELEASE_FROZEN_PUBLISH_TARGET RELEASE_FROZEN_SETUP_TARGET RELEASE_FROZEN_SETUP_MODE FROZEN_RELEASE_ACTIVE; do
    for value in '' 1; do
        if make --no-print-directory -s -C "$fixture" help "$setting=$value" > "$fixture/retired-setting.log" 2>&1; then exit 1; fi
        grep -q "Retired release settings:.*$setting" "$fixture/retired-setting.log"
        [[ ! -e "$ROUTING_EFFECTS" ]]
    done
done
if make --no-print-directory -s -C "$fixture" release-patch RELEASE_GROUP=platform RELEASE_GROUPS= > "$fixture/cleared-roster.log" 2>&1; then exit 1; fi
[[ ! -e "$ROUTING_EFFECTS" ]]
# The outer Makefile admits a whole canonical cohort without a single group.
EXPECTED_GROUP=types,runtime,platform EXPECTED_OPERATION=patch make --no-print-directory -s -C "$fixture" release-patch \
    RELEASE_GROUPS=types,runtime,platform RELEASE_PUBLISH_TARGET=publish RELEASE_REMOTE=review RELEASE_BRANCH=release-review
[[ "$(cat "$ROUTING_OUTPUT")" == types,runtime,platform ]]
grep -q 'frozen patch types,runtime,platform publish=publish' "$ROUTING_EFFECTS"
cp "$ROUTING_EFFECTS" "$fixture/before-batch-refusal"
for roster in runtime,types types,types types,unknown; do
    if make --no-print-directory -s -C "$fixture" release-patch RELEASE_GROUPS="$roster" > "$fixture/batch-refusal.log" 2>&1; then exit 1; fi
    cmp "$ROUTING_EFFECTS" "$fixture/before-batch-refusal"
done
for operation in status resume reconcile retire; do
    EXPECTED_GROUP=batch-reviewed EXPECTED_OPERATION="$operation" make --no-print-directory -s -C "$fixture" "release-batch-$operation" BATCH=batch-reviewed
    [[ "$(cat "$ROUTING_OUTPUT")" == batch-reviewed ]]
done
for group in platform runtime types; do
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
    subset="$(RELEASE_GROUP="$group" bash scripts/release/group.sh primary)"
    outside=ic-platform; [[ "$group" != platform ]] || outside=ic-platform-types
    RELEASE_GROUP="$group" PACKAGE="$subset" bash "$fixture/scripts/release/publish.sh" check
    [[ "$(cat "$ROUTING_OUTPUT")" == "$subset" ]]
    if RELEASE_GROUP="$group" PACKAGE="$outside" bash "$fixture/scripts/release/publish.sh" check > "$fixture/cross-group.log" 2>&1; then exit 1; fi
    grep -q 'outside the publication roster' "$fixture/cross-group.log"
    if RELEASE_GROUP="$group" PACKAGE='' bash "$fixture/scripts/release/publish.sh" check > "$fixture/empty-package.log" 2>&1; then exit 1; fi
done
# Stale tooling stops both Make and direct custom publication before host or
# publication effects. The metadata adapter must also stop before preparation.
cp scripts/release/adapter.sh "$fixture/scripts/release/"
mkdir -p "$fixture/scripts/guarded-empty"
printf '/scripts/guarded-empty/\n' >> "$fixture/.gitignore"
for operation in publish publish-check direct-publication metadata-prepare capture batch-resume; do
    cp "$ROUTING_EFFECTS" "$fixture/before-tooling-refusal"
    refused=false
    case "$operation" in
        publish|publish-check)
            if ! ROUTING_TOOLING_REFUSE=1 make --no-print-directory -s -j2 -C "$fixture" "$operation" RELEASE_GROUP=platform > "$fixture/tooling-refusal.log" 2>&1; then refused=true; fi ;;
        direct-publication)
            if ! ROUTING_TOOLING_REFUSE=1 RELEASE_GROUP=platform bash "$fixture/scripts/release/publish.sh" check > "$fixture/tooling-refusal.log" 2>&1; then refused=true; fi ;;
        metadata-prepare)
            if ! (cd "$fixture" && ROUTING_TOOLING_REFUSE=1 RELEASE_GROUP=platform bash scripts/release/adapter.sh prepare) > "$fixture/tooling-refusal.log" 2>&1; then refused=true; fi ;;
        capture)
            if ! ROUTING_TOOLING_REFUSE=1 make --no-print-directory -s -j2 -C "$fixture" release-patch RELEASE_GROUP=platform > "$fixture/tooling-refusal.log" 2>&1; then refused=true; fi ;;
        batch-resume)
            if ! ROUTING_TOOLING_REFUSE=1 make --no-print-directory -s -j2 -C "$fixture" release-batch-resume BATCH=batch-reviewed > "$fixture/tooling-refusal.log" 2>&1; then refused=true; fi ;;
    esac
    [[ "$refused" == true ]]
    cat "$fixture/before-tooling-refusal" > "$fixture/expected-tooling-refusal"
    printf 'tooling admission\n' >> "$fixture/expected-tooling-refusal"
    cmp "$fixture/expected-tooling-refusal" "$ROUTING_EFFECTS"
    [[ -d "$fixture/scripts/guarded-empty" ]]
done
# Exercise the actual publication verifier and Make Clippy leaf. Only Cargo and
# compiler/tool admission are substitutes; no archive or upload is dispatched.
mkdir -p "$fixture/lint-bin"
cp scripts/release/verify-publication.sh "$fixture/scripts/release/"
printf '#!/bin/sh\nexit 0\n' > "$fixture/scripts/ci/check-platform-compiler.sh"
export PUBLICATION_LINT_TRACE="$fixture/lint-arguments"
cat > "$fixture/lint-bin/cargo" <<'CARGO'
#!/usr/bin/env bash
set -euo pipefail
[[ "$1" == clippy && -z "${CARGO_PUBLISH_PACKAGES+x}" ]]
printf '%s\n' "$@" > "$PUBLICATION_LINT_TRACE"
CARGO
chmod +x "$fixture/lint-bin/cargo"
for group in platform runtime types; do
    roster="$(RELEASE_GROUP="$group" bash scripts/release/group.sh members)"
    subset="${roster%% *}"
    for selection in "$roster" "$subset"; do
        (cd "$fixture" && PATH="$fixture/lint-bin:$PATH" CARGO_PUBLISH_PACKAGES="$roster" CARGO_PUBLISH_SELECTION="$selection" \
            bash scripts/release/verify-publication.sh --no-print-directory clippy) > "$fixture/lint.log" 2>&1
        {
            printf '%s\n' clippy
            for package in $selection; do printf '%s\n' --package "$package"; done
            printf '%s\n' --all-targets --all-features --locked -- -D warnings
        } > "$fixture/lint-expected"
        cmp "$fixture/lint-expected" "$PUBLICATION_LINT_TRACE"
    done
    cp "$PUBLICATION_LINT_TRACE" "$fixture/lint-before-refusal"
    for selection in '' outside-group '--all'; do
        if (cd "$fixture" && PATH="$fixture/lint-bin:$PATH" CARGO_PUBLISH_PACKAGES="$roster" CARGO_PUBLISH_SELECTION="$selection" \
            bash scripts/release/verify-publication.sh --no-print-directory clippy) > "$fixture/lint-refusal.log" 2>&1; then exit 1; fi
        cmp "$fixture/lint-before-refusal" "$PUBLICATION_LINT_TRACE"
    done
done
echo 'Publication lint reaches only admitted packages/subsets, retaining compiler and Clippy flags.'
# Exercise Platform's actual retention prerequisite, with sparse disposable
# evidence; this proves routing without replaying Shared's checker suite.
mkdir -p "$fixture/recovery-evidence"
perl -e 'open my $f, ">", $ARGV[0] or die $!; truncate $f, 268435457 or die $!' "$fixture/recovery-evidence/oversized"
for operation in release-preflight publish publish-check; do
    if make --no-print-directory -s -C "$fixture" "$operation" RELEASE_GROUP=platform > "$fixture/retention-refusal.log" 2>&1; then
        echo "Retained-evidence admission was bypassed by $operation" >&2; exit 1
    fi
    grep -q 'file exceeds 268435456 byte budget' "$fixture/retention-refusal.log"
done
rm "$fixture/recovery-evidence/oversized"
perl -e 'open my $f, ">", $ARGV[0] or die $!; truncate $f, 1048576 or die $!; utime time-31*86400, time-31*86400, $ARGV[0] or die $!' "$fixture/recovery-evidence/expired"
if make --no-print-directory -s -C "$fixture" release-preflight RELEASE_GROUP=platform > "$fixture/retention-refusal.log" 2>&1; then
    echo 'Expired retained evidence bypassed release preflight' >&2; exit 1
fi
grep -q 'bulk evidence older than 30 days' "$fixture/retention-refusal.log"
rm "$fixture/recovery-evidence/expired"
echo 'Release and publication reject oversized/expired retained evidence before their adapters.'
for operation in version files primary; do
    if bash scripts/release/group.sh "$operation" > "$fixture/cli-refusal.log" 2>&1; then exit 1; fi
    grep -q 'Select RELEASE_GROUP=platform, RELEASE_GROUP=runtime or RELEASE_GROUP=types' "$fixture/cli-refusal.log"
done
for operation in preflight prepare check commit-check; do
    if bash scripts/release/adapter.sh "$operation" > "$fixture/adapter-refusal.log" 2>&1; then exit 1; fi
    grep -q 'Select RELEASE_GROUP=platform, RELEASE_GROUP=runtime or RELEASE_GROUP=types' "$fixture/adapter-refusal.log"
done
for operation in check publish; do
    if bash scripts/release/publish.sh "$operation" > "$fixture/publication-refusal.log" 2>&1; then exit 1; fi
    grep -q 'Select RELEASE_GROUP=platform, RELEASE_GROUP=runtime or RELEASE_GROUP=types' "$fixture/publication-refusal.log"
done
echo 'Explicit Make/CLI group admission, all three rosters and same-group publication subsets passed.'
