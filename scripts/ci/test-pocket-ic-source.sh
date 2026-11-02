#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
mkdir -p "$root/target"
fixture="$(mktemp -d "$root/target/pocket-ic-source-test.XXXXXX")"
trap 'rm -rf "$fixture"' EXIT
bash "$root/scripts/ci/check-pocket-ic-source.sh"
for mutation in import body license; do
    cp -R "$root/crates/ic-platform-pocket-ic" "$fixture/$mutation"
    case "$mutation" in
        import) perl -0777 -i -pe 's/ic_management_canister_types::/changed_management_types::/ == 1 or die "missing import\n"' "$fixture/$mutation/src/lib.rs" ;;
        body) printf '\nfn altered_client_body() {}\n' >> "$fixture/$mutation/src/nonblocking.rs" ;;
        license) printf '\nchanged license\n' >> "$fixture/$mutation/LICENSE" ;;
    esac
    if bash "$root/scripts/ci/check-pocket-ic-source.sh" "$fixture/$mutation" > "$fixture/refusal.log" 2>&1; then
        echo "Unexpectedly admitted PocketIC mutation: $mutation" >&2; exit 1
    fi
done
echo 'PocketIC management adaptation passes; changed imports, bodies and licenses refuse.'
