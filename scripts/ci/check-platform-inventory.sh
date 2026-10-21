#!/usr/bin/env bash
# Reads local manifests only; no sibling scan, network, compilation or lock updates.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
mode="${1:---check}"
[[ $# -le 1 && ( "$mode" == --check || "$mode" == --print ) ]] || {
    echo 'usage: check-platform-inventory.sh [--check|--print]' >&2; exit 2;
}
scratch="$(mktemp -d "${TMPDIR:-/tmp}/platform-inventory.XXXXXX")"
trap 'rm -rf "$scratch"' EXIT
cargo metadata --manifest-path "$root/Cargo.toml" --no-deps --format-version 1 --locked --offline > "$scratch/metadata.json"
jq -r -f "$root/scripts/ci/platform-inventory.jq" "$scratch/metadata.json" > "$scratch/current"
if [[ "$mode" == --print ]]; then cat "$scratch/current"; exit; fi
perl -0777 -ne '
    my @blocks = /<!-- BEGIN MANIFEST INVENTORY -->\n(.*?)<!-- END MANIFEST INVENTORY -->/sg;
    @blocks == 1 or die "expected exactly one manifest inventory block\n";
    print $blocks[0];
' "$root/docs/ic-inventory.md" > "$scratch/documented"
diff -u "$scratch/documented" "$scratch/current" || {
    echo 'Inventory drift: review manifest changes and replace the marked block with --print output.' >&2
    exit 1
}
jq -r '.packages[] | select(.name == "ic-platform") | .features | keys[] | select(. != "default")' "$scratch/metadata.json" > "$scratch/features"
perl -ne 'print "$1\n" if /^\| `([^`]+)` \|/' "$root/README.md" | LC_ALL=C sort > "$scratch/readme-features"
diff -u "$scratch/features" "$scratch/readme-features" || {
    echo 'README feature table must cover each Platform feature exactly once.' >&2
    exit 1
}
echo 'Platform manifest inventory matches all workspace features and direct dependencies.'
