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
# Capture both inputs before registry preparation; audit only these snapshots.
for name in workspace runtime; do
    lock="$root/Cargo.lock"
    if [[ "$name" == runtime ]]; then lock="$root/scripts/fixtures/cdk-runtime/Cargo.lock"; fi
    cp "$lock" "$evidence/$name.lock"
done
if [[ "$1" == online ]]; then
    # --no-fetch also disables cargo-audit's index update. Prepare both locked
    # workspaces explicitly, in a fresh Cargo home so records are current and
    # neither preparation nor auditing contends with the user's package cache.
    export CARGO_HOME="$evidence/cargo-home"
    mkdir "$CARGO_HOME"
    for name in workspace runtime; do
        result=0
        lock="$root/Cargo.lock"
        if [[ "$name" == runtime ]]; then
            lock="$root/scripts/fixtures/cdk-runtime/Cargo.lock"
            # The qualification workspace is generated from maintained templates.
            # Reuse its existing prepare mode instead of inventing another renderer.
            bash "$root/scripts/ci/check-platform-runtime.sh" --prepare > "$evidence/$name.registry-prepare.log" 2>&1 || result=$?
        else
            cargo fetch --locked --manifest-path "$root/Cargo.toml" > "$evidence/$name.registry-prepare.log" 2>&1 || result=$?
        fi
        cat "$evidence/$name.registry-prepare.log"
        printf '%s\n' "$result" > "$evidence/$name.registry-prepare.status"
        [[ "$result" == 0 ]] || exit "$result"
        cmp "$lock" "$evidence/$name.lock" || {
            echo "The $name lockfile changed during registry preparation" >&2; exit 1;
        }
    done
fi
status=0
for name in workspace runtime; do
    result=0
    cargo audit --no-fetch --db "$evidence/db" --file "$evidence/$name.lock" --color never \
        > "$evidence/$name.log" 2>&1 || result=$?
    cat "$evidence/$name.log"
    printf '%s\n' "$result" > "$evidence/$name.status"
    if [[ "$result" != 0 ]]; then status=1; fi
    # cargo-audit 0.22.2 can print index lookup errors without a failing exit.
    # Preserve its status separately, but never admit that as complete coverage.
    coverage=complete
    if grep -Eq "couldn't (check if the package is yanked|open crates.io index|update crates.io index)" "$evidence/$name.log"; then
        coverage=incomplete
        status=1
        echo "The $name yanked-package observation is incomplete" >&2
    fi
    printf '%s\n' "$coverage" > "$evidence/$name.index-status"
done
printf 'Advisory evidence: %s (local mode does not establish freshness)\n' "$evidence"
exit "$status"
