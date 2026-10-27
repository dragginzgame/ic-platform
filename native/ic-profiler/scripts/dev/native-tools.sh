#!/usr/bin/env bash
set -euo pipefail

[[ $# == 1 && ( "$1" == install || "$1" == check ) ]] || {
    printf 'usage: native-tools.sh install|check\n' >&2; exit 2;
}
if [[ "$1" == install ]]; then
    if command -v apt-get >/dev/null && command -v dpkg-query >/dev/null; then
        # Native Cargo prerequisites from the pinned IC container, plus CMake
        # for its locked native dependencies. OS repositories own resolution.
        packages=(build-essential pkg-config protobuf-compiler clang cmake libclang-dev libunwind-dev lld)
        missing=()
        for package in "${packages[@]}"; do
            status=0
            state="$(dpkg-query -W -f='${db:Status-Status}' "$package" 2>/dev/null)" || status=$?
            if (( status > 1 )); then
                printf 'Cannot inspect installed native package: %s\n' "$package" >&2
                exit "$status"
            fi
            if [[ "$state" != installed ]]; then
                missing+=("$package")
            fi
        done
        if [[ ${#missing[@]} != 0 ]]; then
            privilege=()
            if [[ "$(id -u)" != 0 ]]; then
                command -v sudo >/dev/null || {
                    printf 'Native setup needs root or sudo to install: %s\n' "${missing[*]}" >&2; exit 1;
                }
                privilege=(sudo)
            fi
            "${privilege[@]}" apt-get install --yes --no-install-recommends "${missing[@]}"
        fi
    else
        printf 'Automatic native package setup supports Debian/Ubuntu; checking the provided environment.\n'
        printf 'See docs/integration.md for other hosts.\n'
    fi
fi

# prost-build honors PROTOC as an explicit executable selection; do not fall
# back to PATH when that selection is empty, missing or broken.
compiler="${PROTOC-protoc}"
if ! version="$("$compiler" --version 2>&1)"; then
    printf 'Native build prerequisite unavailable: %s\n%s\n' "$compiler" "$version" >&2
    printf 'Run make install-native-tools (Debian/Ubuntu) or set PROTOC to a working protoc executable.\nSee docs/integration.md for the pinned IC native environment.\n' >&2
    exit 1
fi
case "$version" in
    'libprotoc '*) printf 'Native protobuf compiler: %s (%s)\n' "$version" "$compiler" ;;
    *) printf 'Native build prerequisite returned an unexpected protoc version: %s\n' "$version" >&2; exit 1 ;;
esac
