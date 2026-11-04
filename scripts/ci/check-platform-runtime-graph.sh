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
allow_signature_ic0=0
if grep -Eq '^ic-canister-sig-creation v1\.3\.1( \(\*\))?$' "$graph"; then
    if ! awk '$1 == "ic0" && $0 !~ /^ic0 v1\.2\.0( \(\*\))?$/ { exit 1 }' "$graph"; then
        echo "Unqualified signature IC0 supplier: $graph" >&2; exit 1
    fi
    allow_signature_ic0=1
fi
if awk -v family=retired-cdk -v allow_signature_ic0="$allow_signature_ic0" -f "$policy" "$graph"; then
    echo "Retired full CDK runtime/generator: $graph" >&2; exit 1
fi
identities="$(awk '$1 == "ic-platform-runtime"' "$graph" | sed 's/ (\*)$//' | LC_ALL=C sort -u)"
[[ -n "$identities" && "$(printf '%s\n' "$identities" | wc -l | tr -d ' ')" == 1 ]] || {
    echo "Runtime must have one canonical Runtime executor owner: $identities" >&2; exit 1;
}
