#!/usr/bin/env bash
# Consumer integration: actual Platform setup/editor/release/publication admission,
# local Git delivery and verified package archives. Product gates and the final
# upload effect are substitutes; no real remote or registry is written.
set -euo pipefail
[[ $# -le 1 ]] || exit 2
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
lock_input="${1:-$root/Cargo.lock}"
[[ -f "$lock_input" && ! -L "$lock_input" ]] || exit 2
unset RELEASE_GROUP RELEASE_SOURCE RELEASE_COMMIT RELEASE_VERSION RELEASE_PREVIOUS
unset RELEASE_GROUPS RELEASE_PUBLISH_TARGET RELEASE_BATCH_ACTIVE
unset MAKEFLAGS MFLAGS MAKEOVERRIDES GNUMAKEFLAGS MAKEFILES RELEASE_MAKE
mkdir -p "$root/target"
fixture="$(mktemp -d "$root/target/platform-frozen-qualification.XXXXXX")"
trap 'printf "Platform frozen integration evidence: %s\n" "$fixture"' EXIT
development="$fixture/development"
mkdir "$development"
cd "$root"
sha256sum Cargo.lock > "$fixture/normal-lock.before"
git rev-parse HEAD > "$fixture/normal-head.before"
git ls-files --cached --others --exclude-standard -z > "$fixture/source-files"
while IFS= read -r -d '' path; do
    [[ -f "$path" ]] || continue
    mkdir -p "$development/$(dirname "$path")"
    cp -p "$path" "$development/$path"
done < "$fixture/source-files"
cp "$lock_input" "$development/Cargo.lock"
sha256sum "$lock_input" > "$fixture/selected-lock.sha256"
# Supply explicit fixture notes without changing the real ledgers.
for entry in types:ic-platform-types runtime:ic-platform-runtime platform:ic-platform; do
    group="${entry%:*}" package="${entry#*:}"
    previous="$(.tools/host/bin/yq -p toml -o json -r '.package.version' "$development/crates/$package/Cargo.toml")"
    next="$(perl scripts/ci/validate-release-inputs.pl next-version "$previous" patch)"
    ledger="$development/releases/$group/CHANGELOG.md"
    if ! grep -Fq "## [$next] - Unreleased" "$ledger"; then
        awk -v version="$next" '!done && /^## \[/ {print "## [" version "] - Unreleased\n\n### 🧪 Testing\n\n- Disposable frozen qualification.\n"; done=1} {print}' "$ledger" > "$fixture/ledger"
        cp "$fixture/ledger" "$ledger"
    fi
    printf '%s\t%s\n' "$group" "$next" >> "$fixture/versions.tsv"
done
export PLATFORM_FROZEN_QUALIFICATION="$fixture"
export PLATFORM_FROZEN_DEVELOPMENT="$development"
cat > "$fixture/make" <<'MAKE'
#!/usr/bin/env bash
set -euo pipefail
if [[ -n "${RELEASE_SETUP_MODE:-}" ]]; then
    for target in "$@"; do
        case "$target" in tools-check|install-tools|install-hooks|prepare-consumer-dependencies)
            printf '%s %s %s\n' "$RELEASE_GROUP" "$RELEASE_SETUP_MODE" "$target" >> "$PLATFORM_FROZEN_QUALIFICATION/setup.tsv" ;;
        esac
    done
fi
exec make "$@"
MAKE
chmod +x "$fixture/make"
export RELEASE_MAKE="$fixture/make"
# Keep the real metadata hooks. Only the costly product gate/receipt body is
# substituted, as in the existing grouped preparation integration fixture.
cat > "$development/scripts/release/validation-gate.sh" <<'GATE'
#!/usr/bin/env bash
set -euo pipefail
[[ "$1" == verify || "$1" == check ]]
! grep -q 'deferred frozen qualification' Cargo.lock
if [[ "$1" == verify ]]; then
    cargo metadata --locked --offline --format-version 1 > /dev/null
    if [[ "$RELEASE_GROUP" == types ]]; then
        printf '\n# deferred frozen qualification\n' >> "$PLATFORM_FROZEN_DEVELOPMENT/Cargo.lock"
    elif [[ "$RELEASE_GROUP" == runtime && -f "$PLATFORM_FROZEN_QUALIFICATION/fail-runtime-once" ]]; then
        rm "$PLATFORM_FROZEN_QUALIFICATION/fail-runtime-once"
        echo 'fixture: interrupted Runtime gate' >&2
        exit 47
    fi
fi
GATE
cat >> "$development/Makefile" <<'MAKE'

.PHONY: qualification-publication
qualification-publication:
	+bash scripts/ci/fixture-frozen-publication.sh
MAKE
cat > "$development/scripts/ci/fixture-frozen-publication.sh" <<'PUBLICATION'
#!/usr/bin/env bash
set -euo pipefail
[[ "$RELEASE_REMOTE" == origin && "$RELEASE_BRANCH" == main && "$RELEASE_KIND" == patch ]]
[[ "$RELEASE_COMMIT" == "$(git rev-parse HEAD)" && "$RELEASE_SOURCE" == "$(git rev-parse HEAD^)" ]]
[[ "$RELEASE_VERSION" == "$(bash scripts/release/group.sh version)" ]]
make --no-print-directory publish-check
if [[ "$RELEASE_GROUP" == types && -f "$PLATFORM_FROZEN_QUALIFICATION/fail-publication-once" ]]; then
    rm "$PLATFORM_FROZEN_QUALIFICATION/fail-publication-once"
    echo 'fixture: uncertain publication after archive receipt' >&2
    exit 48
fi
PUBLICATION
# Substitute only the terminal Cargo upload/check dispatch. The actual group
# publisher, selection admission and mandatory Clippy gate run first. Package
# all five coordinated archives together to qualify unpublished supplier edges;
# no dependency-local patch or source path is distributed.
cat > "$development/scripts/ci/run-cargo-publication.sh" <<'ARCHIVES'
#!/usr/bin/env bash
set -euo pipefail
[[ "$1" == check && "$3" == "$(git rev-parse HEAD)" && "$4" == crates-io ]]
attempt="$2" source="$3"
shift 4
mkdir -p "$attempt"
selectors=()
while [[ $# -gt 0 ]]; do
    case "$1" in
        --package) selectors+=("$2"); shift 2 ;;
        --all-features|--allow-dirty) shift ;;
        *) exit 96 ;;
    esac
