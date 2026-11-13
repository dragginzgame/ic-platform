#!/usr/bin/env bash
set -euo pipefail
# Qualify Platform's actual Make policy and hook, never the caller's index.
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
mkdir -p "$root/target"
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$root/scripts/ci/fixture-lifecycle.sh"
fixture_complete=false
fixture="$(fixture_create "$root/target/platform-formatting.XXXXXX")"
finish() {
    local status=$?
    fixture_completion_status "$status" "$fixture_complete" || status=$?
    if [[ "$status" != 0 ]]; then fixture_replay_logs "$fixture/last.log"; fi
    fixture_finish "$status" "$fixture" 'Platform formatting evidence retained'
}
trap finish EXIT
mkdir -p "$fixture/work" "$fixture/logs" "$fixture/bin"
# Include the current reviewed snapshot and unstaged inputs, without artifacts.
bash "$root/scripts/ci/copy-platform-fixture.sh" "$root" "$fixture/work" "$fixture"
export RUNNER_TEMP="$fixture/logs"
run_make() {
    make --no-print-directory -C "$fixture/work" "$@" > "$fixture/last.log" 2>&1
}
run_make fmt-check
grep -q 'Standalone Rust formatting.*ok' "$fixture/last.log"
grep -q 'running without shortcut' "$fixture/last.log"

source_file=scripts/fixtures/type-benchmark/main.rs
cp "$fixture/work/$source_file" "$fixture/original.rs"
printf '\nfn platform_formatting_fixture( ){ }\n' >> "$fixture/work/$source_file"
if run_make fmt-check; then
    echo 'Unformatted standalone source was admitted' >&2; exit 1
fi
grep -q 'Standalone Rust formatting.*FAILED' "$fixture/last.log"
run_make fmt
run_make fmt-check
[[ ! -e "$fixture/work/target/toko-tooling/formatting.receipt" ]]

# Root templates project children during generation; every moved root and its
# separately formatted child inputs must still participate in the actual gate.
for module in canister/mod.rs driver/mod.rs wire_driver/mod.rs; do
    selected="scripts/fixtures/cdk-runtime/$module"
    cp "$fixture/work/$selected" "$fixture/layout-original.rs"
    printf '\nfn layout_formatting_fixture( ){ }\n' >> "$fixture/work/$selected"
    if run_make fmt-check; then
        echo "Unformatted Runtime template was admitted: $module" >&2; exit 1
    fi
    grep -q 'Standalone Rust formatting.*FAILED' "$fixture/last.log"
    run_make fmt
    run_make fmt-check
    cp "$fixture/layout-original.rs" "$fixture/work/$selected"
done

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
# Tool-version probes are allowed; formatting either selection must stop.
if grep -Eq '^fmt( --(all|package)( |$)|$)' "$fixture/calls"; then
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
fixture_complete=true
