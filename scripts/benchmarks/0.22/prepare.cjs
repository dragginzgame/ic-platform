// Diagnostic source capture; never changes the inspected sibling checkout.
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const cp = require('child_process');
const root = path.resolve(__dirname, '../../..');
const input = process.argv[2] || '/home/adam/projects/canic';
const output = path.resolve(process.argv[3] || path.join(root, 'target/platform022-canic'));
const source = path.join(output, 'source');
const run = (args, cwd = input) => cp.execFileSync(args[0], args.slice(1), { cwd, maxBuffer: 32 * 1024 * 1024 });
const hash = b => crypto.createHash('sha256').update(b).digest('hex');
fs.mkdirSync(output, { recursive: true });
if (fs.existsSync(source)) throw Error('Refuse to overwrite an existing source capture');
const files = run(['git', 'ls-files', '--cached', '--others', '--exclude-standard', '-z']).toString().split('\0').filter(Boolean).sort();
function inventory() {
  return Object.fromEntries(files.map(p => {
    const f = path.join(input, p), s = fs.lstatSync(f, { throwIfNoEntry: false });
    if (!s || s.isDirectory()) return [p, null];
    return [p, s.isSymbolicLink() ? { link: fs.readlinkSync(f) } : hash(fs.readFileSync(f))];
  }));
}
const before = inventory();
fs.writeFileSync(path.join(output, 'source-head'), run(['git', 'rev-parse', 'HEAD']));
fs.writeFileSync(path.join(output, 'source-status.nul'), run(['git', 'status', '--porcelain=v1', '-z', '-uall']));
for (const p of files) {
  const from = path.join(input, p), to = path.join(source, p), s = fs.lstatSync(from, { throwIfNoEntry: false });
  if (!s || s.isDirectory()) continue;
  fs.mkdirSync(path.dirname(to), { recursive: true });
  if (s.isSymbolicLink()) fs.symlinkSync(fs.readlinkSync(from), to);
  else fs.copyFileSync(from, to);
}
const after = inventory();
if (JSON.stringify(before) !== JSON.stringify(after)) throw Error('Canic changed during capture');
fs.writeFileSync(path.join(output, 'source-inventory.json'), JSON.stringify(before, null, 2) + '\n');
fs.writeFileSync(path.join(output, 'source-tar-provenance.json'), JSON.stringify({ input, output, head: run(['git', 'rev-parse', 'HEAD']).toString().trim(), inventorySha256: hash(JSON.stringify(before)) }, null, 2) + '\n');
if (process.argv[4] === '--capture-only') { console.log(source); process.exit(); }
const registry = '/home/adam/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f';
fs.cpSync(path.join(registry, 'ic-auth-0.33.2'), path.join(output, 'auth'), { recursive: true });
let manifest = fs.readFileSync(path.join(source, 'Cargo.toml'), 'utf8');
const catalog = /^ic-platform = \{ version = "[^"]+", default-features = false \}$/m;
if (!catalog.test(manifest)) throw Error('Platform catalog changed; review the benchmark selection');
manifest = manifest.replace(catalog, 'ic-platform = { version = "=0.22.0", default-features = false }');
if (manifest.includes('[patch.crates-io]')) {
  const block = manifest.split('[patch.crates-io]')[1].split(/\n\[/)[0];
  for (const line of block.split('\n').filter(x => x.trim() && !x.trim().startsWith('#'))) {
    if (!/^canic(?:-cli|-control-plane|-contracts|-core|-macros|-host)? = \{ path = "crates\/canic[^"]*" \}$/.test(line)) throw Error('Unexpected source override: ' + line);
  }
  manifest = manifest.replace('[patch.crates-io]\n', '[patch.crates-io]\nic-auth = { path = "../auth" }\n');
} else manifest += '\n[patch.crates-io]\nic-auth = { path = "../auth" }\n';
fs.writeFileSync(path.join(source, 'Cargo.toml'), manifest);
const authManifest = path.join(output, 'auth/Cargo.toml');
const authText = fs.readFileSync(authManifest, 'utf8').replace(
  /(\[dependencies.ic-platform\]\nversion = ")[^"]+("\n)/, '$1=0.22.0$2');
fs.writeFileSync(authManifest, authText);
// Only the benchmark host driver selects explicit, already admitted tools.
const driver = path.join(source, 'crates/canic-host/examples/build_artifact.rs');
let text = fs.readFileSync(driver, 'utf8');
const old = 'let output = build_workspace_canister_artifact(&context)?;';
if (!text.includes(old)) throw Error('Host driver changed; review its public API');
text = text.replace(old, `let toolchain = canic_host::build_toolchain::BuildToolchain::resolve_with_paths(
        profile,
        Some(Path::new("${input}/.tools/ic/bin/ic-wasm")),
        Some(Path::new("${input}/.tools/ic/bin/wasm-opt")),
    )?;
    let output = canic_host::canister_build::CanisterArtifactBuilder::with_toolchain(toolchain)
        .build_workspace_canister_artifact(&context)?;`);
text = text.replace('CanisterBuildProfile, WorkspaceBuildContext, build_workspace_canister_artifact,', 'CanisterBuildProfile, WorkspaceBuildContext,');
fs.writeFileSync(driver, text);
fs.writeFileSync(path.join(output, 'canic-manifest.original'), fs.readFileSync(path.join(source, 'Cargo.toml')));
fs.writeFileSync(path.join(output, 'auth-manifest.original'), fs.readFileSync(path.join(output, 'auth/Cargo.toml')));
console.log(source);
