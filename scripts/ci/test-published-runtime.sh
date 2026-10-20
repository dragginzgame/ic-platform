#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
jq="$root/.tools/host/bin/jq"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/platform-registry-admission.XXXXXX")"
trap 'rm -rf "$fixture"' EXIT
"$jq" -n '{packages: [
    {name: "ic-platform", version: "0.4.0", source: "registry+https://github.com/rust-lang/crates.io-index"},
    {name: "ic-platform-macros", version: "0.4.0", source: "registry+https://github.com/rust-lang/crates.io-index"}
]}' > "$fixture/good.json"
"$jq" -e --arg version 0.4.0 -f "$root/scripts/ci/published-runtime.jq" "$fixture/good.json" > /dev/null
for mutation in \
    '.packages[0].source = null' \
    '.packages[1].source = "git+https://example.invalid/generator"' \
    '.packages[0].version = "0.3.0"' \
    '.packages[1].version = "0.3.0"' \
    '.packages = []' \
    '.packages |= map(select(.name != "ic-platform-macros"))'; do
    "$jq" "$mutation" "$fixture/good.json" > "$fixture/bad.json"
    if "$jq" -e --arg version 0.4.0 -f "$root/scripts/ci/published-runtime.jq" "$fixture/bad.json" > /dev/null; then
        echo "Registry admission accepted: $mutation" >&2; exit 1
    fi
done
echo 'Published runtime admits exact registry identities and refuses replacements/missing versions.'
