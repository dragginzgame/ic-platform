// Source-bound Auth browser-codec qualification using published dependencies.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync} from 'node:fs';
import {resolve} from 'node:path';

const root = process.cwd();
const source = resolve(process.env.AUTH_SOURCE ?? '../ic-auth');
const revision = 'b78991e9833320692921bf766e51ad361c9c8c4d';
const referenceRevision = 'e52a40be75dd9809d0528189d84b43fe96952ad9';
const directory = resolve(`target/auth-wire/${Date.now()}`);
const reference = resolve('target/auth-wire/reference.git');
mkdirSync(directory, {recursive: true});
function run(command, args, cwd = directory) {
  const result = spawnSync(command, args, {cwd, encoding: 'utf8', maxBuffer: 32 * 1024 * 1024});
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stdout + result.stderr);
  return result.stdout.trim();
}
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
run('git', ['archive', `--output=${directory}/source.tar`, revision], source);
run('tar', ['-xf', 'source.tar']);
// This bare object store supplies the consumer's own read-only historical oracle.
if (!existsSync(reference)) run('git', ['init', '--bare', reference]);
const available = spawnSync('git', ['--git-dir', reference, 'cat-file', '-e', referenceRevision]);
if (available.status !== 0) run('git', ['--git-dir', reference, 'fetch', '--depth=1',
  'https://github.com/dragginzgame/ic-auth-legacy.git', referenceRevision]);
const manifest = readFileSync(`${directory}/Cargo.toml`, 'utf8');
assert(manifest.includes('ic-platform-types = { version = "0.20.4",'));
assert(manifest.includes('ic-wire = "0.24.1"'));
writeFileSync(`${directory}/Cargo.toml`, manifest
  .replace('ic-platform-types = { version = "0.20.4",', 'ic-platform-types = { version = "=0.21.1",')
  .replace('ic-wire = "0.24.1"', 'ic-wire = "=0.24.2"'));
run('cargo', ['update', '--offline', '-p', 'ic-wire']);
const metadata = JSON.parse(run('cargo', ['metadata', '--offline', '--locked', '--format-version=1']));
for (const [name, version] of [['ic-wire','0.24.2'], ['ic-platform-types','0.21.1']]) {
  const selected = metadata.packages.find(p => p.name === name && p.version === version);
  assert(selected?.source?.startsWith('registry+'), `${name} must be published`);
}
run('cargo', ['clippy', '--offline', '--locked', '-p', 'ic-auth-wire', '--all-targets', '--features', 'typescript', '--', '-D', 'warnings']);
const tests = run('cargo', ['test', '--offline', '--locked', '-p', 'ic-auth-wire', '--features', 'typescript']);
run('cargo', ['check', '--offline', '--locked', '-p', 'ic-auth-wire', '--lib', '--features', 'typescript', '--target', 'wasm32-unknown-unknown']);
run('cargo', ['run', '--offline', '--locked', '-p', 'ic-auth-wire', '--features', 'typescript', '--example', 'generate', '--', 'target/generated']);
const interoperability = run('node', ['crates/ic-auth-wire/tests/verify-reference.mjs', reference, 'target/generated']);
const modules = ['credential', 'establish', 'logout', 'wire-runtime'];
run('node', [resolve(source, 'packages/client/node_modules/typescript/bin/tsc'),
  '--target', 'ES2023', '--module', 'ESNext', '--strict', '--declaration', '--outDir', 'target/javascript',
  ...modules.map(name => `target/generated/${name}.ts`)]);
for (const name of modules.flatMap(name => [`${name}.js`, `${name}.d.ts`])) {
  assert.deepEqual(readFileSync(`${directory}/target/javascript/${name}`),
    readFileSync(`${directory}/packages/client/src/generated/${name}`), `generated client changed: ${name}`);
}
const evidence = {revision, referenceRevision, packages: {types:'0.21.1', wire:'0.24.2'},
  archiveSha256: hash(readFileSync(`${directory}/source.tar`)), lockSha256: hash(readFileSync(`${directory}/Cargo.lock`)),
  generated: Object.fromEntries(readdirSync(`${directory}/target/generated`).map(name => [name, hash(readFileSync(`${directory}/target/generated/${name}`))])),
  tests, interoperability, status:'passed'};
writeFileSync(`${directory}/verified.json`, JSON.stringify(evidence, null, 2) + '\n');
console.log(JSON.stringify({directory, ...evidence}, null, 2));
