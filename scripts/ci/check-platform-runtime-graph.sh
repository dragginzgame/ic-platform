#!/usr/bin/env bash
set -euo pipefail
# Input: cargo tree --edges normal,build --prefix none for the canister package.
[[ $# -ge 1 && $# -le 3 && -s "$1" ]] || { echo 'Expected a nonempty canister graph [--external-candid [--native-management]]' >&2; exit 2; }
graph="$1"
native_management=0
if [[ $# == 3 ]]; then
    [[ "$2" == --external-candid && "$3" == --native-management ]] || exit 2
    native_management=1
fi
policy="$(cd "$(dirname "$0")" && pwd -P)/platform-graph-policy.awk"
case "${2:-}" in
    '') runtime_mode=wire ;;
    --external-candid) runtime_mode=external ;;
    *) echo 'Unknown runtime graph mode' >&2; exit 2 ;;
esac
exec awk -v runtime_mode="$runtime_mode" -v allow_native_management="$native_management" -f "$policy" "$graph"
