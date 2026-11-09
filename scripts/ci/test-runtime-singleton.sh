#!/usr/bin/env bash
set -euo pipefail
# Use Cargo's real registry executor/links identity, not a simulated resolver.
prepare=false
case "$#:${1:-}" in
    0:) ;;
    1:--prepare) prepare=true ;;
    *) echo "usage: $0 [--prepare]" >&2; exit 2 ;;
esac
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
mkdir -p target
fixture="$(mktemp -d "$root/target/runtime-singleton.XXXXXX")"
finish() {
    result=$?
    if [[ "$result" != 0 ]]; then
        for log in "$fixture"/*.log; do
            [[ -f "$log" ]] || continue
            printf '\nEvidence: %s\n' "$log" >&2
            cat "$log" >&2
        done
    fi
    printf 'Runtime singleton evidence: %s\n' "$fixture"
    exit "$result"
}
trap finish EXIT
mkdir -p "$fixture/src"
printf '// Cargo packaging proof; execution stays in the qualified runtime fixtures.\n' > "$fixture/src/lib.rs"
manifest() {
cat > "$fixture/Cargo.toml" <<MANIFEST
[package]
name = "runtime-singleton-proof"
version = "0.0.0"
edition = "2024"

[package.metadata.toko-tooling.loc]
category = "tests"
[workspace]
[dependencies]
ic-platform = { path = "../../crates/ic-platform", default-features = false, features = [$1] }
ic-cdk-executor = "=2.0.0"
MANIFEST
}
cp Cargo.lock "$fixture/Cargo.lock"
if [[ "$prepare" == true ]]; then
    # The embedded runtime removed this registry supplier from the root lock.
    # Resolve and fetch it only during explicitly requested fixture setup.
    manifest '"ic0"'
    cargo fetch --manifest-path "$fixture/Cargo.toml"
    # The negative legacy-owner case also needs its registry manifest on fresh
    # runners; a missing offline package is not singleton-refusal evidence.
    cat > "$fixture/Cargo.toml" <<'MANIFEST'
[package]
name = "runtime-singleton-proof"
version = "0.0.0"
edition = "2024"

[package.metadata.toko-tooling.loc]
category = "tests"
[workspace]
[dependencies]
ic-platform = { version = "=0.16.0", default-features = false, features = ["executor"] }
MANIFEST
    cargo fetch --manifest-path "$fixture/Cargo.toml"
    echo 'Registry singleton dependencies prepared; normal checks remain offline.'
    exit 0
fi
# Only this disposable lock resolves. The native source profile's unrelated
# registry executor must coexist with every selection lacking Platform's executor.
for features in default ic0; do
    manifest "\"$features\""
    result=0
    cargo tree --manifest-path "$fixture/Cargo.toml" --offline \
        --edges normal,build --prefix none > "$fixture/$features.tree" 2> "$fixture/$features.log" || result=$?
    if [[ "$result" != 0 ]]; then
        cat "$fixture/$features.log" >&2
        echo 'Prepare fixture dependencies with make prepare-consumer-dependencies.' >&2
        exit "$result"
    fi
    grep -q '^ic-cdk-executor v2.0.0$' "$fixture/$features.tree"
    if grep -q '^ic-platform-runtime-guard ' "$fixture/$features.tree"; then
        echo "Non-runtime $features selection acquired an executor guard" >&2; exit 1
    fi
done
manifest '"ic0"'
cargo check --manifest-path "$fixture/Cargo.toml" --frozen \
    --target-dir "$fixture/target" > "$fixture/ic0-check.log" 2>&1
# Reuse the successful native lock: enabling runtime later must still refuse.
for features in executor runtime candid-codec-endpoint; do
    manifest "$(printf '"%s"' "$features" | sed 's/,/", "/g')"
    if cargo tree --manifest-path "$fixture/Cargo.toml" --offline \
        > "$fixture/$features.tree" 2> "$fixture/$features.log"; then
        echo "Cargo admitted mixed executors for $features" >&2; exit 1
    fi
    grep -q 'links to the native library' "$fixture/$features.log"
    grep -q 'ic-platform-runtime-guard' "$fixture/$features.log"
    grep -q 'ic-cdk-executor' "$fixture/$features.log"
done
echo 'Native default/ic0 coexistence and all three runtime singleton refusals passed.'
# Crypto/certification releases may change the facade without changing any
# Runtime/Types requirement or implementation. Both facades delegate one owner.
copy="$fixture-next-platform"
mkdir "$copy"
cp Cargo.toml "$copy/"
cp -R crates "$copy/"
# Edit only the Platform facade literal and its catalog requirement.
perl -0pi -e 's/^(ic-platform = \{ version = ")[^"]+/${1}99.0.0/gm' "$copy/Cargo.toml"
perl -pi -e 's/^version = "[^"]+"$/version = "99.0.0"/' "$copy/crates/ic-platform/Cargo.toml"
# A second facade must depend on the original canonical owners, not copied
# packages with coincident versions. Their version requirements remain untouched.
for package in ic-platform-types ic-platform-runtime ic-platform-macros ic-platform-runtime-guard; do
    cmp "$root/crates/$package/Cargo.toml" "$copy/crates/$package/Cargo.toml"
    PLATFORM_SINGLETON_PACKAGE="$package" PLATFORM_SINGLETON_ROOT="$root" perl -0pi -e '
        my ($p,$r)=@ENV{qw(PLATFORM_SINGLETON_PACKAGE PLATFORM_SINGLETON_ROOT)};
        s/^    "crates\/\Q$p\E",\n//m == 1 or die "missing member $p";
        s/path = "crates\/\Q$p\E"/path = "$r\/crates\/$p"/ == 1 or die "missing catalog $p";
    ' "$copy/Cargo.toml"
done
perl -pi -e 's/^\[package\]$/[package]\nworkspace = "..\/.."/' "$copy/crates/ic-platform/Cargo.toml"
cat > "$fixture/Cargo.toml" <<MANIFEST
[package]
name = "runtime-singleton-proof"
version = "0.0.0"
edition = "2024"

[package.metadata.toko-tooling.loc]
category = "tests"
[workspace]
[dependencies]
current = { package = "ic-platform", path = "$root/crates/ic-platform", default-features = false, features = ["runtime"] }
next = { package = "ic-platform", path = "$copy/crates/ic-platform", default-features = false, features = ["runtime"] }
ic-platform-runtime = { path = "$root/crates/ic-platform-runtime", features = ["runtime"] }
MANIFEST
cat > "$fixture/src/lib.rs" <<'RUST'
// Public handles and calls retain a single owner across facade generations.
pub fn task(value: current::executor::TaskHandle) -> next::executor::TaskHandle { value }
pub fn direct(value: next::executor::TaskHandle) -> ic_platform_runtime::executor::TaskHandle { value }
pub fn call<'a, 'b>(value: current::call::Call<'a, 'b>) -> next::call::Call<'a, 'b> { value }
RUST
cargo check --manifest-path "$fixture/Cargo.toml" --offline --target-dir "$fixture/target" > "$fixture/crypto-break-check.log" 2>&1
cargo tree --manifest-path "$fixture/Cargo.toml" --offline --edges normal,build --prefix none > "$fixture/crypto-break.tree"
[[ "$(awk '$1 == "ic-platform-runtime"' "$fixture/crypto-break.tree" | sed 's/ (\*)$//' | sort -u | wc -l | tr -d ' ')" == 1 ]]
# A genuinely different runtime generation keeps the exact guard and refuses
# coexistence, even when both implementation bodies happen to be unchanged.
other="$fixture-next-runtime"
mkdir "$other"
cp Cargo.toml "$other/"
cp -R crates "$other/"
perl -0pi -e 's/^(ic-platform(?:-runtime|-runtime-guard|-macros) = \{ version = ")(=?)[^"]+/${1}${2}99.0.0/gm' "$other/Cargo.toml"
for package in ic-platform-runtime ic-platform-runtime-guard ic-platform-macros; do
    perl -pi -e 's/^version = "[^"]+"$/version = "99.0.0"/' "$other/crates/$package/Cargo.toml"
done
for manifest_path in "$other"/crates/*/Cargo.toml; do
    perl -pi -e 's/^\[package\]$/[package]\nworkspace = "..\/.."/' "$manifest_path"
done
cat > "$fixture/Cargo.toml" <<MANIFEST
[package]
name = "runtime-singleton-proof"
version = "0.0.0"
edition = "2024"

[package.metadata.toko-tooling.loc]
category = "tests"
[workspace]
[dependencies]
current = { package = "ic-platform-runtime", path = "$root/crates/ic-platform-runtime", features = ["executor"] }
next = { package = "ic-platform-runtime", path = "$other/crates/ic-platform-runtime", features = ["executor"] }
MANIFEST
if cargo tree --manifest-path "$fixture/Cargo.toml" --offline > "$fixture/incompatible.tree" 2> "$fixture/incompatible.log"; then
    echo 'Mixed runtime generations unexpectedly resolved' >&2; exit 1
fi
grep -q 'links to the native library' "$fixture/incompatible.log"
grep -q 'ic-platform-runtime-guard' "$fixture/incompatible.log"
perl -0777 -i -pe 's/^next = .*$/next = { package = "ic-platform", version = "=0.16.0", features = ["executor"] }/m == 1 or die "missing next owner\n"' "$fixture/Cargo.toml"
if cargo tree --manifest-path "$fixture/Cargo.toml" --offline > "$fixture/legacy.tree" 2> "$fixture/legacy.log"; then
    echo 'Legacy embedded executor unexpectedly resolved' >&2; exit 1
fi
grep -q 'links to the native library' "$fixture/legacy.log"
echo 'Crypto facade generations share Runtime types; incompatible and legacy executor owners are refused.'
