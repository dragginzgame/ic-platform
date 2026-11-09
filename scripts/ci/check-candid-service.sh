#!/usr/bin/env bash
# Native external-service equality; setup is explicit and never part of this check.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
cd "$root"
bash scripts/dev/install-ic-tools.sh --consumer "$root" --tool didc --check
mkdir -p target
evidence="$(mktemp -d "$root/target/candid-service.XXXXXX")"
trap 'printf "Native service evidence: %s\n" "$evidence"' EXIT
PLATFORM_SERVICE_EVIDENCE="$evidence" cargo test --locked --offline -p ic-platform \
    --features candid-codec --test candid_service_consumer
"$root/.tools/ic/bin/didc" check --strict "$evidence/actual.did" "$evidence/frozen.did"
for index in 0 1 2 3 4 5; do
    if "$root/.tools/ic/bin/didc" check --strict "$evidence/actual.did" \
        "$evidence/mismatch-$index.did" > "$evidence/refusal-$index.log" 2>&1; then
        echo "didc admitted mismatched or malformed fixture $index" >&2
        exit 1
    fi
done
printf 'Pinned didc accepts the mounted external service and rejects signature, mode, arity, name and malformed DID changes.\n'
