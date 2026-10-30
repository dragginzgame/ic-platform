#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
mode=routine
scope=""
case "${1:-}" in
    "") [[ $# == 0 ]] || exit 2 ;;
    --exhaustive)
        [[ $# == 2 && -n "$2" ]] || { echo 'Select 1–12 comma-separated features for --exhaustive' >&2; exit 2; }
        mode=exhaustive; scope="$2" ;;
    *) echo 'usage: check-platform-features.sh [--exhaustive FEATURE,...]' >&2; exit 2 ;;
esac
mkdir -p target/feature-graphs
cargo metadata --no-deps --format-version 1 --frozen > target/feature-graphs/metadata.json
"$root/.tools/host/bin/jq" -e --slurpfile policy scripts/ci/platform-feature-policy.json \
    -f scripts/ci/platform-feature-contract.jq target/feature-graphs/metadata.json >/dev/null || exit 1

generator_version="$("$root/.tools/host/bin/jq" -r '.packages[] | select(.name == "ic-platform") | .version' target/feature-graphs/metadata.json)"
# Admit the manifest's immutable Host supplier, including the reviewed process
# adoption source. Paths, branches and foreign Git sources are not that contract.
gzip_source="$("$root/.tools/host/bin/jq" -er '
    .packages[] | select(.name == "ic-platform") | .dependencies[] |
    select(.name == "ic-host-artifacts") | .source |
    if . == "registry+https://github.com/rust-lang/crates.io-index" then ""
    else capture("^git\\+(?<url>https://github.com/dragginzgame/ic-tooling\\.git\\?rev=(?<revision>[0-9a-f]{40}))$") |
        "(\(.url)#\(.revision[0:8]))"
    end' target/feature-graphs/metadata.json)"

# Cargo tree prints canonical package names even for renamed deps.
# Cache only within this invocation; never reuse graphs from a previous lockfile.
cache="$(mktemp -d "$root/target/feature-graphs/attempt.XXXXXX")"
"$root/.tools/host/bin/jq" '.packages[] | select(.name == "ic-platform") | .features | keys - ["default"]' target/feature-graphs/metadata.json |
    "$root/.tools/host/bin/jq" --arg scope "$scope" '
        . as $known | if $scope == "" then . else
        ($scope | split(",")) as $selected |
        if ($selected | length) > 12 or any($selected[]; . as $name | . == "" or ($known | index($name)) == null)
        then error("select 1–12 known feature names") else $selected | unique end end' |
    "$root/.tools/host/bin/jq" --arg mode "$mode" --slurpfile policy scripts/ci/platform-feature-policy.json \
        -f scripts/ci/platform-feature-selections.jq > "$cache/selections.json"
selection_count="$("$root/.tools/host/bin/jq" length "$cache/selections.json")"
printf 'Feature matrix mode: %s\n' "$mode"
for target in "$(rustc -vV | sed -n 's/^host: //p')" wasm32-unknown-unknown; do
    mapping="$cache/$target.tsv"
    "$root/.tools/host/bin/jq" -r --arg target "$target" --slurpfile metadata target/feature-graphs/metadata.json \
        --slurpfile policy scripts/ci/platform-feature-policy.json \
        -f scripts/ci/platform-feature-closures.jq "$cache/selections.json" > "$cache/$target-closures.tsv"
    "$root/.tools/host/bin/jq" -r --arg target "$target" --arg projection activation --slurpfile metadata target/feature-graphs/metadata.json \
        --slurpfile policy scripts/ci/platform-feature-policy.json \
        -f scripts/ci/platform-feature-closures.jq "$cache/selections.json" > "$cache/$target-activation.tsv"
    awk -f scripts/ci/platform-feature-cache-keys.awk "$cache/$target-closures.tsv" > "$mapping"
    # Resolve/download once during setup, not in every exhaustive graph query.
    # Keep all selection admissions, including those sharing an equivalent key.
    queries="$(cut -f2 "$mapping" | LC_ALL=C sort -u | wc -l | tr -d ' ')"
    completed=0
    printf 'Feature graphs: %s, %s unique offline queries (%s selections). Evidence: %s\n' \
        "$target" "$queries" "$selection_count" "$cache"
    while IFS=$'\t' read -r selection key; do
        cached="$cache/$target-$key.txt"
        if [[ -f "$cached" ]]; then continue; fi
        args=(tree -p ic-platform --frozen --target "$target" --edges "normal,build" --prefix none)
        if [[ "$selection" != default ]]; then args+=(--no-default-features); fi
        if [[ "$selection" != none && "$selection" != default ]]; then args+=(--features "$selection"); fi
        if ! cargo "${args[@]}" > "$cached"; then
            printf 'Feature query failed: %s / %s. If registry inputs are missing, run cargo fetch --locked --target %s before retrying.\n' "$target" "$selection" "$target" >&2
            exit 1
        fi
        completed=$((completed + 1))
        if (( completed % 100 == 0 || completed == queries )); then
            printf 'Feature graphs: %s %s/%s complete\n' "$target" "$completed" "$queries"
        fi
    done < "$mapping"
    awk -v target="$target" -v cache="$cache" -v activation="$cache/$target-activation.tsv" \
        -v gzip_source="$gzip_source" \
        -v guard="ic-platform-runtime-guard v$generator_version ($root/crates/ic-platform-runtime-guard)" \
        -v generator="ic-platform-macros v$generator_version (proc-macro) ($root/crates/ic-platform-macros)" \
        -f scripts/ci/platform-graph-policy.awk -f scripts/ci/check-platform-feature-graphs.awk "$mapping"
done
printf 'Feature selections, mappings and Cargo graphs retained at %s\n' "$cache"
cargo check -p ic-platform --lib --frozen --no-default-features
cargo check -p ic-platform --lib --frozen --no-default-features --features executor --target wasm32-unknown-unknown
