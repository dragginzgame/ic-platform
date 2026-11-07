#!/usr/bin/env bash
# Consumer group selection and required metadata custody; never mutates versions.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
[[ $# == 1 ]] || exit 2
group="${RELEASE_GROUP:-}"
helper="$root/scripts/ci/cargo-release-group.sh"
case "$group" in platform|runtime|types) ;; *) echo "Select RELEASE_GROUP=platform, RELEASE_GROUP=runtime or RELEASE_GROUP=types (received: $group)" >&2; exit 2 ;; esac
case "$1" in
    version) exec bash "$helper" version "$group" ;;
    plan|primary|changelog|members|files|ready) ;;
    *) exit 2 ;;
esac
# One admitted observation per command; do not resolve the same graph again to
# project adjacent fields. Later release phases still make fresh observations.
plan="$(bash "$helper" plan "$group")"
required="$(jq -cn --arg group "$group" '["scripts/fixtures/cdk-runtime/Cargo.lock", "scripts/fixtures/timers-runtime/Cargo.lock"] +
    (if $group == "runtime" then ["ci/dependency-pinning-exceptions.json"] else [] end)')"
case "$1" in
    primary|changelog|members)
        case "$1" in
            primary|changelog) printf '%s\n' "$plan" | jq -er --arg key "$1" '.[$key]' ;;
            members) printf '%s\n' "$plan" | jq -er '.members | join(" ")' ;;
        esac ;;
    files)
        jq -jr --argjson required "$required" '(.files + ($required - .files))[] | . + "\u0000"' <<< "$plan"
        ;;
    plan|ready)
        if ! jq -e --argjson required "$required" '($required - .files) | length == 0' <<< "$plan" > /dev/null; then
            echo 'release refused: Shared group custody must also admit the required runtime fixture locks and exact guard-pin exception; their checks cannot be omitted' >&2
            exit 1
        fi
        if [[ "$1" == plan ]]; then printf '%s\n' "$plan"; exit 0; fi
        mkdir -p target
        scratch="$(mktemp -d "$PWD/target/platform-group-readiness.XXXXXX")"
        trap 'rm -rf "$scratch"' EXIT
        # shellcheck source=ci/release-editor.env
        source ci/release-editor.env
        editor="$(bash scripts/release/editor.sh check)"
        if ! "$editor" --max-input-bytes "$PLATFORM_RELEASE_EDITOR_MAX_BYTES" group-manifest --help > "$scratch/editor" 2>&1; then
            echo 'release refused: the selected IC Build editor lacks group-manifest; a published group-capable editor must be adopted before grouped release preparation' >&2
            exit 1
        fi
        ;;
esac
