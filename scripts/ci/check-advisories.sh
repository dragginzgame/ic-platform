#!/usr/bin/env bash
set -euo pipefail
# Explicit preparation only; never mutate the user's advisory cache or lockfiles.
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
if [[ $# != 3 ]]; then
    echo 'usage: check-advisories.sh online|local SOURCE NEW_EVIDENCE_DIRECTORY' >&2
    exit 2
fi
cargo audit --version
bash "$root/scripts/ci/prepare-rustsec-db.sh" "$1" "$2" "$3"
evidence="$(cd "$3" && pwd -P)"
cargo audit --version > "$evidence/tool-version"
status=0
for name in workspace runtime; do
    lock="$root/Cargo.lock"
    if [[ "$name" == runtime ]]; then lock="$root/scripts/fixtures/cdk-runtime/Cargo.lock"; fi
    # Preserve precisely the audited input even if a maintainer subsequently updates it.
    cp "$lock" "$evidence/$name.lock"
    result=0
    cargo audit --no-fetch --db "$evidence/db" --file "$evidence/$name.lock" \
        > "$evidence/$name.log" 2>&1 || result=$?
    cat "$evidence/$name.log"
    printf '%s\n' "$result" > "$evidence/$name.status"
    if [[ "$result" != 0 ]]; then status=1; fi
done
printf 'Advisory evidence: %s (local mode does not establish freshness)\n' "$evidence"
exit "$status"
