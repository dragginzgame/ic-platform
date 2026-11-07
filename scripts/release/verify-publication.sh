#!/usr/bin/env bash
# Shared admits the exact publication selection before this verifier runs.
# Carry that selection to the existing Clippy leaf, retaining its flags/compiler.
set -euo pipefail
[[ $# == 2 && "$1" == --no-print-directory && "$2" == clippy ]] || exit 2
selection="${CARGO_PUBLISH_SELECTION:?}"
roster="${CARGO_PUBLISH_PACKAGES:?}"
[[ -n "$selection" && "$selection" != *$'\n'* ]] || exit 2
read -r -a packages <<< "$selection"
[[ ${#packages[@]} != 0 ]] || exit 2
for package in "${packages[@]}"; do
    [[ "$package" =~ ^[A-Za-z][A-Za-z0-9_-]*$ ]] || exit 2
    case " $roster " in
        *" $package "*) ;;
        *) echo 'Publication lint selection is outside the admitted group' >&2; exit 2 ;;
    esac
done
# Do not opt verifier Make into Shared's publication recipes a second time.
unset CARGO_PUBLISH_PACKAGES
exec make "$@" "PLATFORM_CLIPPY_PACKAGES=${packages[*]}"
