#!/usr/bin/env bash
set -euo pipefail
root="${BASH_SOURCE[0]}"
[[ "$root" == /* ]] || root="$PWD/$root"
case "$root" in *$'\n'*|*$'\r'*) echo 'Script path contains CR/LF' >&2; exit 2 ;; esac
root="$(CDPATH='' cd -P "${root%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
case "$root" in *$'\n'*|*$'\r'*) echo 'Resolved root contains CR/LF' >&2; exit 2 ;; esac
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$root/scripts/ci/fixture-lifecycle.sh"
fixture="$(fixture_create "$root/target/blake3-feature-union.XXXXXX")"
fixture_complete=false
trap 'fixture_finish "$?" "$fixture" "BLAKE3 graph fixture retained"' EXIT
mkdir -p "$fixture/consumer/src"
platform_path="$(jq -nr --arg path "$root/crates/ic-platform" '$path | tojson')"
cat > "$fixture/Cargo.toml" <<MANIFEST
[workspace]
members = ["consumer"]
resolver = "3"
[workspace.dependencies]
ic-platform = { path = $platform_path, default-features = false, features = ["blake3"] }
hasher = { package = "blake3", version = "1.8.7", default-features = false }
MANIFEST
cat > "$fixture/consumer/Cargo.toml" <<'MANIFEST'
[package]
name = "primitive-graph-consumer"
version = "0.0.0"
edition = "2024"
[dependencies]
ic-platform.workspace = true
# Deliberate adversarial alias: one identity can acquire extra unified features.
hasher = { workspace = true, optional = true }
[features]
extra = ["dep:hasher", "hasher/pure"]
MANIFEST
printf 'pub fn digest() { let _ = ic_platform::blake3::hash(b"fixture"); }\n' > "$fixture/consumer/src/lib.rs"
cp "$root/Cargo.lock" "$fixture/Cargo.lock"
cargo metadata --manifest-path "$fixture/Cargo.toml" --offline --filter-platform wasm32-unknown-unknown \
    --format-version 1 > "$fixture/metadata.json"
for target in "$(rustc -vV | sed -n 's/^host: //p')" wasm32-unknown-unknown; do
    bash "$root/scripts/ci/check-blake3-graph.sh" "$fixture" primitive-graph-consumer "$target" -
done
if bash "$root/scripts/ci/check-blake3-graph.sh" "$fixture" primitive-graph-consumer \
    wasm32-unknown-unknown extra > "$fixture/extra.log" 2>&1; then
    echo 'Unified extra BLAKE3 feature was admitted' >&2; exit 1
fi
grep -q 'must select only std' "$fixture/extra.log"
grep -q 'pure,std' "$fixture/extra.log"
fixture_complete=true
echo 'Real locked Cargo graphs reject extra features even with one package identity'
