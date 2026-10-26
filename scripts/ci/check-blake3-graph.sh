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
selection=(--package "$package" --target "$target" --no-default-features)
policy=(--consumer "$consumer" "${selection[@]}" --max-count blake3=1 --json --explain)
if [[ "$features" != - ]]; then
    selection+=(--features "$features")
    policy+=(--features "$features")
fi
if ! perl "$root/scripts/ci/check-cargo-graph.pl" "${policy[@]}" > "$evidence/policy.json"; then
    cat "$evidence/policy.json" >&2
    exit 1
fi
cargo tree --manifest-path "$consumer/Cargo.toml" --locked --offline "${selection[@]}" \
    --edges normal,build --prefix none --format '{p}' > "$evidence/packages.txt"
if grep -q '^blake3 ' "$evidence/packages.txt"; then
    cargo tree --manifest-path "$consumer/Cargo.toml" --locked --offline "${selection[@]}" \
        --edges normal,build --invert blake3 --depth 0 --prefix none --format '{f}' > "$evidence/features.txt"
    # Inspect the effective feature union, including command-line/root selections.
    [[ "$(cat "$evidence/features.txt")" == std ]] || {
        echo 'BLAKE3 must select only std in this qualified graph; observed:' >&2
        cat "$evidence/features.txt" >&2
        exit 1
    }
fi
echo 'BLAKE3 identity/feature policy passed (normal/build graph; no size claim)'
