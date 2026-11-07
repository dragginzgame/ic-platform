#!/usr/bin/env bash
# Required leaf of check-release-commands; source admission remains with its plan.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
if [[ "${1:-}" == --execute && $# == 1 ]]; then
    bash scripts/ci/test-platform-validation-plan.sh
    bash scripts/ci/test-platform-validation-recovery.sh
    exit
fi
[[ $# == 0 ]] || exit 2
if [[ -z "${PLATFORM_VALIDATION_ATTEMPT:-}" ]]; then
    echo 'Policy fixtures: fresh execution (outside a committed group qualification).'
    exec bash "$0" --execute
fi
attempt="$PLATFORM_VALIDATION_ATTEMPT"
[[ "$attempt" =~ ^target/release-validation/qualification\.[A-Za-z0-9]+$ && -f "$attempt/plan.json" ]] || exit 2
source="$(jq -r .source "$attempt/plan.json")"
group="$(jq -r .group "$attempt/plan.json")"
[[ "$(git rev-parse HEAD)" == "$source" && "$group" == "${RELEASE_GROUP:-}" ]] || exit 1
mkdir -p "$attempt/stage-admissions"
# Unsupported native hosts run the same leaf freshly; its parent remains required.
if [[ "$(uname -sm)" != 'Linux x86_64' ]]; then
    echo 'Policy fixtures: fresh execution (native host not qualified for reuse).'
    exec bash "$0" --execute
fi
perl scripts/release/policy-fixture-evidence.pl environment "$attempt/stage-environment.json"
perl scripts/ci/run-validation-stage.pl --state target/validation-stages/policy-fixtures \
    --stage release-policy-fixtures --contract platform-policy-fixtures/1 \
    --source "$source" --group "$group" --observe scripts/release/observe-policy-fixtures.pl \
    --environment "$attempt/stage-environment.json" --max-bytes 536870912 \
    --admission-output "$attempt/stage-admissions/release-policy-fixtures.json" \
    -- bash scripts/release/check-policy-fixtures.sh --execute
