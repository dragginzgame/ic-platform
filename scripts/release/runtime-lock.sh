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
# Finish the roster observation before checking or preparing the lock. A
# process substitution hides a failed (or partially emitted) producer from set -e.
groups="$(jq -er '.groups[] | [.name,.primary,(.members|join(" "))] | @tsv' ci/cargo-release-groups.json)"
# Observe each primary once, before the lock is prepared.
roster="$(while IFS=$'\t' read -r owner primary members; do
    version="$(yq -p toml -o json -r '.package.version' "crates/$primary/Cargo.toml")" || exit
    printf '%s\t%s\t%s\n' "$owner" "$version" "$members"
done <<< "$groups")"
lock=scripts/fixtures/cdk-runtime/Cargo.lock
records="$(yq -p toml -o json '.' "$lock" | jq -c '[.package[] | select(.source == null)]')"
# Admit and index every local identity once; duplicates never select a winner.
records="$(jq -ce 'reduce .[] as $p ({}; if has($p.name) then error("duplicate owner: " + $p.name) else .[$p.name] = $p.version end)' <<< "$records")"
while IFS=$'\t' read -r owner version members; do
    packages=()
    for package in $members; do
        observed="$(jq -r --arg name "$package" '.[$name] // empty' <<< "$records")"
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
