#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
[[ $# -le 1 ]] || exit 2
source_root="${1:-$root/crates/ic-platform/src/management_canister_types/maintained}"
manifest="$root/crates/ic-platform/src/management_canister_types/maintained/provenance.json"
mkdir -p "$root/target"
scratch="$(mktemp -d "$root/target/management-source.XXXXXX")"
trap 'rm -rf "$scratch"' EXIT
perl -0777 -e '
    use strict; use warnings;
    local $/; my $source = <>;
    my $current = qq{    #![expect(\n        missing_docs,\n        reason = "The upstream function-reference macro emits undocumented members."\n    )]};
    $source =~ s/\Q$current\E/    #![allow(missing_docs)]/ == 1
        or die "unexpected management callback lint scaffolding\n";
    my $original = qq{#[allow(\n    clippy::upper_case_acronyms,\n    reason = "Retain upstream HTTP method names"\n)]\npub enum HttpMethod};
    $source =~ s/pub enum HttpMethod/$original/ == 1
        or die "missing management HTTP method declaration\n";
    print $source;
' "$source_root/mod.rs" > "$scratch/mod.rs"
for entry in 'maintained_source_sha256 mod.rs' 'license_sha256 LICENSE-APACHE'; do
    read -r field file <<< "$entry"
    checksum="$("$root/.tools/host/bin/jq" -er ".$field" "$manifest")"
    selected="$source_root/$file"
    [[ "$file" != mod.rs ]] || selected="$scratch/mod.rs"
    bash "$root/scripts/ci/verify-file-checksum.sh" sha256 "$checksum" "$selected"
done
printf 'Selected management DTO source and Apache license match the reviewed 0.11.0 import.\n'
