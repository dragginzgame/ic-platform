#!/usr/bin/env bash
set -euo pipefail
# Coordinate the documented exact helper pin with the Platform group release.
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
mode="${1:-}"
case "$mode" in check) [[ $# -eq 1 ]] ;; prepare) [[ $# -eq 2 && "${RELEASE_GROUP:-}" == platform ]] ;; *) exit 2 ;; esac
file=ci/dependency-pinning-exceptions.json
export PATH="$root/.tools/host/bin:$PATH"
YQ="$root/.tools/host/bin/yq"
version="$("$YQ" -p toml -o json -r '.package.version' crates/ic-platform/Cargo.toml)"
observed="$(jq -er '[.[] | select(.rule == "cargo-exact" and .file == "Cargo.toml" and .subject == "ic-platform-runtime-guard")] |
    if length == 1 then .[0].value else error("expected one runtime guard pin exception") end' "$file")"
if [[ "$observed" != "=$version" ]]; then
    if [[ "$mode" != prepare || "$observed" != "=${2:-}" ]]; then
        echo "Runtime guard pin exception is $observed; Platform group is $version" >&2; exit 1
    fi
    temporary="$(mktemp "$root/$file.XXXXXX")"
    trap 'rm -f "$temporary"' EXIT
    jq --arg version "=$version" 'map(if .rule == "cargo-exact" and .file == "Cargo.toml" and .subject == "ic-platform-runtime-guard"
        then .value = $version else . end)' "$file" > "$temporary"
    cat "$temporary" > "$file"
fi
