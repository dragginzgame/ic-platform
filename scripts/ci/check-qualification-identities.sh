#!/usr/bin/env bash
# Read-only identity admission; the caller retains target/profile metadata as evidence.
set -euo pipefail
[[ $# -ge 4 && $# -le 6 ]] || { echo 'usage: check-qualification-identities.sh METADATA SOURCE_ROOT FIXTURE REQUIRED_PACKAGES [source|published [RUNTIME_VERSION]]' >&2; exit 2; }
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
source_root="$(cd "$2" && pwd -P)"
mode="${5:-source}"
case "$mode" in source) ;; published) [[ $# == 6 && -n "$6" ]] ;; *) exit 2 ;; esac
jq -e --arg fixture "$3" --arg source_root "$source_root" --arg mode "$mode" \
    --arg required "$4" --arg runtime_version "${6:-}" -f "$root/scripts/ci/qualification-identities.jq" "$1"
