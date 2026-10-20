#!/usr/bin/env bash
set -euo pipefail
bench="${BENCH_ROOT:?Set BENCH_ROOT to a prepared benchmark directory}"
for revision in base candidate; do
 for selection in runtime creation verification; do
  fixture="$bench/fixtures/$revision-$selection"
  mkdir -p "$fixture/src"
  case "$selection" in runtime) features='"cdk"';; creation) features='"canister-sig-creation"';; verification) features='"signature-verification"';; esac
  cat > "$fixture/Cargo.toml" <<MANIFEST
[package]
name = "platform-benchmark"
version = "0.0.0"
edition = "2024"
[workspace]
[lib]
crate-type = ["cdylib"]
[features]
external-candid = ["ic-platform/candid"]
[dependencies]
ic-platform = { path = "$bench/$revision/crates/ic-platform", default-features = false, features = [$features] }
[profile.release]
opt-level = 3
lto = false
codegen-units = 16
panic = "abort"
strip = "debuginfo"
MANIFEST
  if [[ "$selection" == runtime ]]; then
   cat >> "$fixture/Cargo.toml" <<'MANIFEST'
[dependencies.ic-timers]
version = "=0.29.2"
[dependencies.ic-wire]
version = "=0.4.7"
[dependencies.serde]
version = "1.0"
default-features = false
MANIFEST
   cp "$bench/candidate/scripts/fixtures/cdk-runtime/canister.rs" "$fixture/src/lib.rs"
   cp -R "$bench/candidate/scripts/fixtures/cdk-runtime/"{wire_canister,wire_protocol,protocol,codec} "$fixture/src/"
   cp "$bench/candidate/scripts/fixtures/cdk-runtime/Cargo.lock" "$fixture/Cargo.lock"
  else
   source=canister_sig
   if [[ "$selection" == verification ]]; then source=signature; fi
   cp "$bench/candidate/crates/ic-platform/tests/${source}_consumer.rs" "$fixture/src/lib.rs"
   cp -R "$bench/candidate/crates/ic-platform/tests/fixtures" "$fixture/src/"
   cat >> "$fixture/src/lib.rs" <<'RUST'

#[unsafe(no_mangle)]
pub extern "C" fn benchmark() { roundtrip(); }
RUST
   cp "$bench/$revision/Cargo.lock" "$fixture/Cargo.lock"
  fi
  cargo +1.99.1 metadata --manifest-path "$fixture/Cargo.toml" --offline --filter-platform wasm32-unknown-unknown --format-version 1 > "$fixture/metadata.json"
 done
done
