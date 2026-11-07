#!/usr/bin/env bash
set -euo pipefail
# Preserve the imported ABI, reviewed executor changes and complete license terms.
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
[[ $# -le 1 ]] || exit 2
source_root="${1:-$root/crates/ic-platform-runtime/src}"
jq="$root/.tools/host/bin/jq"
mkdir -p "$root/target"
scratch="$(mktemp -d "$root/target/embedded-source.XXXXXX")"
trap 'rm -rf "$scratch"' EXIT
for module in ic0/upstream executor/runtime; do
    manifest="$root/crates/ic-platform-runtime/src/$module/provenance.json"
    # Capture the complete observation first: process substitution hides reader
    # failures, including a valid prefix followed by failure. Empty rosters refuse.
    sources="$("$jq" -er '.maintained_sources | to_entries[] | [.key, .value] | @tsv' "$manifest")"
    while IFS=$'\t' read -r file checksum; do
        selected="$source_root/$module/$file"
        if [[ "$module/$file" == ic0/upstream/sys/mod.rs ]]; then
            # Only the exact native lint scaffolding changes; preserve the
            # original ABI/function-body fingerprint rather than refreshing it.
            perl -0777 -e '
                use strict; use warnings;
                local $/; my $source = <>;
                my $current = qq{#[expect(\n    unused_variables,\n    reason = "Native IC0 stubs trap before reading the unchanged ABI arguments."\n)]\n#[expect(\n    clippy::too_many_arguments,\n    reason = "The imported System API argument lists are fixed by the IC ABI."\n)]};
                my $original = qq{#[allow(unused_variables)]\n#[allow(clippy::missing_safety_doc)]\n#[allow(clippy::too_many_arguments)]};
                $source =~ s/\Q$current\E/$original/ == 1
                    or die "unexpected native IC0 lint scaffolding\n";
                print $source;
            ' "$selected" > "$scratch/sys.rs"
            selected="$scratch/sys.rs"
        fi
        bash "$root/scripts/ci/verify-file-checksum.sh" sha256 "$checksum" "$selected"
    done <<< "$sources"
    license="$("$jq" -er '.license_sha256' "$manifest")"
    bash "$root/scripts/ci/verify-file-checksum.sh" sha256 "$license" "$source_root/$module/LICENSE-APACHE"
done
echo 'Embedded IC0 ABI, maintained executor and licenses match reviewed sources.'
