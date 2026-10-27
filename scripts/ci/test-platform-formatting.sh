#!/usr/bin/env bash
set -euo pipefail
# Qualify Platform's actual Make policy and hook, never the caller's index.
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/platform-formatting.XXXXXX")"
finish() {
    local status=$?
    if [[ "$status" == 0 ]]; then
        rm -rf "$fixture"
    else
        printf 'Platform formatting evidence retained at: %s\n' "$fixture" >&2
        [[ ! -f "$fixture/last.log" ]] || cat "$fixture/last.log" >&2
    fi
    exit "$status"
}
trap finish EXIT
mkdir -p "$fixture/work" "$fixture/logs" "$fixture/bin"
# Include the current reviewed snapshot and unstaged inputs, without artifacts.
git -C "$root" ls-files --stage -z > "$fixture/index"
: > "$fixture/files"
: > "$fixture/gitlinks"
while IFS= read -r -d '' entry; do
    mode="${entry%% *}"
    path="${entry#*$'\t'}"
    case "$mode" in
        100644|100755) [[ ! -e "$root/$path" ]] || printf '%s\0' "$path" >> "$fixture/files" ;;
        160000) printf '%s\0' "$entry" >> "$fixture/gitlinks" ;;
        *) echo 'Formatting fixture requires regular files or opaque gitlinks' >&2; exit 1 ;;
    esac
done < "$fixture/index"
git -C "$root" ls-files --others --exclude-standard -z >> "$fixture/files"
while IFS= read -r -d '' path; do
    [[ -f "$root/$path" && ! -L "$root/$path" ]] || {
        echo 'Formatting fixture input is not a regular file' >&2; exit 1;
    }
done < "$fixture/files"
(cd "$root" && tar --null -T "$fixture/files" -cf -) |
    (cd "$fixture/work" && tar -xf -)
export RUNNER_TEMP="$fixture/logs"
run_make() {
    make --no-print-directory -C "$fixture/work" "$@" > "$fixture/last.log" 2>&1
}
run_make fmt-check
grep -q 'Standalone Rust formatting.*ok' "$fixture/last.log"
grep -q 'running without shortcut' "$fixture/last.log"

source_file=scripts/fixtures/secp-scaffolding/main.rs
cp "$fixture/work/$source_file" "$fixture/original.rs"
printf '\nfn platform_formatting_fixture( ){ }\n' >> "$fixture/work/$source_file"
if run_make fmt-check; then
    echo 'Unformatted standalone source was admitted' >&2; exit 1
fi
grep -q 'Standalone Rust formatting.*FAILED' "$fixture/last.log"
run_make fmt
run_make fmt-check
[[ ! -e "$fixture/work/target/shared-tooling/formatting.receipt" ]]

# A changed configuration must still be observed for this uncached selection.
printf 'max_width = "invalid"\n' > "$fixture/work/rustfmt.toml"
if run_make fmt-check; then
    echo 'Invalid formatter configuration was admitted' >&2; exit 1
fi
rm "$fixture/work/rustfmt.toml"

real_cargo="$(command -v cargo)"
export PLATFORM_FORMAT_REAL_CARGO="$real_cargo" PLATFORM_FORMAT_CALLS="$fixture/calls"
cat > "$fixture/bin/cargo" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >> "$PLATFORM_FORMAT_CALLS"
if [[ "$1" == sort && "${2:-}" == --workspace ]]; then
    echo 'injected manifest sorting failure' >&2
    exit 47
fi
exec "$PLATFORM_FORMAT_REAL_CARGO" "$@"
STUB
chmod +x "$fixture/bin/cargo"
if run_make fmt-check "FORMAT_CARGO=$fixture/bin/cargo"; then
    echo 'Failed manifest sorting was admitted' >&2; exit 1
fi
grep -q 'FAILED (exit 47,' "$fixture/last.log"
if grep -q '^fmt --all' "$fixture/calls"; then
    echo 'Workspace Rustfmt ran after manifest sorting failed' >&2; exit 1
fi
if grep -q 'Standalone Rust formatting' "$fixture/last.log"; then
    echo 'Standalone Rustfmt ran after manifest sorting failed' >&2; exit 1
fi

# Stage a clean baseline only in this disposable checkout. The real hook must
# repair a selected standalone source without exposing an unrelated working edit.
cp "$fixture/original.rs" "$fixture/work/$source_file"
git -C "$fixture/work" init -q
while IFS= read -r -d '' entry; do
    metadata="${entry%%$'\t'*}"
    path="${entry#*$'\t'}"
    read -r mode object stage <<< "$metadata"
    [[ "$stage" == 0 ]] || exit 1
    git -C "$fixture/work" update-index --add --cacheinfo "$mode,$object,$path"
    mkdir -p "$fixture/work/$path"
done < "$fixture/gitlinks"
git -C "$fixture/work" add --all
git -C "$fixture/work" -c user.name=Fixture -c user.email=fixture@example.invalid \
    -c commit.gpgsign=false -c core.hooksPath=/dev/null commit -qm baseline
printf '\nfn platform_formatting_fixture( ){ }\n' >> "$fixture/work/$source_file"
git -C "$fixture/work" add -- "$source_file"
printf '\nUnstaged fixture edit.\n' >> "$fixture/work/README.md"
cp "$fixture/work/README.md" "$fixture/working-readme"
cp "$fixture/work/Cargo.lock" "$fixture/lock"
(cd "$fixture/work" && bash .githooks/pre-commit) > "$fixture/last.log" 2>&1
grep -q 'Standalone Rust formatting.*ok' "$fixture/last.log"
cmp "$fixture/work/README.md" "$fixture/working-readme"
cmp "$fixture/work/Cargo.lock" "$fixture/lock"
git -C "$fixture/work" diff --quiet -- "$source_file"
git -C "$fixture/work" diff --cached --name-only > "$fixture/selected"
[[ "$(cat "$fixture/selected")" == "$source_file" ]]
git -C "$fixture/work" ls-files --stage -z > "$fixture/final-index"
: > "$fixture/final-gitlinks"
while IFS= read -r -d '' entry; do
    [[ "$entry" != 160000\ * ]] || printf '%s\0' "$entry" >> "$fixture/final-gitlinks"
done < "$fixture/final-index"
cmp "$fixture/gitlinks" "$fixture/final-gitlinks"
run_make fmt-check
printf 'Platform formatting: standalone check/write, configuration, failure stop and staged-hook isolation passed.\n'
