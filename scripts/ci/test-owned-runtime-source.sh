#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/platform-runtime-source-test.XXXXXX")"
trap 'rm -rf "$fixture"' EXIT
for mutation in cleanup protected license; do
    cp -R "$root/crates/ic-platform/src/cdk/upstream" "$fixture/$mutation"
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
for part in creation verification; do
    for file in mod.rs LICENSE; do
        destination="$fixture/signature-$part-${file//./-}"
        cp -R "$root/crates/ic-platform/src/signature_upstream" "$destination"
        printf '\nmutated source\n' >> "$destination/$part/$file"
        if bash "$root/scripts/ci/check-signature-source.sh" "$destination" > "$fixture/log" 2>&1; then
            echo "Unexpectedly admitted signature mutation: $part/$file" >&2; exit 1
        fi
    done
done
printf 'Signature source, vector and license mutation refusals passed.\n'
for mutation in missing duplicate moved; do
    destination="$fixture/signature-admission-$mutation"
    cp -R "$root/crates/ic-platform/src/signature_upstream" "$destination"
    case "$mutation" in
        missing) perl -i -ne 'print unless /Platform: checked DER admission\./' "$destination/creation/mod.rs" ;;
        duplicate) perl -i -pe 'print if /Platform: checked DER admission\./' "$destination/creation/mod.rs" ;;
        moved) perl -0777 -i -pe 's/(    super::admission::canister_sig_der\(pk_der\)\?; \/\/ Platform: checked DER admission\.\n)(    let oid_part = [^\n]+\n)/$2$1/' "$destination/creation/mod.rs" ;;
    esac
    if bash "$root/scripts/ci/check-signature-source.sh" "$destination" > "$fixture/log" 2>&1; then
        echo "Unexpectedly admitted changed DER admission: $mutation" >&2; exit 1
    fi
done
printf 'Missing, duplicate and moved DER admission calls are refused.\n'
