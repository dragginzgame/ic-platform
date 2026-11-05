#!/usr/bin/env bash
# Consumer policy proof: real external DTOs remain admitted, new application
# declarations and stale allowances refuse. Parser fixtures are not compiled.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
mkdir -p "$root/target"
fixture="$(mktemp -d "$root/target/platform-candid-boundary.XXXXXX")"
trap 'printf "Candid policy evidence: %s\n" "$fixture"' EXIT
source_path=crates/ic-platform/src/certified_assets_types/mod.rs
mkdir -p "$fixture/$(dirname "$source_path")"
cp "$root/$source_path" "$fixture/$source_path"
jq --arg path "$source_path" '{schema,allowances:[.allowances[] | select(.path==$path)]}' \
    "$root/ci/candid-boundary.json" > "$fixture/policy.json"
check=(perl "$root/scripts/ci/check-candid-boundary.pl" "$fixture" policy.json "$source_path")
"${check[@]}"
printf '\n#[derive(CandidType)] struct ApplicationCanary;\n' >> "$fixture/$source_path"
if "${check[@]}" > "$fixture/unreviewed.log" 2>&1; then
    echo 'Unreviewed application Candid declaration was admitted' >&2; exit 1
fi
cp "$root/$source_path" "$fixture/$source_path"
sed 's/pub struct GetArgs/pub struct RenamedGetArgs/' "$fixture/$source_path" > "$fixture/changed.rs"
cp "$fixture/changed.rs" "$fixture/$source_path"
if "${check[@]}" > "$fixture/stale.log" 2>&1; then
    echo 'Stale external allowance was admitted' >&2; exit 1
fi
echo 'Platform Candid policy admits external assets and refuses new/stale declarations.'
