#!/usr/bin/env bash
# Real Shared release journal and consumer receipt hooks; substitute metadata
# and product stage bodies. Delivery is only to disposable local bare repos.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
unset RELEASE_SOURCE RELEASE_COMMIT RELEASE_PREPARED_COMMIT RELEASE_PREPARATION_SOURCE RELEASE_GROUP
unset MAKEFLAGS MFLAGS MAKEOVERRIDES RELEASE_MAKE
prepared="$(bash "$root/scripts/ci/test-platform-validation-plan.sh" --prepare)"
fixture="$(printf '%s\n' "$prepared" | sed -n 's/^Platform validation-plan fixture evidence: //p')"
[[ -d "$fixture/repo" ]] || exit 1
trap 'printf "Release receipt recovery evidence: %s\n" "$fixture"' EXIT
seed="$fixture/repo"
cp "$root/scripts/release/validation-gate.sh" "$seed/scripts/release/"
cp "$root/scripts/ci/"{run-release.sh,check-release-tag.sh,validate-release-inputs.pl,StableVersion.pm} "$seed/scripts/ci/"
cat > "$seed/scripts/ci/cargo-release-group.sh" <<'GROUP'
#!/usr/bin/env bash
set -eu
case "$1" in
    version) cat "versions/$2" ;;
    files) printf 'versions/%s\0' "$2" ;;
    *) exit 2 ;;
esac
GROUP
cat > "$seed/scripts/release/group.sh" <<'GROUP'
#!/usr/bin/env bash
exec bash scripts/ci/cargo-release-group.sh "$1" "$RELEASE_GROUP"
GROUP
cat >> "$seed/Makefile" <<'MAKE'
export RELEASE_GROUP RELEASE_SOURCE RELEASE_PREVIOUS RELEASE_VERSION RELEASE_REMOTE RELEASE_BRANCH
release-version:
	@cat versions/$(RELEASE_GROUP)
release-preflight:
	@true
release-prepare-version:
	@bash scripts/release/validation-gate.sh check
	@printf '%s\n' '$(RELEASE_VERSION)' > versions/$(RELEASE_GROUP)
	@test "$${STOP_AFTER_PREPARE:-0}" != 1
release-prepared-check release-commit-check release-committed-check release-tagged-check:
	@bash scripts/release/validation-gate.sh check
release-push-check:
	@bash scripts/release/validation-gate.sh check
	@test "$${STOP_BEFORE_PUSH:-0}" != 1
release-files:
	@bash scripts/release/group.sh files
