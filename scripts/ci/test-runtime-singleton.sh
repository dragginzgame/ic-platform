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
trap 'printf "Runtime singleton evidence: %s\n" "$fixture"' EXIT
mkdir -p "$fixture/src"
printf '// Cargo packaging proof; execution stays in the qualified runtime fixtures.\n' > "$fixture/src/lib.rs"
manifest() {
cat > "$fixture/Cargo.toml" <<MANIFEST
[package]
name = "runtime-singleton-proof"
version = "0.0.0"
edition = "2024"
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
for features in executor runtime cdk; do
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
# Exercise two real facade versions as well as the retired registry executor.
# Only disposable package metadata changes; both copies retain the actual source.
mkdir "$fixture-next-platform"
printf 'Next-version source evidence: %s\n' "$fixture-next-platform"
cp Cargo.toml "$fixture-next-platform/"
cp -R crates "$fixture-next-platform/"
version="$("$root/.tools/host/bin/yq" -p toml -o json -r '.package.version' crates/ic-platform/Cargo.toml)"
next="${version%.*}.$(( ${version##*.} + 1 ))"
PLATFORM_SINGLETON_CURRENT="$version" PLATFORM_SINGLETON_NEXT="$next" perl -0pi -e '
    my ($old, $new) = @ENV{qw(PLATFORM_SINGLETON_CURRENT PLATFORM_SINGLETON_NEXT)};
    my $count = s/^(ic-platform(?:-macros|-runtime-guard)? = \{ version = ")(=?)\Q$old\E(")/$1$2$new$3/gm;
    $count == 2 or die "expected two coordinated dependency requirements\n";
' "$fixture-next-platform/Cargo.toml"
for package in ic-platform ic-platform-macros ic-platform-runtime-guard; do
    PLATFORM_NEXT_VERSION="$next" perl -pi -e 's/^version = "[^" ]+"$/version = "$ENV{PLATFORM_NEXT_VERSION}"/' "$fixture-next-platform/crates/$package/Cargo.toml"
done
# Both facade generations must retain the canonical identity owner, rather than
# copying a second definition along with their runtime source.
PLATFORM_IDENTITY_SOURCE="$root/crates/ic-platform-types" perl -0pi -e '
    s/^    "crates\/ic-platform-types",\n//m == 1 or die "identity member changed\n";
    s/path = "crates\/ic-platform-types"/"path = \"".$ENV{PLATFORM_IDENTITY_SOURCE}."\""/e == 1 or die "identity selection changed\n";
' "$fixture-next-platform/Cargo.toml"
# Bind inherited metadata to the copied workspace when Cargo traverses both
# path suppliers from a consumer nested in the original workspace.
for manifest_path in "$fixture-next-platform"/crates/*/Cargo.toml; do
    perl -pi -e 's/^\[package\]$/[package]\nworkspace = "..\/.."/' "$manifest_path"
done
for selection in native current-runtime next-runtime both-runtime; do
    current=ic0 next_features=ic0
    case "$selection" in
        current-runtime) current=executor ;;
        next-runtime) next_features=executor ;;
        both-runtime) current=executor; next_features=executor ;;
    esac
    cat > "$fixture/Cargo.toml" <<MANIFEST
[package]
name = "runtime-singleton-proof"
version = "0.0.0"
edition = "2024"
[workspace]
[dependencies]
current = { package = "ic-platform", path = "../../crates/ic-platform", default-features = false, features = ["$current"] }
next = { package = "ic-platform", path = "$fixture-next-platform/crates/ic-platform", default-features = false, features = ["$next_features"] }
MANIFEST
    cp "$fixture/Cargo.toml" "$fixture/$selection.Cargo.toml"
    result=0
    cargo tree --manifest-path "$fixture/Cargo.toml" --offline --edges normal,build --prefix none \
        > "$fixture/$selection.tree" 2> "$fixture/$selection.log" || result=$?
    if [[ "$selection" == both-runtime ]]; then
        [[ "$result" != 0 ]]
        grep -q 'links to the native library' "$fixture/$selection.log"
        grep -Fq "ic-platform v$version " "$fixture/$selection.log"
        grep -Fq "ic-platform v$next " "$fixture/$selection.log"
    else
        [[ "$result" == 0 ]] || { cat "$fixture/$selection.log" >&2; exit 1; }
        guards="$(awk '$1 == "ic-platform-runtime-guard" {print $2}' "$fixture/$selection.tree" | LC_ALL=C sort -u)"
        expected=""
        if [[ "$selection" == current-runtime ]]; then expected="v$version"; fi
        if [[ "$selection" == next-runtime ]]; then expected="v$next"; fi
        [[ "$guards" == "$expected" ]] || { echo "Unexpected guards for $selection: $guards" >&2; exit 1; }
    fi
done
echo 'Two Platform versions coexist without runtime or with one runtime; two runtime owners are refused.'
