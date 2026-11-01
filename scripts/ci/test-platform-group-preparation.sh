#!/usr/bin/env bash
# Real metadata/editor and Git delivery in disposable repositories. The fixture
# substitutes tool installation and CI/runtime qualification; it is not a release gate.
set -euo pipefail
unset RELEASE_GROUP RELEASE_COMMIT RELEASE_SOURCE IC_PLATFORM_RELEASE_PREPARE
case "$#:${1:-all}" in
    0:all) groups=(platform runtime types) ;;
    1:platform|1:runtime|1:types) groups=("$1") ;;
    *) echo "usage: $0 [platform|runtime|types]" >&2; exit 2 ;;
esac
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
bash "$root/scripts/ci/check-make-execution.sh"
unset MAKEFLAGS MFLAGS MAKEOVERRIDES
fixture="$(mktemp -d "${TMPDIR:-/tmp}/platform-group-preparation.XXXXXX")"
fixture="$(cd "$fixture" && pwd -P)"
trap 'printf "Grouped preparation evidence: %s\n" "$fixture"' EXIT
cd "$root"
source ci/release-editor.env
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
# Preserve the real Make/adapter environment and test its prepared-graph routing.
# Runtime resolution and installed behavior have their own qualification paths.
cat > "$fixture/source/scripts/ci/check-platform-runtime.sh" <<'RUNTIME'
#!/usr/bin/env bash
set -euo pipefail
[[ $# == 3 && "$1" == --timers && "$2" == --prepared-call && "$3" == --check-graph ]]
RUNTIME
# Inject failure after Shared renders core metadata, inside the consumer's outer
# transaction. The real checkout must retain every original byte.
mv "$fixture/source/scripts/release/runtime-lock.sh" "$fixture/source/scripts/release/runtime-lock-implementation.sh"
cat > "$fixture/source/scripts/release/runtime-lock.sh" <<'LOCK'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$1" == prepare && "${PLATFORM_TEST_FAIL_METADATA:-0}" == 1 ]]; then
    echo 'fixture: metadata rendering failure' >&2
    exit 37
fi
exec bash scripts/release/runtime-lock-implementation.sh "$@"
LOCK
for group in "${groups[@]}"; do
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
    platform_before="$(yq -p toml -o json -r '.package.version' crates/ic-platform/Cargo.toml)"
    types_before="$(yq -p toml -o json -r '.package.version' crates/ic-platform-types/Cargo.toml)"
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
    if PLATFORM_TEST_FAIL_METADATA=1 bash scripts/release/adapter.sh prepare > "$fixture/$group-failure.log" 2>&1; then exit 1; fi
    grep -q 'fixture: metadata rendering failure' "$fixture/$group-failure.log"
    git diff --exit-code HEAD
    bash scripts/release/adapter.sh prepare > "$fixture/$group-prepare.log" 2>&1
    git diff --exit-code -- CHANGELOG.md releases/history.md
    git diff --binary > "$fixture/$group-prepared.patch"
    bash scripts/release/adapter.sh prepare > "$fixture/$group-retry.log" 2>&1
    git diff --binary > "$fixture/$group-retry.patch"
    cmp "$fixture/$group-prepared.patch" "$fixture/$group-retry.patch"
    bash scripts/release/adapter.sh check
    # Only the selected group's member literals, catalog requirements and ledger
    # change. Crypto/certification preparation preserves Runtime and Types.
    for other_group in platform runtime types; do
        [[ "$other_group" != "$group" ]] || continue
        other_members="$(RELEASE_GROUP="$other_group" bash scripts/release/group.sh members)"
        git diff --exit-code -- "releases/$other_group/CHANGELOG.md"
        for package in $other_members; do
            git diff --exit-code -- "crates/$package/Cargo.toml"
            git show HEAD:Cargo.toml | yq -p toml -o json '.' | jq --arg package "$package" '.workspace.dependencies[$package]' > "$fixture/catalog-before"
            yq -p toml -o json '.' Cargo.toml | jq --arg package "$package" '.workspace.dependencies[$package]' > "$fixture/catalog-after"
            cmp "$fixture/catalog-before" "$fixture/catalog-after"
        done
    done
    if [[ "$group" == runtime ]]; then
        if git diff --quiet -- ci/dependency-pinning-exceptions.json; then exit 1; fi
    else
        git diff --exit-code -- ci/dependency-pinning-exceptions.json
    fi
    # Reversing only selected identities must reproduce both lockfiles exactly,
    # including supplier records and dependency references.
    for lock in scripts/fixtures/{cdk-runtime,timers-runtime}/Cargo.lock; do
        git show "HEAD:$lock" > "$fixture/lock-before"
        members="$(bash scripts/release/group.sh members)"
        # shellcheck disable=SC2086 # Group roster is space-delimited package names.
        "$editor" --max-input-bytes "$PLATFORM_RELEASE_EDITOR_MAX_BYTES" lock "$lock" "$next" "$previous" $members > "$fixture/lock-restored"
        cmp "$fixture/lock-before" "$fixture/lock-restored"
    done
    # Use real prepared metadata when group versions differ; the selected batch
    # must use each package's own identity, never the facade's workspace version.
    cargo metadata --no-deps --offline --locked --format-version 1 > "$fixture/$group-publication-metadata.json"
    members="$(bash scripts/release/group.sh members)"
    perl scripts/ci/cargo-publication-selection.pl "$fixture/$group-publication-metadata.json" \
        'ic-platform-types ic-platform-runtime-guard ic-platform-macros ic-platform-runtime ic-platform-pocket-ic ic-platform' \
        "$members" crates-io > "$fixture/$group-publication-selectors"
    tr '\0' '\n' < "$fixture/$group-publication-selectors" > "$fixture/$group-publication-lines"
    [[ "$(wc -l < "$fixture/$group-publication-lines" | tr -d ' ')" == "$(wc -w <<< "$members" | tr -d ' ')" ]]
    for package in $members; do
        grep -Fx "$package@$next" "$fixture/$group-publication-lines"
    done
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
    other=types; [[ "$group" != types ]] || other=platform
    cp ".git/release-state/$group-$next.plan" ".git/release-state/$other-$next.plan"
    if RELEASE_GROUP="$other" bash scripts/ci/run-release.sh resume "$next" origin main > "$fixture/$group-wrong-resume.log" 2>&1; then exit 1; fi
    grep -q 'release group changed' "$fixture/$group-wrong-resume.log"
    [[ "$(git rev-parse HEAD)" == "$commit" ]]
    # A saved unfinished attempt cannot be reinterpreted under a changed policy.
    cp ci/cargo-release-groups.json "$fixture/$group-policy"
    printf '\n' >> ci/cargo-release-groups.json
    if make --no-print-directory release-resume VERSION="$next" RELEASE_GROUP="$group" > "$fixture/$group-policy-refusal.log" 2>&1; then exit 1; fi
    grep -q 'Release policy changed since the saved attempt' "$fixture/$group-policy-refusal.log"
    [[ "$(git --git-dir="$fixture/$group.git" rev-parse refs/heads/main)" != "$commit" ]]
    cp "$fixture/$group-policy" ci/cargo-release-groups.json
    make --no-print-directory release-resume VERSION="$next" RELEASE_GROUP="$group" > "$fixture/$group-resume.log" 2>&1
    [[ "$(git --git-dir="$fixture/$group.git" rev-parse refs/heads/main)" == "$commit" ]]
    [[ "$(git --git-dir="$fixture/$group.git" rev-parse "$group-v$next^{commit}")" == "$commit" ]]
    [[ "$(git rev-parse HEAD)" == "$commit" ]]
    git diff --exit-code HEAD
    # A second resume reconciles the same commit/tag without another bump.
    make --no-print-directory release-resume VERSION="$next" RELEASE_GROUP="$group" > "$fixture/$group-resume-again.log" 2>&1
    [[ "$(git rev-parse HEAD)" == "$commit" ]]
done
echo 'Independent group preparation, failure/rollback, retry, exact custody and local delivery/resume passed.'
