#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
case "${1:-}" in
    "") [[ $# == 0 ]] || exit 2; mode=check ;;
    --prepare) [[ $# == 1 ]] || exit 2; mode=prepare ;;
    *) echo 'usage: check-native-profiler-http.sh [--prepare]' >&2; exit 2 ;;
esac
mkdir -p "$root/target"
fixture="$(mktemp -d "$root/target/native-profiler-http.XXXXXX")"
trap 'printf "Native HTTP evidence retained at %s\n" "$fixture"' EXIT
mkdir -p "$fixture/src"
cp "$root/scripts/fixtures/native-profiler-http/Cargo.toml.in" "$fixture/Cargo.toml"
source="$root/native/ic-profiler/profiler/server/system_profile_api/admission.rs"
cp "$source" "$fixture/src/lib.rs"
# A separate target avoids contention with the consumer library's Cargo builds.
export CARGO_TARGET_DIR="${PLATFORM_HTTP_TARGET_DIR:-$root/target/native-profiler-http-target}"
if [[ "$mode" == prepare ]]; then
    cargo fetch --manifest-path "$fixture/Cargo.toml"
    exit
fi
# Check mode never downloads, installs or initializes the IC submodule.
rustfmt --check --edition 2024 "$source"
cargo metadata --manifest-path "$fixture/Cargo.toml" --format-version 1 --offline > "$fixture/metadata.json"
cargo clippy --manifest-path "$fixture/Cargo.toml" --all-targets --frozen -- -D warnings
cargo test --manifest-path "$fixture/Cargo.toml" --lib --frozen
cmp "$source" "$fixture/src/lib.rs"
