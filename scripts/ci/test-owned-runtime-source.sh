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
for file in mod.rs LICENSE-APACHE; do
    destination="$fixture/management-${file//./-}"
    cp -R "$root/crates/ic-platform/src/management_canister_types/maintained" "$destination"
    printf '\nmutated\n' >> "$destination/$file"
    if bash "$root/scripts/ci/check-management-source.sh" "$destination" > "$fixture/log" 2>&1; then
        echo "Unexpectedly admitted management source/license mutation: $file" >&2; exit 1
    fi
done
destination="$fixture/management-lint"
cp -R "$root/crates/ic-platform/src/management_canister_types/maintained" "$destination"
perl -i -pe 's/The upstream function-reference macro/Changed upstream function-reference macro/' "$destination/mod.rs"
if bash "$root/scripts/ci/check-management-source.sh" "$destination" > "$fixture/log" 2>&1; then
    echo 'Unexpectedly admitted management lint-normalization mutation' >&2; exit 1
fi
printf 'Management DTO source and license mutation refusals passed.\n'
