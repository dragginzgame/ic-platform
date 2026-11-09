#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
mkdir -p "$root/target"
fixture="$(mktemp -d "$root/target/platform-runtime-source-test.XXXXXX")"
trap 'rm -rf "$fixture"' EXIT
for mutation in cleanup protected license; do
    cp -R "$root/crates/ic-platform-runtime/src/cdk/upstream" "$fixture/$mutation"
    case "$mutation" in
        cleanup) perl -i -pe 's/reject_message: "cleanup"/reject_message: "changed cleanup"/' "$fixture/$mutation/call/mod.rs" ;;
        protected) perl -i -pe 's/if !this.completed/if this.completed/' "$fixture/$mutation/futures/mod.rs" ;;
        license) printf '\nchanged\n' >> "$fixture/$mutation/LICENSE-APACHE" ;;
    esac
    if bash "$root/scripts/ci/check-owned-runtime-source.sh" "$fixture/$mutation" > "$fixture/log" 2>&1; then
        echo "Unexpectedly admitted runtime source mutation: $mutation" >&2; exit 1
    fi
done
printf 'Cleanup, protected-spawn and license mutation refusals passed.\n'
for module in ic0/upstream executor/runtime; do
    for file in mod.rs LICENSE-APACHE; do
        destination="$fixture/embedded-${module//\//-}-${file//./-}"
        mkdir -p "$destination/ic0" "$destination/executor"
        cp -R "$root/crates/ic-platform-runtime/src/ic0/upstream" "$destination/ic0/"
        cp -R "$root/crates/ic-platform-runtime/src/executor/runtime" "$destination/executor/"
        printf '\nmutated source\n' >> "$destination/$module/$file"
        if bash "$root/scripts/ci/check-embedded-runtime-source.sh" "$destination" > "$fixture/log" 2>&1; then
            echo "Unexpectedly admitted embedded runtime mutation: $module/$file" >&2; exit 1
        fi
    done
done
for mutation in abi lint; do
    destination="$fixture/embedded-native-$mutation"
    mkdir -p "$destination/ic0" "$destination/executor"
    cp -R "$root/crates/ic-platform-runtime/src/ic0/upstream" "$destination/ic0/"
    cp -R "$root/crates/ic-platform-runtime/src/executor/runtime" "$destination/executor/"
    case "$mutation" in
        abi) perl -i -pe 's/pub unsafe fn msg_arg_data_size\(\) -> usize/pub unsafe fn msg_arg_data_size() -> u32/' "$destination/ic0/upstream/sys/mod.rs" ;;
        lint) perl -i -pe 's/Native IC0 stubs trap/Changed IC0 stubs trap/' "$destination/ic0/upstream/sys/mod.rs" ;;
    esac
    if bash "$root/scripts/ci/check-embedded-runtime-source.sh" "$destination" > "$fixture/log" 2>&1; then
        echo "Unexpectedly admitted native IC0 mutation: $mutation" >&2; exit 1
    fi
done
printf 'Native IC0 ABI and exact lint-normalization mutation refusals passed.\n'
# Exercise failed/empty provenance observations with the real checker and source
# bytes. Only its jq executable is substituted for the explicit reader failures.
roster_root="$fixture/roster"
mkdir -p "$roster_root/scripts/ci" "$roster_root/.tools/host/bin"
cp "$root/scripts/ci/"{check-embedded-runtime-source,verify-file-checksum}.sh "$roster_root/scripts/ci/"
for module in ic0/upstream executor/runtime; do
    mkdir -p "$roster_root/crates/ic-platform-runtime/src/$module"
    cp "$root/crates/ic-platform-runtime/src/$module/provenance.json" "$roster_root/crates/ic-platform-runtime/src/$module/"
done
export PLATFORM_TEST_JQ="$root/.tools/host/bin/jq"
cat > "$roster_root/.tools/host/bin/jq" <<'JQ'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${2:-}" == .maintained_sources* ]]; then
    case "${PLATFORM_TEST_ROSTER:-}" in
        failed) exit 23 ;;
        partial) "$PLATFORM_TEST_JQ" "$@"; exit 23 ;;
    esac
fi
exec "$PLATFORM_TEST_JQ" "$@"
JQ
chmod +x "$roster_root/.tools/host/bin/jq"
for module in ic0/upstream executor/runtime; do
    manifest="$roster_root/crates/ic-platform-runtime/src/$module/provenance.json"
    original="$root/crates/ic-platform-runtime/src/$module/provenance.json"
    for invalid in empty null malformed; do
        case "$invalid" in
            empty) "$PLATFORM_TEST_JQ" '.maintained_sources = {}' "$original" > "$manifest" ;;
            null) "$PLATFORM_TEST_JQ" '.maintained_sources = null' "$original" > "$manifest" ;;
            malformed) printf '{' > "$manifest" ;;
        esac
        if bash "$roster_root/scripts/ci/check-embedded-runtime-source.sh" "$root/crates/ic-platform-runtime/src" > "$fixture/roster-$invalid.log" 2>&1; then
            echo "Unexpectedly admitted invalid source roster: $module/$invalid" >&2; exit 1
        fi
    done
    cp "$original" "$manifest"
done
for invalid in failed partial; do
    result=0
    PLATFORM_TEST_ROSTER="$invalid" bash "$roster_root/scripts/ci/check-embedded-runtime-source.sh" "$root/crates/ic-platform-runtime/src" > "$fixture/roster-$invalid.log" 2>&1 || result=$?
    [[ "$result" == 23 ]]
done
printf 'Failed, partially emitted and empty source-roster refusals passed.\n'
bash "$root/scripts/ci/test-management-source.sh"
