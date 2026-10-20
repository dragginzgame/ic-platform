#!/usr/bin/env bash
set -euo pipefail
# Explicit opt-in; requires prepared nightly, registry sources and C++ compiler.
export RUSTUP_AUTO_INSTALL=0
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
host="$(rustup run nightly-2026-07-29 rustc -vV | sed -n 's/^host: //p')"
fixture="$(mktemp -d "$root/target/parser-fuzz.XXXXXX")"
mkdir -p "$fixture/crates/probe/src" "$fixture/corpus"
version="$("$root/.tools/host/bin/yq" -p toml -r '.workspace.dependencies.libfuzzer-sys' "$root/Cargo.toml")"
cat > "$fixture/Cargo.toml" <<MANIFEST
[workspace]
members = ["crates/probe"]
resolver = "3"
[workspace.dependencies]
ic-platform = { path = "../../crates/ic-platform", default-features = false, features = ["icrc-ledger-types"] }
libfuzzer-sys = "$version"
MANIFEST
cat > "$fixture/crates/probe/Cargo.toml" <<'MANIFEST'
[package]
name = "platform-parser-fuzz"
version = "0.0.0"
edition = "2024"
[dependencies]
ic-platform.workspace = true
libfuzzer-sys.workspace = true
MANIFEST
cp "$root/scripts/fixtures/parser-fuzz/main.rs" "$fixture/crates/probe/src/main.rs"
cp "$root/Cargo.lock" "$fixture/Cargo.lock"
cargo metadata --offline --manifest-path "$fixture/Cargo.toml" --format-version 1 > "$fixture/metadata.json"
printf aaaaa-aa > "$fixture/corpus/management"
printf 2vxsx-fae > "$fixture/corpus/anonymous"
printf '%s' 'k2t6j-2nvnp-4zjm3-25dtz-6xhaa-c7boj-5gayf-oj3xs-i43lp-teztq-6ae-6cc627i.1' > "$fixture/corpus/account"
printf 'Fuzz evidence: %s\n' "$fixture"
RUSTFLAGS='-Cpasses=sancov-module -Cllvm-args=-sanitizer-coverage-level=4 -Cllvm-args=-sanitizer-coverage-inline-8bit-counters -Cllvm-args=-sanitizer-coverage-pc-table -Cllvm-args=-sanitizer-coverage-trace-compares -Zsanitizer=address' \
    CARGO_BUILD_JOBS=4 CARGO_TARGET_DIR="$fixture/target" cargo +nightly-2026-07-29 build --target "$host" --release --frozen --manifest-path "$fixture/Cargo.toml"
"$fixture/target/$host/release/platform-parser-fuzz" "$fixture/corpus" -seed=42 -max_len=512 -timeout=5 -rss_limit_mb=2048 -max_total_time=60 "-artifact_prefix=$fixture/" "$@"
