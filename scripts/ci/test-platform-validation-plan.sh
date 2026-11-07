#!/usr/bin/env bash
# Real selector + Shared plan runner, disposable stage bodies. No product gate claim.
set -euo pipefail
[[ $# == 0 || ( $# == 1 && "$1" == --prepare ) ]] || exit 2
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
mkdir -p "$root/target"
fixture="$(mktemp -d "$root/target/release-plan-test.XXXXXX")"
trap 'printf "Platform validation-plan fixture evidence: %s\n" "$fixture"' EXIT
mkdir -p "$fixture/repo/scripts/ci" "$fixture/repo/scripts/release" "$fixture/repo/ci"
cp "$root/ci/"{release-validation.json,cargo-release-groups.json} "$fixture/repo/ci/"
cp "$root/scripts/release/"{validation-plan.pl,validation-context.pl,validation-receipt.pl,qualify-validation-plan.sh} "$fixture/repo/scripts/release/"
bash "$root/scripts/ci/prepare-script-fixture.sh" "$root" "$fixture/repo" scripts/ci/run-validation-targets.sh scripts/ci/validation-evidence.pl scripts/ci/check-make-execution.sh
# Retain the actual full-gate recipe text so roster drift is checked against it.
perl -0777 -ne 'for my $name (qw(ci check-runtime-all release-verify)) { /(^\Q$name\E:\n(?:\t[^\n]*\n)+)/m or die "missing recipe"; print $1,"\n"; }' "$root/Makefile" > "$fixture/repo/Makefile"
jq -r '.stages[]' "$root/ci/release-validation.json" | while read -r target; do
    case "$target" in ci|check-runtime-all|release-verify) exit 1 ;; esac
    printf '.PHONY: %s\n%s:\n\t@bash fixture-stage.sh %s\n' "$target" "$target" "$target" >> "$fixture/repo/Makefile"
done
cat > "$fixture/repo/fixture-stage.sh" <<'STAGE'
set -eu
printf '%s\n' "$1"
[[ "$1" != "${FAIL_STAGE:-}" ]] || exit 27
if [[ "$1" == "${INTERRUPT_STAGE:-}" ]]; then kill -TERM "$PPID"; exit 143; fi
if [[ "$1" == "${DRIFT_STAGE:-}" ]]; then printf '\nchanged\n' >> crates/ic-platform-runtime/src/lib.rs; fi
STAGE
cd "$fixture/repo"
printf '/target/\n' > .gitignore
for package in ic-platform ic-platform-runtime ic-platform-types; do
    mkdir -p "crates/$package/src"
    printf '// unchanged source\n' > "crates/$package/src/lib.rs"
done
mkdir -p docs/status releases/{platform,runtime,types}
printf 'fixture\n' > docs/status/current.md
printf 'fixture lock\n' > Cargo.lock
git init -q
git config user.name 'Platform validation fixture'
git config user.email fixture@example.invalid
git add .
git commit -qm baseline
base="$(git rev-parse HEAD)"
for group in platform runtime types; do git tag "$group-v1.0.0"; done
if [[ ${1:-} == --prepare ]]; then exit 0; fi
refuse() {
    if "$@" > "$fixture/refusal.log" 2>&1; then echo 'unexpected admission' >&2; exit 1; fi
}
for group in platform runtime types; do
    git checkout -q -B "test-$group" "$base"
    case "$group" in platform) package=ic-platform ;; runtime) package=ic-platform-runtime ;; types) package=ic-platform-types ;; esac
    printf '// owner change\n' >> "crates/$package/src/lib.rs"
    git commit -qam 'owner source change'
    head="$(git rev-parse HEAD)"
    mkdir -p "target/$group"
    RELEASE_PREVIOUS=1.0.0 RELEASE_VERSION=1.0.1 RELEASE_REMOTE=origin RELEASE_BRANCH=main \
        perl scripts/release/validation-plan.pl "$group" "$head" "$group-v1.0.0" "target/$group/release.json" release
    jq -e '.scope=="workspace" and ([.stages[].required] | all)' "target/$group/release.json"
    perl scripts/release/validation-plan.pl "$group" "$head" none "target/$group/no-baseline.json" qualification
    jq -e '.scope=="workspace"' "target/$group/no-baseline.json"
    perl scripts/release/validation-plan.pl "$group" "$head" "$group-v1.0.0" "target/fresh/$group/nested/full.json" qualification
    test -f "target/fresh/$group/nested/full.json.context.json"
    perl scripts/release/validation-plan.pl "$group" "$head" "$group-v1.0.0" "target/$group/full.json" qualification
    jq -e '.scope=="workspace" and ([.stages[].required] | all)' "target/$group/full.json"
    bash scripts/release/qualify-validation-plan.sh "$group" "$group-v1.0.0" qualification > "$fixture/$group-full.log"
    FAIL_STAGE=check-release-commands refuse bash scripts/release/qualify-validation-plan.sh "$group" "$group-v1.0.0" qualification
    FAIL_STAGE=clippy refuse bash scripts/release/qualify-validation-plan.sh "$group" "$group-v1.0.0" qualification
    refuse perl scripts/release/validation-plan.pl unknown "$head" "$group-v1.0.0" "target/$group/unknown.json" qualification
    refuse perl scripts/release/validation-plan.pl "$group" "$base" "$group-v1.0.0" "target/$group/stale.json" qualification
    # Shared lock changes also retain every full-gate obligation.
    printf 'changed lock\n' >> Cargo.lock
    git commit -qam 'shared lock change'
    perl scripts/release/validation-plan.pl "$group" "$(git rev-parse HEAD)" "$group-v1.0.0" "target/$group/shared.json" qualification
    jq -e '.scope=="workspace" and ([.stages[].required] | all)' "target/$group/shared.json"
