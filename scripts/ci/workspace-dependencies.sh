#!/usr/bin/env bash
# Read the root catalog once; preserve complete dependency tables in fixtures.
platform_dependencies_read() {
    # Archived catalogs contain source, not an installed tool bundle. Use the
    # invoking helper's admitted tools while reading the selected source bytes.
    platform_dependency_tools="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)/.tools/host/bin"
    platform_dependencies="$("$platform_dependency_tools/yq" -p toml -o json '.workspace.dependencies' "$1/Cargo.toml")"
}
platform_dependency() {
    # shellcheck disable=SC2016 # $name is a jq binding.
    "$platform_dependency_tools/jq" -er --arg name "$1" '
        .[$name] | if type == "string" then {version: .}
        elif type == "object" then . else error("missing dependency: " + $name) end |
        to_entries | map("\(.key) = \(.value | tojson)") | join(", ")' <<< "$platform_dependencies"
}
platform_dependency_version() {
    # shellcheck disable=SC2016 # $name is a jq binding.
    "$platform_dependency_tools/jq" -er --arg name "$1" '
        .[$name] | if type == "object" then .version else . end |
        if type == "string" and length > 0 then . else error("missing dependency version: " + $name) end' <<< "$platform_dependencies"
}
