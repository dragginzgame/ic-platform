#!/usr/bin/env bash
# Exercise the actual read-only capture without Canic, tools, registry or builds.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
mkdir -p "$root/target"
fixture="$(mktemp -d "$root/target/canic-capture-test.XXXXXX")"
trap 'printf "Capture evidence: %s\n" "$fixture"' EXIT
mkdir "$fixture/input"
git -C "$fixture/input" init -q
printf 'tracked source\n' > "$fixture/input/README.md"
git -C "$fixture/input" add README.md
git -C "$fixture/input" -c user.name='Capture fixture' -c user.email=fixture@example.invalid commit -qm baseline
printf 'untracked source\n' > "$fixture/input/NOTES.md"
capture="$root/scripts/benchmarks/canic-wasm/capture.cjs"
node "$capture" "$fixture/input" "$fixture/good" > "$fixture/positive.log"
cmp "$fixture/input/README.md" "$fixture/good/source/README.md"
cmp "$fixture/input/NOTES.md" "$fixture/good/source/NOTES.md"
git -C "$fixture/input" diff --exit-code
[[ "$(git -C "$fixture/input" status --porcelain)" == '?? NOTES.md' ]]
if node "$capture" "$fixture/input" "$fixture/good" > "$fixture/occupied.log" 2>&1; then
    echo 'Capture overwrote an existing capture' >&2; exit 1
fi
grep -q 'Refuse to overwrite an existing source capture' "$fixture/occupied.log"
mkdir "$fixture/stale"
cp "$fixture/good/source-inventory.json" "$fixture/stale/"
if node "$capture" "$fixture/input" "$fixture/stale" > "$fixture/stale.log" 2>&1; then
    echo 'Capture reused an output with a retained old success manifest' >&2; exit 1
fi
grep -q 'Refuse to overwrite an existing source capture' "$fixture/stale.log"
if node "$capture" "$fixture/input" "$fixture/invalid"$'\npath' > "$fixture/path.log" 2>&1; then
    echo 'Capture admitted a line-break directory' >&2; exit 1
fi
grep -q 'must not contain line breaks' "$fixture/path.log"
ln -s README.md "$fixture/input/link"
if node "$capture" "$fixture/input" "$fixture/linked" > "$fixture/link.log" 2>&1; then
    echo 'Capture admitted a source symlink' >&2; exit 1
fi
grep -q 'requires regular files, not symlinks' "$fixture/link.log"
echo 'Capture retains tracked/untracked bytes, leaves inputs unchanged and refuses invalid paths/links.'
