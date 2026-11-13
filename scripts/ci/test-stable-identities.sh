#!/usr/bin/env bash
# Real facade generations and Wire schema ownership; disposable sources/locks only.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
prepare=false
profiles=('' external-ledger current-runtime)
compiler="$(bash "$root/scripts/ci/read-platform-msrv.sh")"
# shellcheck source=scripts/ci/workspace-dependencies.sh
source "$root/scripts/ci/workspace-dependencies.sh"
case "${1:-}" in '') [[ $# == 0 ]] ;; --prepare) [[ $# == 1 ]]; prepare=true ;; *) exit 2 ;; esac
mkdir -p target
fixture="$(mktemp -d "$root/target/stable-identities.XXXXXX")"
printf 'Stable identity evidence: %s\n' "$fixture"
mkdir -p "$fixture/supplier/src" "$fixture/consumer/src" "$fixture/next"
printf '[workspace]\n[package]\nname="identity-schema-supplier"\nversion="0.0.0"\nedition="2024"\n[package.metadata.toko-tooling.loc]\ncategory="tests"\n[dependencies]\nic-wire="=0.12.1"\n' > "$fixture/supplier/Cargo.toml"
touch "$fixture/supplier/src/lib.rs"
platform_fixture_policy "$root" "$fixture/supplier"
if [[ "$prepare" == true ]]; then
    cargo fetch --manifest-path "$fixture/supplier/Cargo.toml"
    exit
fi
cargo metadata --manifest-path "$fixture/supplier/Cargo.toml" --offline --format-version 1 > "$fixture/supplier.json" 2> "$fixture/supplier.log" || {
    cat "$fixture/supplier.log" >&2
    echo 'Run bash scripts/ci/test-stable-identities.sh --prepare before offline qualification' >&2
    exit 1
}
wire_manifest="$(jq -er '.packages[] | select(.name=="ic-wire" and .version=="0.12.1") | .manifest_path' "$fixture/supplier.json")"
cp -R "${wire_manifest%/*}" "$fixture/wire"
# The actual Wire trait owner keeps its source/implementations. Only its schema
# dependency selects the new canonical type; endpoint migration belongs to Wire.
PLATFORM_IDENTITY_SOURCE="$root/crates/ic-platform-types" perl -0pi -e '
    s/\[dependencies\.ic-platform-types\]\n[^\[]*/"[dependencies.ic-platform-types]\npath = \"".$ENV{PLATFORM_IDENTITY_SOURCE}."\"\noptional = true\ndefault-features = false\n\n"/e == 1 or die "Wire supplier changed\n";
    $_ .= "\n[workspace]\n";
' "$fixture/wire/Cargo.toml"
diff -qr "${wire_manifest%/*}/src" "$fixture/wire/src"
cp Cargo.toml "$fixture/next/"
cp -R crates "$fixture/next/"
version="$("$root/.tools/host/bin/yq" -p toml -o json -r '.package.version' crates/ic-platform/Cargo.toml)"
major="${version%%.*}" minor="${version#*.}"; minor="${minor%%.*}"
next="$major.$((minor + 1)).0"
PLATFORM_IDENTITY_CURRENT="$version" PLATFORM_IDENTITY_NEXT="$next" PLATFORM_IDENTITY_SOURCE="$root/crates/ic-platform-types" perl -0pi -e '
    my ($old,$new,$source)=@ENV{qw(PLATFORM_IDENTITY_CURRENT PLATFORM_IDENTITY_NEXT PLATFORM_IDENTITY_SOURCE)};
    s/^(ic-platform = \{ version = ")(=?)\Q$old\E(")/$1$2$new$3/gm == 1 or die "facade catalog identity changed\n";
    for my $owner (qw(ic-platform-types ic-platform-runtime ic-platform-macros ic-platform-runtime-guard)) {
        my $path=$source =~ s{ic-platform-types$}{$owner}r;
        s/^    "crates\/\Q$owner\E",\n//m == 1 or die "owner member changed\n";
        s/path = "crates\/\Q$owner\E"/path = "$path"/ == 1 or die "owner selection changed\n";
    }
' "$fixture/next/Cargo.toml"
PLATFORM_NEXT_VERSION="$next" perl -pi -e 's/^version = "[^" ]+"$/version = "$ENV{PLATFORM_NEXT_VERSION}"/; s/^\[package\]$/[package]\nworkspace = "..\/.."/' "$fixture/next/crates/ic-platform/Cargo.toml"
cat > "$fixture/consumer/Cargo.toml" <<MANIFEST
[workspace]
[package]
name = "stable-identity-consumer"
version = "0.0.0"
edition = "2024"
rust-version = "$compiler"

[package.metadata.toko-tooling.loc]
category = "tests"
[features]
external-ledger = ["current/icrc-ledger-types"]
current-runtime = ["current/runtime"]
[dependencies]
current = { package = "ic-platform", path = "$root/crates/ic-platform", default-features = false, features = ["principal"] }
next = { package = "ic-platform", path = "$fixture/next/crates/ic-platform", default-features = false, features = ["principal"] }
ic-platform-types = { path = "$root/crates/ic-platform-types" }
ic-wire = { path = "$fixture/wire", features = ["platform-types"] }
MANIFEST
cp scripts/fixtures/stable-identities/{lib,tests}.rs "$fixture/consumer/src/"
platform_fixture_policy "$root" "$fixture/consumer"
cargo metadata --manifest-path "$fixture/consumer/Cargo.toml" --offline --all-features --format-version 1 > "$fixture/metadata.json"
bash "$root/scripts/ci/check-cargo-dependency-boundaries.sh" --consumer "$fixture/consumer" --external-ic
bash "$root/scripts/ci/check-platform-compiler.sh" > "$fixture/compiler.txt"
build="${CARGO_TARGET_DIR:-$root/target/stable-identities-build}"
for features in "${profiles[@]}"; do
    args=(); [[ -z "$features" ]] || args=(--features "$features")
    CARGO_TARGET_DIR="$build/$compiler" cargo clippy --manifest-path "$fixture/consumer/Cargo.toml" --offline --locked --all-targets ${args[@]+"${args[@]}"} -- -D warnings
    CARGO_TARGET_DIR="$build/$compiler" cargo +"$compiler" test --manifest-path "$fixture/consumer/Cargo.toml" --offline --locked ${args[@]+"${args[@]}"}
    CARGO_TARGET_DIR="$build/$compiler" cargo +"$compiler" check --manifest-path "$fixture/consumer/Cargo.toml" --offline --locked --target wasm32-unknown-unknown ${args[@]+"${args[@]}"}
done
cargo tree --manifest-path "$fixture/consumer/Cargo.toml" --offline --locked --edges normal --prefix none > "$fixture/identity.graph"
if grep -Eq '^(candid |ic-cdk|ic-platform-runtime-guard |ic-platform-macros )' "$fixture/identity.graph"; then
    echo 'Identity/schema-only graph acquired a codec/runtime owner' >&2; exit 1
fi
jq -e --arg path "$root/crates/ic-platform-types/Cargo.toml" '
 [.packages[] | select(.name=="ic-platform-types")] | length==1 and .[0].manifest_path==$path
' "$fixture/metadata.json" >/dev/null
printf 'Selected stable-identity profiles passed: %s\n' "${profiles[*]}"
