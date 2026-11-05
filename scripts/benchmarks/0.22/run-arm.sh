#!/usr/bin/env bash
# Diagnostic only: one frozen source path, one native builder, fixed release tools.
set -euo pipefail
[[ $# == 3 || $# == 4 ]] || exit 2
base="$(cd "$1" && pwd -P)" arm="$2" version="$3" platform_source="${4:-}"
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || exit 2
root="$(cd "$(dirname "$0")/../../.." && pwd -P)"
export RUSTUP_TOOLCHAIN=1.99.1
mkdir -p "$base/$arm"
node - "$base" "$version" "$platform_source" <<'NODE'
const fs=require('fs');const [base,version,override]=process.argv.slice(2);
const root=base+'/source/Cargo.toml',auth=base+'/auth/Cargo.toml';
let s=fs.readFileSync(root,'utf8').replace(/^ic-platform = \{ version = "[^"]+", default-features = false \}$/m,`ic-platform = { version = "=${version}", default-features = false }`);
s=s.replace(/^ic-platform(?:-types|-runtime|-runtime-guard|-macros)? = \{ path = "[^"]+" \}\n/gm,'');
if(override) {
    const path=require('path');
    const owners=['ic-platform','ic-platform-types','ic-platform-runtime','ic-platform-runtime-guard','ic-platform-macros'];
    const patches=owners.map(name=>`${name} = { path = "${path.join(path.dirname(override),name)}" }`).join('\n');
    s=s.replace('[patch.crates-io]\n',`[patch.crates-io]\n${patches}\n`);
}
fs.writeFileSync(root,s);
s=fs.readFileSync(auth,'utf8').replace(/(\[dependencies.ic-platform\]\nversion = ")[^"]+("\n)/,`$1=${version}$2`);
fs.writeFileSync(auth,s);
NODE
cd "$base/source"
cargo metadata --offline --format-version 1 > "$base/$arm/metadata.json" 2> "$base/$arm/resolve.log"
cp Cargo.lock "$base/$arm/Cargo.lock"
export CARGO_TARGET_DIR="$base/build"
export RUSTFLAGS='--cfg=cpubits="32"'
# A separately labelled attribution companion, never the primary size control.
if [[ "${PLATFORM_BENCH_SYMBOLS:-0}" == 1 ]]; then
    RUSTFLAGS+=' -Cstrip=none'
fi
export PATH="/home/adam/projects/canic/.tools/ic/bin:$PATH"
builder="$base/build/debug/examples/build_artifact"
sha256sum "$builder" > "$base/$arm/builder.sha256"
read -r -a roles <<<"${PLATFORM_BENCH_ROLES:-issuer receiver}"
for role in "${roles[@]}"; do
    [[ "$role" == issuer || "$role" == receiver ]] || exit 2
    if [[ "$role" == issuer ]]; then package=delegation_issuer_stub; export_name=issuer; config=canisters/test/delegation_issuer_stub/canic.toml
    else package=local_authorization_probe; export_name=app; config=canisters/audit/local_authorization_probe/canic.toml; fi
    cargo tree --locked --offline -p "$package" --target wasm32-unknown-unknown --edges normal --prefix none > "$base/$arm/$role.graph"
    cargo tree --locked --offline -p "$package" --target wasm32-unknown-unknown --edges features --prefix none > "$base/$arm/$role.features"
    node - "$base/$arm/$role.graph" "$version" <<'NODE'
const fs=require('fs'),[graph,version]=process.argv.slice(2);
const selected=new Set(fs.readFileSync(graph,'utf8').split('\n').filter(s=>s.startsWith('ic-platform v')).map(s=>s.split(' ')[1]));
if(selected.size!==1 || !selected.has('v'+version))throw Error('Unexpected Wasm facade selection: '+[...selected]);
NODE
    "$builder" "$export_name" release "$base/source" "$base/source" "$config" --environment local \
        --release-build-id a4c128728412f11837b79ce8562e3115451db387e17361b79b4f15d02cbb36ae \
        > "$base/$arm/$role.log" 2>&1
    artifact="$base/source/.canic/release-builds/a4c128728412f11837b79ce8562e3115451db387e17361b79b4f15d02cbb36ae/artifacts/$export_name/$export_name"
    cp "$artifact.wasm" "$base/$arm/$role.wasm"
    cp "$base/build/wasm32-unknown-unknown/release/$package.wasm" "$base/$arm/$role.raw.wasm"
    for extension in did wire.json wire.ts; do
        [[ ! -f "$artifact.$extension" ]] || cp "$artifact.$extension" "$base/$arm/$role.$extension"
    done
    perl "$root/scripts/benchmarks/0.4/sections.pl" "$base/$arm/$role.wasm" > "$base/$arm/$role.sections.tsv"
    printf '%s %s %s\n' "$arm" "$role" "$(cat "$base/$arm/$role.sections.tsv")"
done
