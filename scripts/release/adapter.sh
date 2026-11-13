#!/usr/bin/env bash
set -euo pipefail
operation="${1:-}"
[[ $# -eq 1 ]] || exit 2
prepare="${IC_PLATFORM_RELEASE_PREPARE:-0}"
unset IC_PLATFORM_RELEASE_PREPARE
# Frozen setup already prepared this candidate's selections before version
# lookup. The ordinary preflight still admits them; it must not install again.
if [[ "${RELEASE_BATCH_ACTIVE:-}" == 1 ]]; then prepare=0; fi
export RELEASE_GROUP
case "$operation" in
    version|files) exec bash scripts/release/group.sh "$operation" ;;
esac
make --no-print-directory toko-tooling-check
projection=plan
case "$operation" in preflight|prepare) projection=ready-plan ;; esac
plan="$(bash scripts/release/group.sh "$projection")"
primary="$(jq -er .primary <<< "$plan")"
bash vendor/toko-tooling/scripts/ci/check-release-delivery.sh ci/release-delivery.txt "${RELEASE_DELIVERY:-direct}"
changelog="$(jq -er .changelog <<< "$plan")"
mkdir -p target
metadata_files="$(mktemp "$PWD/target/platform-release-files.XXXXXX")"
trap 'rm -f "$metadata_files"' EXIT
jq -jr '.files[] | . + "\u0000"' <<< "$plan" > "$metadata_files"
source_args=()
while IFS= read -r -d '' path; do source_args+=(--allow "$path"); done < "$metadata_files"
# The shared transaction installs all selected metadata together, including the
# consumer-rendered qualification lock and pin. Render in committed source with
# the admitted working metadata overlaid; no real source is mutated until every
# candidate and semantic check succeeds.
# shellcheck source=vendor/toko-tooling/scripts/ci/release-metadata-transaction.sh
source vendor/toko-tooling/scripts/ci/release-metadata-transaction.sh
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
        cd -P "$workspace" || exit
        bash vendor/toko-tooling/scripts/ci/cargo-release-metadata.sh prepare "$primary" || exit
        bash scripts/release/runtime-lock.sh prepare "${RELEASE_PREVIOUS:?}" || exit
        if [[ "$RELEASE_GROUP" == runtime ]]; then
            bash scripts/release/runtime-pin.sh prepare "$RELEASE_PREVIOUS" || exit
        fi
        # A minor owner bump may leave a native supplier needing the preceding
        # registry owner. Preserve foreign pins; resolve that distinction offline
        # inside the transaction before installing the qualification lock.
        CARGO_NET_OFFLINE=true bash scripts/ci/check-platform-runtime.sh --refresh-lock || exit
        # The independent benchmark keeps an exact registry baseline alongside
        # local product paths, so it is not a coordinated qualification cohort.
        # Refresh only its local identities within the same metadata transaction.
        cargo metadata --manifest-path qualification/Cargo.toml --offline --format-version 1 > /dev/null || exit
        # Match the normal commit hook before the transaction binds candidate
        # bytes. The metadata editor can compact arrays that cargo-sort expands.
        cargo sort --workspace --grouped || exit
        bash scripts/release/runtime-lock.sh check || exit
        bash scripts/release/runtime-pin.sh check || exit
        bash vendor/toko-tooling/scripts/ci/cargo-release-metadata.sh check "$primary" --manifest-order-grouped
    ) || return
    while IFS= read -r -d '' path; do
        cp -p "$workspace/$path" "$candidate/$path" || return
    done < "$metadata_files"
}
# shellcheck disable=SC2329 # Shared transaction callback.
check_platform_metadata() {
    bash scripts/release/runtime-lock.sh check || return
    bash scripts/release/runtime-pin.sh check || return
    cargo metadata --manifest-path qualification/Cargo.toml --locked --offline --format-version 1 > /dev/null || return
    bash vendor/toko-tooling/scripts/ci/cargo-release-metadata.sh check "$primary" --manifest-order-grouped
}
case "$operation" in
    preflight)
        bash scripts/release/runtime-lock.sh check
        bash scripts/release/runtime-pin.sh check
        bash vendor/toko-tooling/scripts/ci/check-release-source.sh "${source_args[@]}"
        current="$(jq -er .previous <<< "$plan")"
        [[ "$current" == "${RELEASE_PREVIOUS:?}" ]] || exit 1
        bash vendor/toko-tooling/scripts/ci/check-release-changelog.sh "${RELEASE_VERSION:?}" "$current" "${RELEASE_DATE:?}" "$changelog"
        bash vendor/toko-tooling/scripts/ci/cargo-release-preflight.sh "$PWD" "$prepare" \
            --pin vendor/toko-tooling/ci/tool-versions.env --pin ci/release-editor.env \
            --check tools-check --prepare install-tools \
            --prepare prepare-consumer-dependencies --prepare tools-check
        ;;
    prepare)
        bash scripts/release/validation-gate.sh check
        bash vendor/toko-tooling/scripts/ci/check-release-source.sh "${source_args[@]}"
        current="$(jq -er .previous <<< "$plan")"
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
        bash scripts/release/validation-gate.sh check
        bash scripts/release/runtime-lock.sh check
        bash scripts/release/runtime-pin.sh check
        bash vendor/toko-tooling/scripts/ci/check-release-source.sh "${source_args[@]}"
        bash vendor/toko-tooling/scripts/ci/cargo-release-metadata.sh check "$primary" --manifest-order-grouped
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
