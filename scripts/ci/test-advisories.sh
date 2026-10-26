#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
fixture="$(mktemp -d "$root/target/advisory-test.XXXXXX")"
mkdir -p "$fixture/repo/scripts/ci" "$fixture/repo/scripts/fixtures/cdk-runtime" "$fixture/bin"
cp "$root/scripts/ci/check-advisories.sh" "$fixture/repo/scripts/ci/"
printf workspace > "$fixture/repo/Cargo.lock"
printf runtime > "$fixture/repo/scripts/fixtures/cdk-runtime/Cargo.lock"
printf workspace > "$fixture/repo/Cargo.toml"
printf runtime > "$fixture/repo/scripts/fixtures/cdk-runtime/Cargo.toml"
cat > "$fixture/repo/scripts/ci/prepare-rustsec-db.sh" <<'PREPARE'
#!/usr/bin/env bash
set -euo pipefail
[[ "$2" != fail ]] || exit 7
mkdir -p "$3/db"
printf revision > "$3/revision"
PREPARE
cat > "$fixture/repo/scripts/ci/check-platform-runtime.sh" <<'RUNTIME'
#!/usr/bin/env bash
set -euo pipefail
[[ $# == 1 && "$1" == --prepare ]]
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cargo fetch --locked --manifest-path "$root/scripts/fixtures/cdk-runtime/Cargo.toml"
RUNTIME
cat > "$fixture/bin/cargo" <<'AUDIT'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$1" == fetch ]]; then
    [[ "$2" == --locked && "$3" == --manifest-path && $# == 4 ]]
    [[ "$CARGO_HOME" == */cargo-home && -d "$CARGO_HOME" ]]
    cat "$4" >> "$FETCH_CALLS"
    if [[ "${FETCH_FAILURE:-no}" == yes ]]; then exit 7; fi
    if [[ "${FETCH_MUTATE_LOCK:-no}" == yes ]]; then printf changed >> "${4%Cargo.toml}Cargo.lock"; fi
    exit 0
fi
[[ "$1" == audit ]]
if [[ "$2" == --version ]]; then echo cargo-audit-test; exit; fi
[[ "$2" == --no-fetch && "$3" == --db && "$5" == --file ]]
[[ "$7" == --color && "$8" == never && $# == 8 ]]
[[ -d "$4" ]]
cat "$6" >> "$CALLS"
echo 'warning: informational unmaintained package'
if [[ "${YANKED:-no}" == yes ]]; then echo 'warning: package was yanked'; fi
if [[ "${MISSING_INDEX:-no}" == yes ]]; then echo "error: couldn't check if the package is yanked: not found"; fi
if [[ "${INDEX_OPEN_FAILURE:-no}" == yes ]]; then echo "warning: couldn't open crates.io index: unavailable"; fi
if [[ "$6" == */workspace.lock ]]; then exit "${WORKSPACE_STATUS:-0}"; fi
AUDIT
chmod +x "$fixture/bin/cargo"
export PATH="$fixture/bin:$PATH" CALLS="$fixture/calls" FETCH_CALLS="$fixture/fetch-calls"
: > "$FETCH_CALLS"
script="$fixture/repo/scripts/ci/check-advisories.sh"
# Warnings are visible but not promoted to vulnerabilities.
bash "$script" local ok "$fixture/pass" > "$fixture/pass.log"
[[ "$(cat "$CALLS")" == workspaceruntime ]]
[[ ! -s "$FETCH_CALLS" && "$(cat "$fixture/pass/workspace.index-status")" == complete ]]
: > "$CALLS"
bash "$script" online ok "$fixture/online" > "$fixture/online.log"
[[ "$(cat "$FETCH_CALLS")" == workspaceruntime && "$(cat "$CALLS")" == workspaceruntime ]]
cmp "$fixture/repo/Cargo.lock" "$fixture/online/workspace.lock"
cmp "$fixture/repo/scripts/fixtures/cdk-runtime/Cargo.lock" "$fixture/online/runtime.lock"
: > "$CALLS"
YANKED=yes bash "$script" local ok "$fixture/yanked" > "$fixture/yanked.log"
grep -q 'package was yanked' "$fixture/yanked.log"
[[ "$(cat "$fixture/yanked/workspace.index-status")" == complete ]]
for failure in missing open; do
    : > "$CALLS"
    missing=no opening=no
    if [[ "$failure" == missing ]]; then missing=yes; else opening=yes; fi
    status=0
    MISSING_INDEX="$missing" INDEX_OPEN_FAILURE="$opening" bash "$script" local ok "$fixture/$failure" > "$fixture/$failure.log" 2>&1 || status=$?
    [[ "$status" == 1 && "$(cat "$CALLS")" == workspaceruntime ]]
    [[ "$(cat "$fixture/$failure/workspace.status")" == 0 && "$(cat "$fixture/$failure/workspace.index-status")" == incomplete ]]
done
: > "$CALLS"
status=0
WORKSPACE_STATUS=1 bash "$script" local ok "$fixture/fail" > "$fixture/fail.log" || status=$?
[[ "$status" == 1 && "$(cat "$CALLS")" == workspaceruntime ]]
[[ "$(cat "$fixture/fail/workspace.status")" == 1 && "$(cat "$fixture/fail/workspace.index-status")" == complete ]]
: > "$CALLS"
status=0
bash "$script" local fail "$fixture/preparation-failure" > "$fixture/preparation.log" 2>&1 || status=$?
[[ "$status" == 7 && ! -s "$CALLS" ]]
: > "$CALLS"
status=0
FETCH_FAILURE=yes bash "$script" online ok "$fixture/fetch-failure" > "$fixture/fetch-failure.log" 2>&1 || status=$?
[[ "$status" == 7 && ! -s "$CALLS" && "$(cat "$fixture/fetch-failure/workspace.registry-prepare.status")" == 7 ]]
status=0
FETCH_MUTATE_LOCK=yes bash "$script" online ok "$fixture/lock-change" > "$fixture/lock-change.log" 2>&1 || status=$?
[[ "$status" == 1 && ! -s "$CALLS" ]]
grep -q 'lockfile changed' "$fixture/lock-change.log"
printf 'Advisory adapter tests passed: %s\n' "$fixture"
