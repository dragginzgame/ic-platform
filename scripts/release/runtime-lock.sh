#!/usr/bin/env bash
set -euo pipefail
# Update only the selected group's recorded local identities; never resolve a graph.
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
export YQ="$root/.tools/host/bin/yq"
export PATH="$root/.tools/host/bin:$PATH"
mode="${1:-check}"
case "$mode" in check) [[ $# -eq 1 ]] ;; prepare) [[ $# -eq 2 ]] ;; *) exit 2 ;; esac
group="${RELEASE_GROUP:-platform}"
case "$group" in platform|types) ;; *) exit 2 ;; esac
lock=scripts/fixtures/cdk-runtime/Cargo.lock
platform="$(bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml)"
types="$("$YQ" -p toml -o json -r '.package.version' crates/ic-platform-types/Cargo.toml)"
records="$("$YQ" -p toml -o json '.' "$lock" | jq -ce '
    [.package[] | select(.source == null and (.name == "ic-platform" or .name == "ic-platform-macros" or .name == "ic-platform-runtime-guard" or .name == "ic-platform-types"))] as $p |
    if (($p | map(.name) | sort) == ["ic-platform", "ic-platform-macros", "ic-platform-runtime-guard"] or
        ($p | map(.name) | sort) == ["ic-platform", "ic-platform-macros", "ic-platform-runtime-guard", "ic-platform-types"]) and
       ([$p[] | select(.name != "ic-platform-types") | .version] | unique | length) == 1
    then $p else error("invalid qualification package identities") end')"
observed_platform="$(jq -r '.[] | select(.name == "ic-platform") | .version' <<< "$records")"
observed_types="$(jq -r '.[] | select(.name == "ic-platform-types") | .version' <<< "$records")"
if [[ "$observed_platform" != "$platform" &&
      ( "$mode" != prepare || "$group" != platform || "$observed_platform" != "${2:-}" ) ]]; then
    echo "Qualification runtime is $observed_platform; workspace is $platform" >&2; exit 1
fi
if [[ -n "$observed_types" && "$observed_types" != "$types" &&
      ( "$mode" != prepare || "$group" != types || "$observed_types" != "${2:-}" ) ]]; then
    echo "Qualification types are $observed_types; type owner is $types" >&2; exit 1
fi
packages=()
if [[ "$group" == platform ]]; then
    observed="$observed_platform" version="$platform"
    packages=(ic-platform ic-platform-macros ic-platform-runtime-guard)
else
    observed="$observed_types" version="$types"
    # Historical qualification locks predate the type extraction. Preserve them;
    # adding the owner requires a separately qualified actual dependency graph.
    [[ -n "$observed" ]] || exit 0
    packages=(ic-platform-types)
fi
if [[ "$observed" != "$version" ]]; then
    temporary="$(mktemp "$root/$lock.XXXXXX")"
    trap 'rm -f "$temporary"' EXIT
    source "$root/ci/release-editor.env"
    editor="${CARGO_RELEASE_METADATA_EDITOR:-}"
    [[ -n "$editor" ]] || editor="$(bash "$root/scripts/release/editor.sh" check)"
    "$editor" --max-input-bytes "$PLATFORM_RELEASE_EDITOR_MAX_BYTES" lock "$lock" "$observed" "$version" \
        "${packages[@]}" > "$temporary"
    cat "$temporary" > "$lock"
fi
