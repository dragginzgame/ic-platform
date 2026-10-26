#!/usr/bin/env bash
# Shared companions: scripts/ci/check-release-changelog.sh scripts/ci/fixture-lifecycle.sh
set -euo pipefail
root="${BASH_SOURCE[0]}"
[[ "$root" == /* ]] || root="$PWD/$root"
root="$(CDPATH='' cd -P "${root%/*}/../.." && printf '%s/.' "$PWD")"
root="${root%/.}"
# shellcheck source=scripts/ci/fixture-lifecycle.sh
source "$root/scripts/ci/fixture-lifecycle.sh"
fixture="$(fixture_create "${TMPDIR:-/tmp}/release-changelog.XXXXXX")"
fixture_complete=false
trap 'fixture_finish "$?" "$fixture" "Changelog preflight fixture retained"' EXIT
checker="$root/scripts/ci/check-release-changelog.sh"
for scenario in pending empty previous undated whitespace unreleased unreleased-whitespace; do
    printf '# Changelog\n\n' > "$fixture/notes"
    case "$scenario" in
        pending) printf '## [0.1.1]\n\n- Pending notes.\n\n' >> "$fixture/notes" ;;
        empty) printf '## [0.1.1]\n\n' >> "$fixture/notes" ;;
        whitespace) printf '## [0.1.1]  \t\n\n- Pending notes.  \t\n\n' >> "$fixture/notes" ;;
        unreleased) printf '## [0.1.1] - Unreleased\n\n- Pending notes.\n\n' >> "$fixture/notes" ;;
        unreleased-whitespace) printf '##\t[0.1.1]  -\tUnreleased \t\n\n- Pending notes.\n\n' >> "$fixture/notes" ;;
    esac
    printf '## [0.1.0]' >> "$fixture/notes"
    [[ "$scenario" == undated ]] || printf ' - 2026-08-01' >> "$fixture/notes"
    printf '\n\n- Historical bytes.  \t' >> "$fixture/notes"
    cp "$fixture/notes" "$fixture/original"
    chmod 0640 "$fixture/notes"
    bash "$checker" 0.1.1 0.1.0 2026-08-02 "$fixture/notes" > "$fixture/$scenario.log" 2>&1
    cmp "$fixture/original" "$fixture/notes"
    perl -e 'exit(((stat $ARGV[0])[2] & 07777) == 0640 ? 0 : 1)' "$fixture/notes"
done
# A historical unnumbered label cannot veto the current release. Exercise both
# read-only preflight and rendered preparation, preserving every historical byte.
for scenario in dated undated unreleased whitespace previous-top no-lf; do
    case "$scenario" in
        dated|previous-top|no-lf) heading='## [0.1.0] - 2026-08-01' ;;
        undated) heading='## [0.1.0]' ;;
        unreleased) heading='## [0.1.0] - Unreleased' ;;
        whitespace) heading=$'##\t[0.1.0]  -\t2026-08-01  ' ;;
    esac
    printf '%s\n\n## [Draft]\n\n- Archived wording.  \t' "$heading" > "$fixture/history"
    [[ "$scenario" == no-lf ]] || printf '\n' >> "$fixture/history"
    printf '# Changelog\n\n' > "$fixture/notes"
    [[ "$scenario" == previous-top ]] || printf '## [0.1.1]\n\n- Current change.\n\n' >> "$fixture/notes"
    cat "$fixture/history" >> "$fixture/notes"
    cp "$fixture/notes" "$fixture/original"
    bash "$checker" 0.1.1 0.1.0 2026-08-02 "$fixture/notes"
    cmp "$fixture/original" "$fixture/notes"
    printf '# Changelog\n\n## [0.1.1] - 2026-08-02\n\n' > "$fixture/expected"
    [[ "$scenario" == previous-top ]] || printf '%s\n\n' '- Current change.' >> "$fixture/expected"
    cat "$fixture/history" >> "$fixture/expected"
    awk -v version=0.1.1 -v previous=0.1.0 -v date=2026-08-02 \
        -f "$root/scripts/ci/finalize-release-changelog.awk" "$fixture/notes" > "$fixture/rendered"
    cmp "$fixture/expected" "$fixture/rendered"
    awk -v version=0.1.1 -v previous=0.1.0 -v date=2026-08-02 -v allow_finalized=1 \
        -f "$root/scripts/ci/finalize-release-changelog.awk" "$fixture/rendered" > "$fixture/recovered"
    cmp "$fixture/rendered" "$fixture/recovered"
done
for scenario in draft active-draft misplaced finalized conflicting headingless missing symlink directory; do
    printf '# Changelog\n\n' > "$fixture/notes"
    case "$scenario" in
        draft) printf '## [Draft]\n\n- Pending notes.\n' >> "$fixture/notes" ;;
        active-draft) printf '## [0.1.1]\n\n## [Draft]\n\n## [0.1.0] - 2026-08-01\n' >> "$fixture/notes" ;;
        misplaced) printf '## [0.1.0] - 2026-08-01\n\n## [0.1.1]\n\n- Pending notes.\n' >> "$fixture/notes" ;;
        finalized) printf '## [0.1.1] - 2026-08-02\n\n- Finalized notes.\n' >> "$fixture/notes" ;;
        conflicting) printf '## [0.2.0]\n\n- Public contract change.\n' >> "$fixture/notes" ;;
        missing) rm "$fixture/notes" ;;
        symlink) mv "$fixture/notes" "$fixture/target"; ln -s target "$fixture/notes" ;;
        directory) rm "$fixture/notes"; mkdir "$fixture/notes" ;;
    esac
    if bash "$checker" 0.1.1 0.1.0 2026-08-02 "$fixture/notes" > "$fixture/$scenario.log" 2>&1; then exit 1; fi
    case "$scenario" in
        missing) [[ ! -e "$fixture/notes" ]] || exit 1 ;;
        symlink) [[ -L "$fixture/notes" ]] || exit 1; rm "$fixture/notes" ;;
        directory) [[ -d "$fixture/notes" ]] || exit 1; rmdir "$fixture/notes" ;;
    esac
done
printf '# Changelog\n\n## [0.1.1]\n\n- Pending notes.\n' > "$fixture/notes"
cp "$fixture/notes" "$fixture/original"
for date in 0001-01-01 2000-02-29 2024-02-29 9999-12-31; do
    bash "$checker" 0.1.1 0.1.0 "$date" "$fixture/notes" > "$fixture/scalars.out"
    [[ ! -s "$fixture/scalars.out" ]] || exit 1
done
for date in 0000-01-01 1900-02-29 2026-02-29 2026-04-31 2026-13-01 $'2026-08-02\n'; do
    status=0
    bash "$checker" 0.1.1 0.1.0 "$date" "$fixture/notes" \
        > "$fixture/scalars.out" 2> "$fixture/scalars.err" || status=$?
    [[ "$status" == 2 && ! -s "$fixture/scalars.out" ]] || exit 1
    cmp "$fixture/original" "$fixture/notes"
done
for invalid in 18446744073709551616.0.0 0.18446744073709551616.0 0.0.18446744073709551616; do
    for field in version previous; do
        version=0.1.1 previous=0.1.0
        if [[ "$field" == version ]]; then version="$invalid"; else previous="$invalid"; fi
        status=0
        bash "$checker" "$version" "$previous" 2026-08-02 "$fixture/notes" \
            > "$fixture/scalars.out" 2> "$fixture/scalars.err" || status=$?
        [[ "$status" == 2 && ! -s "$fixture/scalars.out" ]] || exit 1
        cmp "$fixture/original" "$fixture/notes"
    done
done
printf '# Changelog\n\n## [18446744073709551615.0.1]\n\n- Exact upper bound.\n' > "$fixture/large-notes"
bash "$checker" 18446744073709551615.0.1 18446744073709551615.0.0 \
    2026-08-02 "$fixture/large-notes" > "$fixture/scalars.out"
[[ ! -s "$fixture/scalars.out" ]] || exit 1
mkdir "$fixture/bin"
for producer in awk cat; do
    printf '%s\n' '#!/usr/bin/env bash' 'printf "## [0.1.1]\\n\\n- Partial notes.\\n"' 'exit 37' > "$fixture/bin/$producer"
    chmod +x "$fixture/bin/$producer"
    status=0
    PATH="$fixture/bin:$PATH" bash "$checker" 0.1.1 0.1.0 2026-08-02 "$fixture/notes" > "$fixture/$producer.log" 2>&1 || status=$?
    [[ "$status" == 37 ]] || exit 1
    cmp "$fixture/original" "$fixture/notes"
    rm "$fixture/bin/$producer"
done
for arguments in '' '0.1.1' '0.1.1 0.1.0 invalid notes' '00.1.1 0.1.0 2026-08-02 notes'; do
    # shellcheck disable=SC2086 # Deliberate argument-count/shape subjects.
    if bash "$checker" $arguments > "$fixture/invalid.log" 2>&1; then exit 1; fi
done
[[ -z "$(find "$fixture" -name '*.tmp.*' -print)" ]] || exit 1
fixture_complete=true
echo 'Read-only changelog admission and failed-producer checks passed'
