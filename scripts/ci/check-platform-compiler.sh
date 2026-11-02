#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
version="$(bash "$root/scripts/ci/read-platform-msrv.sh")"
development="$("$root/.tools/host/bin/yq" -p toml -o yaml -r '.toolchain.channel' rust-toolchain.toml)"
[[ "$development" == "$version" ]] || { echo 'Minimum and pinned development compiler must agree' >&2; exit 1; }
cargo_version="$(cargo --version)"
rustc_version="$("${RUSTC:-rustc}" --version)"
[[ "$cargo_version" == "cargo $version ("* && "$rustc_version" == "rustc $version ("* ]] || {
    printf 'Expected Cargo/rustc %s; observed %s / %s\n' "$version" "$cargo_version" "$rustc_version" >&2
    exit 1
}
printf '%s\n%s\n' "$cargo_version" "$rustc_version"
