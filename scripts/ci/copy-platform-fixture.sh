#!/usr/bin/env bash
# Copy current tracked/untracked regular inputs, preserving opaque gitlink records.
set -euo pipefail
[[ $# == 3 ]] || { echo 'usage: copy-platform-fixture.sh ROOT DESTINATION RECORDS' >&2; exit 2; }
root="$1" destination="$2" records="$3"
mkdir -p "$destination" "$records"
git -C "$root" ls-files --stage -z > "$records/index"
: > "$records/files"
: > "$records/gitlinks"
while IFS= read -r -d '' entry; do
    metadata="${entry%%$'\t'*}" path="${entry#*$'\t'}"
    read -r mode _ stage <<< "$metadata"
    [[ "$stage" == 0 ]] || { echo 'Unmerged fixture input' >&2; exit 1; }
    case "$mode" in
        100644|100755) [[ ! -e "$root/$path" ]] || printf '%s\0' "$path" >> "$records/files" ;;
        160000) printf '%s\0' "$entry" >> "$records/gitlinks" ;;
        *) echo 'Platform fixtures require regular files or opaque gitlinks' >&2; exit 1 ;;
    esac
done < "$records/index"
git -C "$root" ls-files --others --exclude-standard -z >> "$records/files"
while IFS= read -r -d '' path; do
    [[ -f "$root/$path" && ! -L "$root/$path" ]] || {
        echo 'Platform fixture input is not a regular file' >&2; exit 1;
    }
done < "$records/files"
(cd "$root" && tar --null -T "$records/files" -cf -) |
    (cd "$destination" && tar -xf -)
