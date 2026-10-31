#!/usr/bin/env bash
set -euo pipefail
# Coordinate recorded local workspace identities; never resolve dependencies.
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
export YQ="$root/.tools/host/bin/yq"
export PATH="$root/.tools/host/bin:$PATH"
mode="${1:-check}"
case "$mode" in check) [[ $# -eq 1 ]] ;; prepare) [[ $# -eq 2 ]] ;; *) exit 2 ;; esac
lock=scripts/fixtures/cdk-runtime/Cargo.lock
version="$(bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml)"
observed="$("$root/.tools/host/bin/yq" -p toml -o json '.' "$lock" |
    jq -er '
      [.package[] | select(.source == null and (.name == "ic-platform" or .name == "ic-platform-macros" or .name == "ic-platform-runtime-guard" or .name == "ic-platform-types"))] as $p |
      if (($p | map(.name) | sort) == ["ic-platform", "ic-platform-macros", "ic-platform-runtime-guard"] or
          ($p | map(.name) | sort) == ["ic-platform", "ic-platform-macros", "ic-platform-runtime-guard", "ic-platform-types"]) and
         ($p | map(.version) | unique | length) == 1
      then $p[0].version else error("invalid qualification package identities") end')"
if [[ "$observed" != "$version" ]]; then
    if [[ "$mode" != prepare || "$observed" != "${2:-}" ]]; then
        echo "Qualification lock is $observed; workspace is $version" >&2
        exit 1
    fi
    temporary="$(mktemp "$root/$lock.XXXXXX")"
    trap 'rm -f "$temporary"' EXIT
    source "$root/ci/release-editor.env"
    editor="$(bash "$root/scripts/release/editor.sh" check)"
    packages=(ic-platform ic-platform-macros ic-platform-runtime-guard)
    # Historical qualification locks predate the type extraction. If the actual
    # qualified graph records its local owner, advance that package as well.
    if "$YQ" -p toml -o json '.' "$lock" | jq -e 'any(.package[]; .source == null and .name == "ic-platform-types")' > /dev/null; then
        packages+=(ic-platform-types)
    fi
    "$editor" --max-input-bytes "$PLATFORM_RELEASE_EDITOR_MAX_BYTES" lock "$lock" "$observed" "$version" \
        "${packages[@]}" > "$temporary"
    cat "$temporary" > "$lock"
fi
