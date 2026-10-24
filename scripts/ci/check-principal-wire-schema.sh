#!/usr/bin/env bash
set -euo pipefail
# Explicitly prepared TypeScript compiler; no installation or package downloads.
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
published=""
case $# in
    1) ;;
    3)
        [[ "$2" == --published && "$3" =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]] || {
            echo 'Expected --published X.Y.Z' >&2; exit 2;
        }
        published="$3"
        ;;
    *) echo 'usage: check-principal-wire-schema.sh PATH_TO_TYPESCRIPT_TSC_JS [--published X.Y.Z]' >&2; exit 2 ;;
esac
[[ -f "$1" ]] || { echo 'TypeScript compiler must be explicitly prepared' >&2; exit 2; }
compiler="$(cd "$(dirname "$1")" && pwd -P)/$(basename "$1")"
node "$compiler" --version
mkdir -p "$root/target"
fixture="$(mktemp -d "$root/target/principal-wire-schema.XXXXXX")"
mkdir -p "$fixture/src" "$fixture/browser"
wire="$("$root/.tools/host/bin/yq" -p toml -o yaml -r '.workspace.dependencies.ic-wire-metadata.version' "$root/Cargo.toml")"
selection='path = "../../crates/ic-platform"'
if [[ -n "$published" ]]; then selection="version = \"=$published\""; fi
cat > "$fixture/Cargo.toml" <<MANIFEST
[package]
name = "principal-wire-schema-proof"
version = "0.0.0"
edition = "2024"
[workspace]
[dependencies]
ic-platform = { $selection, default-features = false, features = ["wire-schema"] }
ic-wire-metadata = { package = "ic-wire", version = "$wire", features = ["typescript"] }
serde = { version = "1", features = ["derive"] }
serde_bytes = "0.11"
serde_json = "1"
MANIFEST
cp "$root/crates/ic-platform/tests/wire_schema_consumer.rs" "$fixture/src/lib.rs"
cp "$root/Cargo.lock" "$fixture/Cargo.lock"
cargo metadata --manifest-path "$fixture/Cargo.toml" --offline --filter-platform "$(rustc -vV | sed -n 's/^host: //p')" --format-version 1 > "$fixture/metadata.json"
if [[ -n "$published" ]]; then
    # shellcheck disable=SC2016 # $version is a jq variable.
    "$root/.tools/host/bin/jq" -e --arg version "$published" '
        [.packages[] | select(.name == "ic-platform")] as $facades |
        ($facades | length) == 1 and $facades[0].version == $version and
        $facades[0].source == "registry+https://github.com/rust-lang/crates.io-index"
    ' "$fixture/metadata.json" > /dev/null || {
        echo 'Published metadata proof must use the exact registry Platform' >&2; exit 1;
    }
fi
PLATFORM_WIRE_SCHEMA_OUTPUT="$fixture/browser" CARGO_TARGET_DIR="$root/target/consumer-msrv" \
    cargo +1.91.0 test --manifest-path "$fixture/Cargo.toml" --frozen
printf '{"type":"module"}\n' > "$fixture/browser/package.json"
node "$compiler" --strict --target ES2022 --module NodeNext --moduleResolution NodeNext \
    --outDir "$fixture/browser" "$fixture/browser/account.ts" "$fixture/browser/principal.ts" "$fixture/browser/wire-runtime.ts"
cp "$root/scripts/fixtures/wire-schema/check.mjs" "$fixture/browser/check.mjs"
node "$fixture/browser/check.mjs"
printf 'Principal/Account Wire evidence: %s\n' "$fixture"
