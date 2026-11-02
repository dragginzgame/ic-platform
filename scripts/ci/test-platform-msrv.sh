#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
mkdir -p "$root/target"
fixture="$(mktemp -d "$root/target/platform-msrv.XXXXXX")"
trap 'rm -rf "$fixture"' EXIT
for version in 1.99.1 1.100.0; do
    printf '[workspace]\n[workspace.package]\nrust-version = "%s"\n' "$version" > "$fixture/Cargo.toml"
    actual="$(bash "$root/scripts/ci/read-platform-msrv.sh" "$fixture/Cargo.toml")"
    [[ "$actual" == "$version" ]] || exit 1
done
for version in '' 1.99 1.99.1-beta invalid; do
    printf '[workspace]\n[workspace.package]\nrust-version = "%s"\n' "$version" > "$fixture/Cargo.toml"
    if bash "$root/scripts/ci/read-platform-msrv.sh" "$fixture/Cargo.toml" > "$fixture/output" 2> "$fixture/error"; then
        echo 'MSRV reader admitted an unpinned or invalid compiler' >&2; exit 1
    fi
    [[ ! -s "$fixture/output" ]] || exit 1
done
# Exercise actual compiler admission with a later floor, not another literal.
mkdir -p "$fixture/scripts/ci" "$fixture/.tools" "$fixture/bin"
cp "$root/scripts/ci/"{read-platform-msrv.sh,check-platform-compiler.sh} "$fixture/scripts/ci/"
ln -s "$root/.tools/host" "$fixture/.tools/host"
printf '[workspace]\n[workspace.package]\nrust-version = "1.100.0"\n' > "$fixture/Cargo.toml"
printf '[toolchain]\nchannel = "1.100.0"\n' > "$fixture/rust-toolchain.toml"
cat > "$fixture/bin/cargo" <<'CARGO'
#!/usr/bin/env bash
if [[ "$1" == --version ]]; then printf 'cargo %s (fixture)\n' "${FIXTURE_CARGO_VERSION:-1.100.0}"; fi
CARGO
cat > "$fixture/bin/rustc" <<'RUSTC'
#!/usr/bin/env bash
printf 'rustc %s (fixture)\n' "${FIXTURE_RUSTC_VERSION:-1.100.0}"
RUSTC
chmod +x "$fixture/bin/cargo" "$fixture/bin/rustc"
export PATH="$fixture/bin:$PATH"
unset RUSTC
bash "$fixture/scripts/ci/check-platform-compiler.sh"
if FIXTURE_CARGO_VERSION=1.99.1 bash "$fixture/scripts/ci/check-platform-compiler.sh" > "$fixture/log" 2>&1; then exit 1; fi
if FIXTURE_RUSTC_VERSION=1.99.1 bash "$fixture/scripts/ci/check-platform-compiler.sh" > "$fixture/log" 2>&1; then exit 1; fi
printf '[toolchain]\nchannel = "1.99.1"\n' > "$fixture/rust-toolchain.toml"
if bash "$fixture/scripts/ci/check-platform-compiler.sh" > "$fixture/log" 2>&1; then exit 1; fi
# The compatibility command reuses the same native/Wasm targets exactly once.
awk '/^msrv:/ { print }' "$root/Makefile" > "$fixture/Makefile"
printf '\ntest wasm-check:\n\t@echo $@ >> calls\n' >> "$fixture/Makefile"
make --no-print-directory -C "$fixture" msrv
[[ "$(wc -l < "$fixture/calls" | tr -d ' ')" == 2 ]]
grep -qx test "$fixture/calls"
grep -qx wasm-check "$fixture/calls"
# A failed manifest-tool lookup must stop preparation before Cargo sees a
# template with an empty rust-version (the native-provider CI regression).
native="$fixture/native-http"
mkdir -p "$native/scripts/ci" "$native/scripts/fixtures/native-profiler-http" "$native/bin"
cp "$root/scripts/ci/check-native-profiler-http.sh" "$native/scripts/ci/"
cp "$root/scripts/fixtures/native-profiler-http/Cargo.toml.in" "$native/scripts/fixtures/native-profiler-http/"
cat > "$native/scripts/ci/read-platform-msrv.sh" <<'READER'
#!/usr/bin/env bash
echo 'fixture: required manifest tool unavailable' >&2
exit 23
READER
cat > "$native/bin/cargo" <<'CARGO'
#!/usr/bin/env bash
touch "$NATIVE_HTTP_CARGO_MARKER"
CARGO
chmod +x "$native/bin/cargo"
result=0
PATH="$native/bin:$PATH" NATIVE_HTTP_CARGO_MARKER="$native/cargo-called" \
    bash "$native/scripts/ci/check-native-profiler-http.sh" --prepare > "$native/refusal.log" 2>&1 || result=$?
[[ "$result" == 23 && ! -e "$native/cargo-called" ]]
grep -q 'required manifest tool unavailable' "$native/refusal.log"
printf 'Compiler admission follows manifest/toolchain policy, rejects wrong Cargo/rustc, and reuses distinct native/Wasm proofs once\n'
