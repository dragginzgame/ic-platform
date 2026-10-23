#!/usr/bin/env bash
set -euo pipefail
root="${BASH_SOURCE[0]}"
[[ "$root" == /* ]] || root="$PWD/$root"
root="$(CDPATH='' cd -P "${root%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
[[ $# == 1 ]] || exit 2
case "$1" in install|check) ;; *) echo 'usage: editor.sh install|check' >&2; exit 2 ;; esac
# shellcheck source=ci/release-editor.env
source "$root/ci/release-editor.env"
arguments=(--consumer "$root" --package ic-build --version "$PLATFORM_RELEASE_EDITOR_VERSION"
    --bin ic-build-release-metadata --profile debug)
[[ "$1" != check ]] || arguments+=(--check)
exec bash "$root/scripts/dev/install-rust-tools.sh" "${arguments[@]}"
