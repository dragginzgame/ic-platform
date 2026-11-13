#!/usr/bin/env bash
# Exercise Platform's policy decision through Shared's real read-only projection.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
export PATH="$root/.tools/host/bin:$PATH"
mkdir -p "$root/target"
fixture="$(mktemp -d "$root/target/resume-policy.XXXXXX")"
trap 'printf "Resume policy evidence: %s\n" "$fixture"' EXIT
mkdir -p "$fixture/scripts/release" "$fixture/ci"
cp "$root/scripts/release/check-resume-policy.sh" "$fixture/scripts/release/"
bash "$root/vendor/toko-tooling/scripts/ci/prepare-script-fixture.sh" "$root" "$fixture" vendor/toko-tooling/scripts/ci/release-plan.pl
git -C "$fixture" init -q -b main
git -C "$fixture" config user.name Fixture
git -C "$fixture" config user.email fixture@example.invalid
printf '{"groups":[]}\n' > "$fixture/ci/cargo-release-groups.json"
git -C "$fixture" add ci scripts
git -C "$fixture" -c core.hooksPath=/dev/null commit -qm 'Resume policy source'
cd "$fixture"
source_commit="$(git rev-parse HEAD)"
source_tree="$(git write-tree)"
mkdir -p .git/release-state
plan=.git/release-state/platform-1.0.1.plan
write_plan() {
    printf '%s\n' release-group-plan-1 patch 1.0.0 1.0.1 2026-11-08 "$source_commit" origin main "$source_commit" "$source_tree" "$1" platform > "$plan"
}
check() { RELEASE_GROUP=platform bash scripts/release/check-resume-policy.sh 1.0.1; }
refuse() {
    if check > "$fixture/refusal.log" 2>&1; then echo 'Unexpected resume admission' >&2; exit 1; fi
}
write_plan prepare
cp "$plan" "$fixture/journal.original"
check
cmp "$plan" "$fixture/journal.original"
printf ' ' >> ci/cargo-release-groups.json
refuse
grep -q 'Release policy changed since the saved attempt' "$fixture/refusal.log"
cmp "$plan" "$fixture/journal.original"
write_plan complete
check
printf 'unknown-schema\n' > "$plan"
refuse
grep -q 'unknown schema' "$fixture/refusal.log"
write_plan prepare
if RELEASE_GROUP=runtime bash scripts/release/check-resume-policy.sh 1.0.1 > "$fixture/missing.log" 2>&1; then exit 1; fi
[[ "$(git rev-parse HEAD)" == "$source_commit" && "$(git write-tree)" == "$source_tree" ]]
[[ ! -e .git/release-state/lock ]]
echo 'Validated journal projection preserves policy-drift refusal and completed-plan behavior without effects.'
