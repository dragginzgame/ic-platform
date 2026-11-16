#!/usr/bin/env bash
# Retain only bounded, sanitized setup output; never upload raw tool output.
set -uo pipefail
phase=$1
shift
case "$phase" in *[!a-z0-9-]*|'') exit 64;; esac
root=target/ci-setup
if [[ -L target || -L "$root" ]]; then echo 'Unsafe setup evidence directory' >&2; exit 73; fi
mkdir -p "$root" || exit 73
raw=$(mktemp) || exit 73
chmod 600 "$raw"
trap 'rm -f "$raw"' EXIT
"$@" >"$raw" 2>&1
status=$?
if [[ $status -eq 0 ]]; then
  echo "Setup $phase succeeded"
  exit 0
fi
output="$root/$phase.log"
if [[ -L "$output" ]]; then echo "Evidence collection refused symlink; setup status $status" >&2; exit "$status"; fi
# Suppress credential fields and PEM bodies, remove URL credentials/queries.
# Only the last 32 KiB is considered; cap retained lines and line lengths too.
if {
  echo "Setup phase: $phase; original exit status: $status"
  echo 'Bounded final 32 KiB / 100 lines / 500 columns; sensitive lines omitted.'
  LC_ALL=C awk '
    /-----BEGIN / {pem=1}
    pem {if (/-----END /) pem=0; next}
    {lower=tolower($0)}
    lower ~ /token|password|passwd|secret|authorization|private.?key|bearer|credential|cookie/ {print "[sensitive diagnostic omitted]"; next}
    {gsub(/https?:\/\/[^ /]*@/, "https://[redacted]@"); gsub(/\?[^ ]*/, "?[redacted]"); gsub(/[[:cntrl:]]/, ""); print substr($0,1,500)}
  ' "$raw" | tail -c 32768 | tail -n 100
  [[ -s "$raw" ]] || echo 'Underlying command produced no diagnostics.'
} >"$output"; then
  echo "Setup failed; sanitized evidence: $output"
  cat "$output"
else
  echo "Evidence collection failed; original setup status: $status" >&2
fi
exit "$status"
