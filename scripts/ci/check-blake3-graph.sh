#!/usr/bin/env bash
set -euo pipefail
# Explicit consumer package/target/features; no fetch, lock update or compilation.
[[ $# == 4 ]] || { echo 'usage: check-blake3-graph.sh CONSUMER PACKAGE TARGET FEATURES_OR_DASH' >&2; exit 2; }
root="${BASH_SOURCE[0]}"
[[ "$root" == /* ]] || root="$PWD/$root"
case "$root" in *$'\n'*|*$'\r'*) echo 'Script path contains CR/LF' >&2; exit 2 ;; esac
root="$(CDPATH='' cd -P "${root%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
case "$root" in *$'\n'*|*$'\r'*) echo 'Resolved root contains CR/LF' >&2; exit 2 ;; esac
consumer="$1" package="$2" target="$3" features="$4"
mkdir -p "$root/target"
evidence="$(mktemp -d "$root/target/blake3-graph.XXXXXX")"
printf 'BLAKE3 graph evidence: %s\n' "$evidence"
policy=(--consumer "$consumer" --package "$package" --target "$target" --no-default-features --max-count blake3=1 --json --explain --observations)
if [[ "$features" != - ]]; then
    policy+=(--features "$features")
fi
if ! perl "$root/scripts/ci/check-cargo-graph.pl" "${policy[@]}" > "$evidence/policy.json"; then
    cat "$evidence/policy.json" >&2
    exit 1
fi
jq="$root/.tools/host/bin/jq"
"$jq" -r '.observations[].identity' "$evidence/policy.json" > "$evidence/packages.txt"
"$jq" -r '.observations[] | select(.name == "blake3") | .features | join(",")' "$evidence/policy.json" > "$evidence/features.txt"
if grep -q '^blake3 ' "$evidence/packages.txt"; then
    [[ "$(cat "$evidence/features.txt")" == std ]] || {
        echo 'BLAKE3 must select only std in this qualified graph; observed:' >&2
        cat "$evidence/features.txt" >&2
        exit 1
    }
fi
# Optional caller evidence reuses the same complete selected graph.
if [[ -n "${BLAKE3_PACKAGES_OUTPUT:-}" ]]; then cp "$evidence/packages.txt" "$BLAKE3_PACKAGES_OUTPUT"; fi
echo 'BLAKE3 identity/feature policy passed (normal/build graph; no size claim)'
