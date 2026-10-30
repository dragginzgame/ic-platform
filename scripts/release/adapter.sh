#!/usr/bin/env bash
set -euo pipefail
operation="${1:-}"
[[ $# -eq 1 ]] || exit 2
bash scripts/ci/check-release-delivery.sh ci/release-delivery.txt "${RELEASE_DELIVERY:-direct}"
prepare="${IC_PLATFORM_RELEASE_PREPARE:-0}"
unset IC_PLATFORM_RELEASE_PREPARE
case "$operation" in
    version) bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml ;;
    preflight)
        bash scripts/release/runtime-lock.sh check
        bash scripts/release/runtime-pin.sh check
        bash scripts/ci/check-release-source.sh --allow Cargo.toml --allow Cargo.lock --allow CHANGELOG.md --allow scripts/fixtures/cdk-runtime/Cargo.lock --allow ci/dependency-pinning-exceptions.json
        current="$(bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml)"
        [[ "$current" == "${RELEASE_PREVIOUS:?}" ]] || exit 1
        bash scripts/ci/check-release-changelog.sh "${RELEASE_VERSION:?}" "$current" "${RELEASE_DATE:?}" CHANGELOG.md
        bash scripts/ci/cargo-release-preflight.sh "$PWD" "$prepare" \
            --pin ci/tool-versions.env --pin ci/release-editor.env \
            --check tools-check --prepare install-tools \
            --prepare prepare-consumer-dependencies --prepare tools-check
        ;;
    prepare)
        bash scripts/ci/check-release-source.sh --allow Cargo.toml --allow Cargo.lock --allow CHANGELOG.md --allow scripts/fixtures/cdk-runtime/Cargo.lock --allow ci/dependency-pinning-exceptions.json
        current="$(bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml)"
        if [[ "$current" == "${RELEASE_VERSION:?}" ]]; then
            bash scripts/ci/cargo-release-metadata.sh check ic-platform --manifest-order-grouped
        else
            # Explicit setup belongs to admitted preflight; rendering is offline.
            source ci/release-editor.env
            CARGO_RELEASE_METADATA_EDITOR="$(bash scripts/release/editor.sh check)"
            export CARGO_RELEASE_METADATA_EDITOR
            export CARGO_RELEASE_METADATA_MAX_BYTES="$PLATFORM_RELEASE_EDITOR_MAX_BYTES"
            bash scripts/ci/cargo-release-metadata.sh prepare ic-platform
            bash scripts/ci/cargo-release-metadata.sh check ic-platform --manifest-order-grouped
        fi
        bash scripts/release/runtime-lock.sh prepare "${RELEASE_PREVIOUS:?}"
        bash scripts/release/runtime-pin.sh prepare "${RELEASE_PREVIOUS:?}"
        ;;
    check|commit-check)
        bash scripts/release/runtime-lock.sh check
        bash scripts/release/runtime-pin.sh check
        bash scripts/ci/check-release-source.sh --allow Cargo.toml --allow Cargo.lock --allow CHANGELOG.md --allow scripts/fixtures/cdk-runtime/Cargo.lock --allow ci/dependency-pinning-exceptions.json
        bash scripts/ci/cargo-release-metadata.sh check ic-platform --manifest-order-grouped
        if [[ "$operation" == commit-check ]]; then
            [[ -z "${RELEASE_COMMIT:-}" ]] || exit 1
            git diff --quiet -- Cargo.toml Cargo.lock CHANGELOG.md scripts/fixtures/cdk-runtime/Cargo.lock ci/dependency-pinning-exceptions.json
            for path in Cargo.toml Cargo.lock CHANGELOG.md scripts/fixtures/cdk-runtime/Cargo.lock ci/dependency-pinning-exceptions.json; do git cat-file -e ":$path"; done
        fi
        ;;
    *) echo 'usage: adapter.sh version|preflight|prepare|check|commit-check' >&2; exit 2 ;;
esac
