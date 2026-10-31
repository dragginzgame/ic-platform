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
        if [[ "$current" == "${RELEASE_VERSION:?}" ]]; then
            bash scripts/ci/cargo-release-metadata.sh check "$primary" --manifest-order-grouped
        else
            # Explicit setup belongs to admitted preflight; rendering is offline.
            source ci/release-editor.env
            CARGO_RELEASE_METADATA_EDITOR="$(bash scripts/release/editor.sh check)"
            export CARGO_RELEASE_METADATA_EDITOR
            export CARGO_RELEASE_METADATA_MAX_BYTES="$PLATFORM_RELEASE_EDITOR_MAX_BYTES"
            bash scripts/ci/cargo-release-metadata.sh prepare "$primary"
            bash scripts/ci/cargo-release-metadata.sh check "$primary" --manifest-order-grouped
        fi
        if [[ "$RELEASE_GROUP" == platform ]]; then
            bash scripts/release/runtime-lock.sh prepare "${RELEASE_PREVIOUS:?}"
            bash scripts/release/runtime-pin.sh prepare "${RELEASE_PREVIOUS:?}"
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