done
# Group roster and full-gate roster cannot silently lose or gain an obligation.
git checkout -q -B refusal "$base"
jq -cS '.stages += ["new-stage"]' ci/release-validation.json > target/changed.json
cp target/changed.json ci/release-validation.json
git commit -qam 'unmapped stage'
refuse perl scripts/release/validation-plan.pl types "$(git rev-parse HEAD)" types-v1.0.0 target/unknown-stage.json qualification
git checkout -q -B drift "$base"
printf '// changed\n' >> crates/ic-platform-runtime/src/lib.rs
git commit -qam 'client change'
DRIFT_STAGE=clippy refuse bash scripts/release/qualify-validation-plan.sh runtime runtime-v1.0.0 qualification
git restore crates/ic-platform-runtime/src/lib.rs
INTERRUPT_STAGE=clippy refuse bash scripts/release/qualify-validation-plan.sh runtime runtime-v1.0.0 qualification
# A fresh attempt succeeds after an interrupted/failed one; no receipt is reused.
bash scripts/release/qualify-validation-plan.sh runtime runtime-v1.0.0 qualification > "$fixture/retry.log"
receipt="$(sed -n '/\/receipt.json$/p' "$fixture/retry.log")"
attempt="${receipt%/receipt.json}"
run=("$attempt"/logs/validation.*)
cp "${run[0]}/plan-events.tsv" "$fixture/events.original"
sed '/clippy\tPASS/d' "$fixture/events.original" > "${run[0]}/plan-events.tsv"
refuse perl scripts/release/validation-receipt.pl "$attempt/plan.json" "${run[0]}" target/incomplete-receipt.json
cp "$fixture/events.original" "${run[0]}/plan-events.tsv"
[[ ! -e target/incomplete-receipt.json ]]
cp "$attempt/plan.json" "$fixture/plan.original"
jq -cS '.stages[0].required=false' "$fixture/plan.original" > "$attempt/plan.json"
refuse perl scripts/release/validation-receipt.pl "$attempt/plan.json" "${run[0]}" target/omitted-receipt.json
cp "$fixture/plan.original" "$attempt/plan.json"
[[ ! -e target/omitted-receipt.json ]]

# Same-source evidence can be rechecked, but cannot qualify a moved source.
perl scripts/release/validation-receipt.pl "$attempt/plan.json" "${run[0]}" target/readmitted.json
# Platform retains the entire unmodified Shared proof, including the relative
# run/log paths, input checkpoints and evidence inventory; not just a stage list.
jq -cS .validation_evidence "$receipt" > target/retained-proof.json
perl scripts/ci/validation-evidence.pl check "$PWD" "$attempt/plan.json" "$(git rev-parse HEAD)" runtime \
    "${run[0]}" "$attempt/plan.json.context.json" target/retained-proof.json > "$fixture/shared-readmission.json"
cmp target/retained-proof.json "$fixture/shared-readmission.json"
refuse perl scripts/release/validation-receipt.pl "$attempt/plan.json" "${run[0]}" "$receipt" check
RUSTFLAGS='--cfg changed_receipt_environment' refuse perl scripts/release/validation-receipt.pl "$attempt/plan.json" "${run[0]}" target/config-drift.json
printf '// drift\n' >> crates/ic-platform-runtime/src/lib.rs
git commit -qam 'moved source'
refuse perl scripts/release/validation-receipt.pl "$attempt/plan.json" "${run[0]}" target/stale-receipt.json
printf 'Three-group full gates, required-stage failures, drift and recovery passed. Stage bodies were substitutes.\n'
