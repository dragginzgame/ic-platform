#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
[[ $# -le 1 ]] || { echo 'usage: read-platform-msrv.sh [Cargo.toml]' >&2; exit 2; }
manifest="${1:-$root/Cargo.toml}"
CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 cargo locate-project --workspace \
    --message-format plain --manifest-path "$manifest" >/dev/null
msrv="$("$root/.tools/host/bin/yq" -p toml -o yaml -r '.workspace.package."rust-version"' "$manifest")"
stable='^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$'
[[ "$msrv" =~ $stable ]] || { echo 'Expected a pinned workspace.package.rust-version' >&2; exit 1; }
printf '%s\n' "$msrv"
