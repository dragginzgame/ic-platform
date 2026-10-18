#!/usr/bin/env bash
set -euo pipefail
# Local single-package adapter. Cargo owns packaging, verification and upload.
[[ $# == 1 ]] || exit 2
case "$1" in
    check) options=(--dry-run --allow-dirty) ;;
    publish) options=(); bash scripts/ci/check-release-source.sh ;;
    *) echo 'usage: publish.sh check|publish' >&2; exit 2 ;;
esac
version="$(bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml)"
commit="$(git rev-parse --verify HEAD)"
mkdir -p target/publication
attempt="$(mktemp -d target/publication/attempt.XXXXXX)"
printf 'Publication evidence: %s\n' "$attempt"
printf '%s\n' "$1" "$commit" "ic-platform@$version" crates-io > "$attempt/intent"
git status --porcelain=v1 > "$attempt/working-status"
args=(publish --locked --registry crates-io --package "ic-platform@$version" --all-features)
args+=("${options[@]}")
printf '%s\0' cargo "${args[@]}" > "$attempt/arguments"
if [[ "$1" == publish ]]; then
    bash scripts/ci/check-release-source.sh
    [[ "$(git rev-parse --verify HEAD)" == "$commit" ]] || {
        echo 'Publication source changed during preparation' >&2; exit 1;
    }
fi
# A dispatch without status is uncertain: inspect registry state before retrying.
touch "$attempt/dispatch"
set +e
cargo "${args[@]}" 2>&1 | tee "$attempt/cargo.log"
statuses=("${PIPESTATUS[@]}")
set -e
printf '%s\n' "${statuses[0]}" > "$attempt/cargo-status"
printf '%s\n' "${statuses[1]}" > "$attempt/log-status"
status="${statuses[0]}"
[[ "$status" != 0 ]] || status="${statuses[1]}"
printf '%s\n' "$status" > "$attempt/status"
exit "$status"
