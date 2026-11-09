#!/usr/bin/env bash
set -euo pipefail
# Input: cargo tree --edges normal,build --prefix none for the canister package.
[[ $# -ge 1 && $# -le 2 && -s "$1" ]] || { echo 'Expected a nonempty canister graph [--external-candid]' >&2; exit 2; }
graph="$1"
policy="$(cd "$(dirname "$0")" && pwd -P)/platform-graph-policy.awk"
case "${2:-}" in
    '') runtime_mode=wire ;;
    --external-candid) runtime_mode=external ;;
    *) echo 'Unknown runtime graph mode' >&2; exit 2 ;;
esac
exec awk -v runtime_mode="$runtime_mode" -f "$policy" "$graph"
