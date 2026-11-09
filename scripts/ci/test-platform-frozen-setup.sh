#!/usr/bin/env bash
# Exercise the actual setup policy; no downloads, compilation or Git effects.
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
mkdir -p "$root/target"
fixture="$(mktemp -d "$root/target/platform-frozen-setup.XXXXXX")"
trap 'printf "Frozen setup policy evidence: %s\n" "$fixture"' EXIT
mkdir "$fixture/bin"
export FROZEN_SETUP_TRACE="$fixture/events" FROZEN_SETUP_MISSING="$fixture/missing"
cat > "$fixture/bin/make" <<'MAKE'
#!/usr/bin/env bash
set -euo pipefail
target="$2"
printf 'make %s\n' "$target" >> "$FROZEN_SETUP_TRACE"
case "$target" in
    toko-tooling-check) [[ "${FROZEN_SETUP_TOOLING_REFUSE:-0}" == 0 ]] ;;
    tools-check) [[ ! -e "$FROZEN_SETUP_MISSING" ]] ;;
    install-tools) rm "$FROZEN_SETUP_MISSING" ;;
    install-hooks|prepare-consumer-dependencies) ;;
    *) exit 97 ;;
esac
MAKE
cat > "$fixture/bin/cargo" <<'CARGO'
#!/usr/bin/env bash
[[ "$*" == 'fetch --locked' ]] || exit 98
printf 'cargo %s\n' "$*" >> "$FROZEN_SETUP_TRACE"
CARGO
# Only the read-only selected-hook query is needed by the recovery path.
cat > "$fixture/bin/git" <<'GIT'
#!/usr/bin/env bash
[[ "$*" == 'config --local --get core.hooksPath' ]] || exit 99
printf '%s\n' .githooks
GIT
chmod +x "$fixture/bin/"*
export PATH="$fixture/bin:$PATH" RELEASE_MAKE="$fixture/bin/make" RELEASE_BATCH_ACTIVE=1
for group in types runtime platform; do
    export RELEASE_GROUP="$group"
    touch "$FROZEN_SETUP_MISSING"
    : > "$FROZEN_SETUP_TRACE"
    RELEASE_SETUP_MODE=prepare bash "$root/scripts/release/candidate-setup.sh"
    printf 'make toko-tooling-check\nmake tools-check\nmake install-tools\nmake tools-check\nmake install-hooks\ncargo fetch --locked\nmake prepare-consumer-dependencies\n' > "$fixture/expected"
    cmp "$fixture/expected" "$FROZEN_SETUP_TRACE"
    : > "$FROZEN_SETUP_TRACE"
    RELEASE_SETUP_MODE=check bash "$root/scripts/release/candidate-setup.sh"
    printf 'make toko-tooling-check\nmake tools-check\n' > "$fixture/expected"
    cmp "$fixture/expected" "$FROZEN_SETUP_TRACE"
    : > "$FROZEN_SETUP_TRACE"
    RELEASE_SETUP_MODE=prepare bash "$root/scripts/release/candidate-setup.sh"
    printf 'make toko-tooling-check\nmake tools-check\nmake install-hooks\ncargo fetch --locked\nmake prepare-consumer-dependencies\n' > "$fixture/expected"
    cmp "$fixture/expected" "$FROZEN_SETUP_TRACE"
    touch "$FROZEN_SETUP_MISSING"
    : > "$FROZEN_SETUP_TRACE"
    if RELEASE_SETUP_MODE=check bash "$root/scripts/release/candidate-setup.sh"; then exit 1; fi
    printf 'make toko-tooling-check\nmake tools-check\n' > "$fixture/expected"
    cmp "$fixture/expected" "$FROZEN_SETUP_TRACE"
    rm "$FROZEN_SETUP_MISSING"
done
for mode in prepare check; do
    : > "$FROZEN_SETUP_TRACE"
    if FROZEN_SETUP_TOOLING_REFUSE=1 RELEASE_SETUP_MODE="$mode" bash "$root/scripts/release/candidate-setup.sh"; then exit 1; fi
    printf 'make toko-tooling-check\n' > "$fixture/expected"
    cmp "$fixture/expected" "$FROZEN_SETUP_TRACE"
done
: > "$FROZEN_SETUP_TRACE"
if RELEASE_GROUP=unknown RELEASE_SETUP_MODE=prepare bash "$root/scripts/release/candidate-setup.sh"; then exit 1; fi
if RELEASE_SETUP_MODE=invalid bash "$root/scripts/release/candidate-setup.sh"; then exit 1; fi
if RELEASE_BATCH_ACTIVE=0 RELEASE_SETUP_MODE=prepare bash "$root/scripts/release/candidate-setup.sh"; then exit 1; fi
[[ ! -s "$FROZEN_SETUP_TRACE" ]]
echo 'Frozen setup prepares missing tools once; recovery stays offline and refuses missing selections.'
