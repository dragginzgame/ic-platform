#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "$0")/../.." && pwd -P)"
case "${1:-}" in install|check) [[ $# == 1 ]] || exit 2 ;; *) echo 'usage: wire-tools.sh install|check' >&2; exit 2 ;; esac
fixture="$root/scripts/fixtures/wire-schema"
if ! command -v node >/dev/null || ! command -v npm >/dev/null; then
    echo 'Install Node and npm versions from scripts/fixtures/wire-schema/package.json first.' >&2
    exit 1
fi
node - "$fixture/package.json" "$(npm --version)" <<'JS'
const p = require(process.argv[2]);
if (process.versions.node !== p.engines.node || process.argv[3] !== p.engines.npm) {
  console.error(`Wire tools require Node ${p.engines.node} and npm ${p.engines.npm}`);
  process.exit(1);
}
JS
if [[ "$1" == install ]]; then
    npm ci --prefix "$fixture" --ignore-scripts --no-audit --no-fund
fi
compiler="$fixture/node_modules/typescript/lib/tsc.js"
[[ -f "$compiler" ]] || { echo 'Run make install-wire-tools before validation.' >&2; exit 1; }
expected="$(node -p 'require(process.argv[1]).devDependencies.typescript' "$fixture/package.json")"
[[ "$(node "$compiler" --version)" == "Version $expected" ]] || { echo 'TypeScript version mismatch; run make install-wire-tools.' >&2; exit 1; }
echo 'Wire Node/npm/TypeScript tools admitted'
