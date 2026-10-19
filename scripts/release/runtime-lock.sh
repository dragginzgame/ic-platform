#!/usr/bin/env bash
set -euo pipefail
# Keep only the two local package identities coordinated; never resolve dependencies.
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
export YQ="$root/.tools/host/bin/yq"
export PATH="$root/.tools/host/bin:$PATH"
mode="${1:-check}"
case "$mode" in check) [[ $# -eq 1 ]] ;; prepare) [[ $# -eq 2 ]] ;; *) exit 2 ;; esac
lock=scripts/fixtures/cdk-runtime/Cargo.lock
version="$(bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml)"
observed="$("$root/.tools/host/bin/yq" -p toml -o json '.' "$lock" |
    "$root/.tools/host/bin/jq" -er '
      [.package[] | select(.source == null and (.name == "ic-platform" or .name == "ic-platform-macros"))] as $p |
      if ($p | length) == 2 and ($p | map(.name) | unique | length) == 2 and
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
    "$editor" --max-input-bytes "$PLATFORM_RELEASE_EDITOR_MAX_BYTES" lock "$lock" "$observed" "$version" \
        ic-platform ic-platform-macros > "$temporary"
    cat "$temporary" > "$lock"
fi
