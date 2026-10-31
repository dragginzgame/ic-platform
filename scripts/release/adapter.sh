#!/usr/bin/env bash
set -euo pipefail
operation="${1:-}"
[[ $# -eq 1 ]] || exit 2
bash scripts/ci/check-release-delivery.sh ci/release-delivery.txt "${RELEASE_DELIVERY:-direct}"
prepare="${IC_PLATFORM_RELEASE_PREPARE:-0}"
unset IC_PLATFORM_RELEASE_PREPARE
export RELEASE_GROUP="${RELEASE_GROUP:-platform}"
case "$operation" in
    version|files) exec bash scripts/release/group.sh "$operation" ;;
esac
primary="$(bash scripts/release/group.sh primary)"
changelog="$(bash scripts/release/group.sh changelog)"
metadata_files="$(mktemp "${TMPDIR:-/tmp}/platform-release-files.XXXXXX")"
trap 'rm -f "$metadata_files"' EXIT
bash scripts/release/group.sh files > "$metadata_files"
source_args=()
while IFS= read -r -d '' path; do source_args+=(--allow "$path"); done < "$metadata_files"
# The shared transaction installs all selected metadata together, including the
# consumer-rendered qualification lock and pin. Render in committed source with
# the admitted working metadata overlaid; no real source is mutated until every
# candidate and semantic check succeeds.
# shellcheck source=scripts/ci/release-metadata-transaction.sh
source scripts/ci/release-metadata-transaction.sh
# shellcheck disable=SC2329 # Shared transaction callback.
render_platform_metadata() {
    local original="$1" candidate="$2" workspace path
    workspace="${original%/original}/workspace"
    mkdir "$workspace" || return
    git archive HEAD | tar -xf - -C "$workspace" || return
    while IFS= read -r -d '' path; do
        mkdir -p "$workspace/$(dirname "$path")" || return
        cp -p "$original/$path" "$workspace/$path" || return
    done < "$metadata_files"
    ln -s "$PWD/.tools" "$workspace/.tools" || return
    (
        cd "$workspace" || exit
        bash scripts/ci/cargo-release-metadata.sh prepare "$primary" || exit
        bash scripts/release/runtime-lock.sh prepare "${RELEASE_PREVIOUS:?}" || exit
        if [[ "$RELEASE_GROUP" == platform ]]; then
            bash scripts/release/runtime-pin.sh prepare "$RELEASE_PREVIOUS" || exit
        fi
        bash scripts/release/runtime-lock.sh check || exit
        bash scripts/release/runtime-pin.sh check || exit
        bash scripts/ci/cargo-release-metadata.sh check "$primary" --manifest-order-grouped
    ) || return
    while IFS= read -r -d '' path; do
        cp -p "$workspace/$path" "$candidate/$path" || return
    done < "$metadata_files"
}
# shellcheck disable=SC2329 # Shared transaction callback.
check_platform_metadata() {
    bash scripts/release/runtime-lock.sh check || return
    bash scripts/release/runtime-pin.sh check || return
    bash scripts/ci/cargo-release-metadata.sh check "$primary" --manifest-order-grouped
}
case "$operation" in
    preflight)
        bash scripts/release/group.sh ready
        bash scripts/release/runtime-lock.sh check
        bash scripts/release/runtime-pin.sh check
        bash scripts/ci/check-release-source.sh "${source_args[@]}"
        current="$(bash scripts/release/group.sh version)"
        [[ "$current" == "${RELEASE_PREVIOUS:?}" ]] || exit 1
        bash scripts/ci/check-release-changelog.sh "${RELEASE_VERSION:?}" "$current" "${RELEASE_DATE:?}" "$changelog"
        bash scripts/ci/cargo-release-preflight.sh "$PWD" "$prepare" \
            --pin ci/tool-versions.env --pin ci/release-editor.env \
            --check tools-check --prepare install-tools \
            --prepare prepare-consumer-dependencies --prepare tools-check
        ;;
    prepare)
        bash scripts/release/group.sh ready
        bash scripts/ci/check-release-source.sh "${source_args[@]}"
        current="$(bash scripts/release/group.sh version)"
        [[ -z "${RELEASE_COMMIT:-}" ]] || exit 2
        if [[ "$current" == "${RELEASE_VERSION:?}" ]]; then
            check_platform_metadata
        else
            # Tool admission is owned by preflight; candidate rendering is offline.
            source ci/release-editor.env
            CARGO_RELEASE_METADATA_EDITOR="$(bash scripts/release/editor.sh check)"
            export CARGO_RELEASE_METADATA_EDITOR
            export CARGO_RELEASE_METADATA_MAX_BYTES="$PLATFORM_RELEASE_EDITOR_MAX_BYTES"
            files=()
            while IFS= read -r -d '' path; do files+=("$path"); done < "$metadata_files"
            release_metadata_prepare "${TMPDIR:-/tmp}/platform-release-metadata.XXXXXX" \
                render_platform_metadata check_platform_metadata "${files[@]}"
        fi
        ;;
    check|commit-check)
        bash scripts/release/runtime-lock.sh check
        bash scripts/release/runtime-pin.sh check
        bash scripts/ci/check-release-source.sh "${source_args[@]}"
        bash scripts/ci/cargo-release-metadata.sh check "$primary" --manifest-order-grouped
        if [[ "$operation" == commit-check ]]; then
            [[ -z "${RELEASE_COMMIT:-}" ]] || exit 1
            while IFS= read -r -d '' path; do
                git diff --quiet -- "$path"
                git cat-file -e ":$path"
            done < "$metadata_files"
        fi
        ;;
    *) echo 'usage: adapter.sh version|files|preflight|prepare|check|commit-check' >&2; exit 2 ;;
esac
