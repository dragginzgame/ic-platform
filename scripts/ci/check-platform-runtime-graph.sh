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
awk -f "$policy" /dev/null
case "${2:-}" in
    '')
        if grep -Eq '^(candid|candid_derive) ' "$graph"; then
            echo "Wire runtime acquired Candid: $graph" >&2; exit 1
        fi ;;
    --external-candid)
        grep -Eq '^candid ' "$graph" || { echo 'Missing selected external Candid adapter' >&2; exit 1; } ;;
    *) echo 'Unknown runtime graph mode' >&2; exit 2 ;;
esac
if awk -v family=runtime-host -v allow_native_management="$native_management" -f "$policy" "$graph"; then
    echo "Canister graph contains host parsing, Agent or Testkit: $graph" >&2; exit 1
fi
if awk -v family=retired-cdk -f "$policy" "$graph"; then
    echo "Retired full CDK runtime/generator: $graph" >&2; exit 1
fi
identities="$(awk '$1 == "ic-platform"' "$graph" | sed 's/ (\*)$//' | LC_ALL=C sort -u)"
[[ -n "$identities" && "$(printf '%s\n' "$identities" | wc -l | tr -d ' ')" == 1 ]] || {
    echo "Runtime must have one Platform executor owner: $identities" >&2; exit 1;
}
