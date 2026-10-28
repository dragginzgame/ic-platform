#!/usr/bin/env bash
# Exercise the real checker with isolated metadata and documentation mutations.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/platform-inventory-test.XXXXXX")"
trap 'rm -rf "$fixture"' EXIT
mkdir -p "$fixture/scripts/ci" "$fixture/docs" "$fixture/bin"
cp "$root/scripts/ci/check-platform-inventory.sh" "$root/scripts/ci/platform-inventory.jq" "$fixture/scripts/ci/"
cp "$root/docs/ic-inventory.md" "$fixture/docs/"
cp "$root/README.md" "$fixture/"
cargo metadata --manifest-path "$root/Cargo.toml" --no-deps --format-version 1 --locked --offline > "$fixture/original.json"
cp "$fixture/original.json" "$fixture/metadata.json"
cat > "$fixture/bin/cargo" <<'CARGO'
#!/usr/bin/env bash
cat "$INVENTORY_TEST_METADATA"
CARGO
chmod +x "$fixture/bin/cargo"
export INVENTORY_TEST_METADATA="$fixture/metadata.json"
export PATH="$fixture/bin:$PATH"
bash "$fixture/scripts/ci/check-platform-inventory.sh" > "$fixture/log"
for mutation in \
    '.features["new-feature"] = []' \
    '.dependencies[0].req = "=99.0.0"' \
    '.dependencies[0].optional |= not' \
    '.dependencies[0].uses_default_features |= not' \
    '.dependencies[0].target = "cfg(unix)"' \
    '.dependencies[0].kind = "build"' \
    '.dependencies[0].features += ["new-upstream-feature"]' \
    'del(.dependencies[0])'; do
    jq "(.packages[] | select(.name == \"ic-platform\")) |= ($mutation)" "$fixture/original.json" > "$fixture/metadata.json"
    if bash "$fixture/scripts/ci/check-platform-inventory.sh" > "$fixture/log" 2>&1; then
        echo "inventory accepted drift: $mutation" >&2; exit 1
    fi
done
cp "$fixture/original.json" "$fixture/metadata.json"
sed '/^| `stable-structures` |/d' "$root/README.md" > "$fixture/README.md"
if bash "$fixture/scripts/ci/check-platform-inventory.sh" > "$fixture/log" 2>&1; then
    echo 'inventory accepted missing README feature' >&2; exit 1
fi
cp "$root/README.md" "$fixture/README.md"
sed '/<!-- END MANIFEST INVENTORY -->/d' "$root/docs/ic-inventory.md" > "$fixture/docs/ic-inventory.md"
if bash "$fixture/scripts/ci/check-platform-inventory.sh" > "$fixture/log" 2>&1; then
    echo 'inventory accepted malformed document' >&2; exit 1
fi
echo 'Inventory regressions passed: manifest drift, README omission and malformed document refused.'