done
[[ -z "$(git status --porcelain --untracked-files=all)" ]]
cargo package --workspace --all-features --locked --offline > "$attempt/archives.log" 2>&1 || { tail -50 "$attempt/archives.log"; exit 1; }
for selector in "${selectors[@]}"; do
    package="${selector%@*}" version="${selector##*@}"
    archive="target/package/$package-$version.crate"
    mkdir -p "$attempt/archives"
    retained="$attempt/archives/$package-$version.crate"
    cp "$archive" "$retained"
    cmp "$archive" "$retained"
    sha256sum "$retained" >> "$attempt/archives.sha256"
    tar -xOf "$retained" "$package-$version/.cargo_vcs_info.json" > "$attempt/$package.vcs.json"
    jq -e --arg source "$source" '.git.sha1==$source and (.git.dirty!=true)' "$attempt/$package.vcs.json" > /dev/null
    printf '%s\n' "$selector" >> "$attempt/selectors"
done
[[ -z "$(git status --porcelain --untracked-files=all)" ]]
jq -n --arg group "$RELEASE_GROUP" --arg source "$source" --rawfile selectors "$attempt/selectors" \
    '{mode:"local-archive-qualification",group:$group,source:$source,selectors:($selectors|split("\n")|map(select(length>0)))}' > "$attempt/receipt.json"
