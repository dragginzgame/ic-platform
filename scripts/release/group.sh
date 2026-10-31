#!/usr/bin/env bash
# Consumer group selection and required metadata custody; never mutates versions.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
[[ $# == 1 ]] || exit 2
group="${RELEASE_GROUP:-platform}"
helper="$root/scripts/ci/cargo-release-group.sh"
case "$group" in platform|types) ;; *) echo "Unknown Platform release group: $group" >&2; exit 2 ;; esac
case "$1" in
    version) exec bash "$helper" version "$group" ;;
    primary|changelog|members)
        plan="$(bash "$helper" plan "$group")"
        case "$1" in
            primary|changelog) printf '%s\n' "$plan" | jq -er --arg key "$1" '.[$key]' ;;
            members) printf '%s\n' "$plan" | jq -er '.members | join(" ")' ;;
        esac ;;
    files)
        declared="$(mktemp "${TMPDIR:-/tmp}/platform-group-files.XXXXXX")"
        trap 'rm -f "$declared"' EXIT
        bash "$helper" files "$group" > "$declared"
        has_lock=false has_pin=false
        while IFS= read -r -d '' path; do
            printf '%s\0' "$path"
            case "$path" in
                scripts/fixtures/cdk-runtime/Cargo.lock) has_lock=true ;;
                ci/dependency-pinning-exceptions.json) has_pin=true ;;
            esac
        done < "$declared"
        if [[ "$group" == platform ]]; then
            [[ "$has_lock" == true ]] || printf 'scripts/fixtures/cdk-runtime/Cargo.lock\0'
            [[ "$has_pin" == true ]] || printf 'ci/dependency-pinning-exceptions.json\0'
        fi ;;
    ready)
        scratch="$(mktemp -d "${TMPDIR:-/tmp}/platform-group-readiness.XXXXXX")"
        trap 'rm -rf "$scratch"' EXIT
        ready=true
        bash "$helper" files "$group" > "$scratch/declared"
        bash "$0" files > "$scratch/required"
        if ! cmp -s "$scratch/declared" "$scratch/required"; then
            echo 'release refused: Shared group custody must also admit the required runtime fixture lock and exact guard-pin exception; their checks cannot be omitted' >&2
            ready=false
        fi
        # shellcheck source=ci/release-editor.env
        source ci/release-editor.env
        editor="$(bash scripts/release/editor.sh check)"
        if ! "$editor" --max-input-bytes "$PLATFORM_RELEASE_EDITOR_MAX_BYTES" group-manifest --help > "$scratch/editor" 2>&1; then
            echo 'release refused: the selected IC Build editor lacks group-manifest; a published group-capable editor must be adopted before grouped release preparation' >&2
            ready=false
        fi
        [[ "$ready" == true ]] ;;
    *) exit 2 ;;
esac
