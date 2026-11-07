#!/usr/bin/env bash
set -euo pipefail
# Only the selected group's recorded local identities change; never resolve here.
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
export PATH="$root/.tools/host/bin:$PATH"
mode="${1:-check}"
case "$mode" in check) [[ $# == 1 ]] ;; prepare) [[ $# == 2 ]] ;; *) exit 2 ;; esac
group="${RELEASE_GROUP:-}"
if [[ "$mode" == prepare ]]; then
    case "$group" in platform|runtime|types) ;; *) echo 'Select a release group' >&2; exit 2 ;; esac
fi
previous="${2:-}"
# Finish the roster observation before checking or preparing either lock. A
# process substitution hides a failed (or partially emitted) producer from set -e.
roster="$(jq -er '.groups[] | [.name,.primary,(.members|join(" "))] | @tsv' ci/cargo-release-groups.json)"
for lock in scripts/fixtures/{cdk-runtime,timers-runtime}/Cargo.lock; do
    records="$(yq -p toml -o json '.' "$lock" | jq -c '[.package[] | select(.source == null)]')"
    while IFS=$'\t' read -r owner primary members; do
        version="$(yq -p toml -o json -r '.package.version' "crates/$primary/Cargo.toml")"
        packages=()
        for package in $members; do
            observed="$(jq -r --arg name "$package" '[.[] | select(.name == $name)] | if length > 1 then error("duplicate owner") else .[0].version // empty end' <<< "$records")"
            [[ -n "$observed" ]] || { echo "$lock: missing local $package qualification identity" >&2; exit 1; }
            if [[ "$observed" != "$version" ]]; then
                if [[ "$mode" != prepare || "$group" != "$owner" || "$observed" != "$previous" ]]; then
                    echo "$lock: $package is $observed; $owner requires $version" >&2; exit 1
                fi
                packages+=("$package")
            fi
        done
        if [[ ${#packages[@]} != 0 ]]; then
            temporary="$(mktemp "$root/$lock.XXXXXX")"
            trap 'rm -f "$temporary"' EXIT
            source "$root/ci/release-editor.env"
            editor="${CARGO_RELEASE_METADATA_EDITOR:-}"
            [[ -n "$editor" ]] || editor="$(bash scripts/release/editor.sh check)"
            "$editor" --max-input-bytes "$PLATFORM_RELEASE_EDITOR_MAX_BYTES" lock "$lock" "$previous" "$version" "${packages[@]}" > "$temporary"
            cat "$temporary" > "$lock"
            rm -f "$temporary"
        fi
    done <<< "$roster"
done
