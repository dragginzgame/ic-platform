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
admit() {
    "$jq" -e --arg version 0.4.0 --argjson require_generator "$1" \
        -f "$root/scripts/ci/published-runtime.jq" "$2" > /dev/null
}
admit true "$fixture/good.json"
admit false "$fixture/good.json"
for mutation in \
    '.packages[0].source = null' \
    '.packages[1].source = "git+https://example.invalid/generator"' \
    '.packages[0].version = "0.3.0"' \
    '.packages[1].version = "0.3.0"' \
    '.packages = []' \
    '.packages |= map(select(.name != "ic-platform"))' \
    '.packages += [.packages[0]]' \
    '.packages += [.packages[1]]'; do
    "$jq" "$mutation" "$fixture/good.json" > "$fixture/bad.json"
    for required in true false; do
        if admit "$required" "$fixture/bad.json"; then
            echo "Registry admission accepted: $mutation" >&2; exit 1
        fi
    done
done
"$jq" '.packages |= map(select(.name != "ic-platform-macros"))' "$fixture/good.json" > "$fixture/facade.json"
admit false "$fixture/facade.json"
if admit true "$fixture/facade.json"; then
    echo 'Runtime admission accepted a missing generator' >&2; exit 1
fi
"$jq" '.packages += [{name: "ic-platform", version: "0.1.18", source: "registry+https://github.com/rust-lang/crates.io-index"}]' \
    "$fixture/good.json" > "$fixture/timers.json"
admit true "$fixture/timers.json"
if admit false "$fixture/timers.json"; then
    echo 'Standalone admission accepted multiple facades' >&2; exit 1
fi
"$jq" '.packages[2].source = null' "$fixture/timers.json" > "$fixture/bad.json"
if admit true "$fixture/bad.json"; then
    echo 'Runtime admission accepted a replaced transitive facade' >&2; exit 1
fi
echo 'Published runtime admits exact registry identities and refuses replacements/missing versions.'