printf '%s\t%s\t%s\n' "$RELEASE_GROUP" "$source" "$attempt/receipt.json" >> "$PLATFORM_FROZEN_QUALIFICATION/publications.tsv"
ARCHIVES
git -C "$development" init -q -b main
git -C "$development" config user.name 'Platform frozen qualification'
git -C "$development" config user.email 'platform-fixture@example.invalid'
git -C "$development" add .
git -C "$development" commit -qm 'Disposable current Platform frozen integration input'
base="$(git -C "$development" rev-parse HEAD)"
git init -q --bare "$fixture/remote.git"
git -C "$development" remote add origin "$fixture/remote.git"
git -C "$development" push -qu origin main
touch "$fixture/fail-publication-once" "$fixture/fail-runtime-once"
if make --no-print-directory -C "$development" release-patch RELEASE_GROUPS=types,runtime,platform \
    RELEASE_PUBLISH_TARGET=qualification-publication > "$fixture/initial.log" 2>&1; then exit 1; fi
grep -q 'fixture: uncertain publication after archive receipt' "$fixture/initial.log" || { tail -60 "$fixture/initial.log"; exit 1; }
intents=("$development"/.git/release-state/frozen/batch-*/intent.json)
[[ ${#intents[@]} == 1 ]]
intent="${intents[0]}" batch="$(basename "$(dirname "${intents[0]}")")"
candidate="$development/target/release-frozen/$batch"
jq -e '.schema==2 and .phase=="publishing" and .index==0 and (.release_commit|test("^[0-9a-f]{40}$"))' "$intent" > /dev/null
[[ "$(wc -l < "$fixture/publications.tsv" | tr -d ' ')" == 1 ]]
if make --no-print-directory -C "$development" release-batch-resume BATCH="$batch" > "$fixture/uncertain.log" 2>&1; then exit 1; fi
grep -q 'publication outcome uncertain' "$fixture/uncertain.log"
[[ "$(wc -l < "$fixture/publications.tsv" | tr -d ' ')" == 1 ]]
# The local archive receipt above is the explicitly checked fixture substitute
# for a registry observation, not a claim that these versions were uploaded.
if make --no-print-directory -C "$development" release-batch-resume BATCH="$batch" PUBLICATION_RESULT=published > "$fixture/runtime-interruption.log" 2>&1; then exit 1; fi
grep -q 'fixture: interrupted Runtime gate' "$fixture/runtime-interruption.log" || { tail -60 "$fixture/runtime-interruption.log"; exit 1; }
CARGO_NET_OFFLINE=true make --no-print-directory -C "$development" release-batch-resume BATCH="$batch" > "$fixture/resume.log" 2>&1 || { tail -60 "$fixture/resume.log"; exit 1; }
jq -e '.schema==2 and .phase=="complete" and .index==3' "$intent" > /dev/null
[[ "$(git -C "$development" rev-parse HEAD)" == "$base" ]]
grep -q 'deferred frozen qualification' "$development/Cargo.lock"
[[ "$(cut -f1 "$fixture/publications.tsv" | paste -sd, -)" == types,runtime,platform ]]
cp "$fixture/publications.tsv" "$fixture/publications.before-complete-resume.tsv"
make --no-print-directory -C "$development" release-batch-resume BATCH="$batch" > "$fixture/completed-resume.log" 2>&1
cmp "$fixture/publications.before-complete-resume.tsv" "$fixture/publications.tsv"
make --no-print-directory -C "$development" release-batch-status BATCH="$batch" > "$fixture/status.log"
if make --no-print-directory -C "$development" release-batch-reconcile BATCH="$batch" > "$fixture/dirty-reconcile.log" 2>&1; then exit 1; fi
grep -q 'deferred edits remain' "$fixture/dirty-reconcile.log"
# Discard only the already-asserted, test-owned comment in this disposable repo.
git -C "$development" restore -- Cargo.lock
make --no-print-directory -C "$development" release-batch-reconcile BATCH="$batch" > "$fixture/reconcile.log" 2>&1
[[ "$(git -C "$development" rev-parse HEAD)" == "$(jq -r .commit "$intent")" ]]
sha256sum Cargo.lock > "$fixture/normal-lock.after"
cmp "$fixture/normal-lock.before" "$fixture/normal-lock.after"
[[ "$(git -C "$root" rev-parse HEAD)" == "$(cat "$fixture/normal-head.before")" ]]
printf '%s\n' "$candidate" > "$fixture/candidate"
echo 'Platform frozen setup, all three metadata/archive selections, interruption, uncertain publication and reconciliation passed.'
