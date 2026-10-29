#!/usr/bin/env bash
set -euo pipefail
root="${BASH_SOURCE[0]}"
[[ "$root" == /* ]] || root="$PWD/$root"
root="$(CDPATH='' cd -P "${root%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$root/scripts/ci/fixture-lifecycle.sh"
fixture="$(fixture_create "${TMPDIR:-/tmp}/editor-bootstrap.XXXXXX")"
fixture_complete=false
trap 'fixture_finish "$?" "$fixture" "Editor bootstrap fixture retained"' EXIT
consumer="$fixture/consumer space"
mkdir -p "$consumer/scripts/release" "$consumer/scripts/dev" "$consumer/ci" "$fixture/elsewhere"
cp "$root/scripts/release/editor.sh" "$consumer/scripts/release/"
cp "$root/ci/release-editor.env" "$consumer/ci/"
cat > "$consumer/scripts/dev/install-rust-tools.sh" <<'STUB'
set -euo pipefail
[[ "$1" == --consumer && "$2" == "$EDITOR_EXPECT_ROOT" ]]
printf '%s\n' "$@" > "$EDITOR_OBSERVATION"
exit "${EDITOR_TEST_STATUS:-0}"
STUB
export EDITOR_EXPECT_ROOT="$consumer" EDITOR_OBSERVATION="$fixture/arguments"
for mode in install check; do
    for entry in relative absolute; do
        (
            export CDPATH=.
            if [[ "$entry" == relative ]]; then
                cd "$consumer"
                bash scripts/release/editor.sh "$mode"
            else
                cd "$fixture/elsewhere"
                bash "$consumer/scripts/release/editor.sh" "$mode"
            fi
        ) > "$fixture/output" 2> "$fixture/error"
        [[ ! -s "$fixture/output" && ! -s "$fixture/error" ]]
        grep -Fx -- ic-build-release-metadata "$EDITOR_OBSERVATION" > /dev/null
        [[ "$(awk 'previous == "--features" { print } { previous = $0 }' "$EDITOR_OBSERVATION")" == cli ]]
        if [[ "$mode" == check ]]; then
            [[ "$(tail -1 "$EDITOR_OBSERVATION")" == --check ]]
        elif grep -Fx -- --check "$EDITOR_OBSERVATION"; then exit 1; fi
    done
done
status=0
EDITOR_TEST_STATUS=37 bash "$consumer/scripts/release/editor.sh" check || status=$?
[[ "$status" == 37 ]]
fixture_complete=true
echo 'Release editor relative/absolute paths, inherited CDPATH and dispatcher status passed'
