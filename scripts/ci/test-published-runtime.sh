#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
jq="$root/.tools/host/bin/jq"
mkdir -p "$root/target"
fixture="$(mktemp -d "$root/target/platform-registry-admission.XXXXXX")"
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
"$jq" '.packages += [{name: "ic-platform-runtime-guard", version: "0.4.0", source: "registry+https://github.com/rust-lang/crates.io-index"}] |
    .packages[0] += {id: "facade", dependencies: [{name: "ic-platform-runtime-guard"}]} |
    .resolve.nodes = [{id: "facade", features: ["executor"]}]' "$fixture/good.json" > "$fixture/guard.json"
admit true "$fixture/guard.json"
for mutation in '.packages[2].source = null' '.packages[2].version = "0.3.0"' '.packages += [.packages[2]]' '.packages |= map(select(.name != "ic-platform-runtime-guard"))'; do
    "$jq" "$mutation" "$fixture/guard.json" > "$fixture/bad.json"
    if admit true "$fixture/bad.json"; then
        echo "Registry guard admission accepted: $mutation" >&2; exit 1
    fi
done
echo 'Published runtime admits exact registry identities and refuses replacements/missing versions.'
# Independent facades need not share Runtime's version; Runtime/guard/macros do.
"$jq" '.packages[0].version = "9.0.0" | .packages[1].version = "0.4.0" |
    .packages += [{name: "ic-platform-runtime", id: "runtime", version: "0.4.0", source: "registry+https://github.com/rust-lang/crates.io-index"},
    {name: "ic-platform-runtime-guard", version: "0.4.0", source: "registry+https://github.com/rust-lang/crates.io-index"}]' "$fixture/good.json" > "$fixture/runtime.json"
admit_runtime() {
    "$jq" -e --arg version 0.4.0 --argjson runtime_owner true --argjson require_generator true -f "$root/scripts/ci/published-runtime.jq" "$1" > /dev/null
}
admit_runtime "$fixture/runtime.json"
for mutation in '.packages[2].source = null' '.packages[2].version = "0.5.0"' '.packages[3].version = "0.3.0"' '.packages |= map(select(.name != "ic-platform-runtime-guard"))' '.packages |= map(select(.name != "ic-platform-runtime"))' '.packages += [.packages[2]]'; do
    "$jq" "$mutation" "$fixture/runtime.json" > "$fixture/bad.json"
    if admit_runtime "$fixture/bad.json"; then echo "New runtime admission accepted: $mutation" >&2; exit 1; fi
done
"$jq" -e --arg version 9.0.0 --argjson require_generator false -f "$root/scripts/ci/published-runtime.jq" "$fixture/runtime.json" > /dev/null
echo 'Independent published Runtime identities and facade version separation passed.'
