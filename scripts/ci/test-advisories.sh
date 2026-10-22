#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
fixture="$(mktemp -d "$root/target/advisory-test.XXXXXX")"
mkdir -p "$fixture/repo/scripts/ci" "$fixture/repo/scripts/fixtures/cdk-runtime" "$fixture/bin"
cp "$root/scripts/ci/check-advisories.sh" "$fixture/repo/scripts/ci/"
printf workspace > "$fixture/repo/Cargo.lock"
printf runtime > "$fixture/repo/scripts/fixtures/cdk-runtime/Cargo.lock"
cat > "$fixture/repo/scripts/ci/prepare-rustsec-db.sh" <<'PREPARE'
#!/usr/bin/env bash
set -euo pipefail
[[ "$2" != fail ]] || exit 7
mkdir -p "$3/db"
printf revision > "$3/revision"
PREPARE
cat > "$fixture/bin/cargo" <<'AUDIT'
#!/usr/bin/env bash
set -euo pipefail
[[ "$1" == audit ]]
if [[ "$2" == --version ]]; then echo cargo-audit-test; exit; fi
[[ "$2" == --no-fetch && "$3" == --db && "$5" == --file ]]
[[ -d "$4" ]]
cat "$6" >> "$CALLS"
echo 'warning: informational unmaintained package'
if [[ "$6" == */workspace.lock ]]; then exit "${WORKSPACE_STATUS:-0}"; fi
AUDIT
chmod +x "$fixture/bin/cargo"
export PATH="$fixture/bin:$PATH" CALLS="$fixture/calls"
script="$fixture/repo/scripts/ci/check-advisories.sh"
# Warnings are visible but not promoted to vulnerabilities.
bash "$script" local ok "$fixture/pass" > "$fixture/pass.log"
[[ "$(cat "$CALLS")" == workspaceruntime ]]
: > "$CALLS"
status=0
WORKSPACE_STATUS=1 bash "$script" local ok "$fixture/fail" > "$fixture/fail.log" || status=$?
[[ "$status" == 1 && "$(cat "$CALLS")" == workspaceruntime ]]
: > "$CALLS"
status=0
bash "$script" local fail "$fixture/preparation-failure" > "$fixture/preparation.log" 2>&1 || status=$?
[[ "$status" == 7 && ! -s "$CALLS" ]]
printf 'Advisory adapter tests passed: %s\n' "$fixture"
