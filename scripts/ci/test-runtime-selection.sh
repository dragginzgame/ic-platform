#!/usr/bin/env bash
# Selection only: no Cargo resolution, tools, writes or installed runtime.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
# shellcheck source=scripts/ci/runtime-selection.sh
source "$root/scripts/ci/runtime-selection.sh"
selection() {
    runtime_selection "$@"
    printf '%s/%s/%s/%s/%s/%s\n' "$qualification" "$selection_mode" "$transport_mode" "$adapter_mode" "$published" "$mode"
}
[[ "$(selection)" == owner/source/raw/external//run ]]
[[ "$(selection --timers --prepared-call --check-graph)" == timers/source/prepared/external//--check-graph ]]
[[ "$(selection --check-graph --prepared-call --timers)" == "$(selection --timers --prepared-call --check-graph)" ]]
[[ "$(selection --published 0.20.2 --check-graph)" == owner/published/raw/raw/0.20.2/--check-graph ]]
[[ "$(selection --external-adapters --published 0.20.2 --timers --build-only)" == timers/published/raw/external/0.20.2/--build-only ]]
[[ "$(selection --build-only --timers --published 0.20.2 --external-adapters)" == "$(selection --external-adapters --published 0.20.2 --timers --build-only)" ]]
[[ "$(selection --timers --refresh-lock)" == timers/source/raw/external//--refresh-lock ]]
refuse() {
    local result=0
    (runtime_selection "$@") > /dev/null 2>&1 || result=$?
    [[ "$result" == 2 ]] || { echo "Unexpected runtime admission: $* ($result)" >&2; exit 1; }
}
refuse --published
refuse --published 01.2.3
refuse --published 1.2
refuse --prepared-call --published 0.20.2
refuse --published 0.20.2 --prepared-call
refuse --published 0.20.2 --prepare
refuse --prepared-call --refresh-lock
refuse --published 0.20.2 --refresh-lock
refuse --timers --timers
refuse run --check-graph
refuse --unknown
echo 'Runtime selections preserve profiles, accept reordered options and refuse conflicting operations.'
