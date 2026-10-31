#!/usr/bin/env bash
# Read-only migration guard; Shared's runner retains all recovery authorization.
set -euo pipefail
cd "$(dirname "$0")/../.."
case "${RELEASE_GROUP:-}" in platform|types) ;; *) echo 'Select RELEASE_GROUP=platform or RELEASE_GROUP=types' >&2; exit 2 ;; esac
[[ $# == 1 ]] || exit 2
perl scripts/ci/validate-release-inputs.pl version "$1" > /dev/null
plan="$(git rev-parse --git-path release-state)/$RELEASE_GROUP-$1.plan"
[[ -f "$plan" && ! -L "$plan" ]] || { echo "Saved group release plan not found: $plan" >&2; exit 1; }
phase="$(sed -n '11p' "$plan")"
# Completed observations have no remaining release dispatch. Shared still checks
# their full record, source/tree, tag and destination before reconciliation.
[[ "$phase" != complete ]] || exit 0
source_commit="$(sed -n '6p' "$plan")"
[[ "$source_commit" =~ ^([0-9a-f]{40}|[0-9a-f]{64})$ ]] || exit 1
recorded="$(git rev-parse "$source_commit:ci/cargo-release-groups.json")"
current="$(git hash-object ci/cargo-release-groups.json)"
[[ "$recorded" == "$current" ]] || {
    echo 'Release policy changed since the saved attempt; recover using its recorded source and policy. The journal is retained.' >&2
    exit 1
}
