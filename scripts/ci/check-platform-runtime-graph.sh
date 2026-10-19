#!/usr/bin/env bash
set -euo pipefail
# Input: cargo tree --edges normal,build --prefix none for the canister package.
[[ $# -ge 1 && $# -le 2 && -s "$1" ]] || { echo 'Expected a nonempty canister graph [--external-candid]' >&2; exit 2; }
graph="$1"
case "${2:-}" in
    '')
        if grep -Eq '^(candid|candid_derive) ' "$graph"; then
            echo "Wire runtime acquired Candid: $graph" >&2; exit 1
        fi ;;
    --external-candid)
        grep -Eq '^candid ' "$graph" || { echo 'Missing selected external Candid adapter' >&2; exit 1; } ;;
    *) echo 'Unknown runtime graph mode' >&2; exit 2 ;;
esac
if grep -Eq '^(candid_parser|ic-agent|ic-secp256k1|pocket-ic|ic-testkit|ic-testkit-candid) ' "$graph"; then
    echo "Canister graph contains host parsing, Agent or Testkit: $graph" >&2; exit 1
fi
if ! awk '$1 ~ /^ic-cdk/ && $1 != "ic-cdk-executor" { exit 1 }' "$graph"; then
    echo "Retired full CDK runtime/generator: $graph" >&2; exit 1
fi
identities="$(awk '$1 == "ic-cdk-executor" || $1 == "ic0"' "$graph" | sed 's/ (\*)$//' | LC_ALL=C sort -u)"
[[ "$identities" == $'ic-cdk-executor v2.0.0\nic0 v1.2.0' ]] || {
    echo "Unqualified runtime identities: $identities" >&2; exit 1;
}