MAKE
# Record dispatch counts independently of retained evidence so retry proves that
# source validation is not silently rerun after the prepared journal is saved.
cat >> "$seed/fixture-stage.sh" <<'COUNT'
printf '%s\n' "$1" >> target/executed-stages
COUNT
mkdir "$seed/versions"
for group in platform runtime types; do printf '1.0.0\n' > "$seed/versions/$group"; done
printf '#!/usr/bin/env bash\nexit 0\n' > "$fixture/server"
chmod +x "$fixture/server"
export POCKET_IC_BIN="$fixture/server"
cd "$seed"
jq -cnS --arg host "$(perl scripts/release/validation-context.pl | jq -r .host_key)" '{schema:1,qualified:[$host]}' > ci/release-validation-hosts.json
git add .
git commit -qm 'Recovery fixture hooks and qualified-host substitute'
base="$(git rev-parse HEAD)"
for group in platform runtime types; do git tag -f "$group-v1.0.0" "$base" > /dev/null; done
refuse() {
    if "$@" > "$fixture/refusal.log" 2>&1; then echo 'unexpected recovery admission' >&2; exit 1; fi
}
for group in platform runtime types; do
    work="$fixture/$group"
    cp -R "$seed" "$work"
    cd "$work"
    git branch -M main
    git init -q --bare "$fixture/$group.git"
    git remote add origin "$fixture/$group.git"
    git push -qu origin main
    export RELEASE_GROUP="$group"
    case "$group" in platform) package=ic-platform ;; runtime) package=ic-platform-runtime ;; types) package=ic-platform-types ;; esac
    printf '// owner change\n' >> "crates/$package/src/lib.rs"
    git commit -qam 'Scoped owner change'
    source="$(git rev-parse HEAD)"
    FAIL_STAGE=clippy refuse bash scripts/ci/run-release.sh patch origin main
    [[ ! -f ".git/release-state/$group-1.0.1.plan" ]]
    [[ "$(cat "versions/$group")" == 1.0.0 ]]
    failure=check-release-commands; [[ "$group" != platform ]] || failure=''
    FAIL_STAGE="$failure" STOP_AFTER_PREPARE=1 refuse bash scripts/ci/run-release.sh patch origin main
    journal=".git/release-state/$group-1.0.1.validation"
    [[ "$(cat "versions/$group")" == 1.0.1 && -f "$journal" ]]
    receipt="$(cat "$journal")"
    cp "$journal" "$fixture/journal"
    cp "$receipt" "$fixture/receipt"
    attempt="${receipt%/receipt.json}"
    runs=("$attempt"/logs/validation.*)
    RELEASE_SOURCE="$source" RELEASE_PREVIOUS=1.0.0 RELEASE_VERSION=1.0.2 RELEASE_REMOTE=origin RELEASE_BRANCH=main \
        refuse perl scripts/release/validation-receipt.pl "$attempt/plan.json" "${runs[0]}" "$receipt" check
    other=platform; [[ "$group" != platform ]] || other=types
    cp "$journal" ".git/release-state/$other-1.0.1.validation"
    RELEASE_GROUP="$other" RELEASE_SOURCE="$source" RELEASE_PREVIOUS=1.0.0 RELEASE_VERSION=1.0.1 RELEASE_REMOTE=origin RELEASE_BRANCH=main \
        refuse bash scripts/release/validation-gate.sh check
    if [[ "$group" == platform ]]; then
        jq -e '.scope=="workspace" and .release_omission_admitted==false' "$receipt"
    else
        jq -e '.scope=="group" and .release_omission_admitted==true' "$receipt"
    fi
    mv "$journal" "$journal.missing"
    refuse bash scripts/ci/run-release.sh resume 1.0.1 origin main
    mv "$journal.missing" "$journal"
    printf '// worktree drift\n' >> "crates/$package/src/lib.rs"
    refuse bash scripts/ci/run-release.sh resume 1.0.1 origin main
    git add "crates/$package/src/lib.rs"
    git show "HEAD:crates/$package/src/lib.rs" > "crates/$package/src/lib.rs"
    refuse bash scripts/ci/run-release.sh resume 1.0.1 origin main
    git restore --staged --worktree "crates/$package/src/lib.rs"
    RUSTFLAGS='--cfg changed_recovery_environment' refuse bash scripts/ci/run-release.sh resume 1.0.1 origin main
    printf '\n# changed binary\n' >> "$POCKET_IC_BIN"
    refuse bash scripts/ci/run-release.sh resume 1.0.1 origin main
    printf '#!/usr/bin/env bash\nexit 0\n' > "$POCKET_IC_BIN"
    log="$(jq -r '.stages[] | select(.target=="clippy") | .log' "$receipt")"
    cp "$log" "$fixture/log"
    printf '\nchanged log\n' >> "$log"
    refuse bash scripts/ci/run-release.sh resume 1.0.1 origin main
    cp "$fixture/log" "$log"
    jq -cS '.group="wrong"' "$fixture/receipt" > "$receipt"
    refuse bash scripts/ci/run-release.sh resume 1.0.1 origin main
    cp "$fixture/receipt" "$receipt"
    git tag -f "$group-v1.0.0" "$source" > /dev/null
    refuse bash scripts/ci/run-release.sh resume 1.0.1 origin main
    git tag -f "$group-v1.0.0" "$base" > /dev/null
    cp "$journal" "$fixture/redirected-journal"
    rm "$journal"
    ln -s "$fixture/redirected-journal" "$journal"
    refuse bash scripts/ci/run-release.sh resume 1.0.1 origin main
    rm "$journal"
    cp "$fixture/journal" "$journal"
    STOP_BEFORE_PUSH=1 refuse bash scripts/ci/run-release.sh resume 1.0.1 origin main
    [[ "$(git log -1 --format=%s)" == "Release $group 1.0.1" ]]
    [[ "$(git --git-dir="$fixture/$group.git" rev-parse main)" == "$base" ]]
    before="$(wc -l < target/executed-stages)"
    bash scripts/ci/run-release.sh resume 1.0.1 origin main > "$fixture/$group-resume.log" 2>&1
    [[ "$(wc -l < target/executed-stages)" == "$before" ]]
    [[ "$(git --git-dir="$fixture/$group.git" rev-parse main)" == "$(git rev-parse HEAD)" ]]
    for other in platform runtime types; do
        [[ "$other" == "$group" || "$(cat "versions/$other")" == 1.0.0 ]]
    done
    git diff --exit-code HEAD
done
# Existing saved intents predate receipt custody. Admit only the recognized
# former complete hook; do not treat arbitrary historical validation as enough.
cp -R "$seed" "$fixture/legacy"
cd "$fixture/legacy"
rm scripts/release/validation-gate.sh
cat > Makefile <<'LEGACY'
release-verify:
	+VALIDATION_FAILURE_LOG_DIR="$$(git rev-parse --git-path release-state)/validation-failures" bash scripts/ci/run-validation-targets.sh ci
	+$(MAKE) check-runtime-all
LEGACY
git add Makefile scripts/release/validation-gate.sh
git commit -qm 'Historical full-gate fixture'
cp "$root/scripts/release/validation-gate.sh" scripts/release/
export RELEASE_GROUP=platform RELEASE_SOURCE
RELEASE_SOURCE="$(git rev-parse HEAD)"
export RELEASE_PREVIOUS=1.0.0 RELEASE_VERSION=1.0.1 RELEASE_REMOTE=origin RELEASE_BRANCH=main
bash scripts/release/validation-gate.sh check
printf 'release-verify:\n\t@true\n' > Makefile
git add Makefile
git commit -qm 'Unknown historical hook fixture'
RELEASE_SOURCE="$(git rev-parse HEAD)" refuse bash scripts/release/validation-gate.sh check
printf 'Three-group release preparation, commit/tag/push recovery and evidence refusals passed; product stages and metadata were substitutes.\n'
