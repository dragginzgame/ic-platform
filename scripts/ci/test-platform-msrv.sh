#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/platform-msrv.XXXXXX")"
trap 'rm -rf "$fixture"' EXIT
for version in 1.91.0 1.92.0; do
    printf '[workspace]\n[workspace.package]\nrust-version = "%s"\n' "$version" > "$fixture/Cargo.toml"
    actual="$(bash "$root/scripts/ci/read-platform-msrv.sh" "$fixture/Cargo.toml")"
    [[ "$actual" == "$version" ]] || exit 1
done
for version in '' 1.91 1.91.0-beta invalid; do
    printf '[workspace]\n[workspace.package]\nrust-version = "%s"\n' "$version" > "$fixture/Cargo.toml"
    if bash "$root/scripts/ci/read-platform-msrv.sh" "$fixture/Cargo.toml" > "$fixture/output" 2> "$fixture/error"; then
        echo 'MSRV reader admitted an unpinned or invalid compiler' >&2; exit 1
    fi
    [[ ! -s "$fixture/output" ]] || exit 1
done
# Exercise the actual Make recipe with a later floor and a non-compiling Cargo
# adapter. This catches accidental Make expansion of shell variables.
mkdir -p "$fixture/scripts/ci" "$fixture/.tools" "$fixture/bin"
cp "$root/scripts/ci/read-platform-msrv.sh" "$fixture/scripts/ci/"
ln -s "$root/.tools/host" "$fixture/.tools/host"
printf '[workspace]\n[workspace.package]\nrust-version = "1.92.0"\n' > "$fixture/Cargo.toml"
awk '/^msrv:/ { printing=1 } printing && /^[A-Za-z0-9_.-]+:/ && !/^msrv:/ { exit } printing { print }' "$root/Makefile" > "$fixture/Makefile"
printf '\nhost-tools-check:\n\t@:\n' >> "$fixture/Makefile"
cat > "$fixture/bin/cargo" <<'CARGO'
#!/usr/bin/env bash
if [[ "$1" != locate-project ]]; then printf '%s\n' "$*" >> "$MSRV_TEST_CALLS"; fi
CARGO
chmod +x "$fixture/bin/cargo"
PATH="$fixture/bin:$PATH" MSRV_TEST_CALLS="$fixture/calls" make --no-print-directory -C "$fixture" msrv
[[ "$(wc -l < "$fixture/calls" | tr -d ' ')" == 2 ]]
[[ "$(grep -c '^+1.92.0 ' "$fixture/calls")" == 2 ]]
printf 'Current-source MSRV follows the manifest and refuses invalid/unpinned values\n'
