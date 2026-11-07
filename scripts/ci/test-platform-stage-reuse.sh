#!/usr/bin/env bash
# Real policy leaf, Shared stage proofs and complete group receipt dispatch.
# All other product stages and package metadata are disposable substitutes.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
[[ "$(uname -sm)" == 'Linux x86_64' ]] || { echo 'Policy-fixture reuse requires separate native qualification'; exit 0; }
prepared="$(bash "$root/scripts/ci/test-platform-validation-plan.sh" --prepare)"
fixture="$(printf '%s\n' "$prepared" | sed -n 's/^Platform validation-plan fixture evidence: //p')"
trap 'printf "Policy stage reuse evidence: %s\n" "$fixture"' EXIT
repo="$fixture/repo"
# The observer enumerates this complete policy closure; no real Cargo graph is
# used by the leaf, and every other gate stage remains explicitly required.
cp -R "$root/scripts/ci/." "$repo/scripts/ci/"
cp -R "$root/scripts/release/." "$repo/scripts/release/"
for directory in docs rules make .cargo; do mkdir -p "$repo/$directory"; cp -R "$root/$directory/." "$repo/$directory/"; done
cp "$root/"{.shared-tooling.snapshot,DRAGGINZGAME.md,AGENTS.md,README.md,rust-toolchain.toml} "$repo/"
# Keep the fake product stage except for the actual bounded leaf under review.
perl -0pi -e 's/\t\@bash fixture-stage.sh check-release-commands\n/\tbash scripts\/release\/check-policy-fixtures.sh\n/ or die "missing parent stage"' "$repo/Makefile"
# Preserve the real Make export and sealing dispatch. A stage's child Make
# prefixes PATH once more than the parent receipt process; direct sealing from
# that parent must refuse rather than silently rebinding its environment.
sed -n '/^export PATH :=/p' "$root/make/tools.mk" >> "$repo/Makefile"
perl -0777 -ne '/(^check-policy-fixture-evidence:\n(?:\t[^\n]*\n)+)/m or die "missing sealing dispatch"; print $1' "$root/Makefile" >> "$repo/Makefile"
mkdir "$repo/versions"
for group in platform runtime types; do printf '1.0.0\n' > "$repo/versions/$group"; done
cat > "$repo/scripts/release/group.sh" <<'GROUP'
#!/usr/bin/env bash
[[ "$1" == files ]] || exit 2
printf 'versions/%s\0' "$RELEASE_GROUP"
GROUP
cd "$repo"
git add scripts docs rules make .cargo .shared-tooling.snapshot DRAGGINZGAME.md AGENTS.md README.md rust-toolchain.toml Makefile versions
git commit -qm 'Stage reuse contract fixture'
export RELEASE_PREVIOUS=1.0.0 RELEASE_VERSION=1.0.1 RELEASE_REMOTE=origin RELEASE_BRANCH=main
qualify() {
    bash scripts/release/qualify-validation-plan.sh "$1" none release > "$fixture/$2.log" 2>&1
    receipt="$(sed -n '\|^target/release-validation/qualification\.[A-Za-z0-9]*/receipt.json$|p' "$fixture/$2.log")"
    [[ -f "$receipt" ]]
    jq -e '.schema==4 and .scope=="workspace" and .release_omission_admitted==false and (.validation_evidence.stages|length==20)' "$receipt" > /dev/null
    jq -e --arg mode "$3" '.stage_admissions["release-policy-fixtures"].admission.mode==$mode' "$receipt" > /dev/null
}
refuse() { if "$@" > "$fixture/refusal.log" 2>&1; then echo 'unexpected stage admission' >&2; exit 1; fi; }
qualify runtime executed executed
first="$receipt"
# A new source and different group still need fresh full-gate admission. Only
# unchanged policy fixtures reuse their immutable original execution.
printf '1.0.1\n' > versions/runtime
git commit -qam 'Prepared Runtime metadata'
qualify platform reused reused
second="$receipt"
jq -e --slurpfile old "$first" '.stage_admissions["release-policy-fixtures"].origin==$old[0].stage_admissions["release-policy-fixtures"].origin' "$second" > /dev/null
admit() {
    local attempt="${second%/receipt.json}"
    local runs=("$attempt"/logs/validation.*)
    RELEASE_SOURCE="$(jq -r .source "$second")" RELEASE_GROUP=platform \
        perl scripts/release/validation-receipt.pl "$attempt/plan.json" "${runs[0]}" "$second" check
}
admit > "$fixture/readmitted.log"
admission="${second%/receipt.json}/stage-admissions/release-policy-fixtures.json"
refuse perl scripts/release/policy-fixture-evidence.pl seal "$admission" "$(jq -r .source "$second")" platform
mv "$admission" "$admission.saved"
refuse admit
mv "$admission.saved" "$admission"
original_log="$(jq -r '.stage_admissions["release-policy-fixtures"].files|keys[]|select(endswith("command.stdout"))' "$second")"
cp "$original_log" "$fixture/stdout.saved"
printf 'tamper\n' >> "$original_log"
refuse admit
cp "$fixture/stdout.saved" "$original_log"
admit > "$fixture/restored.log"
# Expanded source closure invalidates reuse and a saved receipt. Run the real
# policy leaf again; this is not a version-normalization or manifest exception.
printf 'New policy input\n' > docs/new-policy-input.md
refuse admit
git add docs/new-policy-input.md
git commit -qm 'Expanded policy closure'
qualify types changed-source executed
# A changed bounded tool selection is another distinct execution context.
mkdir -p target/tool-selection
ln -s "$(command -v git)" target/tool-selection/git
PATH="$PWD/target/tool-selection:$PATH" qualify platform changed-tool executed
# Plausible output followed by a primary failure must not publish a receipt or
# admission. The edited body is a failure-injection fixture, not product proof.
cp scripts/ci/test-platform-validation-plan.sh "$fixture/plan-script.saved"
printf '#!/usr/bin/env bash\necho plausible-success\nexit 27\n' > scripts/ci/test-platform-validation-plan.sh
git commit -qam 'Failing policy leaf fixture'
refuse qualify runtime failed executed
failed_attempt="$(sed -n 's/^Group validation evidence: //p' "$fixture/failed.log")"
[[ -n "$failed_attempt" && ! -e "$failed_attempt/receipt.json" && ! -e "$failed_attempt/stage-admissions/release-policy-fixtures.json" ]]
cat > scripts/ci/test-platform-validation-plan.sh <<'INTERRUPTED'
#!/usr/bin/env bash
echo interrupted-output
kill -TERM "$PPID"
exit 143
INTERRUPTED
git commit -qam 'Interrupted policy leaf fixture'
refuse qualify runtime interrupted executed
interrupted_attempt="$(sed -n 's/^Group validation evidence: //p' "$fixture/interrupted.log")"
[[ -n "$interrupted_attempt" && ! -e "$interrupted_attempt/receipt.json" && ! -e "$interrupted_attempt/stage-admissions/release-policy-fixtures.json" ]]
cp "$fixture/plan-script.saved" scripts/ci/test-platform-validation-plan.sh
git commit -qam 'Restore policy leaf after failure'
qualify runtime recovered reused
# Structured counts/timings describe this real leaf, not the substituted gate.
jq -s '[.[]|.stage_admissions["release-policy-fixtures"].admission|{source,group,mode,elapsed_seconds,execution_proof}]' \
    "$first" "$second" "$receipt" > "$fixture/reuse-summary.json"
echo 'Complete group receipts, real leaf execution/reuse, closure/tool drift, missing/tampered evidence and failed-attempt recovery passed.'
