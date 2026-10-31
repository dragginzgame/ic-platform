#!/usr/bin/env bash
# Preserve full workspace qualification; Shared owns publication attempts/custody.
set -euo pipefail
[[ $# == 1 && ( "$1" == check || "$1" == publish ) ]] || exit 2
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
export RELEASE_GROUP="${RELEASE_GROUP:-platform}"
export CARGO_PUBLISH_PACKAGES='ic-platform-types ic-platform-runtime-guard ic-platform-macros ic-platform'
members="$(bash scripts/release/group.sh members)"
if [[ "${PACKAGE+x}" == x ]]; then
    [[ -n "$PACKAGE" ]] || { echo 'Explicit PACKAGE must not be empty' >&2; exit 2; }
    export CARGO_PUBLISH_SELECTION="$PACKAGE"
else
    export CARGO_PUBLISH_SELECTION="$members"
fi
export CARGO_PUBLISH_EVIDENCE_ROOT="${CARGO_PUBLISH_EVIDENCE_ROOT:-$root/target/publication}"
export CARGO_PUBLISH_REGISTRY=crates-io
export CARGO_PUBLISH_VERIFY_TARGETS=clippy
export CARGO_PUBLISH_MAKE="$root/scripts/release/verify-publication.sh"
export CARGO_PUBLISH_ALL_FEATURES=1
export CARGO_PUBLISH_CHECK_ALLOW_DIRTY=1
exec bash scripts/ci/cargo-publication-adapter.sh "$1"
