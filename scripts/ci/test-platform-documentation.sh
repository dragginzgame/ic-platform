#!/usr/bin/env bash
# Check the actual Make roster; mutate only a disposable exported document.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
mkdir -p "$root/target"
fixture="$(mktemp -d "$root/target/platform-doc-links.XXXXXX")"
trap 'printf "Documentation evidence: %s\n" "$fixture"' EXIT
mkdir "$fixture/source"
bash "$root/scripts/ci/copy-platform-fixture.sh" "$root" "$fixture/source" "$fixture/inputs"
make --no-print-directory -s -C "$fixture/source" check-doc-links > "$fixture/positive.log" 2>&1
printf '\n[Broken export fixture](__missing-exported-doc-target__.md)\n' >> "$fixture/source/docs/local-setup.md"
if make --no-print-directory -s -C "$fixture/source" check-doc-links > "$fixture/negative.log" 2>&1; then
    echo 'Platform documentation roster omitted an exported Shared document' >&2; exit 1
fi
grep -q 'docs/local-setup.md: missing local target __missing-exported-doc-target__.md' "$fixture/negative.log"
echo 'Actual Platform documentation gate covers exported Shared docs and refuses missing targets.'
