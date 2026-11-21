// Qualify the pinned real Auth fixture without modifying its checkout.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {mkdirSync, readFileSync, writeFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {resolve} from 'node:path';
const root = process.cwd();
assert(process.env.POCKET_IC_SERVER_URL, 'run through make qualify-auth-runtime to supply the owned server');
function run(command, args, cwd, env = process.env) {
  const result = spawnSync(command, args, {cwd, env, encoding: 'utf8', maxBuffer: 32 * 1024 * 1024});
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stdout + result.stderr);
  return result.stdout.trim();
}
const directory = run('node', ['qualification/auth-runtime/prepare.mjs'], root);
console.log(`Pinned consumer: ${directory}`);
const metadata = JSON.parse(run('cargo', ['metadata', '--manifest-path', 'qualification/Cargo.toml', '--format-version=1'], directory));
for (const [name,version] of [['ic-platform','0.34.19'],['ic-platform-types','0.21.4'],['ic-platform-runtime','0.23.24']]) {
  const packages = metadata.packages.filter(p => p.name === name);
  assert.equal(packages.length,1,`one canonical ${name}`);
  assert.equal(packages[0].version,version);
  assert.equal(packages[0].source,null);
  assert(packages[0].manifest_path.startsWith(`${root}/crates/`));
}
assert(!metadata.packages.some(p => p.name === 'ic-agent'));
const vectors = resolve(directory, 'qualification/target/host-vectors/memory.tsv');
mkdirSync(resolve(vectors, '..'), {recursive:true});
writeFileSync(vectors, run('node', ['crates/ic-auth/tests/verify-memory-fixtures.mjs', '--host'], directory) + '\n');
const env = {...process.env,
  AUTH_HOST_CARGO: run('rustup', ['which', 'cargo'], directory),
  AUTH_HOST_RUSTC: run('rustup', ['which', 'rustc'], directory),
  AUTH_HOST_PACKAGE_ID: run('cargo', ['pkgid', '--locked', '--manifest-path', 'qualification/Cargo.toml', '-p', 'ic-auth-host-fixture'], directory),
  AUTH_HOST_VECTORS: vectors,
};
const tests = run('cargo', ['test', '--locked', '--manifest-path', 'qualification/Cargo.toml', '-p', 'ic-auth-host-conformance', '--test', 'installed', '--', '--ignored'], directory, env);
const hash = file => createHash('sha256').update(readFileSync(file)).digest('hex');
writeFileSync(`${directory}/verified.json`,JSON.stringify({
  source:JSON.parse(readFileSync(`${directory}/source.json`)),
  patchSha256:hash(`${root}/qualification/auth-crypto/consumer.patch`),
  lockSha256:hash(`${directory}/qualification/Cargo.lock`),
  packages:{platform:'0.34.19',types:'0.21.4',runtime:'0.23.24'},tests,status:'passed',
},null,2)+'\n');
console.log(tests);
