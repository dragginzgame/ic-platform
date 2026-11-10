#!/usr/bin/env bash
# Check the actual Make roster using disposable native/baseline documents.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
mkdir -p "$root/target"
fixture="$(mktemp -d "$root/target/platform-doc-links.XXXXXX")"
trap 'printf "Documentation evidence: %s\n" "$fixture"' EXIT
mkdir "$fixture/source"
bash "$root/scripts/ci/copy-platform-fixture.sh" "$root" "$fixture/source" "$fixture/inputs"
make --no-print-directory -s -C "$fixture/source" check-doc-links > "$fixture/positive.log" 2>&1
for document in docs/development/qualification.md TOKO-TOOLING.md; do
    printf '\n[Broken fixture](__missing-doc-target__.md)\n' >> "$fixture/source/$document"
    if make --no-print-directory -s -C "$fixture/source" check-doc-links > "$fixture/negative.log" 2>&1; then
        echo "Platform documentation roster omitted $document" >&2; exit 1
    fi
    grep -Fq "$document: missing local target __missing-doc-target__.md" "$fixture/negative.log"
    cp "$root/$document" "$fixture/source/$document"
done
echo 'Actual documentation gate covers nested native guides and the installed baseline.'
