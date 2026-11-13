#!/usr/bin/env bash
set -euo pipefail
# Explicit opt-in; requires prepared nightly, registry sources and C++ compiler.
export RUSTUP_AUTO_INSTALL=0
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
# shellcheck source=scripts/ci/workspace-dependencies.sh
source "$root/scripts/ci/workspace-dependencies.sh"
platform_dependencies_read "$root"
host="$(rustup run nightly-2026-11-05 rustc -vV | sed -n 's/^host: //p')"
fixture="$(mktemp -d "$root/target/parser-fuzz.XXXXXX")"
mkdir -p "$fixture/crates/probe/src" "$fixture/corpus"
version="$(platform_dependency_version libfuzzer-sys)"
cat > "$fixture/Cargo.toml" <<MANIFEST
[workspace]
members = ["crates/probe"]
resolver = "3"
[workspace.dependencies]
ic-platform = { path = "../../crates/ic-platform", default-features = false, features = ["icrc-ledger-types", "candid-codec", "ic-protocol"] }
libfuzzer-sys = "$version"
MANIFEST
cat > "$fixture/crates/probe/Cargo.toml" <<'MANIFEST'
[package]
name = "platform-parser-fuzz"
version = "0.0.0"
edition = "2024"

[package.metadata.toko-tooling.loc]
category = "tests"
[dependencies]
ic-platform.workspace = true
libfuzzer-sys.workspace = true
MANIFEST
cp "$root/scripts/fixtures/parser-fuzz/main.rs" "$fixture/crates/probe/src/main.rs"
cp "$root/crates/ic-platform/tests/fixtures/candid_malformed.rs" "$fixture/crates/probe/src/candid_malformed.rs"
cp "$root/crates/ic-platform/tests/fixtures/cbor_malformed.rs" "$fixture/crates/probe/src/cbor_malformed.rs"
cp "$root/Cargo.lock" "$fixture/Cargo.lock"
platform_fixture_policy "$root" "$fixture"
cargo metadata --offline --manifest-path "$fixture/Cargo.toml" --format-version 1 > "$fixture/metadata.json"
bash "$root/vendor/toko-tooling/scripts/ci/check-cargo-dependency-boundaries.sh" --consumer "$fixture" --external-ic
printf 'DIDL\000\001\170\052\000\000\000\000\000\000\000' > "$fixture/corpus/candid-one"
printf 'DIDL\001\156\000\001\000\001' > "$fixture/corpus/candid-recursive"
printf 'DIDL\200\200\200\200\200\200\200\200\200\200\000' > "$fixture/corpus/candid-overlong"
printf aaaaa-aa > "$fixture/corpus/management"
printf '\201\000' > "$fixture/corpus/cbor-empty-tree"
printf '\242\144tree\201\000\151signature\100' > "$fixture/corpus/cbor-certificate"
printf '\237\003\137\101a\377\377' > "$fixture/corpus/cbor-chunks"
printf '\241\153certificate\100' > "$fixture/corpus/cbor-read-state"
while IFS=$'\t' read -r name hex rest; do
    [[ "$name" != \#* && -n "$name" ]] || continue
    [[ "$name" =~ ^[a-z0-9-]+$ && "$hex" =~ ^([0-9a-f]{2})+$ ]] || exit 1
    perl -e 'print pack("H*", $ARGV[0])' "$hex" > "$fixture/corpus/ingress-$name"
done < "$root/crates/ic-platform/tests/fixtures/ingress.tsv"
printf 2vxsx-fae > "$fixture/corpus/anonymous"
printf '%s' 'k2t6j-2nvnp-4zjm3-25dtz-6xhaa-c7boj-5gayf-oj3xs-i43lp-teztq-6ae-6cc627i.1' > "$fixture/corpus/account"
printf 'Fuzz evidence: %s\n' "$fixture"
RUSTFLAGS='-Cpasses=sancov-module -Cllvm-args=-sanitizer-coverage-level=4 -Cllvm-args=-sanitizer-coverage-inline-8bit-counters -Cllvm-args=-sanitizer-coverage-pc-table -Cllvm-args=-sanitizer-coverage-trace-compares -Zsanitizer=address' \
    CARGO_BUILD_JOBS=4 CARGO_TARGET_DIR="$fixture/target" cargo +nightly-2026-11-05 build --target "$host" --release --frozen --manifest-path "$fixture/Cargo.toml"
"$fixture/target/$host/release/platform-parser-fuzz" "$fixture/corpus" -seed=42 -max_len=512 -timeout=5 -rss_limit_mb=2048 -max_total_time=60 "-artifact_prefix=$fixture/" "$@" > "$fixture/fuzz.log" 2>&1 || {
    status=$?
    tail -60 "$fixture/fuzz.log"
    exit "$status"
}
tail -5 "$fixture/fuzz.log"
