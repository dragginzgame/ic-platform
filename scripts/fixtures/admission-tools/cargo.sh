#!/usr/bin/env bash
# Sourced by fake Cargo; handled commands exit, test-specific commands continue.
case "$1" in
    --version) printf 'cargo %s (fixture)\n' "${ADMISSION_COMPILER:-${TEST_MSRV:?}}"; exit 0 ;;
    locate-project) exit 0 ;;
    metadata)
        if [[ " $* " == *' --no-deps '* ]]; then
            dependencies='[]'
            if [[ "${ADMISSION_DIRECT_CBOR:-${TEST_DIRECT_CBOR:-no}}" == yes ]]; then
                dependencies='[{"name":"serde_cbor","kind":null}]'
            fi
            printf '{"version":1,"workspace_root":"%s","workspace_members":["probe"],"packages":[{"id":"probe","name":"%s","dependencies":%s}]}\n' \
                "$PWD" "$PLATFORM_ADMISSION_PACKAGE" "$dependencies"
        else printf '{}\n'; fi
        exit 0 ;;
esac
