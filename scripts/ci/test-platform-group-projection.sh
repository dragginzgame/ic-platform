#!/usr/bin/env bash
# Owned projection/custody contract; Shared's graph admission is substituted.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
mkdir -p "$root/target"
fixture="$(mktemp -d "$root/target/group-projection.XXXXXX")"
trap 'printf "Group projection evidence: %s\n" "$fixture"' EXIT
mkdir -p "$fixture/scripts/ci" "$fixture/scripts/release" "$fixture/ci"
cp "$root/scripts/release/group.sh" "$fixture/scripts/release/"
cp "$root/ci/release-editor.env" "$fixture/ci/"
export PROJECTION_PLAN="$fixture/plan.json" PROJECTION_CALLS="$fixture/calls"
cat > "$fixture/scripts/ci/cargo-release-group.sh" <<'HELPER'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >> "$PROJECTION_CALLS"
cat "$PROJECTION_PLAN"
exit "${PROJECTION_FAILURE:-0}"
HELPER
printf '#!/bin/sh\nexit 0\n' > "$fixture/editor"
chmod +x "$fixture/editor"
printf '#!/bin/sh\nprintf "%%s\\n" "%s"\n' "$fixture/editor" > "$fixture/scripts/release/editor.sh"
for group in platform runtime types; do
    export RELEASE_GROUP="$group"
    jq -n --arg group "$group" '{group:$group,primary:("ic-platform-"+$group),previous:"1.2.3",
        changelog:("releases/"+$group+"/CHANGELOG.md"),members:["member"],
        files:(["Cargo.toml","scripts/fixtures/cdk-runtime/Cargo.lock","scripts/fixtures/timers-runtime/Cargo.lock"] +
            if $group=="runtime" then ["ci/dependency-pinning-exceptions.json"] else [] end)}' > "$PROJECTION_PLAN"
    cp "$PROJECTION_PLAN" "$fixture/valid.json"
    for operation in plan primary changelog members files ready ready-plan; do
        : > "$PROJECTION_CALLS"
        bash "$fixture/scripts/release/group.sh" "$operation" > "$fixture/$operation.out"
        [[ "$(wc -l < "$PROJECTION_CALLS")" -eq 1 ]]
    done
    cmp "$PROJECTION_PLAN" "$fixture/plan.out"
    cmp "$PROJECTION_PLAN" "$fixture/ready-plan.out"
    [[ "$(cat "$fixture/primary.out")" == "ic-platform-$group" ]]
    [[ "$(cat "$fixture/members.out")" == member ]]
    jq -jr '.files[] | .+"\u0000"' "$PROJECTION_PLAN" > "$fixture/expected-files"
    cmp "$fixture/expected-files" "$fixture/files.out"
    jq '.files |= map(select(. != "scripts/fixtures/cdk-runtime/Cargo.lock"))' "$fixture/valid.json" > "$PROJECTION_PLAN"
    for operation in plan ready ready-plan; do
        if bash "$fixture/scripts/release/group.sh" "$operation" > "$fixture/refused" 2>&1; then exit 1; fi
        grep -q 'release refused: Shared group custody' "$fixture/refused"
    done
    cp "$fixture/valid.json" "$PROJECTION_PLAN"
    result=0
    PROJECTION_FAILURE=23 bash "$fixture/scripts/release/group.sh" plan > "$fixture/refused" 2>&1 || result=$?
    [[ "$result" == 23 ]]
    jq '.previous="1.2.4"' "$fixture/valid.json" > "$PROJECTION_PLAN"
    bash "$fixture/scripts/release/group.sh" plan | jq -e '.previous=="1.2.4"' > /dev/null
done
echo 'One admitted observation per projection, fresh later observations, custody and reader refusal passed.'
