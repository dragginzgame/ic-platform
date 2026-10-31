#!/usr/bin/env bash
# Real metadata/editor and Git delivery in disposable repositories. The fixture
# substitutes only tool installation/full CI; it is not a production release gate.
set -euo pipefail
unset RELEASE_GROUP RELEASE_COMMIT RELEASE_SOURCE IC_PLATFORM_RELEASE_PREPARE
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
bash "$root/scripts/ci/check-make-execution.sh"
unset MAKEFLAGS MFLAGS MAKEOVERRIDES
fixture="$(mktemp -d "${TMPDIR:-/tmp}/platform-group-preparation.XXXXXX")"
trap 'printf "Grouped preparation evidence: %s\n" "$fixture"' EXIT
cd "$root"
editor="$(bash scripts/release/editor.sh check)"
export PLATFORM_GROUP_TEST_EDITOR="$editor"
export PATH="$root/.tools/host/bin:$PATH"
export YQ="$root/.tools/host/bin/yq"
mkdir "$fixture/source"
# Exercise the current working source, including prepared untracked scripts.
git ls-files --cached --others --exclude-standard -z > "$fixture/source-files"
while IFS= read -r -d '' path; do
    [[ -f "$path" ]] || continue
    mkdir -p "$fixture/source/$(dirname "$path")"
    cp -p "$path" "$fixture/source/$path"
done < "$fixture/source-files"
cat > "$fixture/source/scripts/release/editor.sh" <<'EDITOR'
#!/usr/bin/env bash
set -euo pipefail
[[ "$1" == check ]]
printf '%s\n' "$PLATFORM_GROUP_TEST_EDITOR"
EDITOR
cat > "$fixture/make" <<'MAKE'
#!/usr/bin/env bash
set -euo pipefail
for argument in "$@"; do
    case "$argument" in
        tools-check) exec bash scripts/release/group.sh ready ;;
        release-verify) cargo metadata --locked --offline --format-version 1 > /dev/null; exit ;;
    esac
done
make "$@"
for argument in "$@"; do
    if [[ "$argument" == release-push-check && "${PLATFORM_GROUP_FAIL_PUSH:-0}" == 1 ]]; then
        echo 'fixture: controlled stop before local delivery' >&2
        exit 31
    fi
done
MAKE
chmod +x "$fixture/make"
export RELEASE_MAKE="$fixture/make"
for group in platform types; do
    work="$fixture/$group"
    cp -R "$fixture/source" "$work"
    cd "$work"
    ln -s "$root/.tools" .tools
    git init -q -b main
    git config user.name 'Platform release fixture'
    git config user.email 'platform-fixture@example.invalid'
    printf '/.tools\n' >> .git/info/exclude
    export RELEASE_GROUP="$group"
    previous="$(bash scripts/release/group.sh version)"
    platform_before="$(bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml)"
    kind=minor
    [[ "$group" != types ]] || kind='patch'
    next="$(perl scripts/ci/validate-release-inputs.pl next-version "$previous" "$kind")"
    ledger="$(bash scripts/release/group.sh changelog)"
    awk -v version="$next" '!done && /^## \[/ {print "## [" version "]"; done=1; next} {print}' "$ledger" > "$fixture/ledger"
    cp "$fixture/ledger" "$ledger"
    git add .
    git commit -qm 'Disposable grouped release source'
    git init -q --bare "$fixture/$group.git"
    git remote add origin "$fixture/$group.git"
    git push -qu origin main
    export RELEASE_PREVIOUS="$previous" RELEASE_VERSION="$next"
    RELEASE_DATE="$(date -u +%F)"; export RELEASE_DATE
    bash scripts/release/group.sh files > "$fixture/$group-files"
    bash scripts/release/adapter.sh prepare > "$fixture/$group-prepare.log" 2>&1
    git diff --binary > "$fixture/$group-prepared.patch"
    bash scripts/release/adapter.sh prepare > "$fixture/$group-retry.log" 2>&1
    git diff --binary > "$fixture/$group-retry.patch"
    cmp "$fixture/$group-prepared.patch" "$fixture/$group-retry.patch"
    bash scripts/release/adapter.sh check
    if [[ "$group" == types ]]; then
        git diff --exit-code -- CHANGELOG.md scripts/fixtures/cdk-runtime/Cargo.lock ci/dependency-pinning-exceptions.json
        [[ "$(bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml)" == "$platform_before" ]]
    else
        git diff --exit-code -- crates/ic-platform-types/Cargo.toml crates/ic-platform-types/CHANGELOG.md
        if git diff --quiet -- scripts/fixtures/cdk-runtime/Cargo.lock; then exit 1; fi
        if git diff --quiet -- ci/dependency-pinning-exceptions.json; then exit 1; fi
    fi
    files=()
    while IFS= read -r -d '' path; do files+=("$path"); done < "$fixture/$group-files"
    git add -- "${files[@]}"
    bash scripts/release/adapter.sh commit-check
    git restore --staged --worktree -- "${files[@]}"
    unset RELEASE_PREVIOUS RELEASE_VERSION RELEASE_DATE
    if PLATFORM_GROUP_FAIL_PUSH=1 bash scripts/ci/run-release.sh "$kind" origin main > "$fixture/$group-runner.log" 2>&1; then
        echo 'controlled pre-push failure was not observed' >&2; exit 1
    fi
    grep -q 'fixture: controlled stop before local delivery' "$fixture/$group-runner.log"
    [[ "$(git log -1 --format=%s)" == "Release $group $next" ]]
    commit="$(git rev-parse HEAD)"
    [[ "$(git rev-parse "$group-v$next^{commit}")" == "$commit" ]]
    [[ -f ".git/release-state/$group-$next.plan" ]]
    bash scripts/ci/run-release.sh resume "$next" origin main > "$fixture/$group-resume.log" 2>&1
    [[ "$(git --git-dir="$fixture/$group.git" rev-parse refs/heads/main)" == "$commit" ]]
    [[ "$(git --git-dir="$fixture/$group.git" rev-parse "$group-v$next^{commit}")" == "$commit" ]]
    [[ "$(git rev-parse HEAD)" == "$commit" ]]
    git diff --exit-code HEAD
    # A second resume reconciles the same commit/tag without another bump.
    bash scripts/ci/run-release.sh resume "$next" origin main > "$fixture/$group-resume-again.log" 2>&1
    [[ "$(git rev-parse HEAD)" == "$commit" ]]
done
echo 'Both real group preparations, retry, staged/committed checks and local delivery/resume passed.'
