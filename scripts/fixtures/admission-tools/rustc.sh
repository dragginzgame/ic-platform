#!/usr/bin/env bash
set -euo pipefail
if [[ "$1" == --version ]]; then
    printf 'rustc %s (fixture)\n' "${ADMISSION_COMPILER:-${TEST_MSRV:?}}"
else
    printf 'host: x86_64-unknown-linux-gnu\n'
fi
