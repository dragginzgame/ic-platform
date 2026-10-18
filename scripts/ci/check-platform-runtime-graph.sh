#!/usr/bin/env bash
set -euo pipefail
# Input: cargo tree --edges normal,build --prefix none for the canister package.
[[ $# -eq 1 && -s "$1" ]] || { echo 'Expected a nonempty canister graph' >&2; exit 2; }
graph="$1"
if grep -Eq '^(candid_parser|ic-agent|pocket-ic|ic-testkit|ic-testkit-candid) ' "$graph"; then
    echo "Canister graph contains host parsing, Agent or Testkit: $graph" >&2
    exit 1
fi
if ! awk '
    $1 ~ /^ic-cdk/ && $1 != "ic-cdk-executor" {
        if (($1 != "ic-cdk" && $1 != "ic-cdk-macros") || $2 != "v0.20.3") exit 1
    }
' "$graph"; then
    echo "Unqualified CDK runtime: $graph" >&2
    exit 1
fi
identities="$(awk '$1 == "ic-cdk-executor" || $1 == "ic0"' "$graph" | sed 's/ (\*)$//' | LC_ALL=C sort -u)"
[[ "$identities" == $'ic-cdk-executor v2.0.0\nic0 v1.2.0' ]] || {
    echo "Unqualified runtime identities: $identities" >&2; exit 1;
}
