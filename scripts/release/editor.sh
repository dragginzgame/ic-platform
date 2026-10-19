#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
[[ $# == 1 ]] || exit 2
case "$1" in install|check) ;; *) echo 'usage: editor.sh install|check' >&2; exit 2 ;; esac
# shellcheck source=ci/release-editor.env
source "$root/ci/release-editor.env"
arguments=(--consumer "$root" --package ic-build --version "$PLATFORM_RELEASE_EDITOR_VERSION"
    --bin ic-build-release-metadata --profile debug)
[[ "$1" != check ]] || arguments+=(--check)
exec bash "$root/scripts/dev/install-rust-tools.sh" "${arguments[@]}"
