#!/usr/bin/env bash
# Candidate-local prerequisites only. Shared owns capture, journals and delivery.
set -euo pipefail
[[ $# == 0 ]] || exit 2
cd "$(dirname "$0")/../.."
[[ "${RELEASE_BATCH_ACTIVE:-}" == 1 ]] || { echo 'Candidate setup requires frozen release context' >&2; exit 2; }
case "${RELEASE_GROUP:-}" in types|runtime|platform) ;; *) echo 'Select a saved Platform release group' >&2; exit 2 ;; esac
case "${RELEASE_SETUP_MODE:-}" in prepare|check) ;; *) echo 'Expected frozen setup mode prepare or check' >&2; exit 2 ;; esac
make_bin="${RELEASE_MAKE:-make}"
"$make_bin" --no-print-directory toko-tooling-check
if [[ "$RELEASE_SETUP_MODE" == check ]]; then
    # No installation or graph resolution from partially prepared metadata.
    "$make_bin" --no-print-directory tools-check
    [[ "$(git config --local --get core.hooksPath)" == vendor/toko-tooling/.githooks ]] || { echo 'Candidate hook selection changed' >&2; exit 1; }
else
    # All three groups currently retain the full tool roster. Each owner admits
    # its candidate-local selection first; prepare only a missing selection.
    if ! "$make_bin" --no-print-directory tools-check; then
        "$make_bin" --no-print-directory install-tools
        "$make_bin" --no-print-directory tools-check
    fi
    "$make_bin" --no-print-directory install-hooks
    cargo fetch --locked
    "$make_bin" --no-print-directory prepare-consumer-dependencies
fi
