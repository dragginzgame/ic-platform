#!/usr/bin/env bash
# Execute a fresh plan through Shared; never mutate release metadata or refs.
set -euo pipefail
[[ $# == 3 ]] || { echo 'usage: qualify-validation-plan.sh GROUP GROUP-TAG|none qualification|release' >&2; exit 2; }
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
export RELEASE_GROUP="$1" RELEASE_SOURCE
RELEASE_SOURCE="$(git rev-parse HEAD)"
mkdir -p target/release-validation
attempt="$(mktemp -d target/release-validation/qualification.XXXXXX)"
trap 'printf "Group validation evidence: %s\n" "$attempt"' EXIT
perl scripts/release/validation-plan.pl "$RELEASE_GROUP" "$RELEASE_SOURCE" "$2" "$attempt/plan.json" "$3"
export PLATFORM_VALIDATION_ATTEMPT="$attempt"
VALIDATION_REPOSITORY_ROOT="$PWD" VALIDATION_LOG_DIR="$attempt/logs" bash vendor/toko-tooling/scripts/ci/run-validation-targets.sh --plan "$attempt/plan.json" > "$attempt/runner.log" 2>&1 || {
    cat "$attempt/runner.log" >&2
    exit 1
}
printf 'Platform runner completed status=0\n' >> "$attempt/runner.log"
set -- "$attempt"/logs/validation.*
[[ $# == 1 && -d "$1" ]] || exit 1
perl scripts/release/validation-receipt.pl "$attempt/plan.json" "$1" "$attempt/receipt.json"
for admission in "$attempt"/stage-admissions/*.json; do
    [[ -f "$admission" ]] || continue
    jq -r '"Validation leaf \(.stage): \(.mode), \(.elapsed_seconds)s admission; complete parent gate retained." + (if .mode=="executed" then " No valid exact-input proof matched this closure/tools/environment." else " Original execution and logs retained." end)' "$admission"
done
