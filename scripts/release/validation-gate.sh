#!/usr/bin/env bash
# Consumer receipt admission inside Shared's existing release lock and hooks.
set -euo pipefail
[[ $# == 1 && ( "$1" == verify || "$1" == check ) ]] || exit 2
cd "$(dirname "$0")/../.."
case "${RELEASE_GROUP:-}" in platform|runtime|types) ;; *) exit 2 ;; esac
[[ "${RELEASE_SOURCE:-}" =~ ^([0-9a-f]{40}|[0-9a-f]{64})$ ]] || exit 2
perl scripts/ci/validate-release-inputs.pl version "${RELEASE_VERSION:?}" > /dev/null
perl scripts/ci/validate-release-inputs.pl version "${RELEASE_PREVIOUS:?}" > /dev/null
state="$(git rev-parse --git-path release-state)"
journal="$state/$RELEASE_GROUP-$RELEASE_VERSION.validation"
if [[ "$1" == check ]] && ! git cat-file -e "$RELEASE_SOURCE:scripts/release/validation-gate.sh" 2>/dev/null; then
    # Old saved intents retain their original complete-gate semantics. Shared
    # still requires their exact source, staged tree, group and destination.
    git show "$RELEASE_SOURCE:Makefile" | perl -0777 -ne 'exit(/^release-verify:\n\t\+VALIDATION_FAILURE_LOG_DIR=[^\n]+run-validation-targets\.sh ci\n\t\+\$\(MAKE\) check-runtime-all\n/m ? 0 : 1)' || {
        echo 'Unknown historical release gate; restore its original recovery code.' >&2; exit 1;
    }
    echo 'Legacy full-gate release: no plan receipt is reinterpreted.'
    exit 0
fi
[[ -d "$state" && ! -L "$state" && ! -L "$journal" && ! -d "$journal" ]] || {
    echo 'Unsafe or missing release state directory' >&2; exit 1;
}
if [[ "$1" == verify ]]; then
    [[ "$(git rev-parse HEAD)" == "$RELEASE_SOURCE" ]] || exit 1
    baseline="$RELEASE_GROUP-v$RELEASE_PREVIOUS"
    if ! git show-ref --verify --quiet "refs/tags/$baseline"; then baseline=none; fi
    result="$(bash scripts/release/qualify-validation-plan.sh "$RELEASE_GROUP" "$baseline" release)"
    printf '%s\n' "$result"
    receipt="$(printf '%s\n' "$result" | sed -n '\|^target/release-validation/qualification\.[A-Za-z0-9]*/receipt.json$|p')"
    [[ -n "$receipt" && -f "$receipt" && "$receipt" != *$'\n'* ]] || exit 1
    temporary="$(mktemp "$journal.XXXXXX")"
    digest="$(bash scripts/ci/verify-file-checksum.sh --print sha256 "$receipt")"
    jq -cnS --arg receipt "$receipt" --arg sha256 "$digest" '{schema:1,receipt:$receipt,sha256:$sha256}' > "$temporary"
    mv "$temporary" "$journal"
else
    [[ -f "$journal" ]] || { echo 'Missing release validation receipt; restore retained evidence before resuming.' >&2; exit 1; }
    jq -e '.schema==1 and (.receipt|type=="string") and (.sha256|type=="string")' "$journal" > /dev/null
    receipt="$(jq -r .receipt "$journal")"
    [[ "$receipt" =~ ^target/release-validation/qualification\.[A-Za-z0-9]+/receipt.json$ ]] || exit 1
    [[ ! -L "$receipt" ]] || exit 1
    bash scripts/ci/verify-file-checksum.sh sha256 "$(jq -r .sha256 "$journal")" "$receipt"
    attempt="${receipt%/receipt.json}"
    runs=("$attempt"/logs/validation.*)
    [[ ${#runs[@]} == 1 && -d "${runs[0]}" ]] || exit 1
    perl scripts/release/validation-receipt.pl "$attempt/plan.json" "${runs[0]}" "$receipt" check
fi
